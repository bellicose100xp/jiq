use crate::query::ResultType;
use super::jq_functions::filter_builtins;
use super::result_analyzer::ResultAnalyzer;
use super::autocomplete_state::Suggestion;

/// Context information about what's being typed
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SuggestionContext {
    /// At start or after pipe/operator - suggest functions and patterns
    FunctionContext,
    /// After a dot - suggest field names
    FieldContext,
}

/// Analyze query text and cursor position to determine what to suggest
pub fn get_suggestions(
    query: &str,
    cursor_pos: usize,
    result: Option<&str>,
    result_type: Option<ResultType>,
) -> Vec<Suggestion> {
    // Get the text before cursor
    let before_cursor = &query[..cursor_pos.min(query.len())];

    // Determine context and get the partial word being typed
    let (context, partial) = analyze_context(before_cursor);

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
    }
}

/// Analyze the text before cursor to determine context and partial word
pub fn analyze_context(before_cursor: &str) -> (SuggestionContext, String) {
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

    #[test]
    fn test_empty_query() {
        let (ctx, partial) = analyze_context("");
        assert_eq!(ctx, SuggestionContext::FunctionContext);
        assert_eq!(partial, "");
    }

    #[test]
    fn test_function_context() {
        let (ctx, partial) = analyze_context("ma");
        assert_eq!(ctx, SuggestionContext::FunctionContext);
        assert_eq!(partial, "ma");

        let (ctx, partial) = analyze_context("select");
        assert_eq!(ctx, SuggestionContext::FunctionContext);
        assert_eq!(partial, "select");
    }

    #[test]
    fn test_field_context_with_dot() {
        let (ctx, partial) = analyze_context(".na");
        assert_eq!(ctx, SuggestionContext::FieldContext);
        assert_eq!(partial, "na");

        let (ctx, partial) = analyze_context(".name");
        assert_eq!(ctx, SuggestionContext::FieldContext);
        assert_eq!(partial, "name");
    }

    #[test]
    fn test_just_dot() {
        let (ctx, partial) = analyze_context(".");
        assert_eq!(ctx, SuggestionContext::FieldContext);
        assert_eq!(partial, "");
    }

    #[test]
    fn test_after_pipe() {
        let (ctx, partial) = analyze_context(".name | ma");
        assert_eq!(ctx, SuggestionContext::FunctionContext);
        assert_eq!(partial, "ma");
    }

    #[test]
    fn test_nested_field() {
        let (ctx, partial) = analyze_context(".user.na");
        assert_eq!(ctx, SuggestionContext::FieldContext);
        assert_eq!(partial, "na");
    }

    #[test]
    fn test_array_access() {
        let (ctx, partial) = analyze_context(".items[0].na");
        assert_eq!(ctx, SuggestionContext::FieldContext);
        assert_eq!(partial, "na");
    }

    #[test]
    fn test_in_function_call() {
        let (ctx, partial) = analyze_context("map(.na");
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
        let (ctx, partial) = analyze_context(".services[].capacityProviderStrategy[]?.");
        assert_eq!(ctx, SuggestionContext::FieldContext);
        assert_eq!(partial, "");

        // After []?.b should be FieldContext with partial "b"
        let (ctx, partial) = analyze_context(".services[].capacityProviderStrategy[]?.b");
        assert_eq!(ctx, SuggestionContext::FieldContext);
        assert_eq!(partial, "b");
    }

    #[test]
    fn test_analyze_context_jq_keywords() {
        // jq keywords like "if", "then", "else" should be FunctionContext
        let (ctx, partial) = analyze_context("if");
        assert_eq!(ctx, SuggestionContext::FunctionContext);
        assert_eq!(partial, "if");

        let (ctx, partial) = analyze_context("then");
        assert_eq!(ctx, SuggestionContext::FunctionContext);
        assert_eq!(partial, "then");

        let (ctx, partial) = analyze_context("else");
        assert_eq!(ctx, SuggestionContext::FunctionContext);
        assert_eq!(partial, "else");

        // Partial keywords
        let (ctx, partial) = analyze_context("i");
        assert_eq!(ctx, SuggestionContext::FunctionContext);
        assert_eq!(partial, "i");
    }
}
