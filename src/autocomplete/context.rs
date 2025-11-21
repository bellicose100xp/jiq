use super::jq_functions::filter_builtins;
use super::json_analyzer::JsonAnalyzer;
use super::state::Suggestion;

/// Context information about what's being typed
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SuggestionContext {
    /// At start or after pipe/operator - suggest functions and patterns
    FunctionContext,
    /// After a dot - suggest field names
    FieldContext,
    /// No suggestions needed
    None,
}

/// Analyze query text and cursor position to determine what to suggest
pub fn get_suggestions(
    query: &str,
    cursor_pos: usize,
    json_analyzer: &JsonAnalyzer,
) -> Vec<Suggestion> {
    // Get the text before cursor
    let before_cursor = &query[..cursor_pos.min(query.len())];

    // Determine context and get the partial word being typed
    let (context, partial) = analyze_context(before_cursor);

    match context {
        SuggestionContext::FieldContext => {
            // Suggest JSON fields
            json_analyzer.get_field_suggestions(&partial)
        }
        SuggestionContext::FunctionContext => {
            // Suggest jq functions/patterns/operators
            if partial.is_empty() {
                Vec::new()
            } else {
                filter_builtins(&partial)
            }
        }
        SuggestionContext::None => Vec::new(),
    }
}

/// Analyze the text before cursor to determine context and partial word
fn analyze_context(before_cursor: &str) -> (SuggestionContext, String) {
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

    // Check if the partial starts with a dot (field access)
    if partial.starts_with('.') {
        // Field context - return the part after the LAST dot (for nested fields like .user.na)
        let field_partial = if let Some(last_dot_pos) = partial.rfind('.') {
            partial[last_dot_pos + 1..].to_string()
        } else {
            partial[1..].to_string()
        };
        return (SuggestionContext::FieldContext, field_partial);
    }

    // Check what comes before the partial to determine context
    if start > 0 {
        // Look backwards to see if there's a dot before this position
        let mut j = start;
        while j > 0 && chars[j - 1].is_whitespace() {
            j -= 1;
        }

        if j > 0 && chars[j - 1] == '.' {
            // There's a dot before - we're in field context
            return (SuggestionContext::FieldContext, partial);
        }
    }

    // Otherwise, function context
    (SuggestionContext::FunctionContext, partial)
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
}
