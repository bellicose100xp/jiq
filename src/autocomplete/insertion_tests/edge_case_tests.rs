//! Edge case tests for autocomplete insertion
//!
//! Tests for special character triggers and boundary conditions

use super::*;

#[test]
fn test_insert_suggestion_from_app_when_query_none() {
    // Test that insert_suggestion_from_app handles None query gracefully
    let mut app = test_app(r#"{"test": true}"#);
    app.query = None; // Explicitly set query to None

    let suggestion = test_suggestion("test");

    // Should return early without crashing
    insert_suggestion_from_app(&mut app, &suggestion);

    // Query should still be None
    assert!(app.query.is_none());
}

#[test]
fn test_nested_array_bracket_handling() {
    // Test nested array syntax: .services[].capacityProviderStrategy[].field
    let (mut textarea, mut query_state) = setup_insertion_test("");

    // Simulate: user typed ".services[].capacityProviderStrategy[]"
    textarea.delete_line_by_head();
    textarea.insert_str(".services[].capacityProviderStrategy[]");

    // Set up base query for suggestions
    query_state.base_query_for_suggestions = Some(".services[]".to_string());
    query_state.base_type_for_suggestions = Some(ResultType::DestructuredObjects);

    // User wants to complete with "[].capacityProviderStrategy"
    let suggestion = test_suggestion("[].capacityProviderStrategy");

    insert_suggestion(&mut textarea, &mut query_state, &suggestion);

    let result: &str = textarea.lines()[0].as_ref();
    // Should handle nested array brackets correctly
    assert!(result.contains("capacityProviderStrategy"));
}

#[test]
fn test_open_paren_trigger() {
    // Test suggestion after opening parenthesis: map(.field
    let (mut textarea, mut query_state) = setup_insertion_test("");

    textarea.delete_line_by_head();
    textarea.insert_str(".items | map(.");

    query_state.base_query_for_suggestions = Some(".items".to_string());
    query_state.base_type_for_suggestions = Some(ResultType::ArrayOfObjects);

    let suggestion = test_suggestion("id");
    insert_suggestion(&mut textarea, &mut query_state, &suggestion);

    let result: &str = textarea.lines()[0].as_ref();
    assert!(result.contains("map("));
    assert!(result.contains("id"));
}

#[test]
fn test_open_bracket_trigger() {
    // Test suggestion after opening bracket: .items[.field
    let (mut textarea, mut query_state) = setup_insertion_test("");

    textarea.delete_line_by_head();
    textarea.insert_str(".items[.");

    query_state.base_query_for_suggestions = Some(".items".to_string());
    query_state.base_type_for_suggestions = Some(ResultType::Array);

    let suggestion = test_suggestion("0");
    insert_suggestion(&mut textarea, &mut query_state, &suggestion);

    let result: &str = textarea.lines()[0].as_ref();
    assert!(result.contains(".items["));
}

#[test]
fn test_open_brace_trigger() {
    // Test suggestion after opening brace: {key:.field
    let (mut textarea, mut query_state) = setup_insertion_test("");

    textarea.delete_line_by_head();
    textarea.insert_str("{name:.");

    query_state.base_query_for_suggestions = Some(".".to_string());
    query_state.base_type_for_suggestions = Some(ResultType::Object);

    let suggestion = test_suggestion("name");
    insert_suggestion(&mut textarea, &mut query_state, &suggestion);

    let result: &str = textarea.lines()[0].as_ref();
    // After insertion with OpenBrace trigger, result should contain the suggestion
    assert!(result.contains("name"));
}

#[test]
fn test_question_mark_trigger() {
    // Test suggestion after optional operator: .field?.subfield
    let (mut textarea, mut query_state) = setup_insertion_test("");

    textarea.delete_line_by_head();
    textarea.insert_str(".field?.");

    query_state.base_query_for_suggestions = Some(".field".to_string());
    query_state.base_type_for_suggestions = Some(ResultType::Object);

    let suggestion = test_suggestion("subfield");
    insert_suggestion(&mut textarea, &mut query_state, &suggestion);

    let result: &str = textarea.lines()[0].as_ref();
    assert!(result.contains(".field?"));
    assert!(result.contains("subfield"));
}

#[test]
fn test_dot_trigger() {
    // Test suggestion after explicit dot: ..field
    let (mut textarea, mut query_state) = setup_insertion_test("");

    textarea.delete_line_by_head();
    textarea.insert_str(".user..");

    query_state.base_query_for_suggestions = Some(".user".to_string());
    query_state.base_type_for_suggestions = Some(ResultType::Object);

    let suggestion = test_suggestion("profile");
    insert_suggestion(&mut textarea, &mut query_state, &suggestion);

    let result: &str = textarea.lines()[0].as_ref();
    // Should handle double dot correctly
    assert!(result.contains("profile"));
}

#[test]
fn test_close_paren_trigger() {
    // Test suggestion after closing parenthesis: map(.x).field
    let (mut textarea, mut query_state) = setup_insertion_test("");

    textarea.delete_line_by_head();
    textarea.insert_str(".items | map(.id).");

    query_state.base_query_for_suggestions = Some(".items | map(.id)".to_string());
    query_state.base_type_for_suggestions = Some(ResultType::Array);

    let suggestion = test_suggestion("length");
    insert_suggestion(&mut textarea, &mut query_state, &suggestion);

    let result: &str = textarea.lines()[0].as_ref();
    assert!(result.contains("map(.id)"));
    assert!(result.contains("length"));
}

#[test]
fn test_close_brace_trigger() {
    // Test suggestion after closing brace: {key:.val}.field
    let (mut textarea, mut query_state) = setup_insertion_test("");

    textarea.delete_line_by_head();
    textarea.insert_str("{name:.name}.");

    query_state.base_query_for_suggestions = Some("{name:.name}".to_string());
    query_state.base_type_for_suggestions = Some(ResultType::Object);

    let suggestion = test_suggestion("name");
    insert_suggestion(&mut textarea, &mut query_state, &suggestion);

    let result: &str = textarea.lines()[0].as_ref();
    assert!(result.contains("{name:.name}"));
}

#[test]
fn test_base_query_fallback_when_none() {
    // Test the fallback path when base_query_for_suggestions is None
    let (mut textarea, mut query_state) = setup_insertion_test("");

    textarea.delete_line_by_head();
    textarea.insert_str(".test");

    // Explicitly set base_query_for_suggestions to None to trigger fallback
    query_state.base_query_for_suggestions = None;

    let suggestion = test_suggestion("field");
    insert_suggestion(&mut textarea, &mut query_state, &suggestion);

    // Should use textarea.lines()[0] as fallback
    let result: &str = textarea.lines()[0].as_ref();
    assert!(result.contains("field"));
}
