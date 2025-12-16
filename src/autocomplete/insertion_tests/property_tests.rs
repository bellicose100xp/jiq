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

// **Feature: enhanced-autocomplete, Property 1: Functions requiring arguments get parenthesis appended**
// *For any* jq function marked with `needs_parens = true`, when that function is inserted
// via Tab completion, the resulting query text SHALL end with the function name followed
// by an opening parenthesis `(`.
// **Validates: Requirements 1.1**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop_functions_requiring_args_get_parenthesis(index in 0usize..100) {
        let funcs = get_functions_requiring_args();
        if funcs.is_empty() {
            return Ok(());
        }

        let func = funcs[index % funcs.len()];

        // Create a suggestion with needs_parens = true
        let suggestion = Suggestion::new(func.name, SuggestionType::Function)
            .with_needs_parens(true)
            .with_signature(func.signature);

        // Set up test environment with a partial query that would trigger function context
        // e.g., typing "sel" should complete to "select("
        let partial = &func.name[..func.name.len().min(3)];
        let (mut textarea, mut query_state) = setup_insertion_test(partial);

        // Insert the suggestion
        insert_suggestion(&mut textarea, &mut query_state, &suggestion);

        // Verify the result ends with function name followed by (
        let result = textarea.lines()[0].clone();
        let expected_suffix = format!("{}(", func.name);

        prop_assert!(
            result.ends_with(&expected_suffix),
            "Function '{}' with needs_parens=true should result in '{}' but got '{}'",
            func.name,
            expected_suffix,
            result
        );
    }
}

// **Feature: enhanced-autocomplete, Property 2: Functions not requiring arguments get no parenthesis**
// *For any* jq function marked with `needs_parens = false`, when that function is inserted
// via Tab completion, the resulting query text SHALL contain only the function name
// without any trailing parenthesis.
// **Validates: Requirements 1.2**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop_functions_not_requiring_args_get_no_parenthesis(index in 0usize..100) {
        let funcs = get_functions_not_requiring_args();
        if funcs.is_empty() {
            return Ok(());
        }

        let func = funcs[index % funcs.len()];

        // Create a suggestion with needs_parens = false
        let suggestion = Suggestion::new(func.name, SuggestionType::Function)
            .with_needs_parens(false)
            .with_signature(func.signature);

        // Set up test environment with a partial query
        let partial = &func.name[..func.name.len().min(3)];
        let (mut textarea, mut query_state) = setup_insertion_test(partial);

        // Insert the suggestion
        insert_suggestion(&mut textarea, &mut query_state, &suggestion);

        // Verify the result ends with function name (no parenthesis)
        let result = textarea.lines()[0].clone();

        prop_assert!(
            result.ends_with(func.name),
            "Function '{}' with needs_parens=false should end with '{}' but got '{}'",
            func.name,
            func.name,
            result
        );

        // Also verify it does NOT end with (
        prop_assert!(
            !result.ends_with(&format!("{}(", func.name)),
            "Function '{}' with needs_parens=false should NOT have '(' appended, but got '{}'",
            func.name,
            result
        );
    }
}

// **Feature: enhanced-autocomplete, Property 3: Cursor positioned after parenthesis for argument functions**
// *For any* jq function marked with `needs_parens = true`, after Tab insertion, the cursor
// position SHALL equal the length of the inserted text (function name + opening parenthesis).
// **Validates: Requirements 1.3**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop_cursor_positioned_after_parenthesis(index in 0usize..100) {
        let funcs = get_functions_requiring_args();
        if funcs.is_empty() {
            return Ok(());
        }

        let func = funcs[index % funcs.len()];

        // Create a suggestion with needs_parens = true
        let suggestion = Suggestion::new(func.name, SuggestionType::Function)
            .with_needs_parens(true)
            .with_signature(func.signature);

        // Set up test environment with a partial query
        let partial = &func.name[..func.name.len().min(3)];
        let (mut textarea, mut query_state) = setup_insertion_test(partial);

        // Insert the suggestion
        insert_suggestion(&mut textarea, &mut query_state, &suggestion);

        // Verify cursor position is at the end of the inserted text
        let result = textarea.lines()[0].clone();
        let cursor_col = textarea.cursor().1;
        let expected_cursor_pos = result.len();

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
