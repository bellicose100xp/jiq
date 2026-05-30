//! jiq library - Interactive JSON query tool
//!
//! This library exposes the core functionality of jiq for testing purposes.

pub mod ai;
pub mod app;
pub mod autocomplete;
pub mod clipboard;
pub mod config;
pub mod editor;
pub mod error;
pub mod help;
pub mod history;
pub mod input;
pub mod json;
pub mod json_path;
pub mod layout;
pub mod notification;
pub mod path_at_cursor;
pub mod path_at_cursor_apply;
pub mod query;
pub mod query_undo;
pub mod results;
pub mod save;
pub mod scroll;
pub mod search;
pub mod snippets;
pub mod stats;
pub mod str_utils;
pub mod syntax_highlight;

#[cfg(test)]
pub mod test_utils;
pub mod theme;
pub mod tooltip;
pub mod widgets;

// Re-export commonly used types for convenience
pub use app::{App, Focus, OutputMode};
pub use config::Config;

/// RAII timer that logs `"[TIMING] {label} took {ms}ms"` on drop. When the
/// debug logger is not initialised the line is filtered out; the only cost is
/// one `Instant::now()` call and a no-op log macro.
pub struct Timer {
    label: &'static str,
    start: std::time::Instant,
}

impl Timer {
    pub fn new(label: &'static str) -> Self {
        Self {
            label,
            start: std::time::Instant::now(),
        }
    }
}

impl Drop for Timer {
    fn drop(&mut self) {
        log::debug!(
            "[TIMING] {} took {}ms",
            self.label,
            self.start.elapsed().as_millis()
        );
    }
}
