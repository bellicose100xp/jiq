//! Tests for suggestion application (mouse-click entry point)

use super::*;
use crate::ai::suggestion::{Suggestion, SuggestionType};
use crate::autocomplete::AutocompleteState;
use crate::input::InputState;
use crate::query::QueryState;

// Helper to create a test suggestion
fn make_suggestion(query: &str) -> Suggestion {
    Suggestion {
        query: query.to_string(),
        description: "Test description".to_string(),
        suggestion_type: SuggestionType::Next,
    }
}

// Helper to create test states with valid JSON
fn create_test_states() -> (InputState, QueryState, AutocompleteState) {
    let input_state = InputState::new();
    let query_state = QueryState::new(r#"{"name": "test", "value": 42}"#.to_string());
    let autocomplete_state = AutocompleteState::new();
    (input_state, query_state, autocomplete_state)
}

// Test: apply_clicked_suggestion forwards a clicked suggestion through to
// apply_suggestion, clearing any existing query and inserting the new one.
#[test]
fn test_apply_clicked_suggestion_applies_to_query() {
    let (mut input_state, mut query_state, mut autocomplete_state) = create_test_states();

    // Seed an existing query that must be replaced.
    input_state.textarea.insert_str(".existing");
    assert_eq!(input_state.query(), ".existing");

    let suggestion = make_suggestion(".new.query");
    apply_clicked_suggestion(
        &suggestion,
        &mut input_state,
        &mut query_state,
        &mut autocomplete_state,
    );

    // The old query was cleared and the clicked suggestion's query inserted.
    assert_eq!(input_state.query(), ".new.query");
    // Cursor lands at the end of the inserted query (forwarded behavior).
    assert_eq!(
        input_state.textarea.cursor().1,
        ".new.query".chars().count()
    );
}
