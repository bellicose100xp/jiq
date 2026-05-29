//! jiq library - Interactive JSON query tool
//!
//! This library exposes the core functionality of jiq for testing purposes.

pub mod ai;
pub mod app;
pub mod autocomplete;
pub mod bench_script;
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
pub mod perf;
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
