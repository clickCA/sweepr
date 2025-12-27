use crate::error::{PurgeError, Result};
use ignore::Walk;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct FileDiscovery {
    pub files: Vec<PathBuf>,
    pub entry_points: Vec<PathBuf>,
}

pub struct WorkspaceScanner {
    root: PathBuf,
}

impl WorkspaceScanner {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    /// Discover all JavaScript/TypeScript files in the workspace
    pub fn discover(&self, entry_points: Vec<String>) -> Result<FileDiscovery> {
        let mut files = Vec::new();
        let _ignore_rules = self.load_gitignore()?;

        // Walk the directory
        for entry in Walk::new(&self.root)
            .filter(|entry| entry.as_ref().map_or(false, |e| {
                self.is_js_ts_file(e.path()) && !self.is_in_node_modules(e.path())
            }))
        {
            let entry = entry.map_err(|e| PurgeError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                e.to_string(),
            )))?;

            files.push(entry.path().to_path_buf());
        }

        // Resolve entry points
        let resolved_entry_points = entry_points
            .iter()
            .map(|ep| self.resolve_entry_point(ep))
            .collect::<Result<Vec<PathBuf>>>()?;

        Ok(FileDiscovery {
            files,
            entry_points: resolved_entry_points,
        })
    }

    fn is_js_ts_file(&self, path: &Path) -> bool {
        match path.extension().and_then(|ext| ext.to_str()) {
            Some(ext) => matches!(
                ext,
                "js" | "jsx" | "ts" | "tsx" | "mjs" | "cjs"
            ),
            None => false,
        }
    }

    fn is_in_node_modules(&self, path: &Path) -> bool {
        path.components()
            .any(|c| c.as_os_str() == "node_modules")
    }

    fn resolve_entry_point(&self, entry: &str) -> Result<PathBuf> {
        let path = self.root.join(entry);

        if path.exists() {
            Ok(path)
        } else {
            // Try common extensions
            for ext in &["ts", "js", "tsx", "jsx"] {
                let with_ext = path.with_extension(ext);
                if with_ext.exists() {
                    return Ok(with_ext);
                }
            }

            // Try index files
            for ext in &["ts", "js", "tsx", "jsx"] {
                let index = path.join(format!("index.{}", ext));
                if index.exists() {
                    return Ok(index);
                }
            }

            Err(PurgeError::InvalidEntryPoint(entry.to_string()))
        }
    }

    fn load_gitignore(&self) -> Result<ignore::overrides::Override> {
        let mut override_builder = ignore::overrides::OverrideBuilder::new(&self.root);

        // Add default ignores
        for pattern in &["node_modules", "dist", "build", ".git"] {
            override_builder
                .add(pattern)
                .map_err(|e| PurgeError::Config(e.to_string()))?;
        }

        override_builder
            .build()
            .map_err(|e| PurgeError::Config(e.to_string()))
    }
}
