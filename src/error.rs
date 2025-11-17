use thiserror::Error;

/// Custom error types for jiq
#[derive(Debug, Error)]
pub enum JiqError {
    #[error("jq binary not found in PATH.\n\nInstall jq from: https://jqlang.org/download/")]
    JqNotFound,

    #[error("Invalid JSON input: {0}")]
    InvalidJson(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
