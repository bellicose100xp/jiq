use crate::config::ClipboardBackend;

use super::{osc52, system};

pub type ClipboardResult = Result<(), ClipboardError>;

#[derive(Debug)]
pub enum ClipboardError {
    SystemUnavailable,
    WriteError,
}

pub fn copy_to_clipboard(text: &str, backend: ClipboardBackend) -> ClipboardResult {
    log::debug!("Clipboard copy: backend={:?}, len={}", backend, text.len());
    let result = match backend {
        ClipboardBackend::System => system::copy(text),
        ClipboardBackend::Osc52 => osc52::copy(text),
        ClipboardBackend::Auto => system::copy(text).or_else(|_| {
            log::debug!("System clipboard failed, falling back to OSC52");
            osc52::copy(text)
        }),
    };
    if let Err(ref e) = result {
        log::warn!("Clipboard copy failed: {:?}", e);
    }
    result
}

#[cfg(test)]
#[path = "backend_tests.rs"]
mod backend_tests;
