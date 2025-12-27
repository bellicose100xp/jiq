use super::common::{create_array_of_objects_json, tracker_for};
use crate::autocomplete::*;
use crate::query::ResultType;
use serde_json::Value;
use std::sync::Arc;

fn create_object_json() -> (Arc<Value>, ResultType) {
    let json = r#"{"name": "alice", "age": 30, "active": true}"#;
    let parsed = serde_json::from_str::<Value>(json).unwrap();
    (Arc::new(parsed), ResultType::Object)
}

#[test]
fn test_with_entries_suggests_key_and_value() {
    let (parsed, result_type) = create_object_json();
    let query = "with_entries(.";
    let tracker = tracker_for(query);

    let suggestions = get_suggestions(
        query,
        query.len(),
        Some(parsed),
        Some(result_type),
        &tracker,
    );

    assert!(
        suggestions.iter().any(|s| s.text == ".key"),
        "Should suggest '.key' inside with_entries()"
    );
    assert!(
        suggestions.iter().any(|s| s.text == ".value"),
        "Should suggest '.value' inside with_entries()"
    );
}

#[test]
fn test_with_entries_key_value_appear_first() {
    let (parsed, result_type) = create_object_json();
    let query = "with_entries(.";
    let tracker = tracker_for(query);

    let suggestions = get_suggestions(
        query,
        query.len(),
        Some(parsed),
        Some(result_type),
        &tracker,
    );

    assert!(suggestions.len() >= 2, "Should have at least 2 suggestions");
    assert_eq!(
        suggestions[0].text, ".key",
        "First suggestion should be '.key'"
    );
    assert_eq!(
        suggestions[1].text, ".value",
        "Second suggestion should be '.value'"
    );
}

#[test]
fn test_with_entries_partial_filtering_key() {
    let (parsed, result_type) = create_object_json();
    let query = "with_entries(.ke";
    let tracker = tracker_for(query);

    let suggestions = get_suggestions(
        query,
        query.len(),
        Some(parsed),
        Some(result_type),
        &tracker,
    );

    assert!(
        suggestions.iter().any(|s| s.text == ".key"),
        "Should suggest '.key' for partial 'ke'"
    );
    assert!(
        !suggestions.iter().any(|s| s.text == ".value"),
        "Should not suggest '.value' for partial 'ke'"
    );
}

#[test]
fn test_with_entries_partial_filtering_value() {
    let (parsed, result_type) = create_object_json();
    let query = "with_entries(.val";
    let tracker = tracker_for(query);

    let suggestions = get_suggestions(
        query,
        query.len(),
        Some(parsed),
        Some(result_type),
        &tracker,
    );

    assert!(
        suggestions.iter().any(|s| s.text == ".value"),
        "Should suggest '.value' for partial 'val'"
    );
    assert!(
        !suggestions.iter().any(|s| s.text == ".key"),
        "Should not suggest '.key' for partial 'val'"
    );
}

#[test]
fn test_with_entries_with_nested_select() {
    let (parsed, result_type) = create_object_json();
    let query = "with_entries(select(.";
    let tracker = tracker_for(query);

    let suggestions = get_suggestions(
        query,
        query.len(),
        Some(parsed),
        Some(result_type),
        &tracker,
    );

    assert!(
        suggestions.iter().any(|s| s.text == ".key"),
        "Should suggest '.key' inside with_entries(select())"
    );
    assert!(
        suggestions.iter().any(|s| s.text == ".value"),
        "Should suggest '.value' inside with_entries(select())"
    );
}

#[test]
fn test_with_entries_after_pipe() {
    let (parsed, result_type) = create_object_json();
    let query = "with_entries(. | .";
    let tracker = tracker_for(query);

    let suggestions = get_suggestions(
        query,
        query.len(),
        Some(parsed),
        Some(result_type),
        &tracker,
    );

    assert!(
        suggestions.iter().any(|s| s.text == ".key"),
        "Should suggest '.key' after pipe inside with_entries()"
    );
    assert!(
        suggestions.iter().any(|s| s.text == ".value"),
        "Should suggest '.value' after pipe inside with_entries()"
    );
}

#[test]
fn test_with_entries_closed_context() {
    let (parsed, result_type) = create_object_json();
    let query = "with_entries(.value) | .";
    let tracker = tracker_for(query);

    let suggestions = get_suggestions(
        query,
        query.len(),
        Some(parsed),
        Some(result_type),
        &tracker,
    );

    assert!(
        !suggestions.iter().any(|s| s.text == ".key"),
        "Should NOT suggest '.key' outside with_entries()"
    );
    assert!(
        !suggestions
            .iter()
            .any(|s| s.text == ".value" && s.description.is_some()),
        "Should NOT suggest entry '.value' outside with_entries()"
    );
}

#[test]
fn test_with_entries_data_suggestions_included() {
    let (parsed, result_type) = create_object_json();
    let query = "with_entries(.";
    let tracker = tracker_for(query);

    let suggestions = get_suggestions(
        query,
        query.len(),
        Some(parsed),
        Some(result_type),
        &tracker,
    );

    assert!(
        suggestions.iter().any(|s| s.text == ".name"),
        "Should also include data-driven suggestions like '.name'"
    );
    assert!(
        suggestions.iter().any(|s| s.text == ".age"),
        "Should also include data-driven suggestions like '.age'"
    );
}

#[test]
fn test_with_entries_with_object_construction() {
    let (parsed, result_type) = create_object_json();
    let query = "with_entries({newKey: .";
    let tracker = tracker_for(query);

    let suggestions = get_suggestions(
        query,
        query.len(),
        Some(parsed),
        Some(result_type),
        &tracker,
    );

    assert!(
        suggestions.iter().any(|s| s.text == ".key"),
        "Should suggest '.key' inside object construction within with_entries()"
    );
    assert!(
        suggestions.iter().any(|s| s.text == ".value"),
        "Should suggest '.value' inside object construction within with_entries()"
    );
}

#[test]
fn test_with_entries_key_has_description() {
    let (parsed, result_type) = create_object_json();
    let query = "with_entries(.";
    let tracker = tracker_for(query);

    let suggestions = get_suggestions(
        query,
        query.len(),
        Some(parsed),
        Some(result_type),
        &tracker,
    );

    let key_suggestion = suggestions.iter().find(|s| s.text == ".key");
    assert!(key_suggestion.is_some(), "Should have .key suggestion");
    assert!(
        key_suggestion
            .unwrap()
            .description
            .as_ref()
            .map(|d| d.contains("with_entries"))
            .unwrap_or(false),
        ".key suggestion should have description mentioning with_entries()"
    );
}

#[test]
fn test_with_entries_value_has_description() {
    let (parsed, result_type) = create_object_json();
    let query = "with_entries(.";
    let tracker = tracker_for(query);

    let suggestions = get_suggestions(
        query,
        query.len(),
        Some(parsed),
        Some(result_type),
        &tracker,
    );

    let value_suggestion = suggestions.iter().find(|s| s.text == ".value");
    assert!(value_suggestion.is_some(), "Should have .value suggestion");
    assert!(
        value_suggestion
            .unwrap()
            .description
            .as_ref()
            .map(|d| d.contains("with_entries"))
            .unwrap_or(false),
        ".value suggestion should have description mentioning with_entries()"
    );
}

#[test]
fn test_with_entries_array_input() {
    let (parsed, result_type) = create_array_of_objects_json();
    let query = ".[] | with_entries(.";
    let tracker = tracker_for(query);

    let suggestions = get_suggestions(
        query,
        query.len(),
        Some(parsed),
        Some(result_type),
        &tracker,
    );

    assert!(
        suggestions.iter().any(|s| s.text == ".key"),
        "Should suggest '.key' when with_entries is applied to array elements"
    );
    assert!(
        suggestions.iter().any(|s| s.text == ".value"),
        "Should suggest '.value' when with_entries is applied to array elements"
    );
}

#[test]
fn test_with_entries_no_leading_dot_after_pipe() {
    let (parsed, result_type) = create_object_json();
    let query = "with_entries(.key | ";
    let tracker = tracker_for(query);

    let _suggestions = get_suggestions(
        query,
        query.len(),
        Some(parsed),
        Some(result_type),
        &tracker,
    );

    // After pipe with no dot, we're in function context
    // But with_entries context should still be maintained
    assert!(tracker.is_in_with_entries_context(query.len()));
}

#[test]
fn test_outside_with_entries_no_key_value() {
    let (parsed, result_type) = create_object_json();
    let query = ".";
    let tracker = tracker_for(query);

    let suggestions = get_suggestions(
        query,
        query.len(),
        Some(parsed),
        Some(result_type),
        &tracker,
    );

    // Check that .key and .value with descriptions don't appear outside with_entries
    let key_with_desc = suggestions
        .iter()
        .any(|s| s.text == ".key" && s.description.is_some());
    assert!(
        !key_with_desc,
        "Should not suggest '.key' with description outside with_entries()"
    );
}
