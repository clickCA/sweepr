use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

/// Represents a single file in the project
#[derive(Debug, Clone)]
pub struct FileNode {
    pub path: PathBuf,
    pub is_entry_point: bool,
}

/// Represents an exported symbol
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Symbol {
    pub name: String,
    pub file: PathBuf,
    pub span: (usize, usize),
}

/// Import relationship between files
#[derive(Debug, Clone)]
pub struct ImportEdge {
    pub from: PathBuf,
    pub to: PathBuf,
    pub imported_symbols: Vec<String>,
    pub is_type_only: bool,
}

/// Symbol reference
#[derive(Debug, Clone)]
pub struct SymbolReference {
    pub symbol: String,
    pub file: PathBuf,
    pub span: (usize, usize),
}

/// File Import Graph - tracks how files import each other
#[derive(Debug, Clone)]
pub struct FileImportGraph {
    pub files: HashMap<PathBuf, FileNode>,
    pub imports: Vec<ImportEdge>,
}

impl FileImportGraph {
    pub fn new() -> Self {
        Self {
            files: HashMap::new(),
            imports: Vec::new(),
        }
    }

    pub fn add_file(&mut self, path: PathBuf, is_entry_point: bool) {
        self.files.insert(
            path.clone(),
            FileNode {
                path,
                is_entry_point,
            },
        );
    }

    pub fn add_import(&mut self, edge: ImportEdge) {
        self.imports.push(edge);
    }

    /// Find all files reachable from entry points
    pub fn reachable_files(&self) -> HashSet<PathBuf> {
        let mut reachable = HashSet::new();
        let mut stack: Vec<PathBuf> = self
            .files
            .values()
            .filter(|f| f.is_entry_point)
            .map(|f| f.path.clone())
            .collect();

        while let Some(current) = stack.pop() {
            if reachable.contains(&current) {
                continue;
            }

            reachable.insert(current.clone());

            // Find all files imported by this file
            for edge in &self.imports {
                if edge.from == current {
                    stack.push(edge.to.clone());
                }
            }
        }

        reachable
    }
}

/// Symbol Usage Graph - tracks exports and their references
#[derive(Debug, Clone)]
pub struct SymbolUsageGraph {
    pub exports: HashMap<PathBuf, Vec<Symbol>>,
    pub references: HashMap<PathBuf, Vec<SymbolReference>>,
}

impl SymbolUsageGraph {
    pub fn new() -> Self {
        Self {
            exports: HashMap::new(),
            references: HashMap::new(),
        }
    }

    pub fn add_export(&mut self, file: PathBuf, symbol: Symbol) {
        self.exports
            .entry(file)
            .or_insert_with(Vec::new)
            .push(symbol);
    }

    pub fn add_reference(&mut self, file: PathBuf, reference: SymbolReference) {
        self.references
            .entry(file)
            .or_insert_with(Vec::new)
            .push(reference);
    }

    /// Find unused exports in a file
    pub fn unused_exports_in_file(&self, file: &PathBuf) -> Vec<&Symbol> {
        let exports = self.exports.get(file);
        let mut unused = Vec::new();

        if let Some(exports) = exports {
            for export in exports {
                let mut is_used = false;

                // Check all references across all files
                for (_ref_file, refs) in &self.references {
                    for reference in refs {
                        if reference.symbol == export.name {
                            is_used = true;
                            break;
                        }
                    }
                    if is_used {
                        break;
                    }
                }

                if !is_used {
                    unused.push(export);
                }
            }
        }

        unused
    }
}

/// Dependency Graph - tracks npm package usage
#[derive(Debug, Clone)]
pub struct DependencyGraph {
    pub dependencies: HashMap<String, PackageInfo>,
}

#[derive(Debug, Clone)]
pub struct PackageInfo {
    pub name: String,
    pub version: String,
    pub import_locations: Vec<PathBuf>,
    pub is_used: bool,
}

impl DependencyGraph {
    pub fn new() -> Self {
        Self {
            dependencies: HashMap::new(),
        }
    }

    pub fn add_dependency(&mut self, name: String, version: String) {
        self.dependencies.entry(name.clone()).or_insert_with(|| {
            PackageInfo {
                name: name.clone(),
                version,
                import_locations: Vec::new(),
                is_used: false,
            }
        });
    }

    pub fn record_import(&mut self, package: &str, file: PathBuf) {
        if let Some(dep) = self.dependencies.get_mut(package) {
            dep.import_locations.push(file);
            dep.is_used = true;
        }
    }

    pub fn unused_dependencies(&self) -> Vec<&PackageInfo> {
        self.dependencies
            .values()
            .filter(|dep| !dep.is_used)
            .collect()
    }
}
