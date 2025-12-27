use super::common::tracker_for;
use crate::autocomplete::*;

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
