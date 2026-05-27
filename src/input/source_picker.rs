//! Source picker state.
//!
//! Shown on bare TTY launch (no file argument, no piped stdin, no
//! `--clipboard` / `--paste`) to ask the user where jiq should read
//! JSON from. The clipboard is peeked once at launch so the default
//! selection reflects what's actually available:
//! * Clipboard parses as object/array → default-select Clipboard, show
//!   a preview pane sourced from the cached bytes.
//! * Anything else → default-select Paste, show a one-line failure
//!   context (clipboard unreadable / empty / invalid / primitive).
//!
//! The cached bytes are handed to [`crate::input::FileLoader::from_clipboard_string`]
//! on confirm, so the actual load path never re-reads the clipboard.

use crate::input::loader::ClipboardPeek;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceChoice {
    Clipboard,
    Paste,
}

#[derive(Debug)]
pub struct SourcePickerState {
    /// Currently highlighted option. Mutated by the key handler / mouse
    /// hover; consumed on confirm to pick the load path.
    pub selection: SourceChoice,
    /// Cached clipboard bytes from the launch-time peek. Present iff
    /// the peek returned `Usable`. Consumed on Clipboard-confirm; the
    /// loader never re-reads the system clipboard.
    pub clipboard_cache: Option<String>,
    /// Snapshot of the launch-time peek outcome. Drives the rendered
    /// preview pane and the failure-context line. Refreshed when the
    /// user hits the manual-refresh key.
    pub peek: ClipboardPeek,
}

impl SourcePickerState {
    /// Build a picker from a launch-time clipboard peek. Pre-selection
    /// is data-driven: usable clipboard → Clipboard; anything else →
    /// Paste, so the default choice always matches reality.
    pub fn from_peek(peek: ClipboardPeek) -> Self {
        let (selection, clipboard_cache) = match &peek {
            ClipboardPeek::Usable(bytes) => (SourceChoice::Clipboard, Some(bytes.clone())),
            _ => (SourceChoice::Paste, None),
        };
        Self {
            selection,
            clipboard_cache,
            peek,
        }
    }

    /// Move the highlight to the next option. Two options today, but
    /// the cycle is encoded explicitly so an added third source would
    /// preserve direction-of-travel rather than silently aliasing
    /// `select_previous`.
    pub fn select_next(&mut self) {
        self.selection = match self.selection {
            SourceChoice::Clipboard => SourceChoice::Paste,
            SourceChoice::Paste => SourceChoice::Clipboard,
        };
    }

    /// Move the highlight to the previous option. Inverse cycle of
    /// [`select_next`].
    pub fn select_previous(&mut self) {
        self.selection = match self.selection {
            SourceChoice::Clipboard => SourceChoice::Paste,
            SourceChoice::Paste => SourceChoice::Clipboard,
        };
    }
}

#[cfg(test)]
#[path = "source_picker_tests.rs"]
mod source_picker_tests;
