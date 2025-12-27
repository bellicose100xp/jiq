use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq)]
pub enum JiqError {
    #[error("jq binary not found in PATH.\n\nInstall jq from: https://jqlang.org/download/")]
    JqNotFound,

    #[error("Invalid JSON input: {0}")]
    InvalidJson(String),

    #[error("IO error: {0}")]
    Io(String),
}

impl From<std::io::Error> for JiqError {
    fn from(err: std::io::Error) -> Self {
        JiqError::Io(err.to_string())
    }
}

#[cfg(test)]
#[path = "error_tests.rs"]
mod error_tests;
