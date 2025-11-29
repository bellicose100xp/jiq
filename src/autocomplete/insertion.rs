//! Autocomplete suggestion insertion logic
//!
//! This module handles inserting autocomplete suggestions into the query,
//! managing cursor positioning, and executing the updated query.

use tui_textarea::{CursorMove, TextArea};

use crate::autocomplete::state::Suggestion;
use crate::autocomplete::{analyze_context, find_char_before_field_access, SuggestionContext};
use crate::query::{CharType, QueryState};

#[cfg(debug_assertions)]
use log::debug;

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
    let (context, partial) = analyze_context(before_cursor);

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
            CharType::PipeOperator => {
                #[cfg(debug_assertions)]
                debug!("formula: PipeOperator -> base + middle + ' ' + suggestion");

                // Formula: base + middle + " " + suggestion
                // Trim trailing space from middle to avoid double spaces
                let trimmed_middle = middle_query.trim_end();
                format!("{}{} {}", adjusted_base, trimmed_middle, adjusted_suggestion)
            }
            CharType::Semicolon => {
                #[cfg(debug_assertions)]
                debug!("formula: Semicolon -> base + middle + ' ' + suggestion");

                // Formula: base + middle + " " + suggestion
                // Trim trailing space from middle to avoid double spaces
                let trimmed_middle = middle_query.trim_end();
                format!("{}{} {}", adjusted_base, trimmed_middle, adjusted_suggestion)
            }
            CharType::Comma => {
                #[cfg(debug_assertions)]
                debug!("formula: Comma -> base + middle + ' ' + suggestion");

                // Formula: base + middle + " " + suggestion
                // Trim trailing space from middle to avoid double spaces
                let trimmed_middle = middle_query.trim_end();
                format!("{}{} {}", adjusted_base, trimmed_middle, adjusted_suggestion)
            }
            CharType::Colon => {
                #[cfg(debug_assertions)]
                debug!("formula: Colon -> base + middle + ' ' + suggestion");

                // Formula: base + middle + " " + suggestion
                // Trim trailing space from middle to avoid double spaces
                let trimmed_middle = middle_query.trim_end();
                format!("{}{} {}", adjusted_base, trimmed_middle, adjusted_suggestion)
            }
            CharType::OpenParen => {
                #[cfg(debug_assertions)]
                debug!("formula: OpenParen -> base + middle + suggestion (paren already in middle)");

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
        current_query, before_cursor, partial, trigger_pos_in_before_cursor, base_query.len()
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
    use crate::autocomplete::jq_functions::JQ_FUNCTION_METADATA;
    use crate::autocomplete::state::{Suggestion, SuggestionType};
    use proptest::prelude::*;
    use tui_textarea::TextArea;

    // ============================================================================
    // Property-Based Tests for Insertion Behavior
    // ============================================================================

    // Helper function to get functions requiring arguments
    fn get_functions_requiring_args() -> Vec<&'static crate::autocomplete::jq_functions::JqFunction> {
        JQ_FUNCTION_METADATA
            .iter()
            .filter(|f| f.needs_parens)
            .collect()
    }

    // Helper function to get functions not requiring arguments
    fn get_functions_not_requiring_args() -> Vec<&'static crate::autocomplete::jq_functions::JqFunction> {
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
        assert_eq!(result, " | ", "Middle: pipe with trailing space (before dot)");
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
}
