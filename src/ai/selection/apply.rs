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
#[path = "apply_tests.rs"]
mod apply_tests;
