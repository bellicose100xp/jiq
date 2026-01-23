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
pub mod layout;
pub mod notification;
pub mod query;
pub mod results;
pub mod scroll;
pub mod search;
pub mod snippets;
pub mod stats;
pub mod syntax_highlight;

#[cfg(test)]
pub mod test_utils;
pub mod tooltip;
pub mod widgets;

// Re-export commonly used types for convenience
pub use app::{App, Focus, OutputMode};
pub use config::Config;
