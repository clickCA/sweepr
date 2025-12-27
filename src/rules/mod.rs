use crate::graph::{DependencyGraph, FileImportGraph, SymbolUsageGraph};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnusedDependency {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnusedExport {
    pub name: String,
    pub file: PathBuf,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnusedFile {
    pub path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisReport {
    pub unused_dependencies: Vec<UnusedDependency>,
    pub unused_exports: Vec<UnusedExport>,
    pub unused_files: Vec<UnusedFile>,
}

pub struct RulesEngine;

impl RulesEngine {
    pub fn analyze(
        dependency_graph: &DependencyGraph,
        file_graph: &FileImportGraph,
        symbol_graph: &SymbolUsageGraph,
    ) -> AnalysisReport {
        AnalysisReport {
            unused_dependencies: Self::find_unused_dependencies(dependency_graph),
            unused_exports: Self::find_unused_exports(symbol_graph, file_graph),
            unused_files: Self::find_unused_files(file_graph),
        }
    }

    /// Find dependencies that are never imported
    fn find_unused_dependencies(dependency_graph: &DependencyGraph) -> Vec<UnusedDependency> {
        dependency_graph
            .unused_dependencies()
            .into_iter()
            .map(|dep| UnusedDependency {
                name: dep.name.clone(),
                version: dep.version.clone(),
            })
            .collect()
    }

    /// Find exports that are never referenced
    fn find_unused_exports(
        symbol_graph: &SymbolUsageGraph,
        file_graph: &FileImportGraph,
    ) -> Vec<UnusedExport> {
        let mut unused = Vec::new();

        // Only check files that are reachable
        let reachable = file_graph.reachable_files();

        for file in reachable {
            let exports_in_file = symbol_graph.unused_exports_in_file(&file);

            for export in exports_in_file {
                unused.push(UnusedExport {
                    name: export.name.clone(),
                    file: export.file.clone(),
                    line: export.span.0,
                    column: export.span.1,
                });
            }
        }

        unused
    }

    /// Find files that are not reachable from any entry point
    fn find_unused_files(file_graph: &FileImportGraph) -> Vec<UnusedFile> {
        let reachable = file_graph.reachable_files();

        file_graph
            .files
            .values()
            .filter(|file| !reachable.contains(&file.path) && !file.is_entry_point)
            .map(|file| UnusedFile {
                path: file.path.clone(),
            })
            .collect()
    }
}
