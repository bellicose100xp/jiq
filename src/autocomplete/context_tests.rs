//! Tests for autocomplete context analysis

use super::*;
use proptest::prelude::*;
use serde_json::Value;
use std::sync::Arc;

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
    assert_eq!(
        find_char_before_field_access(".services | .", ""),
        Some('|')
    );
    // `.services | .ser` - should find '|' (go back past partial)
    assert_eq!(
        find_char_before_field_access(".services | .ser", "ser"),
        Some('|')
    );
}

#[test]
fn test_char_before_field_after_dot() {
    // `.services.` - should find 's' (last char of identifier)
    assert_eq!(find_char_before_field_access(".services.", ""), Some('s'));
    // `.services.na` - should find 's' (go back past partial and dot)
    assert_eq!(
        find_char_before_field_access(".services.na", "na"),
        Some('s')
    );
}

#[test]
fn test_char_before_field_after_brackets() {
    // `.services[].` - should find ']'
    assert_eq!(find_char_before_field_access(".services[].", ""), Some(']'));
    // `.services[0].` - should find ']'
    assert_eq!(
        find_char_before_field_access(".services[0].", ""),
        Some(']')
    );
}

#[test]
fn test_char_before_field_after_question() {
    // `.services?.` - should find '?'
    assert_eq!(find_char_before_field_access(".services?.", ""), Some('?'));
    // `.services?.na` - should find '?'
    assert_eq!(
        find_char_before_field_access(".services?.na", "na"),
        Some('?')
    );
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
    assert_eq!(
        find_char_before_field_access("select(.active).", ""),
        Some(')')
    );
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

#[test]
fn test_object_key_context_after_open_brace() {
    let query = "{na";
    let tracker = tracker_for(query);
    let (ctx, partial) = analyze_context(query, &tracker);
    assert_eq!(ctx, SuggestionContext::ObjectKeyContext);
    assert_eq!(partial, "na");
}

#[test]
fn test_object_key_context_after_comma() {
    let query = "{name: .name, ag";
    let tracker = tracker_for(query);
    let (ctx, partial) = analyze_context(query, &tracker);
    assert_eq!(ctx, SuggestionContext::ObjectKeyContext);
    assert_eq!(partial, "ag");
}

#[test]
fn test_array_context_not_object_key() {
    let query = "[1, na";
    let tracker = tracker_for(query);
    let (ctx, partial) = analyze_context(query, &tracker);
    assert_ne!(ctx, SuggestionContext::ObjectKeyContext);
    assert_eq!(partial, "na");
    assert_eq!(ctx, SuggestionContext::FunctionContext);
}

#[test]
fn test_function_call_context_not_object_key() {
    let query = "select(.a, na";
    let tracker = tracker_for(query);
    let (ctx, partial) = analyze_context(query, &tracker);
    assert_ne!(ctx, SuggestionContext::ObjectKeyContext);
    assert_eq!(partial, "na");
    assert_eq!(ctx, SuggestionContext::FunctionContext);
}

#[test]
fn test_nested_object_in_array() {
    let query = "[{na";
    let tracker = tracker_for(query);
    let (ctx, partial) = analyze_context(query, &tracker);
    assert_eq!(ctx, SuggestionContext::ObjectKeyContext);
    assert_eq!(partial, "na");
}

#[test]
fn test_nested_array_in_object() {
    let query = "{items: [na";
    let tracker = tracker_for(query);
    let (ctx, partial) = analyze_context(query, &tracker);
    assert_ne!(ctx, SuggestionContext::ObjectKeyContext);
    assert_eq!(partial, "na");
    assert_eq!(ctx, SuggestionContext::FunctionContext);
}

#[test]
fn test_object_key_empty_partial_no_suggestions() {
    let query = "{";
    let tracker = tracker_for(query);
    let (ctx, partial) = analyze_context(query, &tracker);
    assert_ne!(ctx, SuggestionContext::ObjectKeyContext);
    assert_eq!(partial, "");
}

#[test]
fn test_object_key_after_comma_empty_partial() {
    let query = "{name: .name, ";
    let tracker = tracker_for(query);
    let (ctx, _partial) = analyze_context(query, &tracker);
    assert_ne!(ctx, SuggestionContext::ObjectKeyContext);
}

#[test]
fn test_dot_after_brace_is_field_context() {
    let query = "{.na";
    let tracker = tracker_for(query);
    let (ctx, partial) = analyze_context(query, &tracker);
    assert_eq!(ctx, SuggestionContext::FieldContext);
    assert_eq!(partial, "na");
}

#[test]
fn test_object_key_with_complex_value() {
    let query = "{name: .name | map(.), ag";
    let tracker = tracker_for(query);
    let (ctx, partial) = analyze_context(query, &tracker);
    assert_eq!(ctx, SuggestionContext::ObjectKeyContext);
    assert_eq!(partial, "ag");
}

#[test]
fn test_deeply_nested_object_context() {
    let query = "{a: {b: {c";
    let tracker = tracker_for(query);
    let (ctx, partial) = analyze_context(query, &tracker);
    assert_eq!(ctx, SuggestionContext::ObjectKeyContext);
    assert_eq!(partial, "c");
}

#[test]
fn test_regression_field_context_at_start() {
    let query = ".na";
    let tracker = tracker_for(query);
    let (ctx, partial) = analyze_context(query, &tracker);
    assert_eq!(ctx, SuggestionContext::FieldContext);
    assert_ne!(ctx, SuggestionContext::ObjectKeyContext);
    assert_eq!(partial, "na");
}

#[test]
fn test_regression_field_context_after_pipe() {
    let query = ".services | .na";
    let tracker = tracker_for(query);
    let (ctx, partial) = analyze_context(query, &tracker);
    assert_eq!(ctx, SuggestionContext::FieldContext);
    assert_ne!(ctx, SuggestionContext::ObjectKeyContext);
    assert_eq!(partial, "na");
}

#[test]
fn test_regression_field_context_in_map() {
    let query = "map(.na";
    let tracker = tracker_for(query);
    let (ctx, partial) = analyze_context(query, &tracker);
    assert_eq!(ctx, SuggestionContext::FieldContext);
    assert_ne!(ctx, SuggestionContext::ObjectKeyContext);
    assert_eq!(partial, "na");
}

#[test]
fn test_regression_function_context_at_start() {
    let query = "sel";
    let tracker = tracker_for(query);
    let (ctx, partial) = analyze_context(query, &tracker);
    assert_eq!(ctx, SuggestionContext::FunctionContext);
    assert_ne!(ctx, SuggestionContext::ObjectKeyContext);
    assert_eq!(partial, "sel");
}

#[test]
fn test_regression_function_context_after_pipe() {
    let query = ".services | sel";
    let tracker = tracker_for(query);
    let (ctx, partial) = analyze_context(query, &tracker);
    assert_eq!(ctx, SuggestionContext::FunctionContext);
    assert_ne!(ctx, SuggestionContext::ObjectKeyContext);
    assert_eq!(partial, "sel");
}

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
        let parsed = serde_json::from_str::<Value>(&json_result).ok().map(Arc::new);
        let suggestions = get_suggestions(
            &query,
            query.len(),
            parsed,
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
        let parsed = serde_json::from_str::<Value>(&json_result).ok().map(Arc::new);
        let suggestions = get_suggestions(
            &query,
            query.len(),
            parsed,
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
