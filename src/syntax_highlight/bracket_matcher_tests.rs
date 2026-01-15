//! Tests for bracket_matcher module

use super::*;

#[test]
fn test_cursor_on_opening_paren() {
    let query = "map(.)";
    let result = find_matching_bracket(query, 3);
    assert_eq!(result, Some((3, 5)));
}

#[test]
fn test_cursor_on_closing_paren() {
    let query = "map(.)";
    let result = find_matching_bracket(query, 5);
    assert_eq!(result, Some((3, 5)));
}

#[test]
fn test_cursor_on_opening_square() {
    let query = ".items[]";
    let result = find_matching_bracket(query, 6);
    assert_eq!(result, Some((6, 7)));
}

#[test]
fn test_cursor_on_closing_square() {
    let query = ".items[]";
    let result = find_matching_bracket(query, 7);
    assert_eq!(result, Some((6, 7)));
}

#[test]
fn test_cursor_on_opening_curly() {
    let query = "{name: .x}";
    let result = find_matching_bracket(query, 0);
    assert_eq!(result, Some((0, 9)));
}

#[test]
fn test_cursor_on_closing_curly() {
    let query = "{name: .x}";
    let result = find_matching_bracket(query, 9);
    assert_eq!(result, Some((0, 9)));
}

#[test]
fn test_cursor_not_on_bracket() {
    let query = "map(.)";
    let result = find_matching_bracket(query, 0);
    assert_eq!(result, None);
}

#[test]
fn test_cursor_not_on_bracket_middle() {
    let query = "map(.)";
    let result = find_matching_bracket(query, 4);
    assert_eq!(result, None);
}

#[test]
fn test_empty_query() {
    let query = "";
    let result = find_matching_bracket(query, 0);
    assert_eq!(result, None);
}

#[test]
fn test_cursor_beyond_query_length() {
    let query = "map";
    let result = find_matching_bracket(query, 10);
    assert_eq!(result, None);
}

#[test]
fn test_unmatched_opening_bracket() {
    let query = "map(";
    let result = find_matching_bracket(query, 3);
    assert_eq!(result, None);
}

#[test]
fn test_unmatched_closing_bracket() {
    let query = "map)";
    let result = find_matching_bracket(query, 3);
    assert_eq!(result, None);
}

#[test]
fn test_nested_same_type_brackets() {
    let query = "map(select(.x))";
    let result = find_matching_bracket(query, 3);
    assert_eq!(result, Some((3, 14)));
}

#[test]
fn test_nested_same_type_inner_brackets() {
    let query = "map(select(.x))";
    let result = find_matching_bracket(query, 10);
    assert_eq!(result, Some((10, 13)));
}

#[test]
fn test_mixed_bracket_types() {
    let query = ".items[0] | map({name: .x})";
    let result = find_matching_bracket(query, 6);
    assert_eq!(result, Some((6, 8)));
}

#[test]
fn test_mixed_bracket_types_paren() {
    let query = ".items[0] | map({name: .x})";
    let result = find_matching_bracket(query, 15);
    assert_eq!(result, Some((15, 26)));
}

#[test]
fn test_mixed_bracket_types_curly() {
    let query = ".items[0] | map({name: .x})";
    let result = find_matching_bracket(query, 16);
    assert_eq!(result, Some((16, 25)));
}

#[test]
fn test_deeply_nested_brackets() {
    let query = "map(select(has({a: {b: .x}})))";
    let result = find_matching_bracket(query, 3);
    assert_eq!(result, Some((3, 29)));
}

#[test]
fn test_deeply_nested_innermost_curly() {
    let query = "map(select(has({a: {b: .x}})))";
    let result = find_matching_bracket(query, 19);
    assert_eq!(result, Some((19, 25)));
}

#[test]
fn test_deeply_nested_middle_curly() {
    let query = "map(select(has({a: {b: .x}})))";
    let result = find_matching_bracket(query, 15);
    assert_eq!(result, Some((15, 26)));
}

#[test]
fn test_multiple_bracket_pairs_first() {
    let query = "(.) | (.)";
    let result = find_matching_bracket(query, 0);
    assert_eq!(result, Some((0, 2)));
}

#[test]
fn test_multiple_bracket_pairs_second() {
    let query = "(.) | (.)";
    let result = find_matching_bracket(query, 6);
    assert_eq!(result, Some((6, 8)));
}

#[test]
fn test_adjacent_brackets() {
    let query = "[]{}()";
    let result = find_matching_bracket(query, 0);
    assert_eq!(result, Some((0, 1)));
}

#[test]
fn test_adjacent_brackets_second() {
    let query = "[]{}()";
    let result = find_matching_bracket(query, 2);
    assert_eq!(result, Some((2, 3)));
}

#[test]
fn test_adjacent_brackets_third() {
    let query = "[]{}()";
    let result = find_matching_bracket(query, 4);
    assert_eq!(result, Some((4, 5)));
}

#[test]
fn test_single_pair_at_start() {
    let query = "()";
    let result = find_matching_bracket(query, 0);
    assert_eq!(result, Some((0, 1)));
}

#[test]
fn test_single_pair_at_end() {
    let query = "map()";
    let result = find_matching_bracket(query, 3);
    assert_eq!(result, Some((3, 4)));
}

#[test]
fn test_complex_real_world_query() {
    let query = r#".items[] | select(.price > 100) | {name, price}"#;
    let result = find_matching_bracket(query, 17);
    assert_eq!(result, Some((17, 30)));
}

#[test]
fn test_complex_real_world_query_curly() {
    let query = r#".items[] | select(.price > 100) | {name, price}"#;
    let result = find_matching_bracket(query, 34);
    assert_eq!(result, Some((34, 46)));
}

#[test]
fn test_array_indexing() {
    let query = ".data[0][1][2]";
    let result = find_matching_bracket(query, 5);
    assert_eq!(result, Some((5, 7)));
}

#[test]
fn test_array_indexing_second() {
    let query = ".data[0][1][2]";
    let result = find_matching_bracket(query, 8);
    assert_eq!(result, Some((8, 10)));
}

#[test]
fn test_array_indexing_third() {
    let query = ".data[0][1][2]";
    let result = find_matching_bracket(query, 11);
    assert_eq!(result, Some((11, 13)));
}

#[test]
fn test_function_call_with_multiple_args() {
    let query = "limit(5; map(.x))";
    let result = find_matching_bracket(query, 0);
    assert_eq!(result, None);
}

#[test]
fn test_function_call_with_multiple_args_outer_paren() {
    let query = "limit(5; map(.x))";
    let result = find_matching_bracket(query, 5);
    assert_eq!(result, Some((5, 16)));
}

#[test]
fn test_function_call_with_multiple_args_inner_paren() {
    let query = "limit(5; map(.x))";
    let result = find_matching_bracket(query, 12);
    assert_eq!(result, Some((12, 15)));
}

#[test]
fn test_only_opening_bracket() {
    let query = "(";
    let result = find_matching_bracket(query, 0);
    assert_eq!(result, None);
}

#[test]
fn test_only_closing_bracket() {
    let query = ")";
    let result = find_matching_bracket(query, 0);
    assert_eq!(result, None);
}

#[test]
fn test_mismatched_bracket_types() {
    let query = "(]";
    let result = find_matching_bracket(query, 0);
    assert_eq!(result, None);
}

#[test]
fn test_unicode_characters_before_brackets() {
    let query = "👋(test)";
    let result = find_matching_bracket(query, 1);
    assert_eq!(result, Some((1, 6)));
}

#[test]
fn test_cursor_at_position_zero_on_bracket() {
    let query = "(.test)";
    let result = find_matching_bracket(query, 0);
    assert_eq!(result, Some((0, 6)));
}

#[test]
fn test_cursor_at_last_position_on_bracket() {
    let query = "(test)";
    let result = find_matching_bracket(query, 5);
    assert_eq!(result, Some((0, 5)));
}
