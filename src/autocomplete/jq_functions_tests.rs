//! Tests for jq function metadata and filtering

use super::*;
use proptest::prelude::*;

// Helper function to get functions requiring arguments
fn get_functions_requiring_args() -> Vec<&'static JqFunction> {
    JQ_FUNCTION_METADATA
        .iter()
        .filter(|f| f.needs_parens)
        .collect()
}

// Helper function to get functions not requiring arguments
fn get_functions_not_requiring_args() -> Vec<&'static JqFunction> {
    JQ_FUNCTION_METADATA
        .iter()
        .filter(|f| !f.needs_parens)
        .collect()
}

#[test]
fn test_metadata_list_not_empty() {
    let metadata = get_all_function_metadata();
    assert!(
        !metadata.is_empty(),
        "JQ_FUNCTION_METADATA should not be empty"
    );
}

#[test]
fn test_functions_requiring_args_have_needs_parens_true() {
    // Verify specific functions from requirements 3.2 have needs_parens = true
    let functions_requiring_args = [
        "map",
        "select",
        "sort_by",
        "group_by",
        "unique_by",
        "min_by",
        "max_by",
        "has",
        "contains",
        "test",
        "match",
        "split",
        "join",
        "sub",
        "gsub",
        "with_entries",
        "recurse",
        "walk",
        "until",
        "while",
        "limit",
        "nth",
        "range",
        "getpath",
        "setpath",
        "delpaths",
        "del",
        "ltrimstr",
        "rtrimstr",
        "startswith",
        "endswith",
        "inside",
        "index",
        "rindex",
        "indices",
        "capture",
        "scan",
        "splits",
        "strftime",
        "strptime",
    ];

    for name in functions_requiring_args {
        let func = JQ_FUNCTION_METADATA.iter().find(|f| f.name == name);
        assert!(
            func.is_some(),
            "Function '{}' should be in JQ_FUNCTION_METADATA",
            name
        );
        assert!(
            func.unwrap().needs_parens,
            "Function '{}' should have needs_parens = true",
            name
        );
    }
}

#[test]
fn test_functions_not_requiring_args_have_needs_parens_false() {
    // Verify specific functions from requirements 3.3 have needs_parens = false
    let functions_not_requiring_args = [
        "keys",
        "keys_unsorted",
        "values",
        "sort",
        "reverse",
        "unique",
        "flatten",
        "add",
        "length",
        "first",
        "last",
        "min",
        "max",
        "type",
        "tostring",
        "tonumber",
        "floor",
        "ceil",
        "round",
        "sqrt",
        "abs",
        "now",
        "empty",
        "error",
        "not",
        "ascii_downcase",
        "ascii_upcase",
        "arrays",
        "objects",
        "iterables",
        "booleans",
        "numbers",
        "strings",
        "nulls",
        "scalars",
        "to_entries",
        "from_entries",
        "paths",
        "leaf_paths",
        "transpose",
        "env",
        "fromdate",
        "todate",
    ];

    for name in functions_not_requiring_args {
        let func = JQ_FUNCTION_METADATA.iter().find(|f| f.name == name);
        assert!(
            func.is_some(),
            "Function '{}' should be in JQ_FUNCTION_METADATA",
            name
        );
        assert!(
            !func.unwrap().needs_parens,
            "Function '{}' should have needs_parens = false",
            name
        );
    }
}

proptest! {
    // **Feature: enhanced-autocomplete, Property 6: All functions have complete metadata**
    // *For any* function in the jq builtins list, the function SHALL have both
    // a `signature` field and a `needs_parens` field defined.
    // **Validates: Requirements 3.1**
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop_all_functions_have_complete_metadata(index in 0usize..JQ_FUNCTION_METADATA.len().max(1)) {
        // Skip test if metadata list is empty (will be populated in task 2)
        if JQ_FUNCTION_METADATA.is_empty() {
            return Ok(());
        }

        let func = &JQ_FUNCTION_METADATA[index % JQ_FUNCTION_METADATA.len()];

        // Verify name is not empty
        prop_assert!(!func.name.is_empty(), "Function name should not be empty");

        // Verify signature is not empty
        prop_assert!(!func.signature.is_empty(), "Function {} should have a non-empty signature", func.name);

        // Verify description is not empty
        prop_assert!(!func.description.is_empty(), "Function {} should have a non-empty description", func.name);

        // Verify signature contains the function name
        prop_assert!(
            func.signature.starts_with(func.name),
            "Function {} signature '{}' should start with the function name",
            func.name,
            func.signature
        );
    }

    // **Feature: enhanced-autocomplete, Property 4: Signature format for argument functions**
    // *For any* jq function marked with `needs_parens = true`, the signature field SHALL
    // match the pattern `name(...)` where `name` is the function name and `...` represents
    // one or more parameter indicators.
    // **Validates: Requirements 2.2**
    #[test]
    fn prop_signature_format_for_argument_functions(index in 0usize..100) {
        let funcs = get_functions_requiring_args();
        if funcs.is_empty() {
            return Ok(());
        }

        let func = funcs[index % funcs.len()];

        // Verify signature starts with function name
        prop_assert!(
            func.signature.starts_with(func.name),
            "Function {} signature '{}' should start with the function name",
            func.name,
            func.signature
        );

        // Verify signature contains opening parenthesis after the name
        let after_name = &func.signature[func.name.len()..];
        prop_assert!(
            after_name.starts_with('('),
            "Function {} signature '{}' should have '(' immediately after the name",
            func.name,
            func.signature
        );

        // Verify signature contains closing parenthesis
        prop_assert!(
            func.signature.ends_with(')'),
            "Function {} signature '{}' should end with ')'",
            func.name,
            func.signature
        );

        // Verify there's content between parentheses (parameter indicators)
        let paren_start = func.signature.find('(').unwrap();
        let paren_end = func.signature.rfind(')').unwrap();
        prop_assert!(
            paren_end > paren_start + 1,
            "Function {} signature '{}' should have parameter indicators between parentheses",
            func.name,
            func.signature
        );
    }

    // **Feature: enhanced-autocomplete, Property 5: Signature format for no-argument functions**
    // *For any* jq function marked with `needs_parens = false`, the signature field SHALL
    // equal the function name without any parentheses.
    // **Validates: Requirements 2.3**
    #[test]
    fn prop_signature_format_for_no_argument_functions(index in 0usize..100) {
        let funcs = get_functions_not_requiring_args();
        if funcs.is_empty() {
            return Ok(());
        }

        let func = funcs[index % funcs.len()];

        // Verify signature equals the function name exactly
        prop_assert!(
            func.signature == func.name,
            "Function {} with needs_parens=false should have signature equal to name, but got '{}'",
            func.name,
            func.signature
        );

        // Verify signature does not contain parentheses
        prop_assert!(
            !func.signature.contains('('),
            "Function {} signature '{}' should not contain '('",
            func.name,
            func.signature
        );

        prop_assert!(
            !func.signature.contains(')'),
            "Function {} signature '{}' should not contain ')'",
            func.name,
            func.signature
        );
    }
}
