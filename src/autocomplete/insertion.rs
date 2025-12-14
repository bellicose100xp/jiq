//! Autocomplete suggestion insertion logic
//!
//! This module handles inserting autocomplete suggestions into the query,
//! managing cursor positioning, and executing the updated query.

use tui_textarea::{CursorMove, TextArea};

use crate::app::App;
use crate::autocomplete::autocomplete_state::Suggestion;
use crate::autocomplete::{SuggestionContext, analyze_context, find_char_before_field_access};
use crate::query::{CharType, QueryState};

#[cfg(debug_assertions)]
use log::debug;

/// Insert an autocomplete suggestion from App context
///
/// This function delegates to the existing `insert_suggestion()` function and
/// updates related app state (hide autocomplete, reset scroll, clear error overlay).
/// This pattern allows the App to delegate feature-specific logic to the autocomplete module.
///
/// # Arguments
/// * `app` - Mutable reference to the App struct
/// * `suggestion` - The suggestion to insert
pub fn insert_suggestion_from_app(app: &mut App, suggestion: &Suggestion) {
    // Delegate to existing insert_suggestion function
    insert_suggestion(&mut app.input.textarea, &mut app.query, suggestion);

    // Hide autocomplete and reset scroll/error state
    app.autocomplete.hide();
    app.results_scroll.reset();
    app.error_overlay_visible = false; // Auto-hide error overlay on query change

    // Handle AI state based on query result (clear on success)
    // Note: auto_show_on_error is false here since autocomplete doesn't trigger auto-show
    let cursor_pos = app.input.textarea.cursor().1;
    let query = app.input.textarea.lines()[0].as_ref();
    crate::ai::ai_events::handle_query_result(
        &mut app.ai,
        &app.query.result,
        false, // Don't auto-show on error for autocomplete
        query,
        cursor_pos,
        app.query.executor.json_input(),
    );
}

/// Insert an autocomplete suggestion at the current cursor position
/// Uses explicit state-based formulas for each context type
///
/// Returns the new query string after insertion
pub fn insert_suggestion(
    textarea: &mut TextArea<'_>,
    query_state: &mut QueryState,
    suggestion: &Suggestion,
) {
    let suggestion_text = &suggestion.text;
    // Get base query that produced these suggestions
    let base_query = match &query_state.base_query_for_suggestions {
        Some(b) => b.clone(),
        None => {
            // Fallback to current query if no base (shouldn't happen)
            textarea.lines()[0].clone()
        }
    };

    let query = textarea.lines()[0].clone();
    let cursor_pos = textarea.cursor().1;
    let before_cursor = &query[..cursor_pos.min(query.len())];

    #[cfg(debug_assertions)]
    debug!(
        "insert_suggestion: current_query='{}' base_query='{}' suggestion='{}' cursor_pos={}",
        query, base_query, suggestion_text, cursor_pos
    );

    // Determine the trigger context
    // Note: For insertion, we create a temporary BraceTracker since we only need
    // to distinguish FunctionContext from FieldContext here. ObjectKeyContext
    // handling will be added in task 9.
    let mut temp_tracker = crate::autocomplete::BraceTracker::new();
    temp_tracker.rebuild(before_cursor);
    let (context, partial) = analyze_context(before_cursor, &temp_tracker);

    #[cfg(debug_assertions)]
    debug!(
        "context_analysis: context={:?} partial='{}'",
        context, partial
    );

    // For function/operator context (jq keywords like then, else, end, etc.),
    // we should do simple replacement without adding dots or complex formulas
    if context == SuggestionContext::FunctionContext {
        // Simple replacement: remove the partial and insert the suggestion
        let replacement_start = cursor_pos.saturating_sub(partial.len());

        // Append opening parenthesis if the function requires arguments
        let insert_text = if suggestion.needs_parens {
            format!("{}(", suggestion_text)
        } else {
            suggestion_text.to_string()
        };

        let new_query = format!(
            "{}{}{}",
            &query[..replacement_start],
            insert_text,
            &query[cursor_pos..]
        );

        #[cfg(debug_assertions)]
        debug!(
            "function_context_replacement: partial='{}' new_query='{}'",
            partial, new_query
        );

        // Replace the entire line with new query
        textarea.delete_line_by_head();
        textarea.insert_str(&new_query);

        // Move cursor to end of inserted suggestion (including parenthesis if added)
        let target_pos = replacement_start + insert_text.len();
        move_cursor_to_column(textarea, target_pos);

        // Execute query
        execute_query_and_update(textarea, query_state);
        return;
    }

    // For ObjectKeyContext (object key names inside `{}`),
    // we do simple replacement similar to FunctionContext
    // This handles cases like `{na` -> `{name` or `{name: .name, ag` -> `{name: .name, age`
    if context == SuggestionContext::ObjectKeyContext {
        // Simple replacement: remove the partial and insert the suggestion
        let replacement_start = cursor_pos.saturating_sub(partial.len());

        let new_query = format!(
            "{}{}{}",
            &query[..replacement_start],
            suggestion_text,
            &query[cursor_pos..]
        );

        #[cfg(debug_assertions)]
        debug!(
            "object_key_context_replacement: partial='{}' new_query='{}'",
            partial, new_query
        );

        // Replace the entire line with new query
        textarea.delete_line_by_head();
        textarea.insert_str(&new_query);

        // Move cursor to end of inserted suggestion
        let target_pos = replacement_start + suggestion_text.len();
        move_cursor_to_column(textarea, target_pos);

        // Execute query
        execute_query_and_update(textarea, query_state);
        return;
    }

    // For field context, continue with the existing complex logic
    let char_before = find_char_before_field_access(before_cursor, &partial);
    let trigger_type = QueryState::classify_char(char_before);

    // Extract middle_query: everything between base and current field being typed
    // This preserves complex expressions like if/then/else, functions, etc.
    let mut middle_query = extract_middle_query(&query, &base_query, before_cursor, &partial);
    let mut adjusted_base = base_query.clone();
    let mut adjusted_suggestion = suggestion_text.to_string();

    #[cfg(debug_assertions)]
    debug!(
        "field_context: partial='{}' char_before={:?} trigger_type={:?} middle_query='{}'",
        partial, char_before, trigger_type, middle_query
    );

    // Special handling for CloseBracket trigger with [] in middle_query
    // This handles nested arrays like: .services[].capacityProviderStrategy[].field
    // When user types [], it becomes part of middle_query, but should be part of base
    if trigger_type == CharType::CloseBracket && middle_query == "[]" {
        #[cfg(debug_assertions)]
        debug!("nested_array_adjustment: detected [] in middle_query, moving to base");

        // Move [] from middle to base
        adjusted_base = format!("{}{}", base_query, middle_query);
        middle_query = String::new();

        // Strip [] prefix from suggestion if present (it's already in the query)
        // Also strip the leading dot since CloseBracket formula will add it
        if let Some(stripped) = adjusted_suggestion.strip_prefix("[]") {
            // Strip leading dot if present (e.g., "[].base" -> "base")
            adjusted_suggestion = stripped.strip_prefix('.').unwrap_or(stripped).to_string();

            #[cfg(debug_assertions)]
            debug!("nested_array_adjustment: stripped [] and leading dot from suggestion");
        }

        #[cfg(debug_assertions)]
        debug!(
            "nested_array_adjustment: adjusted_base='{}' adjusted_suggestion='{}' middle_query='{}'",
            adjusted_base, adjusted_suggestion, middle_query
        );
    }

    // Special case: if base is root "." and suggestion starts with ".",
    // replace the base entirely instead of appending
    // This prevents: "." + ".services" = "..services"
    let new_query = if adjusted_base == "."
        && adjusted_suggestion.starts_with('.')
        && middle_query.is_empty()
    {
        #[cfg(debug_assertions)]
        debug!("formula: root_replacement (special case for root '.')");

        adjusted_suggestion.to_string()
    } else {
        // Apply insertion formula: base + middle + (operator) + suggestion
        // The middle preserves complex expressions between base and current field
        let formula_result = match trigger_type {
            CharType::NoOp => {
                // NoOp means continuing a path, but we need to check if suggestion needs a dot
                // - If suggestion starts with special char like [, {, etc., don't add dot
                // - If base is root ".", don't add another dot
                // - Otherwise, add dot for path continuation (like .user.name)
                let needs_dot = !adjusted_suggestion.starts_with('[')
                    && !adjusted_suggestion.starts_with('{')
                    && !adjusted_suggestion.starts_with('.')
                    && adjusted_base != ".";

                if needs_dot {
                    #[cfg(debug_assertions)]
                    debug!("formula: NoOp -> base + middle + '.' + suggestion");

                    format!("{}{}.{}", adjusted_base, middle_query, adjusted_suggestion)
                } else {
                    #[cfg(debug_assertions)]
                    debug!("formula: NoOp -> base + middle + suggestion (no dot added)");

                    format!("{}{}{}", adjusted_base, middle_query, adjusted_suggestion)
                }
            }
            CharType::CloseBracket => {
                #[cfg(debug_assertions)]
                debug!("formula: CloseBracket -> base + middle + '.' + suggestion");

                // Formula: base + middle + "." + suggestion
                format!("{}{}.{}", adjusted_base, middle_query, adjusted_suggestion)
            }
            CharType::PipeOperator | CharType::Semicolon | CharType::Comma | CharType::Colon => {
                #[cfg(debug_assertions)]
                debug!("formula: Separator -> base + middle + ' ' + suggestion");

                // Formula: base + middle + " " + suggestion
                // Trim trailing space from middle to avoid double spaces
                let trimmed_middle = middle_query.trim_end();
                format!(
                    "{}{} {}",
                    adjusted_base, trimmed_middle, adjusted_suggestion
                )
            }
            CharType::OpenParen => {
                #[cfg(debug_assertions)]
                debug!(
                    "formula: OpenParen -> base + middle + suggestion (paren already in middle)"
                );

                // Formula: base + middle + suggestion
                // The ( is already in middle_query, don't add it again
                format!("{}{}{}", adjusted_base, middle_query, adjusted_suggestion)
            }
            CharType::OpenBracket => {
                #[cfg(debug_assertions)]
                debug!(
                    "formula: OpenBracket -> base + middle + suggestion (bracket already in middle)"
                );

                // Formula: base + middle + suggestion
                // The [ is already in middle_query, don't add it again
                format!("{}{}{}", adjusted_base, middle_query, adjusted_suggestion)
            }
            CharType::OpenBrace => {
                #[cfg(debug_assertions)]
                debug!(
                    "formula: OpenBrace -> base + middle + suggestion (brace already in middle)"
                );

                // Formula: base + middle + suggestion
                // The { is already in middle_query, don't add it again
                format!("{}{}{}", adjusted_base, middle_query, adjusted_suggestion)
            }
            CharType::QuestionMark => {
                #[cfg(debug_assertions)]
                debug!("formula: QuestionMark -> base + middle + '.' + suggestion");

                // Formula: base + middle + "." + suggestion
                format!("{}{}.{}", adjusted_base, middle_query, adjusted_suggestion)
            }
            CharType::Dot => {
                #[cfg(debug_assertions)]
                debug!("formula: Dot -> base + middle + suggestion");

                // Formula: base + middle + suggestion
                format!("{}{}{}", adjusted_base, middle_query, adjusted_suggestion)
            }
            CharType::CloseParen | CharType::CloseBrace => {
                #[cfg(debug_assertions)]
                debug!("formula: CloseParen/CloseBrace -> base + middle + '.' + suggestion");

                // Formula: base + middle + "." + suggestion
                format!("{}{}.{}", adjusted_base, middle_query, adjusted_suggestion)
            }
        };

        #[cfg(debug_assertions)]
        debug!(
            "formula_components: base='{}' middle='{}' suggestion='{}'",
            adjusted_base, middle_query, adjusted_suggestion
        );

        formula_result
    };

    #[cfg(debug_assertions)]
    debug!("new_query_constructed: '{}'", new_query);

    // Replace the entire line with new query
    textarea.delete_line_by_head();
    textarea.insert_str(&new_query);

    #[cfg(debug_assertions)]
    debug!("query_after_insertion: '{}'", textarea.lines()[0]);

    // Move cursor to end of query
    let target_pos = new_query.len();
    move_cursor_to_column(textarea, target_pos);

    // Execute query
    execute_query_and_update(textarea, query_state);
}

/// Extract middle query: everything between base and current field being typed
///
/// Examples:
/// - Query: ".services | if has(...) then .ca", base: ".services"
///   → middle: " | if has(...) then "
/// - Query: ".services | .ca", base: ".services"
///   → middle: " | "
/// - Query: ".services.ca", base: ".services"
///   → middle: ""
pub fn extract_middle_query(
    current_query: &str,
    base_query: &str,
    before_cursor: &str,
    partial: &str,
) -> String {
    // Find where base ends in current query
    if !current_query.starts_with(base_query) {
        // Base is not a prefix of current query (shouldn't happen, but handle gracefully)
        return String::new();
    }

    // Find where the trigger char is in before_cursor
    // Middle should be: everything after base, up to but not including trigger char
    // Examples:
    //   Query: ".services | .ca", partial: "ca", base: ".services"
    //   → trigger is the dot at position 11
    //   → middle = query[9..11] = " | " (with trailing space, no dot)
    let trigger_pos_in_before_cursor = if partial.is_empty() {
        // Just typed trigger char - it's the last char
        before_cursor.len().saturating_sub(1)
    } else {
        // Partial being typed - trigger is one char before partial
        before_cursor.len().saturating_sub(partial.len() + 1)
    };

    #[cfg(debug_assertions)]
    debug!(
        "extract_middle_query: current_query='{}' before_cursor='{}' partial='{}' trigger_pos={} base_len={}",
        current_query,
        before_cursor,
        partial,
        trigger_pos_in_before_cursor,
        base_query.len()
    );

    // Middle is everything from end of base to (but not including) trigger
    let base_len = base_query.len();
    if trigger_pos_in_before_cursor <= base_len {
        // Trigger at or before base ends - no middle
        return String::new();
    }

    // Extract middle - preserve all whitespace as it may be significant
    // (e.g., "then " needs the space before the field access)
    let middle = current_query[base_len..trigger_pos_in_before_cursor].to_string();

    #[cfg(debug_assertions)]
    debug!("extract_middle_query: extracted_middle='{}'", middle);

    middle
}

/// Move cursor to a specific column position
pub fn move_cursor_to_column(textarea: &mut TextArea<'_>, target_col: usize) {
    let current_col = textarea.cursor().1;

    match target_col.cmp(&current_col) {
        std::cmp::Ordering::Less => {
            // Move backward
            for _ in 0..(current_col - target_col) {
                textarea.move_cursor(CursorMove::Back);
            }
        }
        std::cmp::Ordering::Greater => {
            // Move forward
            for _ in 0..(target_col - current_col) {
                textarea.move_cursor(CursorMove::Forward);
            }
        }
        std::cmp::Ordering::Equal => {
            // Already at target position
        }
    }
}

/// Execute query and update results
pub fn execute_query_and_update(textarea: &TextArea<'_>, query_state: &mut QueryState) {
    let query_text = textarea.lines()[0].clone();
    query_state.execute(&query_text);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::autocomplete::autocomplete_state::{Suggestion, SuggestionType};
    use crate::autocomplete::jq_functions::JQ_FUNCTION_METADATA;
    use proptest::prelude::*;
    use tui_textarea::TextArea;

    // ============================================================================
    // Property-Based Tests for Insertion Behavior
    // ============================================================================

    // Helper function to get functions requiring arguments
    fn get_functions_requiring_args() -> Vec<&'static crate::autocomplete::jq_functions::JqFunction>
    {
        JQ_FUNCTION_METADATA
            .iter()
            .filter(|f| f.needs_parens)
            .collect()
    }

    // Helper function to get functions not requiring arguments
    fn get_functions_not_requiring_args()
    -> Vec<&'static crate::autocomplete::jq_functions::JqFunction> {
        JQ_FUNCTION_METADATA
            .iter()
            .filter(|f| !f.needs_parens)
            .collect()
    }

    // Helper to create a test environment for insertion
    fn setup_insertion_test(initial_query: &str) -> (TextArea<'static>, crate::query::QueryState) {
        let mut textarea = TextArea::default();
        textarea.insert_str(initial_query);
        let query_state = crate::query::QueryState::new(r#"{"test": true}"#.to_string());
        (textarea, query_state)
    }

    // **Feature: enhanced-autocomplete, Property 1: Functions requiring arguments get parenthesis appended**
    // *For any* jq function marked with `needs_parens = true`, when that function is inserted
    // via Tab completion, the resulting query text SHALL end with the function name followed
    // by an opening parenthesis `(`.
    // **Validates: Requirements 1.1**
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn prop_functions_requiring_args_get_parenthesis(index in 0usize..100) {
            let funcs = get_functions_requiring_args();
            if funcs.is_empty() {
                return Ok(());
            }

            let func = funcs[index % funcs.len()];

            // Create a suggestion with needs_parens = true
            let suggestion = Suggestion::new(func.name, SuggestionType::Function)
                .with_needs_parens(true)
                .with_signature(func.signature);

            // Set up test environment with a partial query that would trigger function context
            // e.g., typing "sel" should complete to "select("
            let partial = &func.name[..func.name.len().min(3)];
            let (mut textarea, mut query_state) = setup_insertion_test(partial);

            // Insert the suggestion
            insert_suggestion(&mut textarea, &mut query_state, &suggestion);

            // Verify the result ends with function name followed by (
            let result = textarea.lines()[0].clone();
            let expected_suffix = format!("{}(", func.name);

            prop_assert!(
                result.ends_with(&expected_suffix),
                "Function '{}' with needs_parens=true should result in '{}' but got '{}'",
                func.name,
                expected_suffix,
                result
            );
        }
    }

    // **Feature: enhanced-autocomplete, Property 2: Functions not requiring arguments get no parenthesis**
    // *For any* jq function marked with `needs_parens = false`, when that function is inserted
    // via Tab completion, the resulting query text SHALL contain only the function name
    // without any trailing parenthesis.
    // **Validates: Requirements 1.2**
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn prop_functions_not_requiring_args_get_no_parenthesis(index in 0usize..100) {
            let funcs = get_functions_not_requiring_args();
            if funcs.is_empty() {
                return Ok(());
            }

            let func = funcs[index % funcs.len()];

            // Create a suggestion with needs_parens = false
            let suggestion = Suggestion::new(func.name, SuggestionType::Function)
                .with_needs_parens(false)
                .with_signature(func.signature);

            // Set up test environment with a partial query
            let partial = &func.name[..func.name.len().min(3)];
            let (mut textarea, mut query_state) = setup_insertion_test(partial);

            // Insert the suggestion
            insert_suggestion(&mut textarea, &mut query_state, &suggestion);

            // Verify the result ends with function name (no parenthesis)
            let result = textarea.lines()[0].clone();

            prop_assert!(
                result.ends_with(func.name),
                "Function '{}' with needs_parens=false should end with '{}' but got '{}'",
                func.name,
                func.name,
                result
            );

            // Also verify it does NOT end with (
            prop_assert!(
                !result.ends_with(&format!("{}(", func.name)),
                "Function '{}' with needs_parens=false should NOT have '(' appended, but got '{}'",
                func.name,
                result
            );
        }
    }

    // **Feature: enhanced-autocomplete, Property 3: Cursor positioned after parenthesis for argument functions**
    // *For any* jq function marked with `needs_parens = true`, after Tab insertion, the cursor
    // position SHALL equal the length of the inserted text (function name + opening parenthesis).
    // **Validates: Requirements 1.3**
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn prop_cursor_positioned_after_parenthesis(index in 0usize..100) {
            let funcs = get_functions_requiring_args();
            if funcs.is_empty() {
                return Ok(());
            }

            let func = funcs[index % funcs.len()];

            // Create a suggestion with needs_parens = true
            let suggestion = Suggestion::new(func.name, SuggestionType::Function)
                .with_needs_parens(true)
                .with_signature(func.signature);

            // Set up test environment with a partial query
            let partial = &func.name[..func.name.len().min(3)];
            let (mut textarea, mut query_state) = setup_insertion_test(partial);

            // Insert the suggestion
            insert_suggestion(&mut textarea, &mut query_state, &suggestion);

            // Verify cursor position is at the end of the inserted text
            let result = textarea.lines()[0].clone();
            let cursor_col = textarea.cursor().1;
            let expected_cursor_pos = result.len();

            prop_assert_eq!(
                cursor_col,
                expected_cursor_pos,
                "Cursor should be at position {} (end of '{}') but was at {}",
                expected_cursor_pos,
                result,
                cursor_col
            );
        }
    }

    // **Feature: object-key-autocomplete, Property 5: ObjectKeyContext insertion replaces partial correctly**
    // *For any* ObjectKeyContext suggestion accepted via Tab, the resulting query SHALL contain
    // the suggestion text at the position where the partial was, with no duplicate characters.
    // **Validates: Requirements 1.5**
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn prop_object_key_context_insertion_replaces_partial(
            has_prefix in prop::bool::ANY,          // Whether to include existing key-value pairs
            prefix_key in "[a-z]{2,6}",             // Key name for prefix (if used)
            partial in "[a-z]{1,4}",                // Partial being typed
            suffix in "[a-z]{1,6}",                 // Suffix to append to partial to form suggestion
        ) {
            // Build the suggestion by appending suffix to partial (ensures suggestion starts with partial)
            let suggestion = format!("{}{}", partial, suffix);

            // Build initial query: `{` or `{key: .key, ` followed by partial
            let prefix = if has_prefix {
                format!("{{{}: .{}, ", prefix_key, prefix_key)
            } else {
                "{".to_string()
            };
            let initial_query = format!("{}{}", prefix, partial);
            let (mut textarea, mut query_state) = setup_insertion_test(&initial_query);

            // Create a field suggestion (ObjectKeyContext suggestions are field names without dots)
            let suggestion_obj = Suggestion::new(&suggestion, SuggestionType::Field);

            // Insert the suggestion
            insert_suggestion(&mut textarea, &mut query_state, &suggestion_obj);

            // Get the result
            let result = textarea.lines()[0].clone();

            // Verify: the result should contain the suggestion at the right position
            // The partial should be replaced by the suggestion, not duplicated
            let expected = format!("{}{}", prefix, suggestion);
            prop_assert_eq!(
                result.clone(),
                expected.clone(),
                "ObjectKeyContext insertion should replace partial '{}' with suggestion '{}'. Initial: '{}', Expected: '{}', Got: '{}'",
                partial,
                suggestion,
                initial_query,
                expected,
                result
            );

            // Verify: cursor should be positioned after the inserted suggestion
            let cursor_col = textarea.cursor().1;
            let expected_cursor_pos = expected.len();
            prop_assert_eq!(
                cursor_col,
                expected_cursor_pos,
                "Cursor should be at position {} (end of '{}') but was at {}",
                expected_cursor_pos,
                result,
                cursor_col
            );
        }
    }

    // ============================================================================
    // Middle Query Extraction Tests
    // ============================================================================

    #[test]
    fn test_extract_middle_query_simple_path() {
        // Simple path: no middle
        let result = extract_middle_query(".services.ca", ".services", ".services.ca", "ca");
        assert_eq!(result, "", "Simple path should have empty middle");
    }

    #[test]
    fn test_extract_middle_query_after_pipe() {
        // After pipe with identity - preserves trailing space
        let result = extract_middle_query(".services | .ca", ".services", ".services | .ca", "ca");
        assert_eq!(
            result, " | ",
            "Middle: pipe with trailing space (before dot)"
        );
    }

    #[test]
    fn test_extract_middle_query_with_if_then() {
        // Complex: if/then between base and current field - preserves trailing space
        let query = ".services | if has(\"x\") then .ca";
        let before_cursor = query;
        let result = extract_middle_query(query, ".services", before_cursor, "ca");
        assert_eq!(
            result, " | if has(\"x\") then ",
            "Middle with trailing space (important for 'then ')"
        );
    }

    #[test]
    fn test_extract_middle_query_with_select() {
        // With select function - preserves trailing space
        let query = ".items | select(.active) | .na";
        let result = extract_middle_query(query, ".items", query, "na");
        assert_eq!(
            result, " | select(.active) | ",
            "Middle: includes pipe with trailing space"
        );
    }

    #[test]
    fn test_extract_middle_query_no_partial() {
        // Just typed dot, no partial yet - preserves trailing space
        let result = extract_middle_query(".services | .", ".services", ".services | .", "");
        assert_eq!(
            result, " | ",
            "Middle with trailing space before trigger dot"
        );
    }

    #[test]
    fn test_extract_middle_query_base_not_prefix() {
        // Edge case: base is not prefix of current query (shouldn't happen)
        let result = extract_middle_query(".items.ca", ".services", ".items.ca", "ca");
        assert_eq!(result, "", "Should return empty if base not a prefix");
    }

    #[test]
    fn test_extract_middle_query_nested_pipes() {
        // Multiple pipes and functions - preserves trailing space
        let query = ".a | .b | map(.c) | .d";
        let result = extract_middle_query(query, ".a", query, "d");
        assert_eq!(result, " | .b | map(.c) | ", "Middle with trailing space");
    }

    // ============================================================================
    // App Integration Tests for Autocomplete Insertion
    // ============================================================================
    // These tests verify the full autocomplete insertion flow through the App struct

    use crate::query::ResultType;
    use crate::test_utils::test_helpers::test_app;

    /// Helper to create a test suggestion from text (for backward compatibility with existing tests)
    fn test_suggestion(text: &str) -> Suggestion {
        Suggestion::new(text, SuggestionType::Field)
    }

    #[test]
    fn test_array_suggestion_appends_to_path() {
        // When accepting [].field suggestion for .services, should produce .services[].field
        let json = r#"{"services": [{"name": "alice"}, {"name": "bob"}, {"name": "charlie"}]}"#;
        let mut app = test_app(json);

        // Step 1: Execute ".services" to cache base
        app.input.textarea.insert_str(".services");
        app.query.execute(".services");

        // Validate cached state after ".services"
        assert_eq!(
            app.query.base_query_for_suggestions,
            Some(".services".to_string()),
            "base_query should be '.services'"
        );
        assert_eq!(
            app.query.base_type_for_suggestions,
            Some(ResultType::ArrayOfObjects),
            "base_type should be ArrayOfObjects"
        );

        // Step 2: Accept autocomplete suggestion "[].name" (no leading dot since after NoOp)
        insert_suggestion_from_app(&mut app, &test_suggestion("[].name"));

        // Should produce .services[].name (append, not replace)
        assert_eq!(app.input.query(), ".services[].name");

        // CRITICAL: Verify the query EXECUTES correctly and returns ALL array elements
        let result = app.query.result.as_ref().unwrap();
        assert!(result.contains("alice"), "Should contain first element");
        assert!(result.contains("bob"), "Should contain second element");
        assert!(result.contains("charlie"), "Should contain third element");

        // Verify it does NOT just return nulls or single value
        let line_count = result.lines().count();
        assert!(
            line_count >= 3,
            "Should return at least 3 lines for 3 array elements"
        );
    }

    #[test]
    fn test_simple_path_continuation_with_dot() {
        // Test simple path continuation: .object.field
        // This is the bug: .services[0].deploymentConfiguration.alarms becomes deploymentConfigurationalarms
        let json = r#"{"user": {"name": "Alice", "age": 30, "address": {"city": "NYC"}}}"#;
        let mut app = test_app(json);

        // Step 1: Execute base query
        app.input.textarea.insert_str(".user");
        app.query.execute(".user");

        // Validate cached state
        assert_eq!(
            app.query.base_query_for_suggestions,
            Some(".user".to_string())
        );
        assert_eq!(
            app.query.base_type_for_suggestions,
            Some(ResultType::Object)
        );

        // Step 2: Type ".na" (partial field access)
        app.input.textarea.insert_str(".na");

        // Step 3: Accept suggestion "name" (no leading dot since continuing path)
        insert_suggestion_from_app(&mut app, &test_suggestion("name"));

        // Should produce: .user.name
        // NOT: .username (missing dot)
        assert_eq!(app.input.query(), ".user.name");

        // Verify execution
        let result = app.query.result.as_ref().unwrap();
        assert!(result.contains("Alice"));
    }

    #[test]
    fn test_array_suggestion_replaces_partial_field() {
        // When user types partial field after array name, accepting [] suggestion should replace partial
        let json = r#"{"services": [{"serviceArn": "arn1"}, {"serviceArn": "arn2"}, {"serviceArn": "arn3"}]}"#;
        let mut app = test_app(json);

        // Step 1: Execute ".services" to cache base
        app.input.textarea.insert_str(".services");
        app.query.execute(".services");

        // Validate cached state
        assert_eq!(
            app.query.base_query_for_suggestions,
            Some(".services".to_string())
        );
        assert_eq!(
            app.query.base_type_for_suggestions,
            Some(ResultType::ArrayOfObjects)
        );

        // Step 2: Type ".s" (partial)
        app.input.textarea.insert_char('.');
        app.input.textarea.insert_char('s');

        // Step 3: Accept autocomplete suggestion "[].serviceArn"
        insert_suggestion_from_app(&mut app, &test_suggestion("[].serviceArn"));

        // Should produce .services[].serviceArn (replace ".s" with "[].serviceArn")
        assert_eq!(app.input.query(), ".services[].serviceArn");

        // CRITICAL: Verify execution returns ALL serviceArns
        let result = app.query.result.as_ref().unwrap();

        assert!(result.contains("arn1"), "Should contain first serviceArn");
        assert!(result.contains("arn2"), "Should contain second serviceArn");
        assert!(result.contains("arn3"), "Should contain third serviceArn");

        // Should NOT have nulls (would indicate query failed to iterate array)
        let null_count = result.matches("null").count();
        assert_eq!(
            null_count, 0,
            "Should not have any null values - query should iterate all array elements"
        );
    }

    #[test]
    fn test_array_suggestion_replaces_trailing_dot() {
        // When user types ".services." (trailing dot, no partial), array suggestion should replace the dot
        let json = r#"{"services": [{"deploymentConfiguration": {"x": 1}}, {"deploymentConfiguration": {"x": 2}}]}"#;
        let mut app = test_app(json);

        // Step 1: Execute ".services" to cache base query and type
        app.input.textarea.insert_str(".services");
        app.query.execute(".services");

        // Validate cached state
        assert_eq!(
            app.query.base_query_for_suggestions,
            Some(".services".to_string()),
            "base_query should be '.services'"
        );
        assert_eq!(
            app.query.base_type_for_suggestions,
            Some(ResultType::ArrayOfObjects),
            "base_type should be ArrayOfObjects"
        );

        // Step 2: Type a dot (syntax error, doesn't update base)
        app.input.textarea.insert_char('.');

        // Step 3: Accept autocomplete suggestion "[].deploymentConfiguration"
        insert_suggestion_from_app(&mut app, &test_suggestion("[].deploymentConfiguration"));

        // Should produce .services[].deploymentConfiguration (NOT .services.[].deploymentConfiguration)
        assert_eq!(app.input.query(), ".services[].deploymentConfiguration");

        // Verify the query executes correctly
        let result = app.query.result.as_ref().unwrap();
        assert!(result.contains("x"));
        assert!(result.contains("1"));
        assert!(result.contains("2"));
    }

    #[test]
    fn test_nested_array_suggestion_replaces_trailing_dot() {
        // Test deeply nested arrays: .services[].capacityProviderStrategy[].
        let json = r#"{"services": [{"capacityProviderStrategy": [{"base": 0, "weight": 1}]}]}"#;
        let mut app = test_app(json);

        // Step 1: Execute base query to cache state
        app.input
            .textarea
            .insert_str(".services[].capacityProviderStrategy[]");
        app.query.execute(".services[].capacityProviderStrategy[]");

        // Validate cached state
        assert_eq!(
            app.query.base_query_for_suggestions,
            Some(".services[].capacityProviderStrategy[]".to_string())
        );
        // With only 1 service, this returns a single object, not destructured
        assert_eq!(
            app.query.base_type_for_suggestions,
            Some(ResultType::Object)
        );

        // Step 2: Type trailing dot
        app.input.textarea.insert_char('.');

        // Step 3: Accept autocomplete suggestion "base"
        // Note: suggestion is "base" (no prefix) since Object after CloseBracket
        insert_suggestion_from_app(&mut app, &test_suggestion("base"));

        // Should produce .services[].capacityProviderStrategy[].base
        assert_eq!(
            app.input.query(),
            ".services[].capacityProviderStrategy[].base"
        );

        // Verify the query executes and returns the base values
        let result = app.query.result.as_ref().unwrap();
        assert!(result.contains("0"));
    }

    #[test]
    fn test_array_suggestion_after_pipe() {
        // After pipe, array suggestions should include leading dot
        let json = r#"{"services": [{"name": "svc1"}]}"#;
        let mut app = test_app(json);

        // Step 1: Execute base query
        app.input.textarea.insert_str(".services");
        app.query.execute(".services");

        // Validate cached state
        assert_eq!(
            app.query.base_query_for_suggestions,
            Some(".services".to_string())
        );
        assert_eq!(
            app.query.base_type_for_suggestions,
            Some(ResultType::ArrayOfObjects)
        );

        // Step 2: Type " | ."
        app.input.textarea.insert_str(" | .");

        // Step 3: Accept autocomplete suggestion ".[].name" (WITH leading dot after pipe)
        insert_suggestion_from_app(&mut app, &test_suggestion(".[].name"));

        // Should produce .services | .[].name (NOT .services | . | .[].name)
        assert_eq!(app.input.query(), ".services | .[].name");

        // Verify execution
        let result = app.query.result.as_ref().unwrap();
        assert!(result.contains("svc1"));
    }

    #[test]
    fn test_array_suggestion_after_pipe_exact_user_flow() {
        // Replicate exact user flow: type partial, select, then pipe
        let json = r#"{"services": [{"capacityProviderStrategy": [{"base": 0}]}]}"#;
        let mut app = test_app(json);

        // Step 1: Type ".ser" (partial)
        app.input.textarea.insert_str(".ser");
        // Note: .ser returns null, base stays at "."

        // Step 2: Select ".services" from autocomplete
        // In real app, user would Tab here with suggestion ".services"
        insert_suggestion_from_app(&mut app, &test_suggestion(".services"));

        // Validate: should now be ".services"
        assert_eq!(app.input.query(), ".services");

        // Step 3: Verify base is now cached after successful execution
        assert_eq!(
            app.query.base_query_for_suggestions,
            Some(".services".to_string()),
            "base should be '.services' after insertion executed it"
        );
        assert_eq!(
            app.query.base_type_for_suggestions,
            Some(ResultType::ArrayOfObjects)
        );

        // Step 4: Type " | ."
        app.input.textarea.insert_str(" | .");

        // Step 5: Select ".[].capacityProviderStrategy"
        insert_suggestion_from_app(&mut app, &test_suggestion(".[].capacityProviderStrategy"));

        // Should produce: .services | .[].capacityProviderStrategy
        // NOT: .services | . | .[].capacityProviderStrategy
        assert_eq!(
            app.input.query(),
            ".services | .[].capacityProviderStrategy"
        );
    }

    #[test]
    fn test_pipe_after_typing_space() {
        // Test typing space then pipe character by character
        let json = r#"{"services": [{"name": "svc1"}]}"#;
        let mut app = test_app(json);

        // Step 1: Type and execute ".services"
        app.input.textarea.insert_str(".services");
        app.query.execute(".services");

        assert_eq!(
            app.query.base_query_for_suggestions,
            Some(".services".to_string())
        );

        // Step 2: Type space (executes ".services ")
        app.input.textarea.insert_char(' ');
        app.query.execute(".services ");

        // Step 3: Type | (executes ".services |" - syntax error, base stays at ".services")
        app.input.textarea.insert_char('|');
        app.query.execute(".services |");

        // Step 4: Type space then dot
        app.input.textarea.insert_str(" .");

        // Step 5: Accept suggestion
        insert_suggestion_from_app(&mut app, &test_suggestion(".[].name"));

        // Should be: base + " | " + suggestion
        // Base is trimmed, so: ".services" + " | " + ".[].name" = ".services | .[].name" ✅
        assert_eq!(app.input.query(), ".services | .[].name");
    }

    #[test]
    fn test_suggestions_persist_when_typing_partial_after_array() {
        // Critical: When typing partial field after [], suggestions should persist
        let json = r#"{"services": [{"capacityProviderStrategy": [{"base": 0, "weight": 1, "capacityProvider": "x"}]}]}"#;
        let mut app = test_app(json);

        // Step 1: Type the full path up to the last array
        app.input
            .textarea
            .insert_str(".services[].capacityProviderStrategy[]");
        app.query.execute(".services[].capacityProviderStrategy[]");
        app.update_autocomplete();

        // Cache should have the array element objects with fields: base, weight, capacityProvider
        assert!(app.query.last_successful_result_unformatted.is_some());
        let cached = app.query.last_successful_result_unformatted.clone();

        // Step 2: Type a dot - should still have cached result
        app.input.textarea.insert_char('.');
        // Query is now ".services[].capacityProviderStrategy[]." which is syntax error
        app.query.execute(".services[].capacityProviderStrategy[].");

        // Cache should NOT be cleared (syntax error doesn't update cache)
        assert_eq!(app.query.last_successful_result_unformatted, cached);

        // Step 3: Type a partial "b" - query returns multiple nulls
        app.input.textarea.insert_char('b');
        // Query is now ".services[].capacityProviderStrategy[].b" which returns multiple nulls
        app.query
            .execute(".services[].capacityProviderStrategy[].b");

        // CRITICAL: Cache should STILL not be cleared (multiple nulls shouldn't overwrite)
        assert_eq!(app.query.last_successful_result_unformatted, cached);

        // Step 4: Update autocomplete - should still show suggestions based on cached result
        app.update_autocomplete();

        // Should have suggestions for the cached object fields
        let suggestions = app.autocomplete.suggestions();
        assert!(
            !suggestions.is_empty(),
            "Suggestions should persist when typing partial that returns null"
        );

        // Should have "base" suggestion (filtered by partial "b")
        assert!(
            suggestions.iter().any(|s| s.text.contains("base")),
            "Should suggest 'base' field when filtering by 'b'"
        );
    }

    #[test]
    fn test_suggestions_persist_with_optional_chaining_and_partial() {
        // Critical: When typing partial after []?, suggestions should persist
        // Realistic scenario: some services have capacityProviderStrategy, some don't
        let json = r#"{
            "services": [
                {
                    "serviceName": "service1",
                    "capacityProviderStrategy": [
                        {"base": 0, "weight": 1, "capacityProvider": "FARGATE"},
                        {"base": 0, "weight": 2, "capacityProvider": "FARGATE_SPOT"}
                    ]
                },
                {
                    "serviceName": "service2"
                },
                {
                    "serviceName": "service3",
                    "capacityProviderStrategy": [
                        {"base": 1, "weight": 3, "capacityProvider": "EC2"}
                    ]
                }
            ]
        }"#;
        let mut app = test_app(json);

        // Step 1: Execute query with optional chaining up to the array
        app.input
            .textarea
            .insert_str(".services[].capacityProviderStrategy[]?");
        app.query.execute(".services[].capacityProviderStrategy[]?");

        // This should return the object with base, weight, capacityProvider fields
        let cached_before_partial = app.query.last_successful_result_unformatted.clone();
        assert!(cached_before_partial.is_some());
        assert!(cached_before_partial.as_ref().unwrap().contains("base"));

        // Step 2: Type a dot
        app.input.textarea.insert_char('.');
        app.query
            .execute(".services[].capacityProviderStrategy[]?.");
        // Syntax error - cache should remain
        assert_eq!(
            app.query.last_successful_result_unformatted,
            cached_before_partial
        );

        // Step 3: Type partial "b"
        app.input.textarea.insert_char('b');
        app.query
            .execute(".services[].capacityProviderStrategy[]?.b");

        // This returns single "null" (not multiple) due to optional chaining
        // Cache should NOT be updated
        assert_eq!(
            app.query.last_successful_result_unformatted, cached_before_partial,
            "Cache should not be overwritten by null result from partial field"
        );

        // Step 4: Update autocomplete
        app.update_autocomplete();

        // Should have suggestions based on the cached object
        let suggestions = app.autocomplete.suggestions();
        assert!(
            !suggestions.is_empty(),
            "Suggestions should persist when typing partial after []?"
        );

        // Should suggest "base" (filtered by partial "b")
        assert!(
            suggestions.iter().any(|s| s.text.contains("base")),
            "Should suggest 'base' field when filtering by 'b' after []?"
        );
    }

    #[test]
    fn test_jq_keyword_autocomplete_no_dot_prefix() {
        // Test that jq keywords like "then", "else", "end" don't get a dot prefix
        let json = r#"{"services": [{"capacityProviderStrategy": [{"base": 0}]}]}"#;
        let mut app = test_app(json);

        // Step 1: Type the beginning of an if statement
        app.input
            .textarea
            .insert_str(".services | if has(\"capacityProviderStrategy\")");
        app.query
            .execute(".services | if has(\"capacityProviderStrategy\")");

        // Step 2: Type partial "the" to trigger autocomplete for "then"
        app.input.textarea.insert_str(" the");

        // Step 3: Accept "then" from autocomplete
        // This should NOT add a dot before "then"
        insert_suggestion_from_app(&mut app, &test_suggestion("then"));

        // Should produce: .services | if has("capacityProviderStrategy") then
        // NOT: .services | if has("capacityProviderStrategy") .then
        assert_eq!(
            app.input.query(),
            ".services | if has(\"capacityProviderStrategy\") then"
        );

        // Verify no extra dot was added
        assert!(
            !app.input.query().contains(" .then"),
            "Should not have dot before 'then' keyword"
        );
    }

    #[test]
    fn test_jq_keyword_else_autocomplete() {
        // Test "else" keyword autocomplete
        let json = r#"{"value": 42}"#;
        let mut app = test_app(json);

        // Type an if-then statement
        app.input
            .textarea
            .insert_str("if .value > 10 then \"high\" el");

        // Accept "else" from autocomplete
        insert_suggestion_from_app(&mut app, &test_suggestion("else"));

        // Should produce: if .value > 10 then "high" else
        // NOT: if .value > 10 then "high" .else
        assert_eq!(app.input.query(), "if .value > 10 then \"high\" else");
        assert!(
            !app.input.query().contains(".else"),
            "Should not have dot before 'else' keyword"
        );
    }

    #[test]
    fn test_jq_keyword_end_autocomplete() {
        // Test "end" keyword autocomplete
        let json = r#"{"value": 42}"#;
        let mut app = test_app(json);

        // Type a complete if-then-else statement
        app.input
            .textarea
            .insert_str("if .value > 10 then \"high\" else \"low\" en");

        // Accept "end" from autocomplete
        insert_suggestion_from_app(&mut app, &test_suggestion("end"));

        // Should produce: if .value > 10 then "high" else "low" end
        // NOT: if .value > 10 then "high" else "low" .end
        assert_eq!(
            app.input.query(),
            "if .value > 10 then \"high\" else \"low\" end"
        );
        assert!(
            !app.input.query().contains(".end"),
            "Should not have dot before 'end' keyword"
        );
    }

    #[test]
    fn test_field_access_after_jq_keyword_preserves_space() {
        // Test that field access after "then" preserves the space
        // Bug: ".services[] | if has(\"x\") then .field" becomes "then.field" (no space)
        let json = r#"{"services": [{"capacityProviderStrategy": [{"base": 0}]}]}"#;
        let mut app = test_app(json);

        // Step 1: Execute base query
        app.input.textarea.insert_str(".services[]");
        app.query.execute(".services[]");

        // Step 2: Type if-then with field access
        app.input
            .textarea
            .insert_str(" | if has(\"capacityProviderStrategy\") then .ca");

        // Step 3: Accept field suggestion (with leading dot as it would come from get_suggestions)
        insert_suggestion_from_app(&mut app, &test_suggestion(".capacityProviderStrategy"));

        // Should produce: .services[] | if has("capacityProviderStrategy") then .capacityProviderStrategy
        // NOT: .services[] | if has("capacityProviderStrategy") thencapacityProviderStrategy
        assert_eq!(
            app.input.query(),
            ".services[] | if has(\"capacityProviderStrategy\") then .capacityProviderStrategy"
        );

        // Verify there's a space before the field name
        assert!(
            app.input.query().contains("then .capacityProviderStrategy"),
            "Should have space between 'then' and field name"
        );
        assert!(
            !app.input.query().contains("thencapacityProviderStrategy"),
            "Should NOT concatenate 'then' with field name"
        );
    }

    #[test]
    fn test_field_access_after_else_preserves_space() {
        // Test that field access after "else" preserves the space
        let json = r#"{"services": [{"name": "test"}]}"#;
        let mut app = test_app(json);

        // Execute base query
        app.input.textarea.insert_str(".services[]");
        app.query.execute(".services[]");

        // Type if-then-else with field access
        app.input
            .textarea
            .insert_str(" | if has(\"name\") then .name else .na");

        // Accept field suggestion (with leading dot as it would come from get_suggestions)
        insert_suggestion_from_app(&mut app, &test_suggestion(".name"));

        // Should have space between "else" and field
        assert!(
            app.input.query().contains("else .name"),
            "Should have space between 'else' and field name"
        );
        assert!(
            !app.input.query().contains("elsename"),
            "Should NOT concatenate 'else' with field name"
        );
    }

    #[test]
    fn test_autocomplete_inside_if_statement() {
        // Autocomplete inside complex query should only replace the local part
        let json = r#"{"services": [{"capacityProviderStrategy": [{"base": 0}]}]}"#;
        let mut app = test_app(json);

        // User types complex query with if/then
        app.input
            .textarea
            .insert_str(".services | if has(\"capacityProviderStrategy\") then .ca");

        // Execute to cache state (this will likely error due to incomplete query)
        app.query
            .execute(".services | if has(\"capacityProviderStrategy\") then .ca");

        // The issue: when Tab is pressed, entire query gets replaced with base + suggestion
        // Expected: only ".ca" should be replaced
        // Actual: entire query replaced with ".services[].capacityProviderStrategy"

        // TODO: This test documents the bug - we need smarter insertion
        // For now, this is a known limitation when using autocomplete inside complex expressions
    }

    #[test]
    fn test_root_field_suggestion() {
        // At root, typing "." and selecting field should replace "." with ".field"
        let json = r#"{"services": [{"name": "test"}], "status": "active"}"#;
        let mut app = test_app(json);

        // Validate initial state
        assert_eq!(
            app.query.base_query_for_suggestions,
            Some(".".to_string()),
            "base_query should be '.' initially"
        );
        assert_eq!(
            app.query.base_type_for_suggestions,
            Some(ResultType::Object),
            "base_type should be Object"
        );

        // User types "."
        app.input.textarea.insert_str(".");

        // Accept suggestion ".services" (with leading dot since at root after NoOp)
        insert_suggestion_from_app(&mut app, &test_suggestion(".services"));

        // Should produce ".services" NOT "..services"
        assert_eq!(app.input.query(), ".services");

        // Verify query executes correctly
        let result = app.query.result.as_ref().unwrap();
        assert!(result.contains("name"));
    }

    #[test]
    fn test_field_suggestion_replaces_from_dot() {
        // When accepting .field suggestion at root, should replace from last dot
        let json = r#"{"name": "test", "age": 30}"#;
        let mut app = test_app(json);

        // Initial state: "." was executed during App::new()
        // Validate initial state
        assert_eq!(
            app.query.base_query_for_suggestions,
            Some(".".to_string()),
            "base_query should be '.' initially"
        );
        assert_eq!(
            app.query.base_type_for_suggestions,
            Some(ResultType::Object),
            "base_type should be Object for root"
        );

        // Simulate: user typed ".na" and cursor is at end
        app.input.textarea.insert_str(".na");

        // Accept autocomplete suggestion "name" (no leading dot since after Dot)
        insert_suggestion_from_app(&mut app, &test_suggestion("name"));

        // Should produce .name (replace from the dot)
        assert_eq!(app.input.query(), ".name");
    }

    #[test]
    fn test_autocomplete_with_real_ecs_like_data() {
        // Test with data structure similar to AWS ECS services
        let json = r#"{
            "services": [
                {"serviceArn": "arn:aws:ecs:region:account:service/cluster/svc1", "serviceName": "service1"},
                {"serviceArn": "arn:aws:ecs:region:account:service/cluster/svc2", "serviceName": "service2"},
                {"serviceArn": "arn:aws:ecs:region:account:service/cluster/svc3", "serviceName": "service3"},
                {"serviceArn": "arn:aws:ecs:region:account:service/cluster/svc4", "serviceName": "service4"},
                {"serviceArn": "arn:aws:ecs:region:account:service/cluster/svc5", "serviceName": "service5"}
            ]
        }"#;
        let mut app = test_app(json);

        // Step 1: Execute ".services" to cache base
        app.input.textarea.insert_str(".services");
        app.query.execute(".services");

        // Validate cached state
        assert_eq!(
            app.query.base_query_for_suggestions,
            Some(".services".to_string()),
            "base_query should be '.services'"
        );
        assert_eq!(
            app.query.base_type_for_suggestions,
            Some(ResultType::ArrayOfObjects),
            "base_type should be ArrayOfObjects"
        );

        // Step 2: Type ".s" (partial)
        app.input.textarea.insert_str(".s");

        // Step 3: Accept "[].serviceArn" (no leading dot since after NoOp)
        insert_suggestion_from_app(&mut app, &test_suggestion("[].serviceArn"));

        let query_text = app.input.query();
        assert_eq!(query_text, ".services[].serviceArn");

        // Verify execution returns ALL 5 serviceArns
        let result = app.query.result.as_ref().unwrap();

        // Check for all service ARNs
        assert!(result.contains("svc1"));
        assert!(result.contains("svc2"));
        assert!(result.contains("svc3"));
        assert!(result.contains("svc4"));
        assert!(result.contains("svc5"));

        // Count non-null values
        let lines: Vec<&str> = result.lines().collect();
        let non_null_lines: Vec<&str> = lines
            .iter()
            .filter(|line| !line.trim().contains("null"))
            .copied()
            .collect();

        assert!(
            non_null_lines.len() >= 5,
            "Should have at least 5 non-null results, got {}",
            non_null_lines.len()
        );
    }

    // ============================================================================
    // ObjectKeyContext Insertion Unit Tests
    // ============================================================================
    // These tests verify ObjectKeyContext insertion behavior per Requirements 1.5

    #[test]
    fn test_object_key_context_insertion_simple() {
        // Test: `{na` + accept "name" → `{name`
        // This tests the basic ObjectKeyContext insertion after opening brace
        let initial_query = "{na";
        let (mut textarea, mut query_state) = setup_insertion_test(initial_query);

        // Create a field suggestion (ObjectKeyContext suggestions are field names without dots)
        let suggestion = Suggestion::new("name", SuggestionType::Field);

        // Insert the suggestion
        insert_suggestion(&mut textarea, &mut query_state, &suggestion);

        // Verify the result
        let result = textarea.lines()[0].clone();
        assert_eq!(
            result, "{name",
            "ObjectKeyContext insertion should replace 'na' with 'name'. Got: '{}'",
            result
        );

        // Verify cursor position is at the end
        let cursor_col = textarea.cursor().1;
        assert_eq!(
            cursor_col, 5,
            "Cursor should be at position 5 (end of '{{name')"
        );
    }

    #[test]
    fn test_object_key_context_insertion_after_comma() {
        // Test: `{name: .name, ag` + accept "age" → `{name: .name, age`
        // This tests ObjectKeyContext insertion after comma in object literal
        let initial_query = "{name: .name, ag";
        let (mut textarea, mut query_state) = setup_insertion_test(initial_query);

        // Create a field suggestion
        let suggestion = Suggestion::new("age", SuggestionType::Field);

        // Insert the suggestion
        insert_suggestion(&mut textarea, &mut query_state, &suggestion);

        // Verify the result
        let result = textarea.lines()[0].clone();
        assert_eq!(
            result, "{name: .name, age",
            "ObjectKeyContext insertion should replace 'ag' with 'age'. Got: '{}'",
            result
        );

        // Verify cursor position is at the end
        let cursor_col = textarea.cursor().1;
        assert_eq!(
            cursor_col, 17,
            "Cursor should be at position 17 (end of '{{name: .name, age')"
        );
    }

    #[test]
    fn test_object_key_context_insertion_with_space_after_comma() {
        // Test: `{name: .name, ag` (with space after comma) + accept "age" → `{name: .name, age`
        // This tests that spaces are preserved correctly
        let initial_query = "{name: .name, ag";
        let (mut textarea, mut query_state) = setup_insertion_test(initial_query);

        let suggestion = Suggestion::new("age", SuggestionType::Field);
        insert_suggestion(&mut textarea, &mut query_state, &suggestion);

        let result = textarea.lines()[0].clone();
        assert_eq!(result, "{name: .name, age");
    }

    #[test]
    fn test_object_key_context_insertion_nested_object() {
        // Test: `{outer: {in` + accept "inner" → `{outer: {inner`
        // This tests ObjectKeyContext in nested object
        let initial_query = "{outer: {in";
        let (mut textarea, mut query_state) = setup_insertion_test(initial_query);

        let suggestion = Suggestion::new("inner", SuggestionType::Field);
        insert_suggestion(&mut textarea, &mut query_state, &suggestion);

        let result = textarea.lines()[0].clone();
        assert_eq!(
            result, "{outer: {inner",
            "ObjectKeyContext insertion in nested object should work. Got: '{}'",
            result
        );
    }

    #[test]
    fn test_object_key_context_insertion_longer_partial() {
        // Test: `{servi` + accept "services" → `{services`
        // This tests with a longer partial
        let initial_query = "{servi";
        let (mut textarea, mut query_state) = setup_insertion_test(initial_query);

        let suggestion = Suggestion::new("services", SuggestionType::Field);
        insert_suggestion(&mut textarea, &mut query_state, &suggestion);

        let result = textarea.lines()[0].clone();
        assert_eq!(
            result, "{services",
            "ObjectKeyContext insertion with longer partial should work. Got: '{}'",
            result
        );
    }

    #[test]
    fn test_object_key_context_insertion_single_char_partial() {
        // Test: `{n` + accept "name" → `{name`
        // This tests with a single character partial
        let initial_query = "{n";
        let (mut textarea, mut query_state) = setup_insertion_test(initial_query);

        let suggestion = Suggestion::new("name", SuggestionType::Field);
        insert_suggestion(&mut textarea, &mut query_state, &suggestion);

        let result = textarea.lines()[0].clone();
        assert_eq!(
            result, "{name",
            "ObjectKeyContext insertion with single char partial should work. Got: '{}'",
            result
        );
    }
}
