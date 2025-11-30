//! Function detection for tooltip
//!
//! Identifies which jq function (if any) the cursor is currently within.
//! This includes both direct cursor placement on a function name AND
//! cursor placement inside a function's parentheses.

use crate::autocomplete::jq_functions::JQ_FUNCTION_METADATA;

/// Detect jq function at cursor position in query string
///
/// Returns the function name if the cursor is:
/// 1. Positioned directly on a recognized jq function name, OR
/// 2. Inside the parentheses of a recognized jq function
///
/// For nested function calls, returns the innermost enclosing function.
///
/// # Arguments
/// * `query` - The query string to search in
/// * `cursor_pos` - The cursor position (0-indexed, in characters)
///
/// # Returns
/// * `Some(&'static str)` - The function name if cursor is within a function context
/// * `None` - If cursor is not within any function context or position is invalid
///
/// # Examples
/// ```ignore
/// // Cursor on function name
/// detect_function_at_cursor("select(.x)", 3) // Some("select")
///
/// // Cursor inside function parentheses
/// detect_function_at_cursor("select(.field)", 8) // Some("select")
///
/// // Nested functions - returns innermost
/// detect_function_at_cursor("select(test(\"pat\"))", 13) // Some("test")
/// ```
pub fn detect_function_at_cursor(query: &str, cursor_pos: usize) -> Option<&'static str> {
    // Handle edge cases
    if query.is_empty() {
        return None;
    }

    // Convert to chars for proper Unicode handling
    let chars: Vec<char> = query.chars().collect();
    let len = chars.len();

    // Cursor can be at position 0 to len (inclusive)
    if cursor_pos > len {
        return None;
    }

    // Phase 1: Check if cursor is directly on a function name
    if let Some(func) = detect_function_at_word(&chars, cursor_pos) {
        return Some(func);
    }

    // Phase 2: Find enclosing function by scanning for unmatched opening paren
    find_enclosing_function(&chars, cursor_pos)
}

/// Phase 1: Detect function when cursor is directly on a word
fn detect_function_at_word(chars: &[char], cursor_pos: usize) -> Option<&'static str> {
    let (start, end) = find_word_boundaries(chars, cursor_pos);

    // If no word found
    if start == end {
        return None;
    }

    // Extract the token
    let token: String = chars[start..end].iter().collect();

    // Look up in JQ_FUNCTION_METADATA
    lookup_function(&token)
}

/// Phase 2: Find the enclosing function by tracking parenthesis nesting
///
/// Scans backwards from cursor position, tracking paren depth.
/// When we find an unmatched '(', we look for the function name before it.
fn find_enclosing_function(chars: &[char], cursor_pos: usize) -> Option<&'static str> {
    let mut depth: i32 = 0;
    let scan_start = cursor_pos.min(chars.len());

    // Scan backwards from cursor position
    for i in (0..scan_start).rev() {
        match chars[i] {
            ')' => depth += 1,
            '(' => {
                depth -= 1;
                // Found an unmatched opening paren
                if depth < 0 {
                    // Look for function name immediately before this '('
                    if let Some(func) = find_function_before_paren(chars, i) {
                        return Some(func);
                    }
                    // Reset depth and continue looking for outer function
                    depth = 0;
                }
            }
            _ => {}
        }
    }

    None
}

/// Find function name immediately before an opening parenthesis
fn find_function_before_paren(chars: &[char], paren_pos: usize) -> Option<&'static str> {
    if paren_pos == 0 {
        return None;
    }

    // Find the end of the word (should be right before the paren)
    let mut end = paren_pos;

    // Skip any whitespace between function name and paren (though jq typically doesn't have this)
    while end > 0 && chars[end - 1].is_whitespace() {
        end -= 1;
    }

    if end == 0 || !is_word_char(chars[end - 1]) {
        return None;
    }

    // Find the start of the word
    let mut start = end - 1;
    while start > 0 && is_word_char(chars[start - 1]) {
        start -= 1;
    }

    // Extract the token
    let token: String = chars[start..end].iter().collect();

    // Look up in JQ_FUNCTION_METADATA
    lookup_function(&token)
}

/// Find word boundaries around a cursor position
///
/// Returns (start, end) indices where the word spans [start, end)
fn find_word_boundaries(chars: &[char], cursor_pos: usize) -> (usize, usize) {
    let len = chars.len();

    if len == 0 {
        return (0, 0);
    }

    // Determine the character position to start from
    let check_pos = if cursor_pos >= len {
        if len > 0 {
            len - 1
        } else {
            return (0, 0);
        }
    } else if !is_word_char(chars[cursor_pos]) {
        // Cursor is on a non-word char, check if previous char is a word char
        if cursor_pos > 0 && is_word_char(chars[cursor_pos - 1]) {
            cursor_pos - 1
        } else {
            return (0, 0);
        }
    } else {
        cursor_pos
    };

    // Now check_pos should be on a word character
    if !is_word_char(chars[check_pos]) {
        return (0, 0);
    }

    // Find start of word (scan backwards)
    let mut start = check_pos;
    while start > 0 && is_word_char(chars[start - 1]) {
        start -= 1;
    }

    // Find end of word (scan forwards)
    let mut end = check_pos + 1;
    while end < len && is_word_char(chars[end]) {
        end += 1;
    }

    (start, end)
}

/// Check if a character is part of a word (alphanumeric or underscore)
fn is_word_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// Look up a token in JQ_FUNCTION_METADATA
fn lookup_function(token: &str) -> Option<&'static str> {
    JQ_FUNCTION_METADATA
        .iter()
        .find(|f| f.name == token)
        .map(|f| f.name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    // ==================== Unit Tests ====================

    // Tests for cursor directly on function name (Phase 1)

    #[test]
    fn test_detect_function_cursor_on_function() {
        assert_eq!(detect_function_at_cursor("select(.x)", 3), Some("select"));
        assert_eq!(detect_function_at_cursor("select(.x)", 0), Some("select"));
        assert_eq!(detect_function_at_cursor("select(.x)", 6), Some("select"));
    }

    #[test]
    fn test_detect_function_multiple_functions_on_name() {
        let query = "map(select(.x))";
        assert_eq!(detect_function_at_cursor(query, 1), Some("map"));
        assert_eq!(detect_function_at_cursor(query, 5), Some("select"));
    }

    #[test]
    fn test_detect_function_with_pipe() {
        let query = ".[] | sort | reverse";
        assert_eq!(detect_function_at_cursor(query, 7), Some("sort"));
        assert_eq!(detect_function_at_cursor(query, 15), Some("reverse"));
    }

    #[test]
    fn test_detect_function_underscore_names() {
        assert_eq!(detect_function_at_cursor("sort_by(.x)", 3), Some("sort_by"));
        assert_eq!(detect_function_at_cursor("to_entries", 5), Some("to_entries"));
    }

    // Tests for cursor inside function parentheses (Phase 2)

    #[test]
    fn test_detect_enclosing_function_simple() {
        // Cursor inside select's parens, on ".field"
        assert_eq!(
            detect_function_at_cursor("select(.field)", 8),
            Some("select")
        );
        // Cursor right after opening paren
        assert_eq!(
            detect_function_at_cursor("select(.field)", 7),
            Some("select")
        );
    }

    #[test]
    fn test_detect_enclosing_function_nested() {
        let query = "select(.field | test(\"pattern\"))";
        // Cursor inside test's parens
        assert_eq!(detect_function_at_cursor(query, 25), Some("test"));
        // Cursor inside select's parens but outside test
        assert_eq!(detect_function_at_cursor(query, 10), Some("select"));
    }

    #[test]
    fn test_detect_enclosing_function_deeply_nested() {
        let query = "map(select(.x | test(\"a\") | contains(\"b\")))";
        // Cursor inside contains
        assert_eq!(detect_function_at_cursor(query, 38), Some("contains"));
        // Cursor inside test
        assert_eq!(detect_function_at_cursor(query, 22), Some("test"));
        // Cursor inside select but outside inner functions
        assert_eq!(detect_function_at_cursor(query, 12), Some("select"));
        // Cursor inside map but outside select
        assert_eq!(detect_function_at_cursor(query, 4), Some("select"));
    }

    #[test]
    fn test_detect_enclosing_after_autocomplete() {
        // Simulates: user typed "sel<tab>" and got "select("
        // Cursor is now right after the opening paren
        assert_eq!(detect_function_at_cursor("select(", 7), Some("select"));
        assert_eq!(detect_function_at_cursor(".[] | map(", 10), Some("map"));
    }

    #[test]
    fn test_detect_enclosing_with_content() {
        // User is typing inside the function
        assert_eq!(
            detect_function_at_cursor("select(.name == \"foo\")", 15),
            Some("select")
        );
        assert_eq!(
            detect_function_at_cursor("map(.price * 2)", 10),
            Some("map")
        );
    }

    // Edge cases

    #[test]
    fn test_detect_function_empty_query() {
        assert_eq!(detect_function_at_cursor("", 0), None);
        assert_eq!(detect_function_at_cursor("", 5), None);
    }

    #[test]
    fn test_detect_function_cursor_outside_bounds() {
        assert_eq!(detect_function_at_cursor("map", 100), None);
    }

    #[test]
    fn test_detect_function_unknown_word() {
        assert_eq!(detect_function_at_cursor("foo", 1), None);
        assert_eq!(detect_function_at_cursor("foo(.x)", 5), None);
    }

    #[test]
    fn test_detect_no_function_context() {
        // Just field access, no function
        assert_eq!(detect_function_at_cursor(".field", 3), None);
        // Array index
        assert_eq!(detect_function_at_cursor(".[0]", 2), None);
        // Pipe without functions
        assert_eq!(detect_function_at_cursor(".a | .b", 5), None);
    }

    #[test]
    fn test_cursor_at_word_end() {
        assert_eq!(detect_function_at_cursor("map", 3), Some("map"));
        assert_eq!(detect_function_at_cursor("select", 6), Some("select"));
    }

    // ==================== Property Tests ====================

    // **Feature: function-tooltip, Property 6: Function detection correctness**
    // *For any* query string and cursor position:
    // - If cursor is directly on a known function name, returns that function
    // - If cursor is inside the parentheses of a known function, returns innermost enclosing function
    // - For cursor positions not within any function context, returns None
    // **Validates: Requirements 1.1, 1.2, 1.4**
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn prop_function_detection_on_name(
            func_index in 0usize..JQ_FUNCTION_METADATA.len(),
            prefix in "[.| ]{0,5}",
            suffix in "[()| .]{0,10}",
            cursor_offset in 0usize..20
        ) {
            let func = &JQ_FUNCTION_METADATA[func_index];
            let func_name = func.name;

            let query = format!("{}{}{}", prefix, func_name, suffix);
            let func_start = prefix.len();
            let func_end = func_start + func_name.len();

            // Test cursor positions within the function name
            if cursor_offset < func_name.len() {
                let cursor_pos = func_start + cursor_offset;
                let result = detect_function_at_cursor(&query, cursor_pos);
                prop_assert_eq!(
                    result,
                    Some(func_name),
                    "Cursor at position {} in '{}' should detect function '{}'",
                    cursor_pos,
                    query,
                    func_name
                );
            }

            // Test cursor at end of function name
            let result_at_end = detect_function_at_cursor(&query, func_end);
            prop_assert_eq!(
                result_at_end,
                Some(func_name),
                "Cursor at end position {} in '{}' should detect function '{}'",
                func_end,
                query,
                func_name
            );
        }

        #[test]
        fn prop_enclosing_function_detection(
            func_index in 0usize..JQ_FUNCTION_METADATA.len(),
            inner_content in "[.a-z0-9]{1,10}",
            cursor_offset in 0usize..10
        ) {
            let func = &JQ_FUNCTION_METADATA[func_index];
            // Only test functions that take arguments
            if !func.needs_parens {
                return Ok(());
            }

            let func_name = func.name;
            // Build query like "select(.field)"
            let query = format!("{}({})", func_name, inner_content);
            let paren_pos = func_name.len();
            let content_start = paren_pos + 1;
            let content_end = content_start + inner_content.len();

            // Test cursor inside the parentheses (on the inner content)
            let cursor_pos = content_start + (cursor_offset % inner_content.len().max(1));
            if cursor_pos < content_end {
                let result = detect_function_at_cursor(&query, cursor_pos);
                prop_assert_eq!(
                    result,
                    Some(func_name),
                    "Cursor at position {} inside '{}' should detect enclosing function '{}'",
                    cursor_pos,
                    query,
                    func_name
                );
            }
        }

        #[test]
        fn prop_nested_returns_innermost(
            outer_index in 0usize..JQ_FUNCTION_METADATA.len(),
            inner_index in 0usize..JQ_FUNCTION_METADATA.len()
        ) {
            let outer = &JQ_FUNCTION_METADATA[outer_index];
            let inner = &JQ_FUNCTION_METADATA[inner_index];

            // Only test functions that take arguments
            if !outer.needs_parens || !inner.needs_parens {
                return Ok(());
            }

            // Build nested query like "map(select(.x))"
            let query = format!("{}({}(.x))", outer.name, inner.name);
            // Cursor inside inner function's parens (on ".x")
            let inner_content_pos = outer.name.len() + 1 + inner.name.len() + 1;

            let result = detect_function_at_cursor(&query, inner_content_pos);
            prop_assert_eq!(
                result,
                Some(inner.name),
                "Cursor inside inner function in '{}' should detect '{}', not '{}'",
                query,
                inner.name,
                outer.name
            );
        }

        #[test]
        fn prop_empty_query_returns_none(cursor_pos in 0usize..100) {
            let result = detect_function_at_cursor("", cursor_pos);
            prop_assert_eq!(result, None, "Empty query should always return None");
        }

        #[test]
        fn prop_cursor_outside_bounds_returns_none(
            query in "[a-z]{1,10}",
            extra_offset in 1usize..100
        ) {
            let len = query.chars().count();
            let cursor_pos = len + extra_offset;
            let result = detect_function_at_cursor(&query, cursor_pos);

            prop_assert_eq!(
                result,
                None,
                "Cursor at position {} (beyond query length {}) should return None",
                cursor_pos,
                len
            );
        }
    }
}
