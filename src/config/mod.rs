use crate::error::{PurgeError, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub entry: Vec<String>,

    #[serde(default)]
    pub ignore: Vec<String>,

    #[serde(default)]
    pub rules: RulesConfig,

    #[serde(default)]
    pub framework: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RulesConfig {
    #[serde(default = "default_true")]
    pub unused_deps: bool,

    #[serde(default = "default_true")]
    pub unused_exports: bool,

    #[serde(default = "default_true")]
    pub unused_files: bool,
}

fn default_true() -> bool {
    true
}

impl Default for Config {
    fn default() -> Self {
        Self {
            entry: vec!["src/index.ts".to_string()],
            ignore: vec![
                "**/*.test.ts".to_string(),
                "**/*.test.js".to_string(),
                "**/*.spec.ts".to_string(),
                "**/*.spec.js".to_string(),
                "**/node_modules/**".to_string(),
            ],
            rules: RulesConfig::default(),
            framework: None,
        }
    }
}

impl Config {
    /// Load configuration from a file
    pub fn load_from_file(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }

        let content = std::fs::read_to_string(path)
            .map_err(|e| PurgeError::Io(e))?;

        // Try to parse as JSON
        if let Ok(config) = serde_json::from_str::<Config>(&content) {
            return Ok(config);
        }

        // If JSON fails, try to extract JSON from a TypeScript config
        // For now, we'll return default if it's not valid JSON
        // In a real implementation, you'd want to strip the export wrapper
        Ok(Self::default())
    }

    /// Find and load config file from the current directory
    pub fn find_and_load() -> Result<Self> {
        let current_dir = std::env::current_dir()
            .map_err(|e| PurgeError::Io(e))?;

        // Check for sweepr.config.json
        let json_config = current_dir.join("sweepr.config.json");
        if json_config.exists() {
            return Self::load_from_file(&json_config);
        }

        // Check for sweepr.config.ts (basic detection)
        let ts_config = current_dir.join("sweepr.config.ts");
        if ts_config.exists() {
            return Self::load_from_file(&ts_config);
        }

        Ok(Self::default())
    }
}
