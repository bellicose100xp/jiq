//! Query application logic for AI suggestions
//!
//! Handles replacing the query input with a selected suggestion,
//! positioning the cursor, hiding autocomplete, and triggering execution.

use crate::ai::suggestion::Suggestion;
use crate::autocomplete::AutocompleteState;
use crate::input::InputState;
use crate::query::QueryState;

/// Result of applying a suggestion
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApplyResult {
    /// Whether the suggestion was successfully applied
    pub applied: bool,
    /// The query that was applied (for verification)
    pub query: String,
}

/// Apply a selected AI suggestion to the query input
///
/// This function:
/// 1. Clears the existing query input completely
/// 2. Inserts the suggestion's query text
/// 3. Positions the cursor at the end of the query
/// 4. Hides the autocomplete popup
/// 5. Triggers query execution
///
/// # Arguments
/// * `suggestion` - The AI suggestion to apply
/// * `input_state` - The input state to modify
/// * `query_state` - The query state for execution
/// * `autocomplete_state` - The autocomplete state to hide
///
/// # Returns
/// `ApplyResult` indicating success and the applied query
///
/// # Requirements
/// - 3.1: Clear Query_Input completely before inserting new query
/// - 3.2: Position cursor at end of inserted query
/// - 3.3: Query_Input contains only suggestion's query text
/// - 3.4: Hide autocomplete popup
/// - 5.1: Execute jq query automatically with new query
/// - 5.2: Update results pane with new output
pub fn apply_suggestion(
    suggestion: &Suggestion,
    input_state: &mut InputState,
    query_state: &mut QueryState,
    autocomplete_state: &mut AutocompleteState,
) -> ApplyResult {
    let query = suggestion.query.clone();

    // Step 1: Clear existing query input completely (Requirement 3.1)
    // Select all text and delete it
    input_state
        .textarea
        .move_cursor(tui_textarea::CursorMove::Head);
    input_state
        .textarea
        .move_cursor(tui_textarea::CursorMove::End);
    // Delete all text by selecting all and replacing with empty
    while !input_state.query().is_empty() {
        input_state.textarea.delete_char();
    }

    // Step 2: Insert suggestion query (Requirement 3.3)
    input_state.textarea.insert_str(&query);

    // Step 3: Cursor is already at end after insert_str (Requirement 3.2)
    // Verify cursor position matches query length
    debug_assert_eq!(
        input_state.textarea.cursor().1,
        query.chars().count(),
        "Cursor should be at end of inserted query"
    );

    // Step 4: Hide autocomplete popup (Requirement 3.4)
    autocomplete_state.hide();

    // Step 5: Trigger query execution (Requirements 5.1, 5.2)
    query_state.execute(&query);

    ApplyResult {
        applied: true,
        query,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::suggestion::SuggestionType;
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
}
