//! Tests for brace tracking in autocomplete

use super::*;
use proptest::prelude::*;

#[test]
fn test_empty_query() {
    let mut tracker = BraceTracker::new();
    tracker.rebuild("");
    assert_eq!(tracker.context_at(0), None);
    assert!(!tracker.is_in_object(0));
}

#[test]
fn test_simple_object() {
    let mut tracker = BraceTracker::new();
    tracker.rebuild("{name");
    assert_eq!(tracker.context_at(1), Some(BraceType::Curly));
    assert!(tracker.is_in_object(1));
    assert!(tracker.is_in_object(5));
}

#[test]
fn test_simple_array() {
    let mut tracker = BraceTracker::new();
    tracker.rebuild("[1, 2");
    assert_eq!(tracker.context_at(1), Some(BraceType::Square));
    assert!(!tracker.is_in_object(1));
}

#[test]
fn test_simple_paren() {
    let mut tracker = BraceTracker::new();
    tracker.rebuild("map(");
    assert_eq!(tracker.context_at(4), Some(BraceType::Paren));
    assert!(!tracker.is_in_object(4));
}

#[test]
fn test_closed_braces() {
    let mut tracker = BraceTracker::new();
    tracker.rebuild("{name: .name}");
    // After the closing brace, we're no longer in object context
    assert_eq!(tracker.context_at(13), None);
}

#[test]
fn test_object_in_array() {
    let mut tracker = BraceTracker::new();
    tracker.rebuild("[{na");
    // Position 2 is inside the object (after '{')
    assert_eq!(tracker.context_at(2), Some(BraceType::Curly));
    assert!(tracker.is_in_object(2));
}

#[test]
fn test_array_in_object() {
    let mut tracker = BraceTracker::new();
    tracker.rebuild("{items: [na");
    // Position 9 is inside the array (after '[')
    assert_eq!(tracker.context_at(9), Some(BraceType::Square));
    assert!(!tracker.is_in_object(9));
}

#[test]
fn test_deep_nesting() {
    let mut tracker = BraceTracker::new();
    tracker.rebuild("{a: [{b: (c");
    // Position 10 is inside the paren
    assert_eq!(tracker.context_at(10), Some(BraceType::Paren));
    assert!(!tracker.is_in_object(10));
}

#[test]
fn test_braces_in_string() {
    let mut tracker = BraceTracker::new();
    tracker.rebuild("\"{braces}\"");
    // Braces inside string should be ignored
    assert_eq!(tracker.context_at(5), None);
    assert!(!tracker.is_in_object(5));
}

#[test]
fn test_escaped_quote_in_string() {
    let mut tracker = BraceTracker::new();
    tracker.rebuild("\"say \\\"hi\\\" {here\"");
    // The { is inside the string, should be ignored
    assert_eq!(tracker.context_at(12), None);
}

#[test]
fn test_escaped_backslash_in_string() {
    let mut tracker = BraceTracker::new();
    tracker.rebuild("\"path\\\\{dir\"");
    // The { is inside the string after \\, should be ignored
    assert_eq!(tracker.context_at(8), None);
}

#[test]
fn test_string_then_real_braces() {
    let mut tracker = BraceTracker::new();
    tracker.rebuild("\"{fake}\" | {real");
    // Position 12 is inside the real object
    assert_eq!(tracker.context_at(12), Some(BraceType::Curly));
    assert!(tracker.is_in_object(12));
}

#[test]
fn test_object_key_after_comma() {
    let mut tracker = BraceTracker::new();
    tracker.rebuild("{name: .name, ag");
    // Position 14 is still inside the object
    assert!(tracker.is_in_object(14));
}

#[test]
fn test_real_jq_pattern_select() {
    let mut tracker = BraceTracker::new();
    tracker.rebuild("select(.active)");
    // After the closing paren, no context
    assert_eq!(tracker.context_at(15), None);
}

#[test]
fn test_real_jq_pattern_map() {
    let mut tracker = BraceTracker::new();
    tracker.rebuild("map({na");
    assert!(tracker.is_in_object(5));

    tracker.rebuild("map({name: .name})");
    assert_eq!(tracker.context_at(18), None);
    assert_eq!(tracker.context_at(5), None);
}

#[test]
fn test_mismatched_braces() {
    let mut tracker = BraceTracker::new();
    tracker.rebuild("{test]");
    assert!(tracker.is_in_object(5));
}

#[test]
fn test_unclosed_string() {
    let mut tracker = BraceTracker::new();
    tracker.rebuild("\"unclosed {");
    assert_eq!(tracker.context_at(10), None);
}

#[test]
fn test_is_stale() {
    let mut tracker = BraceTracker::new();
    tracker.rebuild("{test");
    assert!(!tracker.is_stale("{test"));
    assert!(tracker.is_stale("{test2"));
    assert!(tracker.is_stale(""));
}

#[test]
fn test_context_at_position_zero() {
    let mut tracker = BraceTracker::new();
    tracker.rebuild("{test");
    // Position 0 is before the opening brace
    assert_eq!(tracker.context_at(0), None);
}

proptest! {
    /// **Feature: object-key-autocomplete, Property 4: BraceTracker never panics**
    /// **Validates: Requirements 5.2, 5.3**
    ///
    /// For any arbitrary string input, calling rebuild() and context_at()
    /// shall not panic.
    #[test]
    fn prop_rebuild_never_panics(query in ".*") {
        let mut tracker = BraceTracker::new();
        tracker.rebuild(&query);
        // If we get here without panicking, the test passes
    }

    /// **Feature: object-key-autocomplete, Property 4: BraceTracker never panics**
    /// **Validates: Requirements 5.2, 5.3**
    ///
    /// For any position query on any string, context_at() shall not panic.
    #[test]
    fn prop_context_at_never_panics(query in ".*", pos in 0usize..1000) {
        let mut tracker = BraceTracker::new();
        tracker.rebuild(&query);
        let _ = tracker.context_at(pos);
        let _ = tracker.is_in_object(pos);
        // If we get here without panicking, the test passes
    }

    /// **Feature: object-key-autocomplete, Property 3: Braces inside strings are ignored**
    /// **Validates: Requirements 3.1, 3.2**
    ///
    /// For any query containing a string literal with braces inside,
    /// the BraceTracker shall report the same context as if those braces
    /// were not present.
    #[test]
    fn prop_string_braces_ignored(
        prefix in "[a-z .|]*",
        string_content in "[a-z{}\\[\\]()]*",
        suffix in "[a-z .|]*"
    ) {
        // Build query with braces inside a string
        let query_with_string_braces = format!("{}\"{}\"{}",  prefix, string_content, suffix);
        // Build query with empty string (no braces in string)
        let query_with_empty_string = format!("{}\"\"{}",  prefix, suffix);

        let mut tracker1 = BraceTracker::new();
        let mut tracker2 = BraceTracker::new();

        tracker1.rebuild(&query_with_string_braces);
        tracker2.rebuild(&query_with_empty_string);

        // The context after the string should be the same
        let pos_after_string1 = prefix.len() + string_content.len() + 2; // +2 for quotes
        let pos_after_string2 = prefix.len() + 2; // +2 for quotes

        prop_assert_eq!(
            tracker1.context_at(pos_after_string1),
            tracker2.context_at(pos_after_string2),
            "Context after string should be same regardless of braces inside string"
        );
    }

    /// **Feature: object-key-autocomplete, Property 4: BraceTracker never panics**
    /// **Validates: Requirements 5.2, 5.3**
    ///
    /// is_in_object should be consistent with context_at result.
    #[test]
    fn prop_is_in_object_consistent(query in ".*", pos in 0usize..500) {
        let mut tracker = BraceTracker::new();
        tracker.rebuild(&query);

        let context = tracker.context_at(pos);
        let is_object = tracker.is_in_object(pos);

        prop_assert_eq!(
            is_object,
            context == Some(BraceType::Curly),
            "is_in_object should match context_at == Curly"
        );
    }

    /// Element-context functions should always be detected
    #[test]
    fn prop_element_context_functions_always_detected(
        func in "(map|select|sort_by|group_by|unique_by|min_by|max_by|recurse|walk)",
        partial in "[a-z]{0,5}"
    ) {
        let query = format!("{}(.{}", func, partial);
        let mut tracker = BraceTracker::new();
        tracker.rebuild(&query);
        prop_assert!(
            tracker.is_in_element_context(query.len()),
            "Should detect element context in '{}' at position {}",
            query,
            query.len()
        );
    }

    /// Non-element functions should never trigger element context
    #[test]
    fn prop_non_element_functions_never_detected(
        func in "(limit|has|del|getpath|split|join|test|match)",
        partial in "[a-z]{0,5}"
    ) {
        let query = format!("{}(.{}", func, partial);
        let mut tracker = BraceTracker::new();
        tracker.rebuild(&query);
        prop_assert!(
            !tracker.is_in_element_context(query.len()),
            "Should NOT detect element context in '{}' at position {}",
            query,
            query.len()
        );
    }
}

// ============================================================================
// Element Context Detection Tests
// ============================================================================

#[test]
fn test_element_context_in_map() {
    let mut tracker = BraceTracker::new();
    tracker.rebuild("map(.");
    assert!(tracker.is_in_element_context(5));
}

#[test]
fn test_element_context_in_select() {
    let mut tracker = BraceTracker::new();
    tracker.rebuild("select(.");
    assert!(tracker.is_in_element_context(8));
}

#[test]
fn test_element_context_all_element_functions() {
    let functions = [
        "map",
        "select",
        "sort_by",
        "group_by",
        "unique_by",
        "min_by",
        "max_by",
        "recurse",
        "walk",
    ];
    for func in functions {
        let query = format!("{}(.field", func);
        let mut tracker = BraceTracker::new();
        tracker.rebuild(&query);
        assert!(
            tracker.is_in_element_context(query.len()),
            "Function '{}' should provide element context",
            func
        );
    }
}

#[test]
fn test_no_element_context_outside_function() {
    let mut tracker = BraceTracker::new();
    tracker.rebuild(".field");
    assert!(!tracker.is_in_element_context(6));
}

#[test]
fn test_no_element_context_in_limit() {
    let mut tracker = BraceTracker::new();
    tracker.rebuild("limit(5; .");
    assert!(!tracker.is_in_element_context(10));
}

#[test]
fn test_no_element_context_in_has() {
    let mut tracker = BraceTracker::new();
    tracker.rebuild("has(.");
    assert!(!tracker.is_in_element_context(5));
}

#[test]
fn test_element_context_nested_in_limit() {
    // map(limit(5; .field)) - cursor inside limit but map provides element context
    let mut tracker = BraceTracker::new();
    tracker.rebuild("map(limit(5; .");
    assert!(
        tracker.is_in_element_context(14),
        "Should detect element context from outer map even when inside limit"
    );
}

#[test]
fn test_element_context_with_object_inside() {
    // map({name: .name}) - cursor inside object construction
    let mut tracker = BraceTracker::new();
    tracker.rebuild("map({name: .");
    assert!(tracker.is_in_element_context(12));
}

#[test]
fn test_element_context_with_array_inside() {
    // map([.x, .y]) - cursor inside array construction
    let mut tracker = BraceTracker::new();
    tracker.rebuild("map([.");
    assert!(tracker.is_in_element_context(6));
}

#[test]
fn test_no_element_context_grouping_parens() {
    // (.x + .y) - just grouping, no function
    let mut tracker = BraceTracker::new();
    tracker.rebuild("(.x + .");
    assert!(!tracker.is_in_element_context(7));
}

#[test]
fn test_element_context_after_pipe_in_function() {
    // map(. | .field) - still inside map after pipe
    let mut tracker = BraceTracker::new();
    tracker.rebuild("map(. | .");
    assert!(tracker.is_in_element_context(10));
}

#[test]
fn test_element_context_string_with_paren() {
    // "(" | map(.field) - paren in string should be ignored
    let mut tracker = BraceTracker::new();
    tracker.rebuild("\"(\" | map(.");
    assert!(tracker.is_in_element_context(11));
}

#[test]
fn test_element_context_closed_function() {
    // map(.field) | . - after map is closed
    let mut tracker = BraceTracker::new();
    tracker.rebuild("map(.field) | .");
    assert!(!tracker.is_in_element_context(15));
}

#[test]
fn test_element_context_nested_element_functions() {
    // map(select(.active)) - nested element context functions
    let mut tracker = BraceTracker::new();
    tracker.rebuild("map(select(.");
    assert!(tracker.is_in_element_context(12));
}

#[test]
fn test_element_context_whitespace_before_paren() {
    // map (.field) - space between function and paren
    let mut tracker = BraceTracker::new();
    tracker.rebuild("map (.");
    assert!(tracker.is_in_element_context(6));
}

#[test]
fn test_no_element_context_unknown_function() {
    // myfunc(.field) - user-defined function not in metadata
    let mut tracker = BraceTracker::new();
    tracker.rebuild("myfunc(.");
    assert!(!tracker.is_in_element_context(8));
}

#[test]
fn test_function_context_enum_debug() {
    // Verify FunctionContext can be debugged
    let ctx = FunctionContext::ElementIterator("map");
    assert!(format!("{:?}", ctx).contains("ElementIterator"));
    assert!(format!("{:?}", ctx).contains("map"));
}

#[test]
fn test_brace_info_struct() {
    // Verify BraceInfo fields are accessible
    let info = BraceInfo {
        pos: 5,
        brace_type: BraceType::Paren,
        context: Some(FunctionContext::ElementIterator("select")),
    };
    assert_eq!(info.pos, 5);
    assert_eq!(info.brace_type, BraceType::Paren);
    assert!(matches!(
        info.context,
        Some(FunctionContext::ElementIterator("select"))
    ));
}

// ============================================================================
// With Entries Context Detection Tests
// ============================================================================

#[test]
fn test_with_entries_context_basic() {
    let mut tracker = BraceTracker::new();
    tracker.rebuild("with_entries(.");
    assert!(tracker.is_in_with_entries_context(14));
}

#[test]
fn test_with_entries_context_with_key() {
    let mut tracker = BraceTracker::new();
    tracker.rebuild("with_entries(.key");
    assert!(tracker.is_in_with_entries_context(17));
}

#[test]
fn test_with_entries_context_with_value() {
    let mut tracker = BraceTracker::new();
    tracker.rebuild("with_entries(.value");
    assert!(tracker.is_in_with_entries_context(19));
}

#[test]
fn test_with_entries_context_after_pipe() {
    let mut tracker = BraceTracker::new();
    tracker.rebuild("with_entries(. | .");
    assert!(tracker.is_in_with_entries_context(18));
}

#[test]
fn test_with_entries_context_with_select() {
    let mut tracker = BraceTracker::new();
    tracker.rebuild("with_entries(select(.");
    assert!(tracker.is_in_with_entries_context(21));
    assert!(tracker.is_in_element_context(21));
}

#[test]
fn test_with_entries_context_closed() {
    let mut tracker = BraceTracker::new();
    tracker.rebuild("with_entries(.key) | .");
    assert!(!tracker.is_in_with_entries_context(22));
}

#[test]
fn test_with_entries_context_whitespace_before_paren() {
    let mut tracker = BraceTracker::new();
    tracker.rebuild("with_entries (.");
    assert!(tracker.is_in_with_entries_context(15));
}

#[test]
fn test_with_entries_context_nested() {
    let mut tracker = BraceTracker::new();
    tracker.rebuild("with_entries(with_entries(.");
    assert!(tracker.is_in_with_entries_context(26));
}

#[test]
fn test_with_entries_not_element_context() {
    let mut tracker = BraceTracker::new();
    tracker.rebuild("with_entries(.");
    assert!(!tracker.is_in_element_context(14));
}

#[test]
fn test_with_entries_context_with_object_construction() {
    let mut tracker = BraceTracker::new();
    tracker.rebuild("with_entries({key: .key, value: .");
    assert!(tracker.is_in_with_entries_context(33));
}

#[test]
fn test_no_with_entries_context_outside() {
    let mut tracker = BraceTracker::new();
    tracker.rebuild(".field");
    assert!(!tracker.is_in_with_entries_context(6));
}

#[test]
fn test_function_context_with_entries_debug() {
    let ctx = FunctionContext::WithEntries;
    assert!(format!("{:?}", ctx).contains("WithEntries"));
}
