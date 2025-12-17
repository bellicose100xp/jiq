use base64::{Engine as _, engine::general_purpose::STANDARD};
use std::io::{self, Write};

use super::backend::{ClipboardError, ClipboardResult};

pub fn copy(text: &str) -> ClipboardResult {
    let sequence = encode_osc52(text);

    // Write directly to stdout
    io::stdout()
        .write_all(sequence.as_bytes())
        .map_err(|_| ClipboardError::WriteError)?;

    io::stdout().flush().map_err(|_| ClipboardError::WriteError)
}

pub fn encode_osc52(text: &str) -> String {
    let encoded = STANDARD.encode(text);
    format!("\x1b]52;c;{}\x07", encoded)
}

#[cfg(test)]
#[path = "osc52_tests.rs"]
mod osc52_tests;
