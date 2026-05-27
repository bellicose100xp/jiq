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

use super::loader::scan_json_or_jsonl;

/// Soft cap on pasted content. Larger pastes are rejected with a clear
/// message rather than being shoved into the textarea (which would block
/// the event loop on each insert).
pub const PASTE_RECOVERY_MAX_BYTES: usize = 16 * 1024 * 1024;

/// Why paste-recovery is on screen.
///
/// `Recovery` is the existing failure-recovery path: clipboard couldn't
/// be read, came back empty, or contained invalid/primitive JSON. We
/// render with a red error border and a "No JSON loaded" title because
/// something *did* go wrong.
///
/// `Explicit` is the user deliberately asking for paste mode (via
/// `--paste` or the source picker's Paste option). Nothing has failed;
/// render with a calm cyan border and a neutral "Paste JSON" title so
/// it doesn't look like an error screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PasteRecoveryMode {
    Recovery,
    Explicit,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PasteRecoveryState {
    /// Top-of-screen message. In `Recovery` mode this starts as the
    /// loader's diagnosis line (e.g. "Clipboard does not contain valid
    /// JSON.") and is replaced by the parse error after a failed
    /// submit ("Invalid JSON: expected `,` at line 3 column 5"). In
    /// `Explicit` mode this is empty unless the picker's smart
    /// fallback added a "why we're here" context line, or a parse
    /// error replaced it.
    ///
    /// Empty string means "no info to show" — the renderer suppresses
    /// the top block entirely when this is empty AND the mode is
    /// `Explicit`, so the textarea claims the full screen.
    pub error_message: String,
    /// Whether the user landed here from a clipboard failure
    /// (`Recovery`) or asked for paste explicitly (`Explicit`). Drives
    /// the renderer's title / border color choice.
    pub mode: PasteRecoveryMode,
}

impl PasteRecoveryState {
    /// Construct a recovery-mode state — used when something went
    /// wrong reading the clipboard. `error_message` is shown in the
    /// red top block.
    pub fn new(error_message: impl Into<String>) -> Self {
        Self {
            error_message: error_message.into(),
            mode: PasteRecoveryMode::Recovery,
        }
    }

    /// Construct an explicit-paste state — used when the user asked
    /// for paste mode deliberately (via `--paste` or the picker's
    /// Paste option). The top info block stays hidden because the
    /// textarea's title and placeholder already say "Paste JSON,
    /// press Enter to load"; an extra info box would just repeat that.
    pub fn new_explicit() -> Self {
        Self {
            error_message: String::new(),
            mode: PasteRecoveryMode::Explicit,
        }
    }

    /// Like [`new_explicit`] but with a one-line context describing
    /// why we landed here. Used when jiq jumps straight to the paste
    /// editor because the clipboard wasn't usable on launch (e.g.
    /// "Clipboard is empty — paste below to load."). When `context` is
    /// `None` or empty, behaves exactly like [`new_explicit`].
    pub fn new_explicit_with_context(context: Option<&str>) -> Self {
        let message = match context {
            Some(line) if !line.is_empty() => line.to_string(),
            _ => String::new(),
        };
        Self {
            error_message: message,
            mode: PasteRecoveryMode::Explicit,
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

        match scan_json_or_jsonl(content) {
            Ok(scan) if scan.all_containers => Ok(content.to_string()),
            Ok(_) => {
                let msg =
                    "Input must be a JSON object or array, not a primitive value.".to_string();
                self.error_message = msg.clone();
                Err(msg)
            }
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
