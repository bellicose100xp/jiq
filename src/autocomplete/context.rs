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
            // Extract the path to the current field (for context-aware suggestions)
            let path = extract_path_before_current_field(before_cursor);

            // Use context-aware field suggestions
            json_analyzer.get_contextual_field_suggestions(&path, &partial)
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

/// Extract the jq path before the current field being typed
/// Examples:
///   ".products.ty" -> ".products"
///   ".services[].service" -> ".services[]"
///   ".na" -> ""
///   "." -> ""
fn extract_path_before_current_field(before_cursor: &str) -> String {
    // Find the last dot position
    let last_dot_pos = match before_cursor.rfind('.') {
        Some(pos) => pos,
        None => return String::new(), // No path
    };

    // If the dot is at position 0, we're at root level
    if last_dot_pos == 0 {
        return String::new();
    }

    // Extract everything before the last dot
    let path = &before_cursor[..last_dot_pos];

    // Clean up the path:
    // - Remove trailing pipes, parentheses, etc.
    // - Keep only the valid jq path portion
    extract_clean_path(path)
}

/// Extract a clean jq path from potentially complex query
/// Handles cases like: "map(.items) | .products" -> ".products"
fn extract_clean_path(text: &str) -> String {
    // Find the last occurrence of pipe or other operators that reset context
    let reset_positions = [
        text.rfind('|'),
        text.rfind('('),
        text.rfind(';'),
    ];

    let last_reset = reset_positions
        .iter()
        .filter_map(|&p| p)
        .max()
        .map(|p| p + 1)
        .unwrap_or(0);

    // Extract from last reset point
    let mut path = text[last_reset..].trim().to_string();

    // Remove trailing whitespace and operators
    path = path.trim().to_string();

    // If path doesn't start with a dot and isn't empty, it might be invalid
    // Return empty in that case (conservative approach)
    if !path.is_empty() && !path.starts_with('.') {
        return String::new();
    }

    path
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

    #[test]
    fn test_extract_path_root_level() {
        assert_eq!(extract_path_before_current_field("."), "");
        assert_eq!(extract_path_before_current_field(".na"), "");
    }

    #[test]
    fn test_extract_path_nested() {
        assert_eq!(extract_path_before_current_field(".products.ty"), ".products");
        assert_eq!(extract_path_before_current_field(".services.items."), ".services.items");
    }

    #[test]
    fn test_extract_path_with_array() {
        assert_eq!(extract_path_before_current_field(".services[].service"), ".services[]");
        assert_eq!(extract_path_before_current_field(".items[0].na"), ".items[0]");
    }
}
