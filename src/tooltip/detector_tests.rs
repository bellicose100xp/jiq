//! Tests for tooltip/detector

use super::*;
use proptest::prelude::*;

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
    assert_eq!(
        detect_function_at_cursor("to_entries", 5),
        Some("to_entries")
    );
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

#[test]
fn test_detect_operator_double_slash() {
    // Cursor on first /
    assert_eq!(
        detect_operator_at_cursor(".x // \"default\"", 3),
        Some("//")
    );
    // Cursor on second /
    assert_eq!(
        detect_operator_at_cursor(".x // \"default\"", 4),
        Some("//")
    );
}

#[test]
fn test_detect_operator_pipe_equals() {
    // Cursor on |
    assert_eq!(detect_operator_at_cursor(".x |= . + 1", 3), Some("|="));
    // Cursor on =
    assert_eq!(detect_operator_at_cursor(".x |= . + 1", 4), Some("|="));
}

#[test]
fn test_detect_operator_triple_slash_equals() {
    // Cursor on first /
    assert_eq!(detect_operator_at_cursor(".x //= 0", 3), Some("//="));
    // Cursor on second /
    assert_eq!(detect_operator_at_cursor(".x //= 0", 4), Some("//="));
    // Cursor on =
    assert_eq!(detect_operator_at_cursor(".x //= 0", 5), Some("//="));
}

#[test]
fn test_detect_operator_double_dot() {
    // Cursor on first .
    assert_eq!(detect_operator_at_cursor(".. | numbers", 0), Some(".."));
    // Cursor on second .
    assert_eq!(detect_operator_at_cursor(".. | numbers", 1), Some(".."));
}

#[test]
fn test_detect_operator_no_false_positive_single_slash() {
    // Single / for division should not be detected
    assert_eq!(detect_operator_at_cursor(".x / 2", 3), None);
}

#[test]
fn test_detect_operator_no_false_positive_single_pipe() {
    // Single | for pipe should not be detected
    assert_eq!(detect_operator_at_cursor(".x | .y", 3), None);
}

#[test]
fn test_detect_operator_no_false_positive_single_dot() {
    // Single . for field access should not be detected
    assert_eq!(detect_operator_at_cursor(".field", 0), None);
    assert_eq!(detect_operator_at_cursor(".x.y", 2), None);
}

#[test]
fn test_detect_operator_empty_query() {
    assert_eq!(detect_operator_at_cursor("", 0), None);
}

#[test]
fn test_detect_operator_cursor_outside_bounds() {
    assert_eq!(detect_operator_at_cursor("//", 100), None);
}

#[test]
fn test_detect_operator_at_query_boundaries() {
    // Operator at start
    assert_eq!(detect_operator_at_cursor("// \"default\"", 0), Some("//"));
    assert_eq!(detect_operator_at_cursor("// \"default\"", 1), Some("//"));
    // Operator at end
    assert_eq!(detect_operator_at_cursor(".x //", 3), Some("//"));
    assert_eq!(detect_operator_at_cursor(".x //", 4), Some("//"));
}

#[test]
fn test_detect_operator_triple_dot_not_detected() {
    // ... (three dots) should not be detected as ..
    assert_eq!(detect_operator_at_cursor("...", 0), None);
    assert_eq!(detect_operator_at_cursor("...", 1), None);
    assert_eq!(detect_operator_at_cursor("...", 2), None);
}

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

    // **Feature: operator-tooltips, Property 1: Operator detection correctness**
    // *For any* query string containing a supported operator (`//`, `|=`, `//=`, `..`)
    // and *for any* cursor position on any character of that operator,
    // the detection function SHALL return that operator.
    // **Validates: Requirements 1.1, 2.1, 3.1, 4.1**
    #[test]
    fn prop_operator_detection_correctness(
        op_index in 0usize..4,
        prefix in "[a-z ]{0,5}",
        suffix in "[ a-z0-9\"]{0,10}",
        cursor_offset in 0usize..3
    ) {
        let operators = ["//", "|=", "//=", ".."];
        let op = operators[op_index];

        let query = format!("{}{}{}", prefix, op, suffix);
        let op_start = prefix.len();
        let op_len = op.len();

        // Skip if suffix starts with characters that would extend the operator
        // (e.g., suffix starting with '.' would turn '..' into '...')
        if op == ".." && suffix.starts_with('.') {
            return Ok(());
        }
        // Skip if suffix starts with '=' which would turn '//' into '//='
        if op == "//" && suffix.starts_with('=') {
            return Ok(());
        }

        // Test cursor on each character of the operator
        if cursor_offset < op_len {
            let cursor_pos = op_start + cursor_offset;
            let result = detect_operator_at_cursor(&query, cursor_pos);
            prop_assert_eq!(
                result,
                Some(op),
                "Cursor at position {} in '{}' should detect operator '{}'",
                cursor_pos,
                query,
                op
            );
        }
    }

    // **Feature: operator-tooltips, Property 2: No false positives on similar characters**
    // *For any* query string containing single characters that resemble operators
    // (single `/` for division, single `|` for pipe, single `.` for field access),
    // the detection function SHALL return None when cursor is on those characters.
    // **Validates: Requirements 1.2, 2.2, 4.2**
    #[test]
    fn prop_no_false_positives_single_chars(
        char_index in 0usize..3,
        prefix in "[a-z0-9]{1,5}",
        suffix in "[a-z0-9]{1,5}"
    ) {
        let single_chars = ['/', '|', '.'];
        let single_char = single_chars[char_index];

        // Build query with single character that should NOT be detected as operator
        let query = format!("{}{}{}", prefix, single_char, suffix);
        let char_pos = prefix.len();

        let result = detect_operator_at_cursor(&query, char_pos);
        prop_assert_eq!(
            result,
            None,
            "Single '{}' at position {} in '{}' should NOT be detected as operator",
            single_char,
            char_pos,
            query
        );
    }

    // **Feature: operator-tooltips, Property 3: Multi-char operator detection order**
    // *For any* query containing `//=`, when cursor is on any of the three characters,
    // the detection function SHALL return `//=` (not `//`).
    // **Validates: Requirements 3.2**
    #[test]
    fn prop_multi_char_detection_order(
        prefix in "[a-z ]{0,5}",
        suffix in "[ a-z0-9]{0,5}",
        cursor_offset in 0usize..3
    ) {
        // Test that //= is detected correctly (not as //)
        let query = format!("{}//={}", prefix, suffix);
        let op_start = prefix.len();

        let cursor_pos = op_start + cursor_offset;
        let result = detect_operator_at_cursor(&query, cursor_pos);
        prop_assert_eq!(
            result,
            Some("//="),
            "Cursor at position {} in '{}' should detect '//=' not '//'",
            cursor_pos,
            query
        );
    }
}
