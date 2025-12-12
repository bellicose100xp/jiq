use crate::query::ResultType;
use super::brace_tracker::BraceTracker;
use super::jq_functions::filter_builtins;
use super::result_analyzer::ResultAnalyzer;
use super::autocomplete_state::Suggestion;

/// Context information about what's being typed
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(clippy::enum_variant_names)] // Context suffix is intentional for clarity
pub enum SuggestionContext {
    /// At start or after pipe/operator - suggest functions and patterns
    FunctionContext,
    /// After a dot - suggest field names
    FieldContext,
    /// Inside object literal `{}` where an object key name is expected.
    /// This context is triggered after `{` or `,` inside an object literal
    /// when the user is typing a partial identifier (not starting with `.`).
    /// Suggestions in this context do NOT have a leading dot, enabling
    /// efficient construction of object literals like `{name: .name}`.
    ObjectKeyContext,
}

/// Analyze query text and cursor position to determine what to suggest
pub fn get_suggestions(
    query: &str,
    cursor_pos: usize,
    result: Option<&str>,
    result_type: Option<ResultType>,
    brace_tracker: &BraceTracker,
) -> Vec<Suggestion> {
    // Get the text before cursor
    let before_cursor = &query[..cursor_pos.min(query.len())];

    // Determine context and get the partial word being typed
    let (context, partial) = analyze_context(before_cursor, brace_tracker);

    match context {
        SuggestionContext::FieldContext => {
            // Determine trigger context to decide if suggestions need leading dot
            let char_before_dot = find_char_before_field_access(before_cursor, &partial);

            // Check if there's a dot immediately before the partial (or at cursor if no partial)
            // This helps distinguish: ".services.ca" (continuation) vs "then .ca" (new path)
            let dot_pos = if partial.is_empty() {
                before_cursor.len().saturating_sub(1)
            } else {
                before_cursor.len().saturating_sub(partial.len() + 1)
            };
            let has_immediate_dot = dot_pos < before_cursor.len() 
                && before_cursor.chars().nth(dot_pos) == Some('.');
            
            // Check if there's whitespace between char_before and the dot
            let has_whitespace_before_dot = if dot_pos > 0 && has_immediate_dot {
                before_cursor[..dot_pos].chars().rev().take_while(|c| c.is_whitespace()).count() > 0
            } else {
                false
            };

            // Include leading dot when:
            // 1. After operators (starting new path): |, ;, ,, :, (, [, {
            // 2. At start of query (None)
            // 3. After whitespace + dot (like "then .field") - new path after keyword
            let needs_leading_dot = matches!(
                char_before_dot,
                Some('|') | Some(';') | Some(',') | Some(':') | Some('(') | Some('[') | Some('{') | None
            ) || has_whitespace_before_dot;

            // Generate type-aware suggestions (no mutation needed!)
            let suggestions = if let (Some(result), Some(typ)) = (result, result_type) {
                ResultAnalyzer::analyze_result(result, typ, needs_leading_dot)
            } else {
                Vec::new()
            };

            // Filter suggestions by partial match
            if partial.is_empty() {
                suggestions
            } else {
                suggestions
                    .into_iter()
                    .filter(|s| s.text.to_lowercase().contains(&partial.to_lowercase()))
                    .collect()
            }
        }
        SuggestionContext::FunctionContext => {
            // Suggest jq functions/patterns/operators
            if partial.is_empty() {
                Vec::new()
            } else {
                filter_builtins(&partial)
            }
        }
        SuggestionContext::ObjectKeyContext => {
            // Object key context: suggest field names without leading dot
            // Per requirement 1.3: return empty if partial is empty
            if partial.is_empty() {
                return Vec::new();
            }

            // Generate suggestions without leading dot (needs_leading_dot = false)
            let suggestions = if let (Some(result), Some(typ)) = (result, result_type) {
                ResultAnalyzer::analyze_result(result, typ, false)
            } else {
                Vec::new()
            };

            // Filter suggestions by partial match (same as FieldContext)
            suggestions
                .into_iter()
                .filter(|s| s.text.to_lowercase().contains(&partial.to_lowercase()))
                .collect()
        }
    }
}

/// Analyze the text before cursor to determine context and partial word
pub fn analyze_context(before_cursor: &str, brace_tracker: &BraceTracker) -> (SuggestionContext, String) {
    if before_cursor.is_empty() {
        return (SuggestionContext::FunctionContext, String::new());
    }

    // Find the last "word" being typed by looking backwards
    let chars: Vec<char> = before_cursor.chars().collect();
    let mut i = chars.len();

    // Skip trailing whitespace
    while i > 0 && chars[i - 1].is_whitespace() {
        i -= 1;
    }

    if i == 0 {
        return (SuggestionContext::FunctionContext, String::new());
    }

    // Check if we're in field context (after a dot)
    if i > 0 && chars[i - 1] == '.' {
        // Just typed a dot - suggest all fields
        return (SuggestionContext::FieldContext, String::new());
    }

    // Look for the start of the current token
    let mut start = i;
    while start > 0 {
        let ch = chars[start - 1];

        // Stop at delimiters
        if is_delimiter(ch) {
            break;
        }

        start -= 1;
    }

    // Extract the partial word
    let partial: String = chars[start..i].iter().collect();

    // Check if the partial starts with a dot (field access) or question mark (optional field access)
    if let Some(stripped) = partial.strip_prefix('.') {
        // Field context - return the part after the LAST dot (for nested fields like .user.na)
        let field_partial = if let Some(last_dot_pos) = partial.rfind('.') {
            partial[last_dot_pos + 1..].to_string()
        } else {
            stripped.to_string()
        };
        return (SuggestionContext::FieldContext, field_partial);
    } else if let Some(stripped) = partial.strip_prefix('?') {
        // Optional field access like []?.field
        // If after ? there's a dot, strip it and get the field name
        let field_partial = if let Some(after_dot) = stripped.strip_prefix('.') {
            after_dot.to_string()
        } else {
            // Just ? with no dot yet
            String::new()
        };
        return (SuggestionContext::FieldContext, field_partial);
    }

    // Check what comes before the partial to determine context
    if start > 0 {
        // Look backwards to see if there's a dot or question mark before this position
        let mut j = start;
        while j > 0 && chars[j - 1].is_whitespace() {
            j -= 1;
        }

        if j > 0 {
            let char_before = chars[j - 1];
            // Field context if preceded by dot or question mark
            // Examples: .field, []?.field
            if char_before == '.' || char_before == '?' {
                return (SuggestionContext::FieldContext, partial);
            }

            // Check for ObjectKeyContext: after `{` or `,` when inside an object literal
            // Only if we have a non-empty partial (per requirement 1.3)
            if !partial.is_empty() && (char_before == '{' || char_before == ',') {
                // Use BraceTracker to verify we're actually inside an object literal
                // The position to check is the cursor position (end of before_cursor)
                if brace_tracker.is_in_object(before_cursor.len()) {
                    return (SuggestionContext::ObjectKeyContext, partial);
                }
            }
        }
    }

    // Otherwise, function context
    (SuggestionContext::FunctionContext, partial)
}

/// Find the character that precedes the field access (the dot we're typing after)
/// Returns None if at the start of the query
pub fn find_char_before_field_access(before_cursor: &str, partial: &str) -> Option<char> {
    // We need to find what's before the dot that triggered FieldContext
    // If we have a partial (like "ser" in ".services | .ser"), go back past it
    // If no partial (like in ".services | ."), we're right after the dot

    let search_end = if partial.is_empty() {
        // No partial - we just typed the dot, so look before it
        before_cursor.len().saturating_sub(1)
    } else {
        // Have partial - go back past partial and the dot
        before_cursor.len().saturating_sub(partial.len() + 1)
    };

    if search_end == 0 {
        return None; // At start of query
    }

    // Search backwards for the first non-whitespace character
    let chars: Vec<char> = before_cursor[..search_end].chars().collect();
    for i in (0..chars.len()).rev() {
        let ch = chars[i];
        if !ch.is_whitespace() {
            return Some(ch);
        }
    }

    None
}

/// Check if a character is a delimiter
fn is_delimiter(ch: char) -> bool {
    matches!(
        ch,
        '|' | ';'
            | '('
            | ')'
            | '['
            | ']'
            | '{'
            | '}'
            | ','
            | ' '
            | '\t'
            | '\n'
            | '\r'
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    /// Helper to create a BraceTracker initialized with the given query
    fn tracker_for(query: &str) -> BraceTracker {
        let mut tracker = BraceTracker::new();
        tracker.rebuild(query);
        tracker
    }

    #[test]
    fn test_empty_query() {
        let tracker = tracker_for("");
        let (ctx, partial) = analyze_context("", &tracker);
        assert_eq!(ctx, SuggestionContext::FunctionContext);
        assert_eq!(partial, "");
    }

    #[test]
    fn test_function_context() {
        let tracker = tracker_for("ma");
        let (ctx, partial) = analyze_context("ma", &tracker);
        assert_eq!(ctx, SuggestionContext::FunctionContext);
        assert_eq!(partial, "ma");

        let tracker = tracker_for("select");
        let (ctx, partial) = analyze_context("select", &tracker);
        assert_eq!(ctx, SuggestionContext::FunctionContext);
        assert_eq!(partial, "select");
    }

    #[test]
    fn test_field_context_with_dot() {
        let tracker = tracker_for(".na");
        let (ctx, partial) = analyze_context(".na", &tracker);
        assert_eq!(ctx, SuggestionContext::FieldContext);
        assert_eq!(partial, "na");

        let tracker = tracker_for(".name");
        let (ctx, partial) = analyze_context(".name", &tracker);
        assert_eq!(ctx, SuggestionContext::FieldContext);
        assert_eq!(partial, "name");
    }

    #[test]
    fn test_just_dot() {
        let tracker = tracker_for(".");
        let (ctx, partial) = analyze_context(".", &tracker);
        assert_eq!(ctx, SuggestionContext::FieldContext);
        assert_eq!(partial, "");
    }

    #[test]
    fn test_after_pipe() {
        let tracker = tracker_for(".name | ma");
        let (ctx, partial) = analyze_context(".name | ma", &tracker);
        assert_eq!(ctx, SuggestionContext::FunctionContext);
        assert_eq!(partial, "ma");
    }

    #[test]
    fn test_nested_field() {
        let tracker = tracker_for(".user.na");
        let (ctx, partial) = analyze_context(".user.na", &tracker);
        assert_eq!(ctx, SuggestionContext::FieldContext);
        assert_eq!(partial, "na");
    }

    #[test]
    fn test_array_access() {
        let tracker = tracker_for(".items[0].na");
        let (ctx, partial) = analyze_context(".items[0].na", &tracker);
        assert_eq!(ctx, SuggestionContext::FieldContext);
        assert_eq!(partial, "na");
    }

    #[test]
    fn test_in_function_call() {
        let tracker = tracker_for("map(.na");
        let (ctx, partial) = analyze_context("map(.na", &tracker);
        assert_eq!(ctx, SuggestionContext::FieldContext);
        assert_eq!(partial, "na");
    }

    // Tests for find_char_before_field_access helper
    #[test]
    fn test_char_before_field_after_pipe() {
        // `.services | .` - should find '|'
        assert_eq!(find_char_before_field_access(".services | .", ""), Some('|'));
        // `.services | .ser` - should find '|' (go back past partial)
        assert_eq!(find_char_before_field_access(".services | .ser", "ser"), Some('|'));
    }

    #[test]
    fn test_char_before_field_after_dot() {
        // `.services.` - should find 's' (last char of identifier)
        assert_eq!(find_char_before_field_access(".services.", ""), Some('s'));
        // `.services.na` - should find 's' (go back past partial and dot)
        assert_eq!(find_char_before_field_access(".services.na", "na"), Some('s'));
    }

    #[test]
    fn test_char_before_field_after_brackets() {
        // `.services[].` - should find ']'
        assert_eq!(find_char_before_field_access(".services[].", ""), Some(']'));
        // `.services[0].` - should find ']'
        assert_eq!(find_char_before_field_access(".services[0].", ""), Some(']'));
    }

    #[test]
    fn test_char_before_field_after_question() {
        // `.services?.` - should find '?'
        assert_eq!(find_char_before_field_access(".services?.", ""), Some('?'));
        // `.services?.na` - should find '?'
        assert_eq!(find_char_before_field_access(".services?.na", "na"), Some('?'));
    }

    #[test]
    fn test_char_before_field_in_constructors() {
        // `[.` - should find '['
        assert_eq!(find_char_before_field_access("[.", ""), Some('['));
        // `[.a, .` - should find ','
        assert_eq!(find_char_before_field_access("[.a, .", ""), Some(','));
        // `{name: .` - should find ':'
        assert_eq!(find_char_before_field_access("{name: .", ""), Some(':'));
        // `{.` - should find '{'
        assert_eq!(find_char_before_field_access("{.", ""), Some('{'));
    }

    #[test]
    fn test_char_before_field_in_functions() {
        // `map(.` - should find '('
        assert_eq!(find_char_before_field_access("map(.", ""), Some('('));
        // `select(.active).` - should find ')'
        assert_eq!(find_char_before_field_access("select(.active).", ""), Some(')'));
    }

    #[test]
    fn test_char_before_field_with_semicolon() {
        // `.a; .` - should find ';'
        assert_eq!(find_char_before_field_access(".a; .", ""), Some(';'));
    }

    #[test]
    fn test_char_before_field_at_start() {
        // `.` at start - should return None
        assert_eq!(find_char_before_field_access(".", ""), None);
        // `.na` at start - should return None
        assert_eq!(find_char_before_field_access(".na", "na"), None);
    }

    #[test]
    fn test_analyze_context_after_optional_array() {
        // After []?. should be FieldContext
        let query = ".services[].capacityProviderStrategy[]?.";
        let tracker = tracker_for(query);
        let (ctx, partial) = analyze_context(query, &tracker);
        assert_eq!(ctx, SuggestionContext::FieldContext);
        assert_eq!(partial, "");

        // After []?.b should be FieldContext with partial "b"
        let query = ".services[].capacityProviderStrategy[]?.b";
        let tracker = tracker_for(query);
        let (ctx, partial) = analyze_context(query, &tracker);
        assert_eq!(ctx, SuggestionContext::FieldContext);
        assert_eq!(partial, "b");
    }

    #[test]
    fn test_analyze_context_jq_keywords() {
        // jq keywords like "if", "then", "else" should be FunctionContext
        let tracker = tracker_for("if");
        let (ctx, partial) = analyze_context("if", &tracker);
        assert_eq!(ctx, SuggestionContext::FunctionContext);
        assert_eq!(partial, "if");

        let tracker = tracker_for("then");
        let (ctx, partial) = analyze_context("then", &tracker);
        assert_eq!(ctx, SuggestionContext::FunctionContext);
        assert_eq!(partial, "then");

        let tracker = tracker_for("else");
        let (ctx, partial) = analyze_context("else", &tracker);
        assert_eq!(ctx, SuggestionContext::FunctionContext);
        assert_eq!(partial, "else");

        // Partial keywords
        let tracker = tracker_for("i");
        let (ctx, partial) = analyze_context("i", &tracker);
        assert_eq!(ctx, SuggestionContext::FunctionContext);
        assert_eq!(partial, "i");
    }

    // ========== ObjectKeyContext Unit Tests ==========

    #[test]
    fn test_object_key_context_after_open_brace() {
        // `{na` should return ObjectKeyContext
        let query = "{na";
        let tracker = tracker_for(query);
        let (ctx, partial) = analyze_context(query, &tracker);
        assert_eq!(ctx, SuggestionContext::ObjectKeyContext);
        assert_eq!(partial, "na");
    }

    #[test]
    fn test_object_key_context_after_comma() {
        // `{name: .name, ag` should return ObjectKeyContext
        let query = "{name: .name, ag";
        let tracker = tracker_for(query);
        let (ctx, partial) = analyze_context(query, &tracker);
        assert_eq!(ctx, SuggestionContext::ObjectKeyContext);
        assert_eq!(partial, "ag");
    }

    #[test]
    fn test_array_context_not_object_key() {
        // `[1, na` should NOT return ObjectKeyContext
        let query = "[1, na";
        let tracker = tracker_for(query);
        let (ctx, partial) = analyze_context(query, &tracker);
        assert_ne!(ctx, SuggestionContext::ObjectKeyContext);
        assert_eq!(partial, "na");
        // Should be FunctionContext since it's not a field access
        assert_eq!(ctx, SuggestionContext::FunctionContext);
    }

    #[test]
    fn test_function_call_context_not_object_key() {
        // `select(.a, na` should NOT return ObjectKeyContext
        let query = "select(.a, na";
        let tracker = tracker_for(query);
        let (ctx, partial) = analyze_context(query, &tracker);
        assert_ne!(ctx, SuggestionContext::ObjectKeyContext);
        assert_eq!(partial, "na");
        // Should be FunctionContext since it's inside a function call
        assert_eq!(ctx, SuggestionContext::FunctionContext);
    }

    #[test]
    fn test_nested_object_in_array() {
        // `[{na` should return ObjectKeyContext (innermost is object)
        let query = "[{na";
        let tracker = tracker_for(query);
        let (ctx, partial) = analyze_context(query, &tracker);
        assert_eq!(ctx, SuggestionContext::ObjectKeyContext);
        assert_eq!(partial, "na");
    }

    #[test]
    fn test_nested_array_in_object() {
        // `{items: [na` should NOT return ObjectKeyContext (innermost is array)
        let query = "{items: [na";
        let tracker = tracker_for(query);
        let (ctx, partial) = analyze_context(query, &tracker);
        assert_ne!(ctx, SuggestionContext::ObjectKeyContext);
        assert_eq!(partial, "na");
        // Should be FunctionContext since innermost context is array
        assert_eq!(ctx, SuggestionContext::FunctionContext);
    }

    #[test]
    fn test_object_key_empty_partial_no_suggestions() {
        // `{` alone should NOT return ObjectKeyContext (no partial)
        let query = "{";
        let tracker = tracker_for(query);
        let (ctx, partial) = analyze_context(query, &tracker);
        // With empty partial, we don't trigger ObjectKeyContext per requirement 1.3
        assert_ne!(ctx, SuggestionContext::ObjectKeyContext);
        assert_eq!(partial, "");
    }

    #[test]
    fn test_object_key_after_comma_empty_partial() {
        // `{name: .name, ` should NOT return ObjectKeyContext (no partial)
        let query = "{name: .name, ";
        let tracker = tracker_for(query);
        let (ctx, _partial) = analyze_context(query, &tracker);
        // With empty partial, we don't trigger ObjectKeyContext
        assert_ne!(ctx, SuggestionContext::ObjectKeyContext);
    }

    #[test]
    fn test_dot_after_brace_is_field_context() {
        // `{.na` should return FieldContext (not ObjectKeyContext)
        let query = "{.na";
        let tracker = tracker_for(query);
        let (ctx, partial) = analyze_context(query, &tracker);
        assert_eq!(ctx, SuggestionContext::FieldContext);
        assert_eq!(partial, "na");
    }

    #[test]
    fn test_object_key_with_complex_value() {
        // `{name: .name | map(.), ag` should return ObjectKeyContext
        let query = "{name: .name | map(.), ag";
        let tracker = tracker_for(query);
        let (ctx, partial) = analyze_context(query, &tracker);
        assert_eq!(ctx, SuggestionContext::ObjectKeyContext);
        assert_eq!(partial, "ag");
    }

    #[test]
    fn test_deeply_nested_object_context() {
        // `{a: {b: {c` should return ObjectKeyContext
        let query = "{a: {b: {c";
        let tracker = tracker_for(query);
        let (ctx, partial) = analyze_context(query, &tracker);
        assert_eq!(ctx, SuggestionContext::ObjectKeyContext);
        assert_eq!(partial, "c");
    }

    // ========== Regression Tests for Existing Behavior ==========
    // These tests verify that the ObjectKeyContext feature doesn't break
    // existing FieldContext and FunctionContext behavior.
    // Requirements: 4.1, 4.2, 4.3, 4.4, 4.5

    #[test]
    fn test_regression_field_context_at_start() {
        // `.na` at start should return FieldContext (not ObjectKeyContext)
        // Validates: Requirement 4.1
        let query = ".na";
        let tracker = tracker_for(query);
        let (ctx, partial) = analyze_context(query, &tracker);
        assert_eq!(ctx, SuggestionContext::FieldContext);
        assert_ne!(ctx, SuggestionContext::ObjectKeyContext);
        assert_eq!(partial, "na");
    }

    #[test]
    fn test_regression_field_context_after_pipe() {
        // `.services | .na` should return FieldContext
        // Validates: Requirement 4.2
        let query = ".services | .na";
        let tracker = tracker_for(query);
        let (ctx, partial) = analyze_context(query, &tracker);
        assert_eq!(ctx, SuggestionContext::FieldContext);
        assert_ne!(ctx, SuggestionContext::ObjectKeyContext);
        assert_eq!(partial, "na");
    }

    #[test]
    fn test_regression_field_context_in_map() {
        // `map(.na` should return FieldContext
        // Validates: Requirement 4.3
        let query = "map(.na";
        let tracker = tracker_for(query);
        let (ctx, partial) = analyze_context(query, &tracker);
        assert_eq!(ctx, SuggestionContext::FieldContext);
        assert_ne!(ctx, SuggestionContext::ObjectKeyContext);
        assert_eq!(partial, "na");
    }

    #[test]
    fn test_regression_function_context_at_start() {
        // `sel` at start should return FunctionContext
        // Validates: Requirement 4.4
        let query = "sel";
        let tracker = tracker_for(query);
        let (ctx, partial) = analyze_context(query, &tracker);
        assert_eq!(ctx, SuggestionContext::FunctionContext);
        assert_ne!(ctx, SuggestionContext::ObjectKeyContext);
        assert_eq!(partial, "sel");
    }

    #[test]
    fn test_regression_function_context_after_pipe() {
        // `.services | sel` should return FunctionContext
        // Validates: Requirement 4.5
        let query = ".services | sel";
        let tracker = tracker_for(query);
        let (ctx, partial) = analyze_context(query, &tracker);
        assert_eq!(ctx, SuggestionContext::FunctionContext);
        assert_ne!(ctx, SuggestionContext::ObjectKeyContext);
        assert_eq!(partial, "sel");
    }

    // ========== Property-Based Tests ==========

    proptest! {
        /// **Feature: object-key-autocomplete, Property 1: ObjectKeyContext suggestions have no leading dot**
        /// **Validates: Requirements 1.1, 1.2**
        ///
        /// For any query where the cursor is in ObjectKeyContext (after `{` or `,` inside
        /// an object literal with a partial typed), all returned suggestions SHALL NOT
        /// start with a `.` character.
        #[test]
        fn prop_object_key_context_suggestions_no_leading_dot(
            partial in "[a-z]{1,10}",
            field_names in prop::collection::vec("[a-z]{1,10}", 1..5),
        ) {
            use crate::query::ResultType;

            // Build a query that triggers ObjectKeyContext: `{<partial>`
            let query = format!("{{{}", partial);
            let tracker = tracker_for(&query);

            // Build a mock JSON result with the field names
            let json_fields: Vec<String> = field_names
                .iter()
                .map(|name| format!("\"{}\": \"value\"", name))
                .collect();
            let json_result = format!("{{{}}}", json_fields.join(", "));

            // Get suggestions
            let suggestions = get_suggestions(
                &query,
                query.len(),
                Some(&json_result),
                Some(ResultType::Object),
                &tracker,
            );

            // All suggestions should NOT start with a dot
            for suggestion in &suggestions {
                prop_assert!(
                    !suggestion.text.starts_with('.'),
                    "ObjectKeyContext suggestion '{}' should NOT start with '.', query: '{}'",
                    suggestion.text,
                    query
                );
            }
        }

        /// **Feature: object-key-autocomplete, Property 1: ObjectKeyContext suggestions have no leading dot**
        /// **Validates: Requirements 1.1, 1.2**
        ///
        /// For any query with comma inside object context, suggestions should not have leading dot.
        #[test]
        fn prop_object_key_context_after_comma_no_leading_dot(
            first_key in "[a-z]{1,8}",
            partial in "[a-z]{1,10}",
            field_names in prop::collection::vec("[a-z]{1,10}", 1..5),
        ) {
            use crate::query::ResultType;

            // Build a query that triggers ObjectKeyContext after comma: `{key: .key, <partial>`
            let query = format!("{{{}: .{}, {}", first_key, first_key, partial);
            let tracker = tracker_for(&query);

            // Build a mock JSON result with the field names
            let json_fields: Vec<String> = field_names
                .iter()
                .map(|name| format!("\"{}\": \"value\"", name))
                .collect();
            let json_result = format!("{{{}}}", json_fields.join(", "));

            // Get suggestions
            let suggestions = get_suggestions(
                &query,
                query.len(),
                Some(&json_result),
                Some(ResultType::Object),
                &tracker,
            );

            // All suggestions should NOT start with a dot
            for suggestion in &suggestions {
                prop_assert!(
                    !suggestion.text.starts_with('.'),
                    "ObjectKeyContext suggestion '{}' should NOT start with '.', query: '{}'",
                    suggestion.text,
                    query
                );
            }
        }

        /// **Feature: object-key-autocomplete, Property 2: Non-object contexts never return ObjectKeyContext**
        /// **Validates: Requirements 2.1, 2.2**
        ///
        /// For any query where the innermost unclosed brace is `[` (array) or `(` (paren),
        /// the analyze_context() function shall NOT return ObjectKeyContext.
        #[test]
        fn prop_non_object_contexts_never_return_object_key_context(
            prefix in "[a-z.| ]*",
            partial in "[a-z]{1,10}",
            brace_type in prop_oneof![Just('['), Just('(')],
        ) {
            // Build a query that ends with an array or paren context followed by a partial
            // Examples: "[na", "select(na", ".items | [na", "map(na"
            let query = format!("{}{}{}", prefix, brace_type, partial);
            
            let tracker = tracker_for(&query);
            let (ctx, _) = analyze_context(&query, &tracker);
            
            // Should never be ObjectKeyContext when inside array or paren
            prop_assert_ne!(
                ctx,
                SuggestionContext::ObjectKeyContext,
                "Query '{}' with innermost brace '{}' should NOT return ObjectKeyContext, got {:?}",
                query,
                brace_type,
                ctx
            );
        }

        /// **Feature: object-key-autocomplete, Property 2: Non-object contexts never return ObjectKeyContext**
        /// **Validates: Requirements 2.1, 2.2**
        ///
        /// For any query with comma inside array or paren context,
        /// the analyze_context() function shall NOT return ObjectKeyContext.
        #[test]
        fn prop_comma_in_non_object_context_not_object_key(
            prefix in "[a-z.| ]*",
            inner in "[a-z0-9., ]{0,20}",
            partial in "[a-z]{1,10}",
            brace_type in prop_oneof![Just('['), Just('(')],
        ) {
            // Build a query with comma inside array or paren
            // Examples: "[1, na", "select(.a, na", ".items | [.x, na"
            let query = format!("{}{}{}, {}", prefix, brace_type, inner, partial);
            
            let tracker = tracker_for(&query);
            let (ctx, _) = analyze_context(&query, &tracker);
            
            // Should never be ObjectKeyContext when comma is inside array or paren
            prop_assert_ne!(
                ctx,
                SuggestionContext::ObjectKeyContext,
                "Query '{}' with comma inside '{}' should NOT return ObjectKeyContext, got {:?}",
                query,
                brace_type,
                ctx
            );
        }

        /// **Feature: object-key-autocomplete, Property 6: Existing FieldContext behavior preserved**
        /// **Validates: Requirements 4.1, 4.2, 4.3**
        ///
        /// For any query starting with `.` followed by a partial (e.g., `.na`),
        /// the analyze_context() function SHALL return FieldContext, not ObjectKeyContext.
        #[test]
        fn prop_field_context_preserved_at_start(
            partial in "[a-z]{1,10}",
        ) {
            // Build a query that starts with dot followed by partial: `.na`
            let query = format!(".{}", partial);
            let tracker = tracker_for(&query);
            let (ctx, returned_partial) = analyze_context(&query, &tracker);
            
            // Should always be FieldContext, never ObjectKeyContext
            prop_assert_eq!(
                ctx,
                SuggestionContext::FieldContext,
                "Query '{}' starting with '.' should return FieldContext, got {:?}",
                query,
                ctx
            );
            
            // The partial should match what we typed
            prop_assert!(
                returned_partial == partial,
                "Query '{}' should return partial '{}', got '{}'",
                query,
                partial,
                returned_partial
            );
        }

        /// **Feature: object-key-autocomplete, Property 6: Existing FieldContext behavior preserved**
        /// **Validates: Requirements 4.1, 4.2, 4.3**
        ///
        /// For any query with pipe followed by dot and partial (e.g., `.services | .na`),
        /// the analyze_context() function SHALL return FieldContext.
        #[test]
        fn prop_field_context_preserved_after_pipe(
            field1 in "[a-z]{1,8}",
            partial in "[a-z]{1,10}",
        ) {
            // Build a query like `.services | .na`
            let query = format!(".{} | .{}", field1, partial);
            let tracker = tracker_for(&query);
            let (ctx, returned_partial) = analyze_context(&query, &tracker);
            
            // Should always be FieldContext
            prop_assert_eq!(
                ctx,
                SuggestionContext::FieldContext,
                "Query '{}' with pipe and dot should return FieldContext, got {:?}",
                query,
                ctx
            );
            
            // The partial should match what we typed after the last dot
            prop_assert!(
                returned_partial == partial,
                "Query '{}' should return partial '{}', got '{}'",
                query,
                partial,
                returned_partial
            );
        }

        /// **Feature: object-key-autocomplete, Property 6: Existing FieldContext behavior preserved**
        /// **Validates: Requirements 4.1, 4.2, 4.3**
        ///
        /// For any query with function call containing dot field access (e.g., `map(.na`),
        /// the analyze_context() function SHALL return FieldContext.
        #[test]
        fn prop_field_context_preserved_in_function_call(
            func in "(map|select|sort_by|group_by|unique_by|min_by|max_by)",
            partial in "[a-z]{1,10}",
        ) {
            // Build a query like `map(.na`
            let query = format!("{}(.{}", func, partial);
            let tracker = tracker_for(&query);
            let (ctx, returned_partial) = analyze_context(&query, &tracker);
            
            // Should always be FieldContext
            prop_assert_eq!(
                ctx,
                SuggestionContext::FieldContext,
                "Query '{}' with function call and dot should return FieldContext, got {:?}",
                query,
                ctx
            );
            
            // The partial should match what we typed after the dot
            prop_assert!(
                returned_partial == partial,
                "Query '{}' should return partial '{}', got '{}'",
                query,
                partial,
                returned_partial
            );
        }

        /// **Feature: object-key-autocomplete, Property 7: Existing FunctionContext behavior preserved**
        /// **Validates: Requirements 4.4, 4.5**
        ///
        /// For any query with a partial identifier not preceded by `.` and not inside object braces
        /// (e.g., `sel`), the analyze_context() function SHALL return FunctionContext.
        #[test]
        fn prop_function_context_preserved_at_start(
            partial in "[a-z]{1,10}",
        ) {
            // Build a query that is just a partial function name: `sel`
            let query = partial.clone();
            let tracker = tracker_for(&query);
            let (ctx, returned_partial) = analyze_context(&query, &tracker);
            
            // Should always be FunctionContext
            prop_assert_eq!(
                ctx,
                SuggestionContext::FunctionContext,
                "Query '{}' (bare identifier) should return FunctionContext, got {:?}",
                query,
                ctx
            );
            
            // The partial should match what we typed
            prop_assert!(
                returned_partial == partial,
                "Query '{}' should return partial '{}', got '{}'",
                query,
                partial,
                returned_partial
            );
        }

        /// **Feature: object-key-autocomplete, Property 7: Existing FunctionContext behavior preserved**
        /// **Validates: Requirements 4.4, 4.5**
        ///
        /// For any query with pipe followed by a partial identifier (e.g., `.services | sel`),
        /// the analyze_context() function SHALL return FunctionContext.
        #[test]
        fn prop_function_context_preserved_after_pipe(
            field in "[a-z]{1,8}",
            partial in "[a-z]{1,10}",
        ) {
            // Build a query like `.services | sel`
            let query = format!(".{} | {}", field, partial);
            let tracker = tracker_for(&query);
            let (ctx, returned_partial) = analyze_context(&query, &tracker);
            
            // Should always be FunctionContext
            prop_assert_eq!(
                ctx,
                SuggestionContext::FunctionContext,
                "Query '{}' with pipe and bare identifier should return FunctionContext, got {:?}",
                query,
                ctx
            );
            
            // The partial should match what we typed after the pipe
            prop_assert!(
                returned_partial == partial,
                "Query '{}' should return partial '{}', got '{}'",
                query,
                partial,
                returned_partial
            );
        }
    }
}
