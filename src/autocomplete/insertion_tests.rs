//! Tests for autocomplete insertion functionality

// Test submodules
#[path = "insertion_tests/cursor_positioning_tests.rs"]
mod cursor_positioning_tests;
#[path = "insertion_tests/edge_case_tests.rs"]
mod edge_case_tests;
#[path = "insertion_tests/field_context_tests.rs"]
mod field_context_tests;
#[path = "insertion_tests/function_context_tests.rs"]
mod function_context_tests;
#[path = "insertion_tests/mid_query_insertion_tests.rs"]
mod mid_query_insertion_tests;
#[path = "insertion_tests/property_tests.rs"]
mod property_tests;
#[path = "insertion_tests/query_execution_tests.rs"]
mod query_execution_tests;
#[path = "insertion_tests/variable_insertion_tests.rs"]
mod variable_insertion_tests;

// Common test utilities
pub(crate) use super::insertion::*;
pub(crate) use crate::autocomplete::autocomplete_state::{Suggestion, SuggestionType};
pub(crate) use crate::query::ResultType;
pub(crate) use crate::test_utils::test_helpers::{execute_query_and_wait, test_app};
pub(crate) use tui_textarea::TextArea;

/// Helper to create a test environment for insertion
pub(crate) fn setup_insertion_test(
    initial_query: &str,
) -> (TextArea<'static>, crate::query::QueryState) {
    let mut textarea = TextArea::default();
    textarea.insert_str(initial_query);
    let query_state = crate::query::QueryState::new(r#"{"test": true}"#.to_string());
    (textarea, query_state)
}

/// Helper to create a test suggestion from text (for backward compatibility with existing tests)
pub(crate) fn test_suggestion(text: &str) -> Suggestion {
    Suggestion::new(text, SuggestionType::Field)
}
