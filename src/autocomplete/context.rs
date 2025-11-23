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
///   "[.name, .age" -> "" (inside array constructor, use root context)
///   ".users | [.id, ." -> ".users |" (pipe before constructor)
///   "{name: .user.na" -> ".user" (nested path inside object constructor)
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

    // Check if we're inside an array or object constructor
    if let Some(constructor_pos) = find_unmatched_constructor_start(path) {
        // We're inside a constructor like [...] or {...}

        // Look at what's inside the constructor (after the opening bracket)
        let inside_constructor = &path[constructor_pos + 1..];

        // For constructors, elements are separated by commas
        // Find the last comma to get the current element's path
        let current_element = if let Some(comma_pos) = inside_constructor.rfind(',') {
            &inside_constructor[comma_pos + 1..]
        } else {
            inside_constructor
        };

        // Clean the current element's path (handles nested paths and object key:value syntax)
        let element_path = extract_clean_path(current_element);

        if !element_path.is_empty() {
            // We have a path within the current constructor element (e.g., [.user.name -> ".user")
            return element_path;
        }

        // No path in current element - use context before the constructor
        // (e.g., .users | [.name -> ".users |")
        if constructor_pos == 0 {
            // Constructor at the start, use root context
            return String::new();
        }

        // Extract and clean the path before the constructor
        let before_constructor = &path[..constructor_pos];
        return extract_clean_path(before_constructor);
    }

    // Normal path extraction (not inside constructor)
    extract_clean_path(path)
}

/// Extract a clean jq path from potentially complex query
/// Now keeps pipes (handled by json_analyzer), only strips parentheses and semicolons
/// Also handles colons (for object constructor values) and strips leading constructors
/// Examples:
///   ".data | select(.field" -> ".data |" (preserves context before function)
///   ".data.users | .[]" -> ".data.users | .[]" (keeps everything)
///   "name: .user" -> ".user" (strips object key)
///   "[.path" -> ".path" (strips leading constructor)
///   ".items[0:5]" -> ".items[0:5]" (keeps array slice colon)
fn extract_clean_path(text: &str) -> String {
    // Check if we're inside an UNMATCHED parenthesis (function call)
    // Only unmatched '(' should trigger context preservation
    if let Some(unmatched_paren_pos) = find_unmatched_paren_start(text) {
        let before_paren = &text[..unmatched_paren_pos];

        // Look for a pipe before the function call
        // For ".data | select(.field" we want to return ".data |"
        // This preserves the piped context for suggestions inside the function
        if let Some(pipe_pos) = before_paren.rfind('|') {
            // Return everything up to and including the pipe
            return text[..=pipe_pos].trim().to_string();
        }

        // No pipe before the parenthesis - function called at root level
        return String::new();
    }

    // If we have matched parentheses and the text ends with |,
    // we need to recursively clean what's before the last pipe
    // This handles cases like ".buildings[] | select(...) | " -> ".buildings[] |"
    if text.trim().ends_with('|') && text.contains('(')
        && let Some(last_pipe_pos) = text.rfind('|')
    {
        let before_last_pipe = text[..last_pipe_pos].trim();
        // Recursively clean what's before the pipe
        // This will strip any complete function calls
        let cleaned = extract_clean_path(before_last_pipe);
        if !cleaned.is_empty() {
            // Check if cleaned already ends with |, if so don't add another
            if cleaned.trim().ends_with('|') {
                return cleaned;
            } else {
                return format!("{} |", cleaned);
            }
        } else {
            return String::new();
        }
    }

    // If we have matched parentheses, check if this is a standalone function call
    // Strip it entirely (e.g., "select(.active)" -> "", "map(.items | .name)" -> "")
    // or if it ends with a function call after a pipe (e.g., ".users | map(.items)" -> ".users |")
    if text.contains('(') && text.contains(')') && text.trim().ends_with(')') {
        // Check if parens are balanced (all matched)
        let open_count = text.matches('(').count();
        let close_count = text.matches(')').count();
        if open_count == close_count {
            let trimmed = text.trim();

            // Check if there's a pipe before the function (outside all matched parens)
            if let Some(last_pipe_pos) = find_last_pipe_outside_parens(trimmed) {
                // Check if everything after the pipe is a function call
                let after_pipe = trimmed[last_pipe_pos + 1..].trim();
                if let Some(paren_pos) = after_pipe.find('(') {
                    let func_name = &after_pipe[..paren_pos];
                    if func_name.chars().all(|c| c.is_alphanumeric() || c == '_' || c.is_whitespace()) {
                        // Return everything up to and including the pipe
                        return trimmed[..=last_pipe_pos].trim().to_string();
                    }
                }
            } else {
                // No pipe, check if the whole thing is a function call
                if let Some(paren_pos) = trimmed.find('(') {
                    let before_paren = &trimmed[..paren_pos];
                    if before_paren.chars().all(|c| c.is_alphanumeric() || c == '_') {
                        return String::new();
                    }
                }
            }
        }
    }

    // Original logic for non-function cases
    let reset_positions = [
        text.rfind(';'),
        find_last_colon_outside_brackets(text), // Only colons outside brackets
    ];

    let last_reset = reset_positions
        .iter()
        .filter_map(|&p| p)
        .max()
        .map(|p| p + 1)
        .unwrap_or(0);

    // Extract from last reset point (keeps pipes intact)
    let mut path = text[last_reset..].trim().to_string();

    // Strip any leading constructor brackets that might remain
    // This handles cases like "[[.name" -> "[" -> ""
    while path.starts_with('[') || path.starts_with('{') {
        path = path[1..].trim().to_string();
    }

    path
}

/// Find the last colon that's outside of brackets
/// This distinguishes object constructor colons from array slicing colons
/// Returns None if no such colon exists
///
/// Examples:
///   "name: .value" -> Some(4) (object constructor)
///   ".items[0:5]" -> None (colon is inside brackets, used for slicing)
///   "{a: .x, b: .y[0:5]}" -> position of last ':' outside brackets (before .y)
fn find_last_colon_outside_brackets(text: &str) -> Option<usize> {
    let mut bracket_depth: i32 = 0;
    let mut last_colon_pos = None;

    for (byte_pos, ch) in text.char_indices() {
        match ch {
            '[' | '(' => bracket_depth += 1,
            ']' | ')' => bracket_depth = bracket_depth.saturating_sub(1),
            ':' if bracket_depth == 0 => {
                // Colon outside brackets - this is an object constructor colon
                last_colon_pos = Some(byte_pos);
            }
            _ => {}
        }
    }

    last_colon_pos
}

/// Find the last pipe '|' that's outside all matched parentheses
/// Returns the BYTE position of the last pipe outside parens, or None if no such pipe exists
///
/// Examples:
///   ".data | select(.x | .y)" -> Some(6) (pipe after .data, not the one inside select)
///   "select(.x | .y) | .z" -> Some(16) (pipe after the select)
///   ".x | .y" -> Some(3) (the pipe)
///   "select(.x)" -> None (no pipe outside parens)
fn find_last_pipe_outside_parens(text: &str) -> Option<usize> {
    let mut paren_depth: i32 = 0;

    // Scan backwards to find pipes outside parentheses
    let chars: Vec<(usize, char)> = text.char_indices().collect();

    for i in (0..chars.len()).rev() {
        let (byte_pos, ch) = chars[i];
        match ch {
            ')' => paren_depth += 1,
            '(' => paren_depth = paren_depth.saturating_sub(1),
            '|' if paren_depth == 0 => {
                // Found a pipe outside all parentheses
                return Some(byte_pos);
            }
            _ => {}
        }
    }

    None
}

/// Find the position of an unmatched opening parenthesis '('
/// Returns the BYTE position of the innermost (rightmost) unmatched '(', or None if all are matched
/// This is used to detect if we're inside a function call
///
/// Examples:
///   "select(.name" -> Some(6) (unmatched '(' from select)
///   "select(.name | contains(" -> Some(19) (unmatched '(' from contains, the innermost)
///   "select(.name)" -> None (matched parentheses)
///   ".data | map(.items) | [" -> None (map's parens are matched)
fn find_unmatched_paren_start(text: &str) -> Option<usize> {
    let mut depth = 0;

    // Scan backwards to find the innermost unmatched opening parenthesis
    let chars: Vec<(usize, char)> = text.char_indices().collect();

    for i in (0..chars.len()).rev() {
        let (byte_pos, ch) = chars[i];
        match ch {
            ')' => depth += 1,
            '(' => {
                if depth == 0 {
                    // Found an unmatched opening parenthesis - return immediately
                    // This gives us the innermost (rightmost) unmatched paren
                    return Some(byte_pos);
                } else {
                    depth -= 1;
                }
            }
            _ => {}
        }
    }

    None
}

/// Find the position of an unmatched opening bracket '[' or brace '{'
/// Returns the BYTE position of the OUTERMOST unmatched opener, or None if all are matched
/// This is used to detect if we're inside an array or object constructor
///
/// When there are nested constructors like [{...}], this returns the position of '['
/// not the '{', because '[' is the outermost constructor boundary.
///
/// Examples:
///   "[.name, .age" -> Some(0) (unmatched '[')
///   ".users | [.id, " -> Some(9) (unmatched '[')
///   ".items[0].na" -> None (the '[' is matched by ']')
///   "{name: .user" -> Some(0) (unmatched '{')
///   "[{id: .x}, {name: ." -> Some(0) (outermost '[', not inner '{')
fn find_unmatched_constructor_start(text: &str) -> Option<usize> {
    let mut depth_square = 0; // Track [ ] pairs
    let mut depth_curly = 0;  // Track { } pairs
    let mut outermost_pos = None; // Track the outermost unmatched constructor

    // Scan backwards using char_indices to get both char and byte positions
    // char_indices returns (byte_pos, char) tuples
    let chars: Vec<(usize, char)> = text.char_indices().collect();

    // Scan backwards from the end to find ALL unmatched constructors
    // Keep the outermost one (earliest in the string)
    for i in (0..chars.len()).rev() {
        let (byte_pos, ch) = chars[i];
        match ch {
            ']' => depth_square += 1,
            '[' => {
                if depth_square == 0 {
                    // Found an unmatched opening bracket
                    outermost_pos = Some(byte_pos); // Update to this position (earlier in string)
                }
                else {
                    depth_square -= 1;
                }
            }
            '}' => depth_curly += 1,
            '{' => {
                if depth_curly == 0 {
                    // Found an unmatched opening brace
                    // Only update if we haven't found an unmatched '[' yet
                    // (because '[' is always more outer than '{')
                    if outermost_pos.is_none() {
                        outermost_pos = Some(byte_pos);
                    }
                } else {
                    depth_curly -= 1;
                }
            }
            _ => {}
        }
    }

    outermost_pos
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
    if let Some(stripped) = partial.strip_prefix('.') {
        // Field context - return the part after the LAST dot (for nested fields like .user.na)
        let field_partial = if let Some(last_dot_pos) = partial.rfind('.') {
            partial[last_dot_pos + 1..].to_string()
        } else {
            stripped.to_string()
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

    #[test]
    fn test_extract_path_with_pipe() {
        // Pipes should now be kept in the path (json_analyzer handles them)
        assert_eq!(extract_path_before_current_field(".data.users | .[]"), ".data.users |");
        assert_eq!(extract_path_before_current_field(".data | .items | .[]"), ".data | .items |");
        assert_eq!(extract_path_before_current_field(".org.hq.facilities.buildings | ."), ".org.hq.facilities.buildings |");
    }

    #[test]
    fn test_extract_path_with_parentheses() {
        // Inside function at root level
        assert_eq!(extract_path_before_current_field("map(.items"), "");
        assert_eq!(extract_path_before_current_field("select(.active"), "");

        // After a function, back at root level
        assert_eq!(extract_path_before_current_field("select(.active) | .na"), "");
    }

    #[test]
    fn test_extract_path_with_mixed_operators() {
        // Inside a function that's after a pipe - preserves context
        assert_eq!(extract_path_before_current_field(".data | map(.items"), ".data |");

        // After a complete function with internal pipes - strips the function
        assert_eq!(
            extract_path_before_current_field("map(.items | .name) | .f"),
            ""
        );
    }

    // Tests for array constructor contexts
    #[test]
    fn test_array_constructor_simple() {
        // Inside array constructor at root level
        assert_eq!(extract_path_before_current_field("[.name"), "");
        assert_eq!(extract_path_before_current_field("[.name, .age"), "");
        assert_eq!(extract_path_before_current_field("[.name, .age, ."), "");
    }

    #[test]
    fn test_array_constructor_after_pipe() {
        // Array constructor after pipe - should use context before the constructor
        assert_eq!(extract_path_before_current_field(".users | [.name"), ".users |");
        assert_eq!(extract_path_before_current_field(".users | [.name, .age"), ".users |");
        assert_eq!(extract_path_before_current_field(".data.items | [.id, ."), ".data.items |");
    }

    #[test]
    fn test_array_constructor_nested_path() {
        // Nested paths inside array constructor
        assert_eq!(extract_path_before_current_field("[.user.name"), ".user");
        assert_eq!(extract_path_before_current_field("[.data.items[0].id"), ".data.items[0]");
    }

    // Tests for object constructor contexts
    #[test]
    fn test_object_constructor_simple() {
        // Inside object constructor at root level
        assert_eq!(extract_path_before_current_field("{name: .name"), "");
        assert_eq!(extract_path_before_current_field("{name: .name, age: .age"), "");
    }

    #[test]
    fn test_object_constructor_nested_path() {
        // Nested paths inside object constructor
        assert_eq!(extract_path_before_current_field("{name: .user.name"), ".user");
        assert_eq!(extract_path_before_current_field("{id: .data.id, name: .data.name"), ".data");
    }

    #[test]
    fn test_object_constructor_after_pipe() {
        // Object constructor after pipe
        assert_eq!(extract_path_before_current_field(".users | {name: .name"), ".users |");
        assert_eq!(extract_path_before_current_field(".items | {id: .id, title: ."), ".items |");
    }

    // Test to ensure matched brackets don't trigger constructor detection
    #[test]
    fn test_matched_brackets_not_constructor() {
        // These should work as normal (brackets are matched, so no constructor)
        assert_eq!(extract_path_before_current_field(".items[].name"), ".items[]");
        assert_eq!(extract_path_before_current_field(".data[0].user.name"), ".data[0].user");
    }

    // Tests for find_last_pipe_outside_parens helper
    #[test]
    fn test_find_last_pipe_outside_parens() {
        // Pipe outside all parens
        assert_eq!(find_last_pipe_outside_parens(".data | select(.x)"), Some(6));
        assert_eq!(find_last_pipe_outside_parens(".x | .y"), Some(3));

        // Pipe inside function should be ignored, find the one outside
        assert_eq!(find_last_pipe_outside_parens(".data | select(.x | .y)"), Some(6));
        assert_eq!(find_last_pipe_outside_parens("select(.x | .y) | .z"), Some(16));

        // Complex nested case
        assert_eq!(
            find_last_pipe_outside_parens(".buildings[] | select(.name | contains(\"Main\"))"),
            Some(13) // The pipe after .buildings[]
        );

        // No pipe outside parens
        assert_eq!(find_last_pipe_outside_parens("select(.x)"), None);
        assert_eq!(find_last_pipe_outside_parens("map(.x | .y)"), None);
    }

    // Tests for find_unmatched_paren_start helper
    #[test]
    fn test_find_unmatched_paren_start() {
        // Unmatched parentheses
        assert_eq!(find_unmatched_paren_start("select(.name"), Some(6));
        assert_eq!(find_unmatched_paren_start(".data | select(.active"), Some(14));
        assert_eq!(find_unmatched_paren_start("select(.name | contains("), Some(23));

        // Matched parentheses
        assert_eq!(find_unmatched_paren_start("select(.name)"), None);
        assert_eq!(find_unmatched_paren_start(".data | map(.items)"), None);
        assert_eq!(find_unmatched_paren_start(".data | map(.items) | [.id"), None);

        // Nested matched parentheses
        assert_eq!(find_unmatched_paren_start("map(select(.x))"), None);
    }

    // Tests for find_unmatched_constructor_start helper
    #[test]
    fn test_find_unmatched_constructor_start() {
        // Array constructors
        assert_eq!(find_unmatched_constructor_start("[.name"), Some(0));
        assert_eq!(find_unmatched_constructor_start(".users | [.id"), Some(9));

        // Object constructors
        assert_eq!(find_unmatched_constructor_start("{name: .x"), Some(0));
        assert_eq!(find_unmatched_constructor_start(".data | {id: .id"), Some(8));

        // Matched brackets (not constructors)
        assert_eq!(find_unmatched_constructor_start(".items[]"), None);
        assert_eq!(find_unmatched_constructor_start(".data[0]"), None);
        assert_eq!(find_unmatched_constructor_start(".items[].name"), None);

        // Nested matched brackets
        assert_eq!(find_unmatched_constructor_start(".a[.b[0]]"), None);
    }

    // Complex mixed scenarios
    #[test]
    fn test_complex_constructor_scenarios() {
        // Array inside array
        assert_eq!(extract_path_before_current_field("[[.name"), "");

        // Array constructor after function - strips the function, preserves data path
        assert_eq!(extract_path_before_current_field(".users | map(.items) | [.id"), ".users |");

        // Object with array value
        assert_eq!(extract_path_before_current_field("{tags: [.tag"), "");
    }

    // Tests for select() and function contexts - Bug fix for autosuggestion issue
    #[test]
    fn test_select_with_piped_context() {
        // THE MAIN BUG: Inside select() - should preserve context before select
        // This was showing root-level .organization instead of building fields
        assert_eq!(
            extract_path_before_current_field(".buildings[] | select(."),
            ".buildings[] |"
        );
        assert_eq!(
            extract_path_before_current_field(".organization.headquarters.facilities.buildings[] | select(."),
            ".organization.headquarters.facilities.buildings[] |"
        );

        // Nested path inside select
        assert_eq!(
            extract_path_before_current_field(".buildings[] | select(.name"),
            ".buildings[] |"
        );

        // After the select, when typing the last | .name from the bug report
        // The select() function should be stripped since it doesn't change the type
        assert_eq!(
            extract_path_before_current_field(".organization.headquarters.facilities.buildings[] | select(.name | contains(\"Main\")) | .name"),
            ".organization.headquarters.facilities.buildings[] |"
        );
    }

    #[test]
    fn test_map_with_piped_context() {
        // Inside map() - should preserve context before map
        assert_eq!(
            extract_path_before_current_field(".items | map(."),
            ".items |"
        );
        assert_eq!(
            extract_path_before_current_field(".data.users | map(.profile"),
            ".data.users |"
        );
    }

    #[test]
    fn test_nested_functions_with_pipes() {
        // Inside inner function
        assert_eq!(
            extract_path_before_current_field(".data | select(.active) | map(."),
            ".data | select(.active) |"
        );
    }

    #[test]
    fn test_array_iterator_after_select() {
        // Inside select - preserves context before select
        assert_eq!(
            extract_path_before_current_field(".buildings[] | select(.name"),
            ".buildings[] |"
        );

        // Inside nested function within select - should preserve context before the nested function
        assert_eq!(
            extract_path_before_current_field(".buildings[] | select(.name | contains(."),
            ".buildings[] | select(.name |"
        );

        // After complete select with nested functions (all parens matched)
        // The select should be stripped to get the actual data type
        assert_eq!(
            extract_path_before_current_field(".buildings[] | select(.name | contains(\"Main\")) | ."),
            ".buildings[] |"
        );
    }

    // Tests for Unicode/multi-byte character handling (Bug fix verification)
    #[test]
    fn test_unicode_before_constructor() {
        // Main goal: verify no panic with multi-byte UTF-8 characters

        // Accented characters (2-byte UTF-8) - 'é' is 2 bytes, '[' is at byte 5
        assert_eq!(find_unmatched_constructor_start("café[.name"), Some(5));
        // "café" is extracted as path before constructor (not a valid jq path, but doesn't panic)
        let _result = extract_path_before_current_field("café[.name");

        // Emoji (4-byte UTF-8) - emoji is 4 bytes, '[' is at byte 4
        assert_eq!(find_unmatched_constructor_start("👍[.id"), Some(4));
        let _result = extract_path_before_current_field("👍[.name");

        // Valid jq with mixed multi-byte characters
        assert_eq!(extract_path_before_current_field(".user.ñame | [.id"), ".user.ñame |");

        // Chinese characters (3-byte UTF-8) - each character is 3 bytes
        // "用户" = 6 bytes, '[' is at byte 6
        assert_eq!(find_unmatched_constructor_start("用户[.name"), Some(6));
    }

    // Tests for array slicing with colons (Bug fix verification)
    #[test]
    fn test_array_slicing_with_colons() {
        // Simple array slice
        assert_eq!(extract_path_before_current_field(".text[0:5].length"), ".text[0:5]");

        // Nested array slice
        assert_eq!(extract_path_before_current_field(".data.items[2:10].name"), ".data.items[2:10]");

        // Array slice with negative indices
        assert_eq!(extract_path_before_current_field(".array[-5:-1].value"), ".array[-5:-1]");

        // Multiple slices
        assert_eq!(extract_path_before_current_field(".a[0:5].b[1:3].c"), ".a[0:5].b[1:3]");
    }

    // Tests for find_last_colon_outside_brackets helper
    #[test]
    fn test_find_last_colon_outside_brackets() {
        // Object constructor colon (should find it)
        assert_eq!(find_last_colon_outside_brackets("name: .value"), Some(4));
        // "a: .x, b: .y" - colons at positions 1 and 8, last one is at 8
        assert_eq!(find_last_colon_outside_brackets("a: .x, b: .y"), Some(8));

        // Array slicing colon (should NOT find it - inside brackets)
        assert_eq!(find_last_colon_outside_brackets(".items[0:5]"), None);
        assert_eq!(find_last_colon_outside_brackets(".data[1:10]"), None);

        // Mixed: object constructor with array slice inside
        assert_eq!(find_last_colon_outside_brackets("id: .data[0:5]"), Some(2)); // The ':' after id

        // No colons
        assert_eq!(find_last_colon_outside_brackets(".simple.path"), None);

        // Colon inside function call
        assert_eq!(find_last_colon_outside_brackets("func(.x[0:5])"), None);
    }

    // Edge case tests
    #[test]
    fn test_edge_cases() {
        // Empty constructor
        assert_eq!(extract_path_before_current_field("[."), "");
        assert_eq!(extract_path_before_current_field("{name: ."), "");

        // Whitespace in constructors (just verify it doesn't panic)
        let _result = extract_path_before_current_field("[  .name  ,  .age");
        let _result2 = extract_path_before_current_field("{  id  :  .value  ,  name  :  .");

        // Very deeply nested - should find OUTERMOST unmatched bracket (at position 0)
        assert_eq!(find_unmatched_constructor_start("[[[[[.x"), Some(0));

        // Mixed brackets and braces - should find outermost '['
        assert_eq!(extract_path_before_current_field(".data | [{id: .id}, {name: ."), ".data |");
    }

    // Tests for additional jq syntax edge cases (verify no panics)
    #[test]
    fn test_jq_edge_cases_no_panic() {
        // Closed constructor before pipe (minor issue: creates invalid path, but doesn't crash)
        let _result = extract_path_before_current_field("[.a, .b] | {x: .");

        // Alternative operator // (not handled, but safe)
        let _result = extract_path_before_current_field(".field // .default");

        // Optional field access ?
        let _result = extract_path_before_current_field(".field?.sub");
        assert_eq!(analyze_context(".field?").1, "field?");

        // Recursive descent ..
        let _result = extract_path_before_current_field("..field");

        // Complex slicing variations
        let _result = extract_path_before_current_field(".arr[:.5].x");
        let _result = extract_path_before_current_field(".arr[2:].x");
        let _result = extract_path_before_current_field(".arr[:].x");

        // Assignment-like operators
        let _result = extract_path_before_current_field(".field |= .transform");

        // Multiple pipes
        let _result = extract_path_before_current_field(".a | .b | .c | .d");

        // Deeply nested with pipes
        let _result = extract_path_before_current_field(".a | [.b | {c: .d | .e}]");

        // Empty array/object constructors
        let _result = extract_path_before_current_field("[] | .");
        let _result = extract_path_before_current_field("{} | .");
    }
}
