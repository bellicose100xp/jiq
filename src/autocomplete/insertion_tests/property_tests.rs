//! Property-based tests for insertion behavior

use super::*;
use crate::autocomplete::jq_functions::JQ_FUNCTION_METADATA;
use proptest::prelude::*;

// Helper function to get functions requiring arguments
fn get_functions_requiring_args() -> Vec<&'static crate::autocomplete::jq_functions::JqFunction> {
    JQ_FUNCTION_METADATA
        .iter()
        .filter(|f| f.needs_parens)
        .collect()
}

// Helper function to get functions not requiring arguments
fn get_functions_not_requiring_args() -> Vec<&'static crate::autocomplete::jq_functions::JqFunction>
{
    JQ_FUNCTION_METADATA
        .iter()
        .filter(|f| !f.needs_parens)
        .collect()
}

#[test]
fn test_all_functions_requiring_args_get_parenthesis() {
    let funcs = get_functions_requiring_args();
    assert!(
        !funcs.is_empty(),
        "Should have functions requiring arguments"
    );

    for func in funcs {
        let suggestion = Suggestion::new(func.name, SuggestionType::Function)
            .with_needs_parens(true)
            .with_signature(func.signature);

        let partial = &func.name[..func.name.len().min(3)];
        let (mut textarea, mut query_state) = setup_insertion_test(partial);

        insert_suggestion(&mut textarea, &mut query_state, &suggestion);

        let result = textarea.lines()[0].clone();
        let expected_suffix = format!("{}(", func.name);

        assert!(
            result.ends_with(&expected_suffix),
            "Function '{}' with needs_parens=true should result in '{}' but got '{}'",
            func.name,
            expected_suffix,
            result
        );
    }
}

#[test]
fn test_all_functions_not_requiring_args_get_no_parenthesis() {
    let funcs = get_functions_not_requiring_args();
    assert!(
        !funcs.is_empty(),
        "Should have functions not requiring arguments"
    );

    for func in funcs {
        let suggestion = Suggestion::new(func.name, SuggestionType::Function)
            .with_needs_parens(false)
            .with_signature(func.signature);

        let partial = &func.name[..func.name.len().min(3)];
        let (mut textarea, mut query_state) = setup_insertion_test(partial);

        insert_suggestion(&mut textarea, &mut query_state, &suggestion);

        let result = textarea.lines()[0].clone();

        assert!(
            result.ends_with(func.name),
            "Function '{}' with needs_parens=false should end with '{}' but got '{}'",
            func.name,
            func.name,
            result
        );

        assert!(
            !result.ends_with(&format!("{}(", func.name)),
            "Function '{}' with needs_parens=false should NOT have '(' appended, but got '{}'",
            func.name,
            result
        );
    }
}

#[test]
fn test_all_functions_cursor_positioned_correctly() {
    let funcs = get_functions_requiring_args();
    assert!(
        !funcs.is_empty(),
        "Should have functions requiring arguments"
    );

    for func in funcs {
        let suggestion = Suggestion::new(func.name, SuggestionType::Function)
            .with_needs_parens(true)
            .with_signature(func.signature);

        let partial = &func.name[..func.name.len().min(3)];
        let (mut textarea, mut query_state) = setup_insertion_test(partial);

        insert_suggestion(&mut textarea, &mut query_state, &suggestion);

        let result = textarea.lines()[0].clone();
        let cursor_col = textarea.cursor().1;
        let expected_cursor_pos = result.len();

        assert_eq!(
            cursor_col, expected_cursor_pos,
            "Cursor should be at position {} (end of '{}') but was at {}",
            expected_cursor_pos, result, cursor_col
        );
    }
}

// **Feature: object-key-autocomplete, Property 5: ObjectKeyContext insertion replaces partial correctly**
// *For any* ObjectKeyContext suggestion accepted via Tab, the resulting query SHALL contain
// the suggestion text at the position where the partial was, with no duplicate characters.
// **Validates: Requirements 1.5**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop_object_key_context_insertion_replaces_partial(
        has_prefix in prop::bool::ANY,          // Whether to include existing key-value pairs
        prefix_key in "[a-z]{2,6}",             // Key name for prefix (if used)
        partial in "[a-z]{1,4}",                // Partial being typed
        suffix in "[a-z]{1,6}",                 // Suffix to append to partial to form suggestion
    ) {
        // Build the suggestion by appending suffix to partial (ensures suggestion starts with partial)
        let suggestion = format!("{}{}", partial, suffix);

        // Build initial query: `{` or `{key: .key, ` followed by partial
        let prefix = if has_prefix {
            format!("{{{}: .{}, ", prefix_key, prefix_key)
        } else {
            "{".to_string()
        };
        let initial_query = format!("{}{}", prefix, partial);
        let (mut textarea, mut query_state) = setup_insertion_test(&initial_query);

        // Create a field suggestion (ObjectKeyContext suggestions are field names without dots)
        let suggestion_obj = Suggestion::new(&suggestion, SuggestionType::Field);

        // Insert the suggestion
        insert_suggestion(&mut textarea, &mut query_state, &suggestion_obj);

        // Get the result
        let result = textarea.lines()[0].clone();

        // Verify: the result should contain the suggestion at the right position
        // The partial should be replaced by the suggestion, not duplicated
        let expected = format!("{}{}", prefix, suggestion);
        prop_assert_eq!(
            result.clone(),
            expected.clone(),
            "ObjectKeyContext insertion should replace partial '{}' with suggestion '{}'. Initial: '{}', Expected: '{}', Got: '{}'",
            partial,
            suggestion,
            initial_query,
            expected,
            result
        );

        // Verify: cursor should be positioned after the inserted suggestion
        let cursor_col = textarea.cursor().1;
        let expected_cursor_pos = expected.len();
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
// ObjectKeyContext Insertion Unit Tests
// ============================================================================
// These tests verify ObjectKeyContext insertion behavior per Requirements 1.5

#[test]
fn test_object_key_context_insertion_simple() {
    // Test: `{na` + accept "name" → `{name`
    // This tests the basic ObjectKeyContext insertion after opening brace
    let initial_query = "{na";
    let (mut textarea, mut query_state) = setup_insertion_test(initial_query);

    // Create a field suggestion (ObjectKeyContext suggestions are field names without dots)
    let suggestion = Suggestion::new("name", SuggestionType::Field);

    // Insert the suggestion
    insert_suggestion(&mut textarea, &mut query_state, &suggestion);

    // Verify the result
    let result = textarea.lines()[0].clone();
    assert_eq!(
        result, "{name",
        "ObjectKeyContext insertion should replace 'na' with 'name'. Got: '{}'",
        result
    );

    // Verify cursor position is at the end
    let cursor_col = textarea.cursor().1;
    assert_eq!(
        cursor_col, 5,
        "Cursor should be at position 5 (end of '{{name')"
    );
}

#[test]
fn test_object_key_context_insertion_after_comma() {
    // Test: `{name: .name, ag` + accept "age" → `{name: .name, age`
    // This tests ObjectKeyContext insertion after comma in object literal
    let initial_query = "{name: .name, ag";
    let (mut textarea, mut query_state) = setup_insertion_test(initial_query);

    // Create a field suggestion
    let suggestion = Suggestion::new("age", SuggestionType::Field);

    // Insert the suggestion
    insert_suggestion(&mut textarea, &mut query_state, &suggestion);

    // Verify the result
    let result = textarea.lines()[0].clone();
    assert_eq!(
        result, "{name: .name, age",
        "ObjectKeyContext insertion should replace 'ag' with 'age'. Got: '{}'",
        result
    );

    // Verify cursor position is at the end
    let cursor_col = textarea.cursor().1;
    assert_eq!(
        cursor_col, 17,
        "Cursor should be at position 17 (end of '{{name: .name, age')"
    );
}

#[test]
fn test_object_key_context_insertion_with_space_after_comma() {
    // Test: `{name: .name, ag` (with space after comma) + accept "age" → `{name: .name, age`
    // This tests that spaces are preserved correctly
    let initial_query = "{name: .name, ag";
    let (mut textarea, mut query_state) = setup_insertion_test(initial_query);

    let suggestion = Suggestion::new("age", SuggestionType::Field);
    insert_suggestion(&mut textarea, &mut query_state, &suggestion);

    let result = textarea.lines()[0].clone();
    assert_eq!(result, "{name: .name, age");
}

#[test]
fn test_object_key_context_insertion_nested_object() {
    // Test: `{outer: {in` + accept "inner" → `{outer: {inner`
    // This tests ObjectKeyContext in nested object
    let initial_query = "{outer: {in";
    let (mut textarea, mut query_state) = setup_insertion_test(initial_query);

    let suggestion = Suggestion::new("inner", SuggestionType::Field);
    insert_suggestion(&mut textarea, &mut query_state, &suggestion);

    let result = textarea.lines()[0].clone();
    assert_eq!(
        result, "{outer: {inner",
        "ObjectKeyContext insertion in nested object should work. Got: '{}'",
        result
    );
}

#[test]
fn test_object_key_context_insertion_longer_partial() {
    // Test: `{servi` + accept "services" → `{services`
    // This tests with a longer partial
    let initial_query = "{servi";
    let (mut textarea, mut query_state) = setup_insertion_test(initial_query);

    let suggestion = Suggestion::new("services", SuggestionType::Field);
    insert_suggestion(&mut textarea, &mut query_state, &suggestion);

    let result = textarea.lines()[0].clone();
    assert_eq!(
        result, "{services",
        "ObjectKeyContext insertion with longer partial should work. Got: '{}'",
        result
    );
}

#[test]
fn test_object_key_context_insertion_single_char_partial() {
    // Test: `{n` + accept "name" → `{name`
    // This tests with a single character partial
    let initial_query = "{n";
    let (mut textarea, mut query_state) = setup_insertion_test(initial_query);

    let suggestion = Suggestion::new("name", SuggestionType::Field);
    insert_suggestion(&mut textarea, &mut query_state, &suggestion);

    let result = textarea.lines()[0].clone();
    assert_eq!(
        result, "{name",
        "ObjectKeyContext insertion with single char partial should work. Got: '{}'",
        result
    );
}
