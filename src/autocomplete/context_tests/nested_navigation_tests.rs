/// Integration tests for Phase 3: Nested path navigation for autocomplete.
///
/// These tests verify that autocomplete correctly suggests nested fields
/// in non-executing contexts (map, select, array builders, object builders).
use super::common::{create_array_of_objects_json, tracker_for};
use crate::autocomplete::*;
use crate::query::ResultType;
use serde_json::Value;
use std::sync::Arc;

fn create_nested_object_json() -> (Arc<Value>, ResultType) {
    let json = r#"{
        "user": {
            "profile": {
                "name": "Alice",
                "age": 30
            },
            "settings": {
                "theme": "dark",
                "lang": "en"
            }
        },
        "orders": [
            {"id": 1, "items": [{"sku": "A1", "qty": 2}], "status": "shipped"},
            {"id": 2, "items": [{"sku": "B2", "qty": 1}], "status": "pending"}
        ],
        "meta": {"version": "1.0"}
    }"#;
    let parsed = serde_json::from_str::<Value>(json).unwrap();
    (Arc::new(parsed), ResultType::Object)
}

fn create_nested_array_json() -> (Arc<Value>, ResultType) {
    let json = r#"[
        {
            "user": {
                "profile": {
                    "name": "Alice",
                    "age": 30
                },
                "settings": {
                    "theme": "dark",
                    "lang": "en"
                }
            },
            "orders": [
                {"id": 1, "items": [{"sku": "A1", "qty": 2}], "status": "shipped"}
            ]
        }
    ]"#;
    let parsed = serde_json::from_str::<Value>(json).unwrap();
    (Arc::new(parsed), ResultType::ArrayOfObjects)
}

mod nested_field_suggestions {
    use super::*;

    #[test]
    fn test_map_with_nested_path_suggests_nested_fields() {
        // map() requires array input - element context prepends ArrayIterator
        let (parsed, result_type) = create_nested_array_json();
        let query = "map(.user.profile.";
        let tracker = tracker_for(query);

        let suggestions = get_suggestions(
            query,
            query.len(),
            Some(parsed.clone()),
            Some(result_type),
            Some(parsed),
            &tracker,
        );

        // Should suggest fields from [].user.profile (name, age)
        assert!(
            suggestions.iter().any(|s| s.text.contains("name")),
            "Should suggest 'name' from nested path .user.profile. Got: {:?}",
            suggestions.iter().map(|s| &s.text).collect::<Vec<_>>()
        );
        assert!(
            suggestions.iter().any(|s| s.text.contains("age")),
            "Should suggest 'age' from nested path .user.profile"
        );
    }

    #[test]
    fn test_array_builder_with_nested_path() {
        let (parsed, result_type) = create_nested_object_json();
        let query = "[.user.settings.";
        let tracker = tracker_for(query);

        let suggestions = get_suggestions(
            query,
            query.len(),
            Some(parsed.clone()),
            Some(result_type),
            Some(parsed),
            &tracker,
        );

        // Should suggest fields from .user.settings (theme, lang)
        assert!(
            suggestions.iter().any(|s| s.text.contains("theme")),
            "Should suggest 'theme' from nested path .user.settings"
        );
        assert!(
            suggestions.iter().any(|s| s.text.contains("lang")),
            "Should suggest 'lang' from nested path .user.settings"
        );
    }

    #[test]
    fn test_object_builder_with_nested_path() {
        let (parsed, result_type) = create_nested_object_json();
        let query = "{x: .user.profile.";
        let tracker = tracker_for(query);

        let suggestions = get_suggestions(
            query,
            query.len(),
            Some(parsed.clone()),
            Some(result_type),
            Some(parsed),
            &tracker,
        );

        // Should suggest fields from .user.profile
        assert!(
            suggestions.iter().any(|s| s.text.contains("name")),
            "Should suggest 'name' from nested path in object builder"
        );
        assert!(
            suggestions.iter().any(|s| s.text.contains("age")),
            "Should suggest 'age' from nested path in object builder"
        );
    }
}

mod array_navigation {
    use super::*;

    #[test]
    fn test_map_with_array_iteration() {
        // map() requires array input - element context prepends ArrayIterator
        let (parsed, result_type) = create_nested_array_json();
        let query = "map(.orders[].items[].";
        let tracker = tracker_for(query);

        let suggestions = get_suggestions(
            query,
            query.len(),
            Some(parsed.clone()),
            Some(result_type),
            Some(parsed),
            &tracker,
        );

        // Should suggest fields from [].orders[].items[] (sku, qty)
        assert!(
            suggestions.iter().any(|s| s.text.contains("sku")),
            "Should suggest 'sku' from array path. Got: {:?}",
            suggestions.iter().map(|s| &s.text).collect::<Vec<_>>()
        );
        assert!(
            suggestions.iter().any(|s| s.text.contains("qty")),
            "Should suggest 'qty' from array path"
        );
    }

    #[test]
    fn test_array_index_navigation() {
        let (parsed, result_type) = create_nested_object_json();
        let query = "[.orders[0].";
        let tracker = tracker_for(query);

        let suggestions = get_suggestions(
            query,
            query.len(),
            Some(parsed.clone()),
            Some(result_type),
            Some(parsed),
            &tracker,
        );

        // Should suggest fields from .orders[0] (id, items, status)
        assert!(
            suggestions.iter().any(|s| s.text.contains("id")),
            "Should suggest 'id' from array index path"
        );
        assert!(
            suggestions.iter().any(|s| s.text.contains("status")),
            "Should suggest 'status' from array index path"
        );
    }
}

mod element_context_with_nested_path {
    use super::*;

    #[test]
    fn test_map_element_context_prepends_array_iterator() {
        let (parsed, result_type) = create_array_of_objects_json();
        let query = "map(.";
        let tracker = tracker_for(query);

        let suggestions = get_suggestions(
            query,
            query.len(),
            Some(parsed.clone()),
            Some(result_type),
            Some(parsed),
            &tracker,
        );

        // Should suggest element fields directly (name, age) without .[]
        let field_suggestions: Vec<_> = suggestions.iter().filter(|s| s.text != ".[]").collect();
        assert!(
            !field_suggestions.is_empty(),
            "Should have field suggestions"
        );

        for suggestion in &field_suggestions {
            assert!(
                !suggestion.text.contains("[]."),
                "Inside map(), suggestion '{}' should not contain '[].'",
                suggestion.text
            );
        }
    }

    #[test]
    fn test_select_element_context() {
        let (parsed, result_type) = create_array_of_objects_json();
        let query = "select(.";
        let tracker = tracker_for(query);

        let suggestions = get_suggestions(
            query,
            query.len(),
            Some(parsed.clone()),
            Some(result_type),
            Some(parsed),
            &tracker,
        );

        let field_suggestions: Vec<_> = suggestions.iter().filter(|s| s.text != ".[]").collect();
        assert!(
            !field_suggestions.is_empty(),
            "Should have field suggestions"
        );

        for suggestion in &field_suggestions {
            assert!(
                !suggestion.text.contains("[]."),
                "Inside select(), suggestion '{}' should not contain '[].'",
                suggestion.text
            );
        }
    }
}

mod pipe_boundary {
    use super::*;

    #[test]
    fn test_pipe_inside_function_resets_path() {
        let (parsed, result_type) = create_nested_object_json();
        let query = "map(.user | .";
        let tracker = tracker_for(query);

        let suggestions = get_suggestions(
            query,
            query.len(),
            Some(parsed.clone()),
            Some(result_type),
            Some(parsed),
            &tracker,
        );

        // After pipe, path extraction should start from the pipe
        // The path is just "." so it should show root-level fields or element fields
        assert!(
            !suggestions.is_empty(),
            "Should have suggestions after pipe inside function"
        );
    }
}

mod fallback_behavior {
    use super::*;

    #[test]
    fn test_nonexistent_path_falls_back() {
        let (parsed, result_type) = create_nested_object_json();
        let query = "map(.nonexistent.path.";
        let tracker = tracker_for(query);

        let _suggestions = get_suggestions(
            query,
            query.len(),
            Some(parsed.clone()),
            Some(result_type),
            Some(parsed),
            &tracker,
        );

        // Navigation fails, should fall back to standard suggestions
        // This verifies graceful degradation
        // May return empty or fallback suggestions
        // The key is it shouldn't panic
    }

    #[test]
    fn test_empty_original_json_uses_result() {
        let (parsed, result_type) = create_nested_object_json();
        let query = "map(.user.profile.";
        let tracker = tracker_for(query);

        // Pass None for original_json to test fallback to result_parsed
        let _suggestions = get_suggestions(
            query,
            query.len(),
            Some(parsed),
            Some(result_type),
            None, // No original_json
            &tracker,
        );

        // Should still attempt navigation using result_parsed
        // May succeed or fail depending on implementation
        // The key is it shouldn't panic
    }
}

mod regression_tests {
    use super::*;

    #[test]
    fn test_executing_context_uses_cache_directly() {
        let (parsed, result_type) = create_nested_object_json();
        let query = ".";
        let tracker = tracker_for(query);

        let suggestions = get_suggestions(
            query,
            query.len(),
            Some(parsed.clone()),
            Some(result_type),
            Some(parsed),
            &tracker,
        );

        // Executing context at end should use cache directly
        assert!(
            suggestions.iter().any(|s| s.text.contains("user")),
            "Should suggest top-level 'user' field"
        );
        assert!(
            suggestions.iter().any(|s| s.text.contains("orders")),
            "Should suggest top-level 'orders' field"
        );
        assert!(
            suggestions.iter().any(|s| s.text.contains("meta")),
            "Should suggest top-level 'meta' field"
        );
    }

    #[test]
    fn test_function_context_unchanged() {
        let (parsed, result_type) = create_nested_object_json();
        let query = "ma";
        let tracker = tracker_for(query);

        let suggestions = get_suggestions(
            query,
            query.len(),
            Some(parsed.clone()),
            Some(result_type),
            Some(parsed),
            &tracker,
        );

        assert!(
            suggestions.iter().any(|s| s.text == "map"),
            "Should still suggest 'map' function"
        );
    }

    #[test]
    fn test_variable_context_unchanged() {
        let query = ". as $x | $";
        let tracker = tracker_for(query);

        let suggestions = get_suggestions(query, query.len(), None, None, None, &tracker);

        assert!(
            suggestions.iter().any(|s| s.text == "$x"),
            "Should still suggest defined variable '$x'"
        );
    }
}
