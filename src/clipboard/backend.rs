use crate::config::ClipboardBackend;

use super::{osc52, system};

pub type ClipboardResult = Result<(), ClipboardError>;

#[derive(Debug)]
pub enum ClipboardError {
    SystemUnavailable,
    WriteError,
}

pub fn copy_to_clipboard(text: &str, backend: ClipboardBackend) -> ClipboardResult {
    match backend {
        ClipboardBackend::System => system::copy(text),
        ClipboardBackend::Osc52 => osc52::copy(text),
        ClipboardBackend::Auto => system::copy(text).or_else(|_| osc52::copy(text)),
    }
}

#[cfg(test)]
#[path = "backend_tests.rs"]
mod backend_tests;
