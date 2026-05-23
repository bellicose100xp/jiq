//! Path-at-cursor lookup for the results pane.
//!
//! Computes the jq path or RFC 6901 JSON Pointer of the value pretty-printed
//! on a given line of the current result. Backed by a single-entry cache so
//! repeated reads on the same row do not re-walk the value tree.

use crate::json_path::{JsonPath, path_at_line};
use serde_json::Value;
use std::sync::Arc;

/// Single-entry path cache: at most one (result-generation, cursor row, path).
#[derive(Debug, Clone, Default)]
pub struct PathAtCursorCache {
    row: u32,
    cached: Option<JsonPath>,
    has_cache: bool,
}

impl PathAtCursorCache {
    pub fn new() -> Self {
        Self::default()
    }

    /// Drop the cached entry. Called whenever the underlying result changes
    /// (so a stale path string never crosses a result boundary).
    pub fn invalidate(&mut self) {
        self.has_cache = false;
        self.cached = None;
    }

    /// Resolve the path for the cursor row, reusing the cache when possible.
    /// Returns `None` when the cursor row maps to no path.
    pub fn resolve(&mut self, value: &Arc<Value>, cursor_row: u32) -> Option<JsonPath> {
        if self.has_cache && self.row == cursor_row {
            return self.cached.clone();
        }
        let path = path_at_line(value, cursor_row as usize);
        self.row = cursor_row;
        self.cached = path.clone();
        self.has_cache = true;
        path
    }
}

#[cfg(test)]
#[path = "path_at_cursor_tests.rs"]
mod path_at_cursor_tests;
