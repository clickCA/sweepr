use thiserror::Error;

#[derive(Error, Debug)]
pub enum PurgeError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Parse error in {path}: {message}")]
    ParseError { path: String, message: String },

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("Invalid entry point: {0}")]
    InvalidEntryPoint(String),
}

pub type Result<T> = std::result::Result<T, PurgeError>;
