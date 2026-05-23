//! Bounded undo ring for `>` (drill-in) on the results pane.
//!
//! Each `>` press pushes a snapshot of the prior input AND the prior
//! results-pane viewport (cursor row + vertical/horizontal scroll), so a
//! subsequent `<` returns the user not just to the old query but to the
//! exact spot they were looking at. The ring's invariant is that when the
//! user fires `<`, the current input must still match the most recent
//! snapshot's `expected_after` — otherwise the user has manually edited
//! the textarea since the last drill-in, the snapshot's "go back to where
//! it came from" promise is no longer meaningful, and the ring is dropped
//! to avoid restoring a query that diverges from what's on screen.

use std::collections::VecDeque;

/// Maximum number of `>`-snapshots retained per session. Drill chains
/// deeper than this discard the oldest entries silently.
const MAX_DEPTH: usize = 20;

/// Restored viewport state to apply on `<`. Captured at `>`-time and
/// returned as part of [`PopOutcome::Restored`].
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ViewportState {
    pub cursor_row: u32,
    pub scroll_offset: u16,
    pub h_offset: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Snapshot {
    prev: String,
    expected_after: String,
    viewport: ViewportState,
}

#[derive(Debug, Default)]
pub struct QueryUndoRing {
    entries: VecDeque<Snapshot>,
}

/// Outcome of `pop_if_matches`: drives the user-visible notification path
/// in `>` / `<` handlers without leaking ring internals.
#[derive(Debug, PartialEq, Eq)]
pub enum PopOutcome {
    /// Snapshot popped successfully — restore the contained query string
    /// and viewport state.
    Restored {
        query: String,
        viewport: ViewportState,
    },
    /// Ring was empty.
    Empty,
    /// Top snapshot's `expected_after` did not match the current input,
    /// so the ring was cleared. The user manually edited the query
    /// between drill-ins.
    Invalidated,
}

impl QueryUndoRing {
    pub fn new() -> Self {
        Self::default()
    }

    /// True when no snapshot is stacked. Used by the renderer to decide
    /// whether to show the `< Back` border hint.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Push a `>`-snapshot. `prev` is the input as it was *before* the
    /// drill-in; `expected_after` is what the input becomes immediately
    /// after the drill-in (used to detect manual edits on the next `<`);
    /// `viewport` is the results-pane state to restore on `<`.
    pub fn push(
        &mut self,
        prev: impl Into<String>,
        expected_after: impl Into<String>,
        viewport: ViewportState,
    ) {
        if self.entries.len() == MAX_DEPTH {
            self.entries.pop_front();
        }
        self.entries.push_back(Snapshot {
            prev: prev.into(),
            expected_after: expected_after.into(),
            viewport,
        });
    }

    /// Try to pop the top snapshot. If `current` equals the expected post-
    /// apply string for the top snapshot, return its `prev` and viewport.
    /// Otherwise the user has typed in the input field since the last `>`,
    /// so clear the whole ring and signal `Invalidated`.
    pub fn pop_if_matches(&mut self, current: &str) -> PopOutcome {
        let top_expected = match self.entries.back() {
            Some(s) => s.expected_after.clone(),
            None => return PopOutcome::Empty,
        };
        if top_expected != current {
            self.entries.clear();
            return PopOutcome::Invalidated;
        }
        let snap = self.entries.pop_back().unwrap();
        PopOutcome::Restored {
            query: snap.prev,
            viewport: snap.viewport,
        }
    }

    #[cfg(test)]
    pub fn depth(&self) -> usize {
        self.entries.len()
    }
}

#[cfg(test)]
#[path = "query_undo_tests.rs"]
mod query_undo_tests;
