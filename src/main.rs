mod cli;
mod config;
mod error;
mod graph;
mod parser;
mod reporter;
mod rules;
mod scanner;

use crate::config::Config;
use crate::error::{PurgeError, Result};
use crate::graph::{DependencyGraph, FileImportGraph, SymbolUsageGraph};
use crate::reporter::{CliReporter, JsonReporter, Reporter};
use crate::rules::RulesEngine;
use crate::scanner::WorkspaceScanner;
use clap::Parser;
use std::time::Instant;

#[derive(Parser, Debug)]
#[command(name = "sweepr")]
#[command(about = "Blazing-fast dead code elimination for JavaScript and TypeScript", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Parser, Debug)]
enum Commands {
    /// Check for unused code (read-only, no modifications)
    Check {
        /// Output results in JSON format
        #[arg(short, long)]
        json: bool,

        /// Custom entry points
        #[arg(short, long)]
        entry: Vec<String>,
    },

    /// Fix unused code (safe modifications only)
    Fix {
        /// Allow dangerous operations (file deletion)
        #[arg(long, name = "unsafe")]
        allow_unsafe: bool,

        /// Output results in JSON format
        #[arg(short, long)]
        json: bool,

        /// Custom entry points
        #[arg(short, long)]
        entry: Vec<String>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    tracing_subscriber::fmt::init();

    match cli.command {
        Commands::Check { json, entry } => {
            run_check(json, entry)?;
        }
        Commands::Fix { allow_unsafe: _, json, entry } => {
            run_check(json, entry)?;
            // TODO: Implement fix functionality
            eprintln!("⚠️  Fix functionality is not yet implemented");
        }
    }

    Ok(())
}

fn run_check(json: bool, entry_points: Vec<String>) -> Result<()> {
    let start = Instant::now();

    // Load configuration
    let config = Config::find_and_load()?;

    // Determine entry points
    let entry_points = if entry_points.is_empty() {
        config.entry
    } else {
        entry_points
    };

    println!("🚀 Scanning workspace...");

    // Scan workspace
    let current_dir = std::env::current_dir()?;
    let scanner = WorkspaceScanner::new(current_dir);
    let discovery = scanner.discover(entry_points)?;

    println!("  📄 Found {} files", discovery.files.len());
    println!("  🎯 Entry points: {}", discovery.entry_points.len());
    println!();

    println!("🔬 Analyzing code...");

    // Parse all files
    let files = discovery.files.clone();
    let parsed_files = parser::AstAnalyzer::parse_files_parallel(files)?;

    println!("  ✓ Parsed {} files", parsed_files.len());

    // Build graphs
    let mut file_graph = FileImportGraph::new();
    let mut symbol_graph = SymbolUsageGraph::new();
    let mut dependency_graph = DependencyGraph::new();

    // Add files to graph
    for file in &discovery.files {
        file_graph.add_file(file.clone(), discovery.entry_points.contains(file));
    }

    // Process parsed files
    for parsed_file in &parsed_files {
        // Add imports to file graph
        for import in &parsed_file.imports {
            file_graph.add_import(import.clone());
        }

        // Add exports to symbol graph
        for export in &parsed_file.exports {
            symbol_graph.add_export(parsed_file.path.clone(), export.clone());
        }

        // Add references to symbol graph
        for reference in &parsed_file.references {
            symbol_graph.add_reference(parsed_file.path.clone(), reference.clone());
        }
    }

    println!("  ✓ Built analysis graphs");

    // Load package.json dependencies
    if let Ok(deps) = load_dependencies() {
        for (name, version) in deps {
            dependency_graph.add_dependency(name, version);
        }

        // Record imports from parsed files
        for parsed_file in &parsed_files {
            for import in &parsed_file.imports {
                let source = import.to.to_string_lossy().to_string();
                if let Some(package_name) = extract_package_name(&source) {
                    dependency_graph.record_import(&package_name, parsed_file.path.clone());
                }
            }
        }

        println!("  ✓ Loaded {} dependencies", dependency_graph.dependencies.len());
    }

    println!();

    // Run analysis
    let analysis = RulesEngine::analyze(&dependency_graph, &file_graph, &symbol_graph);

    // Generate report
    let duration = start.elapsed();

    if json {
        let reporter = JsonReporter;
        reporter.report(&analysis)?;
    } else {
        let reporter = CliReporter;
        reporter.report(&analysis)?;
        println!("⏱️  Completed in {:.2?}", duration);
    }

    Ok(())
}

fn load_dependencies() -> Result<Vec<(String, String)>> {
    let current_dir = std::env::current_dir()?;
    let package_json_path = current_dir.join("package.json");

    if !package_json_path.exists() {
        return Ok(Vec::new());
    }

    let content = std::fs::read_to_string(&package_json_path)
        .map_err(|e| PurgeError::Io(e))?;

    let json: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| PurgeError::Config(format!("Invalid package.json: {}", e)))?;

    let mut dependencies = Vec::new();

    if let Some(deps) = json.get("dependencies").and_then(|d| d.as_object()) {
        for (name, version) in deps {
            if let Some(version_str) = version.as_str() {
                dependencies.push((name.clone(), version_str.to_string()));
            }
        }
    }

    if let Some(dev_deps) = json.get("devDependencies").and_then(|d| d.as_object()) {
        for (name, version) in dev_deps {
            if let Some(version_str) = version.as_str() {
                dependencies.push((name.clone(), version_str.to_string()));
            }
        }
    }

    Ok(dependencies)
}

fn extract_package_name(import_path: &str) -> Option<String> {
    // If it's not a relative path, it might be a package
    if !import_path.starts_with('.') && !import_path.starts_with('/') {
        // Extract the package name (handle scoped packages like @scope/name)
        let parts: Vec<&str> = import_path.split('/').collect();

        if import_path.starts_with('@') && parts.len() >= 2 {
            // Scoped package: @scope/name
            Some(format!("{}/{}", parts[0], parts[1]))
        } else if !import_path.starts_with('@') && parts.len() >= 1 {
            // Regular package
            Some(parts[0].to_string())
        } else {
            None
        }
    } else {
        None
    }
}

