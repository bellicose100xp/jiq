use super::common::tracker_for;
use crate::autocomplete::*;

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

#[test]
fn test_analyze_context_after_optional_array() {
    let query = ".services[].capacityProviderStrategy[]?.";
    let tracker = tracker_for(query);
    let (ctx, partial) = analyze_context(query, &tracker);
    assert_eq!(ctx, SuggestionContext::FieldContext);
    assert_eq!(partial, "");

    let query = ".services[].capacityProviderStrategy[]?.b";
    let tracker = tracker_for(query);
    let (ctx, partial) = analyze_context(query, &tracker);
    assert_eq!(ctx, SuggestionContext::FieldContext);
    assert_eq!(partial, "b");
}

#[test]
fn test_analyze_context_jq_keywords() {
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

    let tracker = tracker_for("i");
    let (ctx, partial) = analyze_context("i", &tracker);
    assert_eq!(ctx, SuggestionContext::FunctionContext);
    assert_eq!(partial, "i");
}
