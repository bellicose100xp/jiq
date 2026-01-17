use super::common::{empty_field_names, tracker_for};
use crate::autocomplete::*;
use crate::query::ResultType;
use serde_json::Value;
use std::sync::Arc;

#[test]
fn test_get_suggestions_with_no_result() {
    let query = ".";
    let tracker = tracker_for(query);

    let suggestions = get_suggestions(
        query,
        query.len(),
        None,
        None,
        None,
        empty_field_names(),
        &tracker,
    );

    assert_eq!(
        suggestions.len(),
        0,
        "Should return empty suggestions when no result is available"
    );
}

#[test]
fn test_get_suggestions_with_result_none_type_none() {
    let json = r#"{"name": "test"}"#;
    let parsed = Arc::new(serde_json::from_str::<Value>(json).unwrap());
    let query = ".";
    let tracker = tracker_for(query);

    let suggestions = get_suggestions(
        query,
        query.len(),
        Some(parsed),
        None,
        None,
        empty_field_names(),
        &tracker,
    );

    assert_eq!(
        suggestions.len(),
        0,
        "Should return empty suggestions when result_type is None"
    );
}

#[test]
fn test_get_suggestions_with_result_type_none_result() {
    let query = ".";
    let tracker = tracker_for(query);

    let suggestions = get_suggestions(
        query,
        query.len(),
        None,
        Some(ResultType::Object),
        None,
        empty_field_names(),
        &tracker,
    );

    assert_eq!(
        suggestions.len(),
        0,
        "Should return empty suggestions when result_parsed is None"
    );
}

#[test]
fn test_analyze_context_simple_dot_field() {
    let tracker = tracker_for(".x");
    let (ctx, partial) = analyze_context(".x", &tracker);

    assert_eq!(ctx, SuggestionContext::FieldContext);
    assert_eq!(partial, "x", "Should extract field name after single dot");
}
