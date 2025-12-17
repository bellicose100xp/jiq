//! Tests for apply.rs

use super::*;
use crate::ai::suggestion::{Suggestion, SuggestionType};
use crate::autocomplete::AutocompleteState;
use crate::input::InputState;
use crate::query::QueryState;
use proptest::prelude::*;

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

// =========================================================================
// Unit Tests
// =========================================================================

#[test]
fn test_apply_suggestion_clears_existing_query() {
    let (mut input_state, mut query_state, mut autocomplete_state) = create_test_states();

    // Set up existing query
    input_state.textarea.insert_str(".existing.query");
    assert_eq!(input_state.query(), ".existing.query");

    let suggestion = make_suggestion(".new.query");
    let result = apply_suggestion(
        &suggestion,
        &mut input_state,
        &mut query_state,
        &mut autocomplete_state,
    );

    assert!(result.applied);
    assert_eq!(input_state.query(), ".new.query");
}

#[test]
fn test_apply_suggestion_cursor_at_end() {
    let (mut input_state, mut query_state, mut autocomplete_state) = create_test_states();

    let suggestion = make_suggestion(".name");
    apply_suggestion(
        &suggestion,
        &mut input_state,
        &mut query_state,
        &mut autocomplete_state,
    );

    // Cursor should be at position 5 (length of ".name")
    assert_eq!(input_state.textarea.cursor().1, 5);
}

#[test]
fn test_apply_suggestion_hides_autocomplete() {
    let (mut input_state, mut query_state, mut autocomplete_state) = create_test_states();

    // Make autocomplete visible
    autocomplete_state.update_suggestions(vec![crate::autocomplete::Suggestion::new(
        "test",
        crate::autocomplete::SuggestionType::Field,
    )]);
    assert!(autocomplete_state.is_visible());

    let suggestion = make_suggestion(".name");
    apply_suggestion(
        &suggestion,
        &mut input_state,
        &mut query_state,
        &mut autocomplete_state,
    );

    assert!(!autocomplete_state.is_visible());
}

#[test]
fn test_apply_suggestion_executes_query() {
    let (mut input_state, mut query_state, mut autocomplete_state) = create_test_states();

    let suggestion = make_suggestion(".name");
    apply_suggestion(
        &suggestion,
        &mut input_state,
        &mut query_state,
        &mut autocomplete_state,
    );

    // Query should have been executed
    assert!(query_state.result.is_ok());
    // Result should contain "test" (the value of .name in our test JSON)
    let result = query_state.result.as_ref().unwrap();
    assert!(result.contains("test"));
}

#[test]
fn test_apply_suggestion_returns_applied_query() {
    let (mut input_state, mut query_state, mut autocomplete_state) = create_test_states();

    let suggestion = make_suggestion(".value");
    let result = apply_suggestion(
        &suggestion,
        &mut input_state,
        &mut query_state,
        &mut autocomplete_state,
    );

    assert!(result.applied);
    assert_eq!(result.query, ".value");
}

#[test]
fn test_apply_suggestion_with_empty_initial_query() {
    let (mut input_state, mut query_state, mut autocomplete_state) = create_test_states();

    // Input is empty by default
    assert_eq!(input_state.query(), "");

    let suggestion = make_suggestion(".name");
    let result = apply_suggestion(
        &suggestion,
        &mut input_state,
        &mut query_state,
        &mut autocomplete_state,
    );

    assert!(result.applied);
    assert_eq!(input_state.query(), ".name");
}

#[test]
fn test_apply_suggestion_with_complex_query() {
    let (mut input_state, mut query_state, mut autocomplete_state) = create_test_states();

    let complex_query = r#".users[] | select(.active == true) | {name, email}"#;
    let suggestion = make_suggestion(complex_query);
    let result = apply_suggestion(
        &suggestion,
        &mut input_state,
        &mut query_state,
        &mut autocomplete_state,
    );

    assert!(result.applied);
    assert_eq!(input_state.query(), complex_query);
    assert_eq!(
        input_state.textarea.cursor().1,
        complex_query.chars().count()
    );
}

// =========================================================================
// Property-Based Tests
// =========================================================================

// **Feature: ai-assistant-phase3, Property 3: Full query replacement with cursor positioning**
// *For any* selected suggestion and any existing query text, applying the suggestion
// should result in the query input containing exactly the suggestion's query text
// with no additional characters, and the cursor positioned at the end.
// **Validates: Requirements 3.1, 3.2, 3.3**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop_full_query_replacement_with_cursor(
        existing_query in "\\.[a-zA-Z0-9_]{0,20}",
        suggestion_query in "\\.[a-zA-Z0-9_|\\[\\] ]{1,30}",
    ) {
        let (mut input_state, mut query_state, mut autocomplete_state) = create_test_states();

        // Set up existing query
        input_state.textarea.insert_str(&existing_query);

        let suggestion = make_suggestion(&suggestion_query);
        let result = apply_suggestion(&suggestion, &mut input_state, &mut query_state, &mut autocomplete_state);

        // Property 1: Suggestion was applied
        prop_assert!(result.applied, "Suggestion should be applied");

        // Property 2: Query input contains exactly the suggestion's query (Req 3.1, 3.3)
        prop_assert_eq!(
            input_state.query(),
            suggestion_query.as_str(),
            "Query input should contain exactly the suggestion's query"
        );

        // Property 3: Cursor is at end of query (Req 3.2)
        let expected_cursor_pos = suggestion_query.chars().count();
        prop_assert_eq!(
            input_state.textarea.cursor().1,
            expected_cursor_pos,
            "Cursor should be at end of inserted query"
        );
    }
}

// **Feature: ai-assistant-phase3, Property 8: Autocomplete hiding on selection**
// *For any* suggestion application, the autocomplete popup should transition
// to hidden state after the query is replaced.
// **Validates: Requirements 3.4**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop_autocomplete_hiding_on_selection(
        suggestion_query in "\\.[a-zA-Z0-9_]{1,20}",
        num_autocomplete_items in 1usize..10,
    ) {
        let (mut input_state, mut query_state, mut autocomplete_state) = create_test_states();

        // Set up autocomplete with some suggestions
        let autocomplete_suggestions: Vec<crate::autocomplete::Suggestion> = (0..num_autocomplete_items)
            .map(|i| crate::autocomplete::Suggestion::new(
                format!("field{}", i),
                crate::autocomplete::SuggestionType::Field,
            ))
            .collect();
        autocomplete_state.update_suggestions(autocomplete_suggestions);

        // Verify autocomplete is visible before
        prop_assert!(
            autocomplete_state.is_visible(),
            "Autocomplete should be visible before applying suggestion"
        );

        let suggestion = make_suggestion(&suggestion_query);
        apply_suggestion(&suggestion, &mut input_state, &mut query_state, &mut autocomplete_state);

        // Property: Autocomplete should be hidden after applying suggestion
        prop_assert!(
            !autocomplete_state.is_visible(),
            "Autocomplete should be hidden after applying suggestion"
        );
    }
}

// **Feature: ai-assistant-phase3, Property 9: Automatic execution after selection**
// *For any* applied suggestion, the jq query execution should be triggered
// automatically with the new query text.
// **Validates: Requirements 5.1, 5.2, 5.3**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop_automatic_execution_after_selection(
        // Use simple valid jq queries that will execute successfully
        field_name in "[a-z]{1,10}",
    ) {
        // Create test JSON with the field we'll query
        let json = format!(r#"{{"{}": "test_value"}}"#, field_name);
        let mut query_state = QueryState::new(json);
        let mut input_state = InputState::new();
        let mut autocomplete_state = AutocompleteState::new();

        let suggestion_query = format!(".{}", field_name);
        let suggestion = make_suggestion(&suggestion_query);

        apply_suggestion(&suggestion, &mut input_state, &mut query_state, &mut autocomplete_state);

        // Property 1: Query was executed (result is Ok)
        prop_assert!(
            query_state.result.is_ok(),
            "Query should execute successfully after applying suggestion"
        );

        // Property 2: Result contains expected value
        let result = query_state.result.as_ref().unwrap();
        prop_assert!(
            result.contains("test_value"),
            "Result should contain the queried value"
        );
    }
}
