//! Tests for jq function metadata and filtering

use super::*;

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
        "tojson",
        "fromjson",
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

#[test]
fn test_all_functions_have_complete_metadata() {
    assert!(
        !JQ_FUNCTION_METADATA.is_empty(),
        "Function metadata should not be empty"
    );

    for func in JQ_FUNCTION_METADATA {
        assert!(!func.name.is_empty(), "Function name should not be empty");
        assert!(
            !func.signature.is_empty(),
            "Function {} should have a non-empty signature",
            func.name
        );
        assert!(
            !func.description.is_empty(),
            "Function {} should have a non-empty description",
            func.name
        );
        assert!(
            func.signature.starts_with(func.name),
            "Function {} signature '{}' should start with the function name",
            func.name,
            func.signature
        );
    }
}

#[test]
fn test_signature_format_for_all_argument_functions() {
    let funcs = get_functions_requiring_args();
    assert!(
        !funcs.is_empty(),
        "Should have functions requiring arguments"
    );

    for func in funcs {
        assert!(
            func.signature.starts_with(func.name),
            "Function {} signature '{}' should start with the function name",
            func.name,
            func.signature
        );

        let after_name = &func.signature[func.name.len()..];
        assert!(
            after_name.starts_with('('),
            "Function {} signature '{}' should have '(' immediately after the name",
            func.name,
            func.signature
        );

        assert!(
            func.signature.ends_with(')'),
            "Function {} signature '{}' should end with ')'",
            func.name,
            func.signature
        );

        let paren_start = func.signature.find('(').unwrap();
        let paren_end = func.signature.rfind(')').unwrap();
        assert!(
            paren_end > paren_start + 1,
            "Function {} signature '{}' should have parameter indicators between parentheses",
            func.name,
            func.signature
        );
    }
}

#[test]
fn test_signature_format_for_all_no_argument_functions() {
    let funcs = get_functions_not_requiring_args();
    assert!(
        !funcs.is_empty(),
        "Should have functions not requiring arguments"
    );

    for func in funcs {
        assert!(
            func.signature == func.name,
            "Function {} with needs_parens=false should have signature equal to name, but got '{}'",
            func.name,
            func.signature
        );

        assert!(
            !func.signature.contains('('),
            "Function {} signature '{}' should not contain '('",
            func.name,
            func.signature
        );

        assert!(
            !func.signature.contains(')'),
            "Function {} signature '{}' should not contain ')'",
            func.name,
            func.signature
        );
    }
}

#[test]
fn test_all_element_context_functions_recognized() {
    let element_funcs = [
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

    for func in element_funcs {
        assert!(
            is_element_context_function(func),
            "Function '{}' should be recognized as element context function",
            func
        );
    }
}

#[test]
fn test_all_non_element_functions_not_recognized() {
    let non_element_funcs = [
        "limit", "has", "del", "getpath", "split", "join", "test", "match", "keys", "values",
        "length", "first", "last",
    ];

    for func in non_element_funcs {
        assert!(
            !is_element_context_function(func),
            "Function '{}' should NOT be recognized as element context function",
            func
        );
    }
}

// ============================================================================
// Element Context Functions Tests
// ============================================================================

#[test]
fn test_element_context_functions_contains_map() {
    assert!(
        ELEMENT_CONTEXT_FUNCTIONS.contains("map"),
        "ELEMENT_CONTEXT_FUNCTIONS should contain 'map'"
    );
}

#[test]
fn test_element_context_functions_contains_all_expected() {
    let expected = [
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

    for func in expected {
        assert!(
            ELEMENT_CONTEXT_FUNCTIONS.contains(func),
            "ELEMENT_CONTEXT_FUNCTIONS should contain '{}'",
            func
        );
    }
}

#[test]
fn test_element_context_functions_excludes_non_element() {
    let non_element = [
        "limit", "has", "del", "getpath", "keys", "values", "length", "add",
    ];

    for func in non_element {
        assert!(
            !ELEMENT_CONTEXT_FUNCTIONS.contains(func),
            "ELEMENT_CONTEXT_FUNCTIONS should NOT contain '{}'",
            func
        );
    }
}

#[test]
fn test_is_element_context_function_helper() {
    assert!(is_element_context_function("map"));
    assert!(is_element_context_function("select"));
    assert!(is_element_context_function("sort_by"));
    assert!(is_element_context_function("group_by"));
    assert!(is_element_context_function("unique_by"));
    assert!(is_element_context_function("min_by"));
    assert!(is_element_context_function("max_by"));
    assert!(is_element_context_function("recurse"));
    assert!(is_element_context_function("walk"));

    assert!(!is_element_context_function("limit"));
    assert!(!is_element_context_function("has"));
    assert!(!is_element_context_function("del"));
    assert!(!is_element_context_function("unknown_function"));
}

#[test]
fn test_element_context_functions_count() {
    assert_eq!(
        ELEMENT_CONTEXT_FUNCTIONS.len(),
        9,
        "ELEMENT_CONTEXT_FUNCTIONS should contain exactly 9 functions"
    );
}

#[test]
fn test_element_context_functions_all_in_metadata() {
    for func in ELEMENT_CONTEXT_FUNCTIONS.iter() {
        let found = JQ_FUNCTION_METADATA.iter().any(|f| f.name == *func);
        assert!(
            found,
            "Element context function '{}' should be in JQ_FUNCTION_METADATA",
            func
        );
    }
}

// ============================================================================
// filter_builtins Tests
// ============================================================================

#[test]
fn test_filter_builtins_with_empty_prefix() {
    let results = filter_builtins("");
    assert!(
        results.is_empty(),
        "filter_builtins with empty prefix should return empty vec"
    );
}

#[test]
fn test_filter_builtins_with_valid_prefix() {
    let results = filter_builtins("ma");
    assert!(
        !results.is_empty(),
        "Should find functions starting with 'ma'"
    );
    assert!(
        results.iter().any(|s| s.text == "map"),
        "Should include 'map' function"
    );
    assert!(
        results.iter().any(|s| s.text == "max"),
        "Should include 'max' function"
    );
}

#[test]
fn test_filter_builtins_case_insensitive() {
    let results_lower = filter_builtins("ma");
    let results_upper = filter_builtins("MA");
    assert_eq!(
        results_lower.len(),
        results_upper.len(),
        "Case should not affect filtering"
    );
}

#[test]
fn test_filter_builtins_no_matches() {
    let results = filter_builtins("zzz_nonexistent");
    assert!(
        results.is_empty(),
        "Should return empty vec for non-matching prefix"
    );
}

// ============================================================================
// JqFunction Constructor Tests
// ============================================================================

#[test]
fn test_jq_function_new() {
    let func = JqFunction::new("test_func", "test_func(arg)", "Test description", true);
    assert_eq!(func.name, "test_func");
    assert_eq!(func.signature, "test_func(arg)");
    assert_eq!(func.description, "Test description");
    assert!(func.needs_parens);
}

#[test]
fn test_jq_function_new_without_parens() {
    let func = JqFunction::new("keys", "keys", "Get object keys", false);
    assert_eq!(func.name, "keys");
    assert_eq!(func.signature, "keys");
    assert_eq!(func.description, "Get object keys");
    assert!(!func.needs_parens);
}
