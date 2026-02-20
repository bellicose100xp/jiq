/// Integration tests for Phase 3: Nested path navigation for autocomplete.
///
/// These tests verify that autocomplete correctly suggests nested fields
/// in non-executing contexts (map, select, array builders, object builders).
use super::common::{
    create_array_of_objects_json, empty_field_names, field_names_from, tracker_for,
};
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
            empty_field_names(),
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
            empty_field_names(),
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
            empty_field_names(),
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
            empty_field_names(),
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
            empty_field_names(),
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
            empty_field_names(),
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
            empty_field_names(),
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
            Some(parsed.clone()),
            field_names_from(&parsed),
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
            empty_field_names(),
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
            empty_field_names(),
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
            empty_field_names(),
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
            empty_field_names(),
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

        let suggestions = get_suggestions(
            query,
            query.len(),
            None,
            None,
            None,
            empty_field_names(),
            &tracker,
        );

        assert!(
            suggestions.iter().any(|s| s.text == "$x"),
            "Should still suggest defined variable '$x'"
        );
    }
}

/// Phase 7: Streaming result context tests
/// Tests for element-context functions after streaming operations (.services[] | select(.))
mod streaming_result_context {
    use super::*;

    fn create_services_json() -> (Arc<Value>, Arc<Value>) {
        let json = r#"{
            "services": [
                {
                    "serviceName": "inventory-manager",
                    "serviceArn": "arn:aws:ecs:us-east-1:111:service/svc1",
                    "status": "ACTIVE",
                    "strategy": "ROLLING",
                    "deploymentConfiguration": {
                        "maximumPercent": 200,
                        "minimumHealthyPercent": 100,
                        "strategy": "ROLLING"
                    }
                },
                {
                    "serviceName": "order-processor",
                    "serviceArn": "arn:aws:ecs:us-east-1:111:service/svc2",
                    "status": "ACTIVE",
                    "strategy": "BLUE_GREEN",
                    "deploymentConfiguration": {
                        "maximumPercent": 200,
                        "minimumHealthyPercent": 50,
                        "strategy": "BLUE_GREEN"
                    }
                }
            ]
        }"#;
        let original = Arc::new(serde_json::from_str::<Value>(json).unwrap());

        // Simulate the result of .services[] - a single service object (DestructuredObjects)
        let service_element = r#"{
            "serviceName": "inventory-manager",
            "serviceArn": "arn:aws:ecs:us-east-1:111:service/svc1",
            "status": "ACTIVE",
            "strategy": "ROLLING",
            "deploymentConfiguration": {
                "maximumPercent": 200,
                "minimumHealthyPercent": 100,
                "strategy": "ROLLING"
            }
        }"#;
        let result = Arc::new(serde_json::from_str::<Value>(service_element).unwrap());

        (original, result)
    }

    #[test]
    fn test_streaming_then_select_suggests_element_fields() {
        // Bug case: .services[] | select(.
        // result_type is DestructuredObjects (streaming), so we should NOT prepend ArrayIterator
        let (original, result) = create_services_json();
        let query = ".services[] | select(.";
        let tracker = tracker_for(query);

        let suggestions = get_suggestions(
            query,
            query.len(),
            Some(result),
            Some(ResultType::DestructuredObjects), // Key: streaming result
            Some(original),
            empty_field_names(),
            &tracker,
        );

        // Should suggest service element fields
        assert!(
            suggestions.iter().any(|s| s.text.contains("serviceName")),
            "Should suggest 'serviceName' from service element. Got: {:?}",
            suggestions.iter().map(|s| &s.text).collect::<Vec<_>>()
        );
        assert!(
            suggestions.iter().any(|s| s.text.contains("status")),
            "Should suggest 'status' from service element"
        );
        assert!(
            suggestions.iter().any(|s| s.text.contains("strategy")),
            "Should suggest 'strategy' from service element"
        );
    }

    #[test]
    fn test_streaming_then_select_with_partial_filters_correctly() {
        // Bug case: .services[] | select(.stra
        let (original, result) = create_services_json();
        let query = ".services[] | select(.stra";
        let tracker = tracker_for(query);

        let suggestions = get_suggestions(
            query,
            query.len(),
            Some(result),
            Some(ResultType::DestructuredObjects),
            Some(original),
            empty_field_names(),
            &tracker,
        );

        // Should filter to only 'strategy' field
        assert!(
            suggestions.iter().any(|s| s.text.contains("strategy")),
            "Should suggest 'strategy' matching partial '.stra'. Got: {:?}",
            suggestions.iter().map(|s| &s.text).collect::<Vec<_>>()
        );
        // Should NOT suggest non-matching fields
        assert!(
            !suggestions.iter().any(|s| s.text.contains("serviceName")),
            "Should NOT suggest 'serviceName' (doesn't match 'stra')"
        );
    }

    #[test]
    fn test_streaming_then_select_nested_path() {
        // .services[] | select(.deploymentConfiguration.
        let (original, result) = create_services_json();
        let query = ".services[] | select(.deploymentConfiguration.";
        let tracker = tracker_for(query);

        let suggestions = get_suggestions(
            query,
            query.len(),
            Some(result),
            Some(ResultType::DestructuredObjects),
            Some(original),
            empty_field_names(),
            &tracker,
        );

        // Should suggest fields from deploymentConfiguration
        assert!(
            suggestions
                .iter()
                .any(|s| s.text.contains("maximumPercent")),
            "Should suggest 'maximumPercent' from nested path. Got: {:?}",
            suggestions.iter().map(|s| &s.text).collect::<Vec<_>>()
        );
        assert!(
            suggestions
                .iter()
                .any(|s| s.text.contains("minimumHealthyPercent")),
            "Should suggest 'minimumHealthyPercent' from nested path"
        );
    }

    #[test]
    fn test_non_streaming_select_still_prepends_array_iterator() {
        // .services | select(. - without streaming, result is ArrayOfObjects
        // Should still prepend ArrayIterator
        let (original, _result) = create_services_json();
        let services_array = original.get("services").unwrap().clone();
        let result = Arc::new(services_array);
        let query = "select(.";
        let tracker = tracker_for(query);

        let suggestions = get_suggestions(
            query,
            query.len(),
            Some(result),
            Some(ResultType::ArrayOfObjects), // Non-streaming: array
            Some(original),
            empty_field_names(),
            &tracker,
        );

        // Should suggest element fields (prepended ArrayIterator works)
        assert!(
            suggestions.iter().any(|s| s.text.contains("serviceName")),
            "Non-streaming select should still show element fields. Got: {:?}",
            suggestions.iter().map(|s| &s.text).collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_variable_binding_with_streaming_select() {
        // "ROLLING" as $st | .services[] | select(.
        let (original, result) = create_services_json();
        let query = r#""ROLLING" as $st | .services[] | select(."#;
        let tracker = tracker_for(query);

        let suggestions = get_suggestions(
            query,
            query.len(),
            Some(result),
            Some(ResultType::DestructuredObjects),
            Some(original),
            empty_field_names(),
            &tracker,
        );

        // Should suggest service element fields
        assert!(
            suggestions.iter().any(|s| s.text.contains("serviceName")),
            "Variable binding + streaming + select should show element fields. Got: {:?}",
            suggestions.iter().map(|s| &s.text).collect::<Vec<_>>()
        );
        assert!(
            suggestions.iter().any(|s| s.text.contains("strategy")),
            "Should suggest 'strategy' field"
        );
    }
}

/// Tests for multi-element array navigation (heterogeneous arrays)
mod multi_element_navigation {
    use super::*;

    fn create_heterogeneous_array_json() -> (Arc<Value>, ResultType) {
        let json = r#"[
            {"name": "svc1", "status": "ACTIVE"},
            {"name": "svc2", "extra_key": true, "region": "us-east-1"},
            {"name": "svc3", "priority": 5}
        ]"#;
        let parsed = serde_json::from_str::<Value>(json).unwrap();
        (Arc::new(parsed), ResultType::ArrayOfObjects)
    }

    fn create_nested_heterogeneous_json() -> (Arc<Value>, ResultType) {
        let json = r#"{
            "services": [
                {"serviceName": "svc1", "status": "ACTIVE"},
                {"serviceName": "svc2", "extra_service_key": "special"}
            ]
        }"#;
        let parsed = serde_json::from_str::<Value>(json).unwrap();
        (Arc::new(parsed), ResultType::Object)
    }

    #[test]
    fn test_map_with_heterogeneous_array_suggests_union_keys() {
        let (parsed, result_type) = create_heterogeneous_array_json();
        let query = "map(.";
        let tracker = tracker_for(query);

        let suggestions = get_suggestions(
            query,
            query.len(),
            Some(parsed.clone()),
            Some(result_type),
            Some(parsed),
            empty_field_names(),
            &tracker,
        );

        assert!(
            suggestions.iter().any(|s| s.text.contains("name")),
            "Should suggest 'name' from first element. Got: {:?}",
            suggestions.iter().map(|s| &s.text).collect::<Vec<_>>()
        );
        assert!(
            suggestions.iter().any(|s| s.text.contains("extra_key")),
            "Should suggest 'extra_key' from second element"
        );
        assert!(
            suggestions.iter().any(|s| s.text.contains("priority")),
            "Should suggest 'priority' from third element"
        );
    }

    #[test]
    fn test_select_with_heterogeneous_array_suggests_union_keys() {
        let (parsed, result_type) = create_heterogeneous_array_json();
        let query = "select(.";
        let tracker = tracker_for(query);

        let suggestions = get_suggestions(
            query,
            query.len(),
            Some(parsed.clone()),
            Some(result_type),
            Some(parsed),
            empty_field_names(),
            &tracker,
        );

        assert!(
            suggestions.iter().any(|s| s.text.contains("status")),
            "Should suggest 'status' from first element. Got: {:?}",
            suggestions.iter().map(|s| &s.text).collect::<Vec<_>>()
        );
        assert!(
            suggestions.iter().any(|s| s.text.contains("region")),
            "Should suggest 'region' from second element"
        );
    }

    #[test]
    fn test_nested_array_path_suggests_union_keys() {
        let (parsed, result_type) = create_nested_heterogeneous_json();
        let query = "[.services[].";
        let tracker = tracker_for(query);

        let suggestions = get_suggestions(
            query,
            query.len(),
            Some(parsed.clone()),
            Some(result_type),
            Some(parsed),
            empty_field_names(),
            &tracker,
        );

        assert!(
            suggestions.iter().any(|s| s.text.contains("serviceName")),
            "Should suggest 'serviceName' from first service. Got: {:?}",
            suggestions.iter().map(|s| &s.text).collect::<Vec<_>>()
        );
        assert!(
            suggestions
                .iter()
                .any(|s| s.text.contains("extra_service_key")),
            "Should suggest 'extra_service_key' from second service"
        );
    }

    #[test]
    fn test_original_json_fallback_uses_multi_element() {
        let (parsed, result_type) = create_nested_heterogeneous_json();
        let all_fields = field_names_from(&parsed);
        let query = "map(.services[].";
        let tracker = tracker_for(query);

        let suggestions = get_suggestions(
            query,
            query.len(),
            Some(parsed.clone()),
            Some(result_type),
            Some(parsed),
            all_fields,
            &tracker,
        );

        assert!(
            suggestions
                .iter()
                .any(|s| s.text.contains("extra_service_key")),
            "Original JSON fallback should also show keys from non-first elements. Got: {:?}",
            suggestions.iter().map(|s| &s.text).collect::<Vec<_>>()
        );
    }
}

/// Integration tests for the canonical issue #145 scenarios
mod issue_145_integration {
    use super::*;

    fn create_canonical_json() -> (Arc<Value>, ResultType) {
        let json = r#"{
            "services": [
                {
                    "serviceName": "inventory-manager",
                    "status": "ACTIVE",
                    "deployments": [
                        {
                            "id": "deploy-1",
                            "tasks": [
                                {"taskArn": "arn:task:1", "cpu": 256}
                            ]
                        }
                    ]
                },
                {
                    "serviceName": "order-processor",
                    "status": "ACTIVE",
                    "extra_service_key": "special_value",
                    "deployments": [
                        {
                            "id": "deploy-2",
                            "tasks": [
                                {"taskArn": "arn:task:2", "payload": {"data": "test"}}
                            ]
                        }
                    ]
                }
            ]
        }"#;
        let parsed = serde_json::from_str::<Value>(json).unwrap();
        (Arc::new(parsed), ResultType::Object)
    }

    #[test]
    fn test_services_iterator_includes_extra_service_key() {
        let (parsed, result_type) = create_canonical_json();
        let query = "[.services[].";
        let tracker = tracker_for(query);

        let suggestions = get_suggestions(
            query,
            query.len(),
            Some(parsed.clone()),
            Some(result_type),
            Some(parsed),
            empty_field_names(),
            &tracker,
        );

        assert!(
            suggestions
                .iter()
                .any(|s| s.text.contains("extra_service_key")),
            ".services[]. should include extra_service_key. Got: {:?}",
            suggestions.iter().map(|s| &s.text).collect::<Vec<_>>()
        );
        assert!(
            suggestions.iter().any(|s| s.text.contains("serviceName")),
            "Should also include serviceName from first element"
        );
    }

    #[test]
    fn test_select_with_services_includes_extra_key() {
        let (parsed, result_type) = create_canonical_json();
        let all_fields = field_names_from(&parsed);

        let query = "[.services[] | select(.";
        let tracker = tracker_for(query);
        let suggestions = get_suggestions(
            query,
            query.len(),
            Some(parsed.clone()),
            Some(result_type),
            Some(parsed),
            all_fields,
            &tracker,
        );

        assert!(
            suggestions
                .iter()
                .any(|s| s.text.contains("extra_service_key")),
            "select() should include extra_service_key from second service. Got: {:?}",
            suggestions.iter().map(|s| &s.text).collect::<Vec<_>>()
        );
        assert!(
            suggestions.iter().any(|s| s.text.contains("serviceName")),
            "select() should include serviceName from first service"
        );
    }

    #[test]
    fn test_nested_array_path_surfaces_non_first_keys() {
        let (parsed, result_type) = create_canonical_json();
        let all_fields = field_names_from(&parsed);
        let query = "[.services[].deployments[].tasks[].";
        let tracker = tracker_for(query);

        let suggestions = get_suggestions(
            query,
            query.len(),
            Some(parsed.clone()),
            Some(result_type),
            Some(parsed),
            all_fields,
            &tracker,
        );

        assert!(
            suggestions.iter().any(|s| s.text.contains("taskArn")),
            "Should include taskArn from first task. Got: {:?}",
            suggestions.iter().map(|s| &s.text).collect::<Vec<_>>()
        );
        assert!(
            suggestions.iter().any(|s| s.text.contains("payload")),
            "Should include payload from second service's task"
        );
    }

    #[test]
    fn test_invalid_path_falls_back_cleanly() {
        let (parsed, result_type) = create_canonical_json();
        let all_fields = field_names_from(&parsed);
        let query = "[.nonexistent[].";
        let tracker = tracker_for(query);

        let suggestions = get_suggestions(
            query,
            query.len(),
            Some(parsed.clone()),
            Some(result_type),
            Some(parsed),
            all_fields,
            &tracker,
        );

        // Should not panic; may return fallback or empty suggestions
        let _ = suggestions;
    }
}
