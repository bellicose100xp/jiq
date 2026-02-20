//! Tests for entry context detection (to_entries, with_entries)

use super::common::{DEFAULT_ARRAY_SAMPLE_SIZE, empty_field_names, field_names_from, tracker_for};
use crate::autocomplete::*;
use crate::query::ResultType;
use serde_json::Value;
use std::sync::Arc;

fn create_object_json() -> (Arc<Value>, ResultType) {
    let json = r#"{"name": "alice", "age": 30, "active": true}"#;
    let parsed = serde_json::from_str::<Value>(json).unwrap();
    (Arc::new(parsed), ResultType::Object)
}

// ============================================================================
// detect_entry_context unit tests
// ============================================================================

mod detect_entry_context_tests {
    use crate::autocomplete::{EntryContext, detect_entry_context};

    #[test]
    fn test_no_entry_context_at_root() {
        assert_eq!(detect_entry_context(".", 1), EntryContext::None);
        assert_eq!(detect_entry_context(".name", 5), EntryContext::None);
    }

    #[test]
    fn test_with_entries_direct_context() {
        assert_eq!(
            detect_entry_context("with_entries(.", 14),
            EntryContext::Direct
        );
        assert_eq!(
            detect_entry_context("with_entries(.key", 17),
            EntryContext::Direct
        );
    }

    #[test]
    fn test_with_entries_after_pipe() {
        assert_eq!(
            detect_entry_context("with_entries(. | .", 18),
            EntryContext::Direct
        );
    }

    #[test]
    fn test_with_entries_opaque_after_value_pipe() {
        assert_eq!(
            detect_entry_context("with_entries(.value | .", 23),
            EntryContext::OpaqueValue
        );
    }

    #[test]
    fn test_with_entries_opaque_after_value_map() {
        assert_eq!(
            detect_entry_context("with_entries(.value | map(.", 27),
            EntryContext::OpaqueValue
        );
    }

    #[test]
    fn test_with_entries_direct_value_without_transformation() {
        // Just .value without further navigation is Direct (we can still suggest .key/.value)
        assert_eq!(
            detect_entry_context("with_entries(.value", 19),
            EntryContext::Direct
        );
    }

    #[test]
    fn test_with_entries_none_after_value_field_access() {
        // After .value.field, we've navigated into the value structure
        assert_eq!(
            detect_entry_context("with_entries(.value.", 20),
            EntryContext::None
        );
    }

    #[test]
    fn test_with_entries_closed_no_context() {
        assert_eq!(
            detect_entry_context("with_entries(.value) | .", 24),
            EntryContext::None
        );
    }

    #[test]
    fn test_with_entries_nested() {
        assert_eq!(
            detect_entry_context("with_entries(with_entries(.", 27),
            EntryContext::Direct
        );
    }

    #[test]
    fn test_with_entries_with_whitespace() {
        assert_eq!(
            detect_entry_context("with_entries (.", 15),
            EntryContext::Direct
        );
    }

    #[test]
    fn test_to_entries_with_array_iteration() {
        assert_eq!(
            detect_entry_context("to_entries | .[].", 17),
            EntryContext::Direct
        );
    }

    #[test]
    fn test_to_entries_with_map() {
        assert_eq!(
            detect_entry_context("to_entries | map(.", 18),
            EntryContext::Direct
        );
    }

    #[test]
    fn test_to_entries_with_map_value_pipe() {
        assert_eq!(
            detect_entry_context("to_entries | map(.value | .", 27),
            EntryContext::OpaqueValue
        );
    }

    #[test]
    fn test_to_entries_opaque_with_nested_select() {
        assert_eq!(
            detect_entry_context("to_entries | map(.value | select(.", 34),
            EntryContext::OpaqueValue
        );
    }

    #[test]
    fn test_to_entries_complex_opaque_pattern() {
        // The pattern from the plan: to_entries | map({k: .key, v: .value | map(select(.
        assert_eq!(
            detect_entry_context("to_entries | map({k: .key, v: .value | map(select(.", 51),
            EntryContext::OpaqueValue
        );
    }

    #[test]
    fn test_string_literal_false_positive_prevention() {
        // .value inside a string should not trigger entry context
        assert_eq!(
            detect_entry_context("with_entries(\"has.value\" | .", 28),
            EntryContext::Direct
        );
    }

    #[test]
    fn test_to_entries_in_string_no_detection() {
        // "to_entries" in a string should not trigger detection
        assert_eq!(
            detect_entry_context("\"to_entries\" | .[].", 19),
            EntryContext::None
        );
    }

    #[test]
    fn test_values_not_value() {
        // .values (plural) should not trigger entry context
        assert_eq!(
            detect_entry_context("with_entries(.values", 20),
            EntryContext::Direct
        );
    }

    #[test]
    fn test_nested_functions_trigger_opaque() {
        let nested_patterns = [
            ("with_entries(.value | map(.", EntryContext::OpaqueValue),
            ("with_entries(.value | select(.", EntryContext::OpaqueValue),
            ("with_entries(.value | sort_by(.", EntryContext::OpaqueValue),
            (
                "with_entries(.value | group_by(.",
                EntryContext::OpaqueValue,
            ),
            (
                "with_entries(.value | unique_by(.",
                EntryContext::OpaqueValue,
            ),
        ];

        for (query, expected) in nested_patterns {
            assert_eq!(
                detect_entry_context(query, query.len()),
                expected,
                "Pattern '{}' should return {:?}",
                query,
                expected
            );
        }
    }
}

// ============================================================================
// Integration tests - get_suggestions with entry context
// ============================================================================

#[test]
fn test_to_entries_array_iteration_suggests_key_value() {
    let (parsed, result_type) = create_object_json();
    let query = "to_entries | .[].";
    let tracker = tracker_for(query);

    let suggestions = get_suggestions(
        query,
        query.len(),
        Some(parsed.clone()),
        Some(result_type.clone()),
        Some(parsed),
        empty_field_names(),
        &tracker,
        DEFAULT_ARRAY_SAMPLE_SIZE,
    );

    // No leading dot because user already typed the dot in ".[].""
    assert!(
        suggestions.iter().any(|s| s.text == "key"),
        "Should suggest 'key' for to_entries | .[]."
    );
    assert!(
        suggestions.iter().any(|s| s.text == "value"),
        "Should suggest 'value' for to_entries | .[]."
    );
}

#[test]
fn test_to_entries_map_suggests_key_value() {
    let (parsed, result_type) = create_object_json();
    let query = "to_entries | map(.";
    let tracker = tracker_for(query);

    let suggestions = get_suggestions(
        query,
        query.len(),
        Some(parsed.clone()),
        Some(result_type.clone()),
        Some(parsed),
        empty_field_names(),
        &tracker,
        DEFAULT_ARRAY_SAMPLE_SIZE,
    );

    assert!(
        suggestions.iter().any(|s| s.text == ".key"),
        "Should suggest '.key' for to_entries | map(."
    );
    assert!(
        suggestions.iter().any(|s| s.text == ".value"),
        "Should suggest '.value' for to_entries | map(."
    );
}

#[test]
fn test_to_entries_opaque_value_shows_all_fields() {
    let json = r#"{"services": {"web": {"port": 8080}, "db": {"port": 5432}}}"#;
    let parsed = Arc::new(serde_json::from_str::<Value>(json).unwrap());
    let all_fields = field_names_from(&parsed);

    let query = "to_entries | map(.value | .";
    let tracker = tracker_for(query);

    let suggestions = get_suggestions(
        query,
        query.len(),
        Some(parsed.clone()),
        Some(ResultType::Object),
        Some(parsed),
        all_fields,
        &tracker,
        DEFAULT_ARRAY_SAMPLE_SIZE,
    );

    // Should show all fields since we can't determine the structure after .value | .
    assert!(
        suggestions.iter().any(|s| s.text == ".services"),
        "Should include original JSON fields in opaque context"
    );
    // Should NOT show .key/.value since we're in opaque context
    let key_with_desc = suggestions
        .iter()
        .any(|s| s.text == ".key" && s.description.is_some());
    assert!(
        !key_with_desc,
        "Should NOT suggest entry '.key' in opaque context"
    );
}

#[test]
fn test_to_entries_complex_pattern_shows_all_fields() {
    let json = r#"{"name": "test", "items": [1, 2, 3]}"#;
    let parsed = Arc::new(serde_json::from_str::<Value>(json).unwrap());
    let all_fields = field_names_from(&parsed);

    // Complex pattern from the plan
    let query = "to_entries | map({service: .key, config: .value | map(select(.";
    let tracker = tracker_for(query);

    let suggestions = get_suggestions(
        query,
        query.len(),
        Some(parsed.clone()),
        Some(ResultType::Object),
        Some(parsed),
        all_fields,
        &tracker,
        DEFAULT_ARRAY_SAMPLE_SIZE,
    );

    // In opaque context, should fall back to all fields
    assert!(
        suggestions.iter().any(|s| s.text == ".name"),
        "Should include original JSON fields in complex opaque pattern"
    );
}

#[test]
fn test_key_value_have_correct_descriptions() {
    let (parsed, result_type) = create_object_json();
    let query = "to_entries | map(.";
    let tracker = tracker_for(query);

    let suggestions = get_suggestions(
        query,
        query.len(),
        Some(parsed.clone()),
        Some(result_type.clone()),
        Some(parsed),
        empty_field_names(),
        &tracker,
        DEFAULT_ARRAY_SAMPLE_SIZE,
    );

    let key_suggestion = suggestions.iter().find(|s| s.text == ".key");
    assert!(
        key_suggestion
            .and_then(|s| s.description.as_ref())
            .map(|d| d.contains("to_entries") || d.contains("with_entries"))
            .unwrap_or(false),
        ".key should have description mentioning entry context"
    );

    let value_suggestion = suggestions.iter().find(|s| s.text == ".value");
    assert!(
        value_suggestion
            .and_then(|s| s.description.as_ref())
            .map(|d| d.contains("to_entries") || d.contains("with_entries"))
            .unwrap_or(false),
        ".value should have description mentioning entry context"
    );
}

#[test]
fn test_key_value_appear_first_in_to_entries() {
    let (parsed, result_type) = create_object_json();
    let query = "to_entries | map(.";
    let tracker = tracker_for(query);

    let suggestions = get_suggestions(
        query,
        query.len(),
        Some(parsed.clone()),
        Some(result_type.clone()),
        Some(parsed),
        empty_field_names(),
        &tracker,
        DEFAULT_ARRAY_SAMPLE_SIZE,
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
fn test_no_duplicate_key_value_suggestions() {
    // Simulate a scenario where the result already has key/value from entry structure
    // The result of to_entries is [{key: ..., value: ...}, ...]
    let json = r#"[{"key": "name", "value": "alice"}, {"key": "age", "value": 30}]"#;
    let parsed = Arc::new(serde_json::from_str::<Value>(json).unwrap());

    let query = "to_entries | .[].";
    let tracker = tracker_for(query);

    let suggestions = get_suggestions(
        query,
        query.len(),
        Some(parsed.clone()),
        Some(ResultType::ArrayOfObjects),
        Some(parsed),
        empty_field_names(),
        &tracker,
        DEFAULT_ARRAY_SAMPLE_SIZE,
    );

    // Count occurrences of key and value suggestions
    let key_count = suggestions.iter().filter(|s| s.text == "key").count();
    let value_count = suggestions.iter().filter(|s| s.text == "value").count();

    assert_eq!(
        key_count, 1,
        "Should have exactly one 'key' suggestion, not duplicates"
    );
    assert_eq!(
        value_count, 1,
        "Should have exactly one 'value' suggestion, not duplicates"
    );

    // Verify they have the entry context description
    let key_suggestion = suggestions.iter().find(|s| s.text == "key").unwrap();
    assert!(
        key_suggestion
            .description
            .as_ref()
            .map(|d| d.contains("to_entries") || d.contains("with_entries"))
            .unwrap_or(false),
        "key suggestion should have entry context description"
    );
}
