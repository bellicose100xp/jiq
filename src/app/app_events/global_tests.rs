//! Tests for global key handlers
//!
//! This module contains all tests for the global key handler functionality.
//! Tests are organized into separate submodules for better organization.

// Re-export test modules
#[path = "global_tests/ai_suggestion_tests.rs"]
mod ai_suggestion_tests;
#[path = "global_tests/autocomplete_tests.rs"]
mod autocomplete_tests;
#[path = "global_tests/error_overlay_tests.rs"]
mod error_overlay_tests;
#[path = "global_tests/global_key_tests.rs"]
mod global_key_tests;
#[path = "global_tests/help_popup_tests.rs"]
mod help_popup_tests;

// Re-export common test utilities for use in submodules
pub(crate) use crate::app::app_state::{App, Focus, OutputMode};
pub(crate) use crate::editor::EditorMode;
pub(crate) use crate::test_utils::test_helpers::{
    TEST_JSON, app_with_query, execute_query_and_wait, key, key_with_mods, test_app,
    wait_for_query_completion,
};
pub(crate) use ratatui::crossterm::event::{KeyCode, KeyModifiers};

// Helper to execute any pending debounced query and wait for completion
// In tests, we need to manually trigger execution and poll for results
pub(crate) fn flush_debounced_query(app: &mut App) {
    if app.debouncer.has_pending() {
        crate::editor::editor_events::execute_query(app);
        app.debouncer.mark_executed();

        // Wait for async query to complete
        assert!(
            wait_for_query_completion(app, 2000),
            "Query did not complete within timeout"
        );
    }
}
