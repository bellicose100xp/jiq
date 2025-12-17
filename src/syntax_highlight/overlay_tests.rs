//! Tests for syntax_highlight/overlay

#[path = "overlay_tests/unit_tests.rs"]
mod unit_tests;

#[path = "overlay_tests/snapshot_tests.rs"]
mod snapshot_tests;

// Re-export common test utilities
pub(crate) use super::*;
pub(crate) use ratatui::style::{Color, Modifier, Style};
pub(crate) use ratatui::text::Span;

// Re-export snapshot helpers from parent module
pub(crate) use crate::syntax_highlight::snapshot_helpers::serialize_spans;
