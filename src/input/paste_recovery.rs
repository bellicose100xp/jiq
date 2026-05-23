//! Paste-recovery state.
//!
//! When jiq is launched with no file argument and no piped stdin, and the
//! clipboard auto-load fails (unreadable, empty, or non-JSON content), the
//! app drops into this in-app paste-recovery flow: render a multi-line
//! input area where the user pastes JSON, then presses Enter to load.
//!
//! Editing reuses jiq's existing query input infrastructure: the same
//! `app.input.textarea` and `app.input.editor_mode`. That gives recovery
//! every VIM binding the query input has — operators, char-search, text
//! objects, dd/cc/D/C/dw/ci"/da[/;/,/u/Ctrl+r — for free. After a
//! successful submit the recovery state is consumed and the textarea is
//! cleared so the user lands on an empty query input as if the JSON had
//! loaded normally.
//!
//! On valid JSON/JSONL the recovery state is consumed. On invalid
//! content the parse error is shown in the top "No JSON loaded" block
//! and the user can edit and resubmit.

use crate::error::JiqError;

use super::loader::validate_json_or_jsonl;

/// Soft cap on pasted content. Larger pastes are rejected with a clear
/// message rather than being shoved into the textarea (which would block
/// the event loop on each insert).
pub const PASTE_RECOVERY_MAX_BYTES: usize = 16 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PasteRecoveryState {
    /// Top-of-screen error message. Starts as the loader's diagnosis line
    /// (e.g. "Clipboard does not contain valid JSON.") and is replaced by
    /// the parse error after a failed submit attempt
    /// ("Invalid JSON: expected `,` at line 3 column 5").
    pub error_message: String,
}

impl PasteRecoveryState {
    pub fn new(error_message: impl Into<String>) -> Self {
        Self {
            error_message: error_message.into(),
        }
    }

    /// Validate raw textarea content as JSON or JSONL.
    ///
    /// On success returns the JSON string for downstream use; the caller
    /// should consume the recovery state. On failure updates
    /// `error_message` so the top block shows the parse error and
    /// returns the message.
    pub fn try_submit(&mut self, content: &str) -> Result<String, String> {
        if content.trim().is_empty() {
            let msg = "Input is empty. Paste JSON and try again.".to_string();
            self.error_message = msg.clone();
            return Err(msg);
        }

        match validate_json_or_jsonl(content) {
            Ok(()) => Ok(content.to_string()),
            Err(JiqError::InvalidJson(detail)) => {
                let msg = format!("Invalid JSON: {}", detail);
                self.error_message = msg.clone();
                Err(msg)
            }
            Err(e) => {
                let msg = format!("Invalid JSON: {}", e);
                self.error_message = msg.clone();
                Err(msg)
            }
        }
    }
}

/// Normalise pasted line endings: CRLF -> LF, lone CR -> LF. JSON spec
/// accepts \n, \r, and \r\n as whitespace, but `tui-textarea::insert_str`
/// keeps a literal `\r` inside what it considers a single line, which
/// then breaks downstream rendering and parsing.
pub(crate) fn normalise_newlines(text: &str) -> String {
    if !text.contains('\r') {
        return text.to_string();
    }
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\r' {
            if chars.peek() == Some(&'\n') {
                chars.next();
            }
            out.push('\n');
        } else {
            out.push(c);
        }
    }
    out
}

#[cfg(test)]
#[path = "paste_recovery_tests.rs"]
mod paste_recovery_tests;
