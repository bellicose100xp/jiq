//! Bounded undo ring for `>` (drill-in) on the results pane.
//!
//! Each `>` press pushes a snapshot of the prior input AND the prior
//! results-pane viewport (cursor row + vertical/horizontal scroll), so a
//! subsequent `<` returns the user not just to the old query but to the
//! exact spot they were looking at. `<` always pops the most recent
//! snapshot, even if the user manually edited the textarea between
//! drill-ins — the trade-off is a simpler mental model (`<` always undoes
//! a `>`) over preserving intermediate edits.

use std::collections::VecDeque;

/// Maximum number of `>`-snapshots retained per session. Drill chains
/// deeper than this discard the oldest entries silently.
const MAX_DEPTH: usize = 20;

/// Restored viewport state to apply on `<`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ViewportState {
    pub cursor_row: u32,
    pub scroll_offset: u16,
    pub h_offset: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Snapshot {
    prev: String,
    viewport: ViewportState,
}

#[derive(Debug, Default)]
pub struct QueryUndoRing {
    entries: VecDeque<Snapshot>,
}

impl QueryUndoRing {
    pub fn new() -> Self {
        Self::default()
    }

    /// True when no snapshot is stacked. Used by the renderer to decide
    /// whether to show the `< back` border hint.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Push a `>`-snapshot. `prev` is the input as it was *before* the
    /// drill-in; `viewport` is the results-pane state to restore on `<`.
    pub fn push(&mut self, prev: impl Into<String>, viewport: ViewportState) {
        if self.entries.len() == MAX_DEPTH {
            self.entries.pop_front();
        }
        self.entries.push_back(Snapshot {
            prev: prev.into(),
            viewport,
        });
    }

    /// Pop the most recent snapshot. Returns `None` when the ring is
    /// empty.
    pub fn pop(&mut self) -> Option<(String, ViewportState)> {
        self.entries.pop_back().map(|s| (s.prev, s.viewport))
    }

    #[cfg(test)]
    pub fn depth(&self) -> usize {
        self.entries.len()
    }
}

#[cfg(test)]
#[path = "query_undo_tests.rs"]
mod query_undo_tests;
