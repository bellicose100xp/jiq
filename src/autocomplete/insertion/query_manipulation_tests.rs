//! Tests for query string manipulation utilities

use super::*;

#[test]
fn test_extract_middle_query_simple_path() {
    // Simple path: no middle
    let result = extract_middle_query(".services.ca", ".services", ".services.ca", "ca");
    assert_eq!(result, "", "Simple path should have empty middle");
}

#[test]
fn test_extract_middle_query_after_pipe() {
    // After pipe with identity - preserves trailing space
    let result = extract_middle_query(".services | .ca", ".services", ".services | .ca", "ca");
    assert_eq!(
        result, " | ",
        "Middle: pipe with trailing space (before dot)"
    );
}

#[test]
fn test_extract_middle_query_with_if_then() {
    // Complex: if/then between base and current field - preserves trailing space
    let query = ".services | if has(\"x\") then .ca";
    let before_cursor = query;
    let result = extract_middle_query(query, ".services", before_cursor, "ca");
    assert_eq!(
        result, " | if has(\"x\") then ",
        "Middle with trailing space (important for 'then ')"
    );
}

#[test]
fn test_extract_middle_query_with_select() {
    // With select function - preserves trailing space
    let query = ".items | select(.active) | .na";
    let result = extract_middle_query(query, ".items", query, "na");
    assert_eq!(
        result, " | select(.active) | ",
        "Middle: includes pipe with trailing space"
    );
}

#[test]
fn test_extract_middle_query_no_partial() {
    // Just typed dot, no partial yet - preserves trailing space
    let result = extract_middle_query(".services | .", ".services", ".services | .", "");
    assert_eq!(
        result, " | ",
        "Middle with trailing space before trigger dot"
    );
}

#[test]
fn test_extract_middle_query_base_not_prefix() {
    // Edge case: base is not prefix of current query (shouldn't happen)
    let result = extract_middle_query(".items.ca", ".services", ".items.ca", "ca");
    assert_eq!(result, "", "Should return empty if base not a prefix");
}

#[test]
fn test_extract_middle_query_nested_pipes() {
    // Multiple pipes and functions - preserves trailing space
    let query = ".a | .b | map(.c) | .d";
    let result = extract_middle_query(query, ".a", query, "d");
    assert_eq!(result, " | .b | map(.c) | ", "Middle with trailing space");
}
