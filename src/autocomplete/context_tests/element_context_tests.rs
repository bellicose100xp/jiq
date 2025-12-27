use super::common::{create_array_of_objects_json, tracker_for};
use crate::autocomplete::*;
use crate::query::ResultType;
use serde_json::Value;
use std::sync::Arc;

#[test]
fn test_suggestions_inside_map_returns_element_fields() {
    let (parsed, result_type) = create_array_of_objects_json();
    let query = "map(.";
    let tracker = tracker_for(query);

    let suggestions = get_suggestions(
        query,
        query.len(),
        Some(parsed),
        Some(result_type),
        &tracker,
    );

    let field_suggestions: Vec<_> = suggestions.iter().filter(|s| s.text != ".[]").collect();

    assert!(
        !field_suggestions.is_empty(),
        "Should have field suggestions"
    );

    for suggestion in field_suggestions {
        assert!(
            !suggestion.text.contains("[]."),
            "Inside map(), suggestion '{}' should not contain '[].'",
            suggestion.text
        );
        assert!(
            suggestion.text.starts_with('.'),
            "Field suggestion '{}' should start with '.'",
            suggestion.text
        );
    }
}

#[test]
fn test_suggestions_inside_select_returns_element_fields() {
    let (parsed, result_type) = create_array_of_objects_json();
    let query = "select(.";
    let tracker = tracker_for(query);

    let suggestions = get_suggestions(
        query,
        query.len(),
        Some(parsed),
        Some(result_type),
        &tracker,
    );

    let field_suggestions: Vec<_> = suggestions.iter().filter(|s| s.text != ".[]").collect();

    assert!(
        !field_suggestions.is_empty(),
        "Should have field suggestions"
    );

    for suggestion in field_suggestions {
        assert!(
            !suggestion.text.contains("[]."),
            "Inside select(), suggestion '{}' should not contain '[].'",
            suggestion.text
        );
    }
}

#[test]
fn test_suggestions_outside_function_returns_array_fields() {
    let (parsed, result_type) = create_array_of_objects_json();
    let query = ".";
    let tracker = tracker_for(query);

    let suggestions = get_suggestions(
        query,
        query.len(),
        Some(parsed),
        Some(result_type),
        &tracker,
    );

    let field_suggestions: Vec<_> = suggestions.iter().filter(|s| s.text != ".[]").collect();

    assert!(
        !field_suggestions.is_empty(),
        "Should have field suggestions"
    );

    for suggestion in field_suggestions {
        assert!(
            suggestion.text.contains("[]."),
            "Outside function, suggestion '{}' should contain '[].'",
            suggestion.text
        );
    }
}

#[test]
fn test_suggestions_inside_nested_element_functions() {
    let (parsed, result_type) = create_array_of_objects_json();
    let query = "map(select(.";
    let tracker = tracker_for(query);

    let suggestions = get_suggestions(
        query,
        query.len(),
        Some(parsed),
        Some(result_type),
        &tracker,
    );

    let field_suggestions: Vec<_> = suggestions.iter().filter(|s| s.text != ".[]").collect();

    for suggestion in field_suggestions {
        assert!(
            !suggestion.text.contains("[]."),
            "Inside nested element functions, suggestion '{}' should not contain '[].'",
            suggestion.text
        );
    }
}

#[test]
fn test_suggestions_inside_map_with_object_construction() {
    let (parsed, result_type) = create_array_of_objects_json();
    let query = "map({name: .";
    let tracker = tracker_for(query);

    let suggestions = get_suggestions(
        query,
        query.len(),
        Some(parsed),
        Some(result_type),
        &tracker,
    );

    let field_suggestions: Vec<_> = suggestions.iter().filter(|s| s.text != ".[]").collect();

    for suggestion in field_suggestions {
        assert!(
            !suggestion.text.contains("[]."),
            "Inside map() with object, suggestion '{}' should not contain '[].'",
            suggestion.text
        );
    }
}

#[test]
fn test_suggestions_partial_field_filtering_in_element_context() {
    let (parsed, result_type) = create_array_of_objects_json();
    let query = "map(.na";
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
        "Should suggest '.name' for partial 'na'"
    );

    assert!(
        !suggestions.iter().any(|s| s.text == ".age"),
        "Should not suggest '.age' for partial 'na'"
    );
}

#[test]
fn test_suggestions_after_pipe_in_element_context() {
    let (parsed, result_type) = create_array_of_objects_json();
    let query = "map(. | .";
    let tracker = tracker_for(query);

    let suggestions = get_suggestions(
        query,
        query.len(),
        Some(parsed),
        Some(result_type),
        &tracker,
    );

    let field_suggestions: Vec<_> = suggestions.iter().filter(|s| s.text != ".[]").collect();

    for suggestion in field_suggestions {
        assert!(
            !suggestion.text.contains("[]."),
            "After pipe inside map(), suggestion '{}' should not contain '[].'",
            suggestion.text
        );
    }
}

#[test]
fn test_suggestions_all_element_functions() {
    let (parsed, result_type) = create_array_of_objects_json();
    let element_functions = [
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

    for func in element_functions {
        let query = format!("{}(.", func);
        let tracker = tracker_for(&query);

        let suggestions = get_suggestions(
            &query,
            query.len(),
            Some(parsed.clone()),
            Some(result_type.clone()),
            &tracker,
        );

        let field_suggestions: Vec<_> = suggestions.iter().filter(|s| s.text != ".[]").collect();

        for suggestion in &field_suggestions {
            assert!(
                !suggestion.text.contains("[]."),
                "Inside {}(), suggestion '{}' should not contain '[].'",
                func,
                suggestion.text
            );
        }
    }
}

#[test]
fn test_suggestions_non_element_functions_have_brackets() {
    let (parsed, result_type) = create_array_of_objects_json();
    let non_element_functions = ["limit", "has", "del"];

    for func in non_element_functions {
        let query = format!("{}(.", func);
        let tracker = tracker_for(&query);

        let suggestions = get_suggestions(
            &query,
            query.len(),
            Some(parsed.clone()),
            Some(result_type.clone()),
            &tracker,
        );

        let field_suggestions: Vec<_> = suggestions.iter().filter(|s| s.text != ".[]").collect();

        for suggestion in &field_suggestions {
            assert!(
                suggestion.text.contains("[]."),
                "Inside {}() (non-element function), suggestion '{}' should contain '[].'",
                func,
                suggestion.text
            );
        }
    }
}

#[test]
fn test_regression_existing_field_suggestions_unchanged() {
    let json = r#"{"name": "test", "value": 42}"#;
    let parsed = Arc::new(serde_json::from_str::<Value>(json).unwrap());
    let query = ".";
    let tracker = tracker_for(query);

    let suggestions = get_suggestions(
        query,
        query.len(),
        Some(parsed),
        Some(ResultType::Object),
        &tracker,
    );

    assert!(
        suggestions.iter().any(|s| s.text == ".name"),
        "Should suggest '.name' for object"
    );
    assert!(
        suggestions.iter().any(|s| s.text == ".value"),
        "Should suggest '.value' for object"
    );
}

#[test]
fn test_regression_object_key_context_unchanged() {
    let json = r#"{"name": "test", "value": 42}"#;
    let parsed = Arc::new(serde_json::from_str::<Value>(json).unwrap());
    let query = "{na";
    let tracker = tracker_for(query);

    let suggestions = get_suggestions(
        query,
        query.len(),
        Some(parsed),
        Some(ResultType::Object),
        &tracker,
    );

    for suggestion in &suggestions {
        assert!(
            !suggestion.text.starts_with('.'),
            "ObjectKeyContext suggestion '{}' should not start with '.'",
            suggestion.text
        );
    }
}

#[test]
fn test_regression_function_context_unchanged() {
    let json = r#"{"name": "test"}"#;
    let parsed = Arc::new(serde_json::from_str::<Value>(json).unwrap());
    let query = "ma";
    let tracker = tracker_for(query);

    let suggestions = get_suggestions(
        query,
        query.len(),
        Some(parsed),
        Some(ResultType::Object),
        &tracker,
    );

    assert!(
        suggestions.iter().any(|s| s.text == "map"),
        "Should suggest 'map' function for partial 'ma'"
    );
}

#[test]
fn test_object_key_context_does_not_suggest_iterator() {
    let json = r#"[{"name": "test", "id": 1}]"#;
    let parsed = Arc::new(serde_json::from_str::<Value>(json).unwrap());
    let query = "{na";
    let tracker = tracker_for(query);

    let suggestions = get_suggestions(
        query,
        query.len(),
        Some(parsed),
        Some(ResultType::ArrayOfObjects),
        &tracker,
    );

    assert!(
        !suggestions
            .iter()
            .any(|s| s.text == "[]" || s.text == ".[]"),
        "Should not suggest [] or .[] in object key context"
    );
}

#[test]
fn test_field_context_inside_object_suggests_array_fields() {
    let json = r#"[{"serviceName": "test", "id": 1}]"#;
    let parsed = Arc::new(serde_json::from_str::<Value>(json).unwrap());
    let query = "{.ser";
    let tracker = tracker_for(query);

    let suggestions = get_suggestions(
        query,
        query.len(),
        Some(parsed),
        Some(ResultType::ArrayOfObjects),
        &tracker,
    );

    assert!(
        suggestions.iter().any(|s| s.text == ".[].serviceName"),
        "Should suggest .[].serviceName inside object construction"
    );
}
