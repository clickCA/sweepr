use crate::rules::AnalysisReport;
use std::io::{self, Write};

pub trait Reporter {
    fn report(&self, report: &AnalysisReport) -> io::Result<()>;
}

pub struct CliReporter;

impl Reporter for CliReporter {
    fn report(&self, report: &AnalysisReport) -> io::Result<()> {
        let stdout = io::stdout();
        let mut handle = stdout.lock();

        writeln!(handle, "\n🔍 Sweepr Analysis Report\n")?;

        // Unused dependencies
        if !report.unused_dependencies.is_empty() {
            writeln!(handle, "❌ Unused Dependencies ({})", report.unused_dependencies.len())?;
            writeln!(handle, "────────────────────────────────")?;
            for dep in &report.unused_dependencies {
                writeln!(handle, "  • {}@{}", dep.name, dep.version)?;
            }
            writeln!(handle)?;
        }

        // Unused exports
        if !report.unused_exports.is_empty() {
            writeln!(handle, "📦 Unused Exports ({})", report.unused_exports.len())?;
            writeln!(handle, "────────────────────────────────")?;
            for export in &report.unused_exports {
                writeln!(
                    handle,
                    "  • {} in {}:{}",
                    export.name,
                    export.file.display(),
                    export.line
                )?;
            }
            writeln!(handle)?;
        }

        // Unused files
        if !report.unused_files.is_empty() {
            writeln!(handle, "📄 Unused Files ({})", report.unused_files.len())?;
            writeln!(handle, "────────────────────────────────")?;
            for file in &report.unused_files {
                writeln!(handle, "  • {}", file.path.display())?;
            }
            writeln!(handle)?;
        }

        if report.unused_dependencies.is_empty()
            && report.unused_exports.is_empty()
            && report.unused_files.is_empty()
        {
            writeln!(handle, "✅ No unused code found! Your project is clean.\n")?;
        } else {
            let total = report.unused_dependencies.len()
                + report.unused_exports.len()
                + report.unused_files.len();
            writeln!(handle, "📊 Summary: {} issues found\n", total)?;
        }

        Ok(())
    }
}

pub struct JsonReporter;

impl Reporter for JsonReporter {
    fn report(&self, report: &AnalysisReport) -> io::Result<()> {
        let json = serde_json::to_string_pretty(report)?;
        println!("{}", json);
        Ok(())
    }
}
