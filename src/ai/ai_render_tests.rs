//! Tests for AI render module
//!
//! This module organizes tests for the AI rendering functionality.

// Module declarations using #[path] attributes
#[path = "ai_render_tests/content_tests.rs"]
mod content_tests;
#[path = "ai_render_tests/height_persistence_tests.rs"]
mod height_persistence_tests;
#[path = "ai_render_tests/layout_tests.rs"]
mod layout_tests;
#[path = "ai_render_tests/model_name_tests.rs"]
mod model_name_tests;
#[path = "ai_render_tests/snapshot_tests.rs"]
mod snapshot_tests;
#[path = "ai_render_tests/spacing_tests.rs"]
mod spacing_tests;
#[path = "ai_render_tests/widget_background_unit_tests.rs"]
mod widget_background_unit_tests;
#[path = "ai_render_tests/widget_selection_tests.rs"]
mod widget_selection_tests;

// Re-export items from parent module for test use
pub(crate) use super::ai_render::*;
pub(crate) use super::ai_state::AiState;
