use arboard::Clipboard;

use super::backend::{ClipboardError, ClipboardResult};

pub fn copy(text: &str) -> ClipboardResult {
    let mut clipboard = Clipboard::new().map_err(|_| ClipboardError::SystemUnavailable)?;

    clipboard
        .set_text(text)
        .map_err(|_| ClipboardError::WriteError)
}

#[cfg(test)]
#[path = "system_tests.rs"]
mod system_tests;
