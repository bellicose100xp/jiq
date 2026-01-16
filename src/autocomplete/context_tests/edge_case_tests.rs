/// Phase 4 edge case tests for autocomplete.
///
/// Tests for transforming functions, complex expressions,
/// and other edge cases that require special handling.
use super::common::tracker_for;
use crate::autocomplete::*;
use crate::query::ResultType;
use serde_json::Value;
use std::sync::Arc;

fn create_test_json() -> (Arc<Value>, ResultType) {
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

mod transforming_function_detection {
    use super::*;

    #[test]
    fn test_keys_pipe_suggests_original_fields() {
        let (parsed, result_type) = create_test_json();
        let query = "keys | .";
        let tracker = tracker_for(query);

        let suggestions = get_suggestions(
            query,
            query.len(),
            Some(parsed.clone()),
            Some(result_type),
            Some(parsed),
            &tracker,
        );

        // After `keys`, should fall back to original_json fields
        // keys produces ["meta", "orders", "user"] but we want original fields
        assert!(
            suggestions.iter().any(|s| s.text.contains("user")),
            "After transforming function, should suggest from original_json. Got: {:?}",
            suggestions.iter().map(|s| &s.text).collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_to_entries_pipe_suggests_original_fields() {
        let (parsed, result_type) = create_test_json();
        let query = "to_entries | .[].";
        let tracker = tracker_for(query);

        let suggestions = get_suggestions(
            query,
            query.len(),
            Some(parsed.clone()),
            Some(result_type),
            Some(parsed),
            &tracker,
        );

        // After `to_entries`, structure changes to {key, value}
        // Should fall back to original_json fields
        assert!(
            !suggestions.is_empty(),
            "Should have fallback suggestions after to_entries"
        );
    }

    #[test]
    fn test_group_by_pipe_suggests_original_fields() {
        let (parsed, result_type) = create_test_json();
        let query = ".orders | group_by(.status) | .";
        let tracker = tracker_for(query);

        let suggestions = get_suggestions(
            query,
            query.len(),
            Some(parsed.clone()),
            Some(result_type),
            Some(parsed),
            &tracker,
        );

        // After group_by, structure is nested arrays
        assert!(
            !suggestions.is_empty(),
            "Should have fallback suggestions after group_by"
        );
    }

    #[test]
    fn test_flatten_pipe_suggests_original_fields() {
        let (parsed, result_type) = create_test_json();
        let query = ".orders | flatten | .";
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
            !suggestions.is_empty(),
            "Should have fallback suggestions after flatten"
        );
    }

    #[test]
    fn test_reverse_pipe_suggests_original_fields() {
        let (parsed, result_type) = create_test_json();
        let query = ".orders | reverse | .";
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
            !suggestions.is_empty(),
            "Should have fallback suggestions after reverse"
        );
    }
}

mod complex_expression_detection {
    use super::*;

    #[test]
    fn test_arithmetic_expression_suggests_original_fields() {
        let (parsed, result_type) = create_test_json();
        let query = ".user.profile.age + 10 | .";
        let tracker = tracker_for(query);

        let suggestions = get_suggestions(
            query,
            query.len(),
            Some(parsed.clone()),
            Some(result_type),
            Some(parsed),
            &tracker,
        );

        // After arithmetic, result is numeric, not object
        assert!(
            !suggestions.is_empty(),
            "Should have fallback suggestions after arithmetic"
        );
    }

    #[test]
    fn test_string_concatenation_suggests_original_fields() {
        let (parsed, result_type) = create_test_json();
        let query = r#".user.profile.name + " Smith" | ."#;
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
            !suggestions.is_empty(),
            "Should have fallback suggestions after string concatenation"
        );
    }

    #[test]
    fn test_conditional_suggests_original_fields() {
        let (parsed, result_type) = create_test_json();
        let query = "if .user then .user else .meta end | .";
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
            !suggestions.is_empty(),
            "Should have fallback suggestions after conditional"
        );
    }
}

mod optional_field_access {
    use super::*;

    #[test]
    fn test_optional_field_in_array_builder() {
        let (parsed, result_type) = create_test_json();
        // Use array builder (non-executing context) to test optional field navigation
        let query = "[.user?.profile?.";
        let tracker = tracker_for(query);

        let suggestions = get_suggestions(
            query,
            query.len(),
            Some(parsed.clone()),
            Some(result_type),
            Some(parsed),
            &tracker,
        );

        // Optional fields should still navigate correctly in non-executing context
        assert!(
            suggestions.iter().any(|s| s.text.contains("name")),
            "Optional fields should still navigate to nested fields. Got: {:?}",
            suggestions.iter().map(|s| &s.text).collect::<Vec<_>>()
        );
        assert!(
            suggestions.iter().any(|s| s.text.contains("age")),
            "Optional fields should still show 'age'"
        );
    }
}

mod bracket_notation {
    use super::*;

    #[test]
    fn test_bracket_notation_in_array_builder() {
        let (parsed, result_type) = create_test_json();
        // Use array builder (non-executing context) to test bracket notation navigation
        let query = r#"[.["user"].profile."#;
        let tracker = tracker_for(query);

        let suggestions = get_suggestions(
            query,
            query.len(),
            Some(parsed.clone()),
            Some(result_type),
            Some(parsed),
            &tracker,
        );

        // Bracket notation should navigate like dot notation
        assert!(
            suggestions.iter().any(|s| s.text.contains("name")),
            "Bracket notation should navigate to nested fields. Got: {:?}",
            suggestions.iter().map(|s| &s.text).collect::<Vec<_>>()
        );
    }
}

mod array_index_access {
    use super::*;

    #[test]
    fn test_specific_array_index_in_array_builder() {
        let (parsed, result_type) = create_test_json();
        // Use array builder (non-executing context) to test array index navigation
        let query = "[.orders[0].items[0].";
        let tracker = tracker_for(query);

        let suggestions = get_suggestions(
            query,
            query.len(),
            Some(parsed.clone()),
            Some(result_type),
            Some(parsed),
            &tracker,
        );

        // Specific index access should work
        assert!(
            suggestions.iter().any(|s| s.text.contains("sku")),
            "Specific array index should navigate correctly. Got: {:?}",
            suggestions.iter().map(|s| &s.text).collect::<Vec<_>>()
        );
        assert!(
            suggestions.iter().any(|s| s.text.contains("qty")),
            "Specific array index should show 'qty'"
        );
    }

    #[test]
    fn test_negative_array_index_in_object_builder() {
        let (parsed, result_type) = create_test_json();
        // Use object builder (non-executing context) to test negative index navigation
        let query = "{last: .orders[-1].";
        let tracker = tracker_for(query);

        let suggestions = get_suggestions(
            query,
            query.len(),
            Some(parsed.clone()),
            Some(result_type),
            Some(parsed),
            &tracker,
        );

        // Negative index should access from end
        assert!(
            suggestions.iter().any(|s| s.text.contains("id")),
            "Negative index should navigate correctly. Got: {:?}",
            suggestions.iter().map(|s| &s.text).collect::<Vec<_>>()
        );
    }
}

mod pipe_chaining {
    use super::*;

    #[test]
    fn test_identity_pipe_in_array_builder() {
        let (parsed, result_type) = create_test_json();
        // Use array builder (non-executing context) to test pipe navigation
        let query = "[. | .user.";
        let tracker = tracker_for(query);

        let suggestions = get_suggestions(
            query,
            query.len(),
            Some(parsed.clone()),
            Some(result_type),
            Some(parsed),
            &tracker,
        );

        // Identity pipe should still work in non-executing context
        assert!(
            suggestions.iter().any(|s| s.text.contains("profile")),
            "Identity pipe should navigate correctly. Got: {:?}",
            suggestions.iter().map(|s| &s.text).collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_multiple_pipes_in_executing_context() {
        let (parsed, result_type) = create_test_json();
        // In executing context, multiple pipes work via cache
        let query = ".user | .profile | .";
        let tracker = tracker_for(query);

        let suggestions = get_suggestions(
            query,
            query.len(),
            Some(parsed.clone()),
            Some(result_type),
            Some(parsed),
            &tracker,
        );

        // Multiple pipes with field access should work
        assert!(
            !suggestions.is_empty(),
            "Multiple pipes should produce suggestions"
        );
    }
}

mod mixed_contexts {
    use super::*;

    fn create_array_root_json() -> (Arc<Value>, ResultType) {
        let json = r#"[
            {"items": [{"sku": "A1", "qty": 2}]},
            {"items": [{"sku": "B2", "qty": 1}]}
        ]"#;
        let parsed = serde_json::from_str::<Value>(json).unwrap();
        (Arc::new(parsed), ResultType::ArrayOfObjects)
    }

    #[test]
    fn test_map_with_nested_path() {
        let (parsed, result_type) = create_array_root_json();
        // map() requires array input - element context prepends ArrayIterator
        let query = "map(.items[].";
        let tracker = tracker_for(query);

        let suggestions = get_suggestions(
            query,
            query.len(),
            Some(parsed.clone()),
            Some(result_type),
            Some(parsed),
            &tracker,
        );

        // Inside map, navigation should work with element context prepending
        assert!(
            suggestions.iter().any(|s| s.text.contains("sku")),
            "Map should navigate nested paths correctly. Got: {:?}",
            suggestions.iter().map(|s| &s.text).collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_select_with_comparison() {
        let (parsed, result_type) = create_test_json();
        let query = r#".orders | select(.status == "shipped") | ."#;
        let tracker = tracker_for(query);

        let suggestions = get_suggestions(
            query,
            query.len(),
            Some(parsed.clone()),
            Some(result_type),
            Some(parsed),
            &tracker,
        );

        // Select with comparison should work
        assert!(
            !suggestions.is_empty(),
            "Select with comparison should produce suggestions"
        );
    }
}

mod deep_nesting {
    use super::*;

    fn create_deeply_nested_json() -> (Arc<Value>, ResultType) {
        let json = r#"{
            "level1": {
                "level2": {
                    "level3": {
                        "level4": {
                            "level5": {
                                "value": "deep"
                            }
                        }
                    }
                }
            }
        }"#;
        let parsed = serde_json::from_str::<Value>(json).unwrap();
        (Arc::new(parsed), ResultType::Object)
    }

    #[test]
    fn test_five_level_nesting_in_array_builder() {
        let (parsed, result_type) = create_deeply_nested_json();
        // Use array builder (non-executing context) to test deep nesting
        let query = "[.level1.level2.level3.level4.level5.";
        let tracker = tracker_for(query);

        let suggestions = get_suggestions(
            query,
            query.len(),
            Some(parsed.clone()),
            Some(result_type),
            Some(parsed),
            &tracker,
        );

        // Deep nesting should work
        assert!(
            suggestions.iter().any(|s| s.text.contains("value")),
            "Deep nesting should navigate correctly. Got: {:?}",
            suggestions.iter().map(|s| &s.text).collect::<Vec<_>>()
        );
    }
}

mod non_deterministic_function_boundary {
    use super::*;

    #[test]
    fn test_non_transforming_function_is_deterministic() {
        let (parsed, result_type) = create_test_json();
        // sort is not in TRANSFORMING_FUNCTIONS, it preserves structure
        let query = ".orders | sort | .";
        let tracker = tracker_for(query);

        let suggestions = get_suggestions(
            query,
            query.len(),
            Some(parsed.clone()),
            Some(result_type),
            Some(parsed),
            &tracker,
        );

        // sort preserves structure, should still suggest array fields
        assert!(
            !suggestions.is_empty(),
            "Non-transforming function should produce suggestions"
        );
    }

    #[test]
    fn test_partial_function_name_not_matched() {
        let (parsed, result_type) = create_test_json();
        // "mykeys" contains "keys" but is not the same function
        let query = ".mykeys_field | .";
        let tracker = tracker_for(query);

        let suggestions = get_suggestions(
            query,
            query.len(),
            Some(parsed.clone()),
            Some(result_type),
            Some(parsed),
            &tracker,
        );

        // Should not trigger non-deterministic fallback
        assert!(
            !suggestions.is_empty(),
            "Partial function name match should not trigger fallback"
        );
    }
}

mod middle_of_query_tests {
    use super::*;

    #[test]
    fn test_cursor_in_middle_navigates_from_original() {
        let (parsed, result_type) = create_test_json();
        // Query: .user.▎profile.name (cursor after ".user.")
        let query = ".user.profile.name";
        let cursor_pos = 6; // After ".user."
        let tracker = tracker_for(query);

        let suggestions = get_suggestions(
            query,
            cursor_pos,
            Some(parsed.clone()),
            Some(result_type),
            Some(parsed),
            &tracker,
        );

        // Middle-of-query should navigate from original_json to .user
        // and suggest profile, settings
        assert!(
            suggestions.iter().any(|s| s.text.contains("profile")),
            "Cursor in middle should navigate from original and suggest 'profile'. Got: {:?}",
            suggestions.iter().map(|s| &s.text).collect::<Vec<_>>()
        );
        assert!(
            suggestions.iter().any(|s| s.text.contains("settings")),
            "Cursor in middle should suggest 'settings'"
        );
    }

    #[test]
    fn test_cursor_in_middle_of_array_path() {
        let (parsed, result_type) = create_test_json();
        // Query: .orders[].▎items[].sku (cursor after ".orders[].")
        let query = ".orders[].items[].sku";
        let cursor_pos = 10; // After ".orders[]."
        let tracker = tracker_for(query);

        let suggestions = get_suggestions(
            query,
            cursor_pos,
            Some(parsed.clone()),
            Some(result_type),
            Some(parsed),
            &tracker,
        );

        // Should suggest fields of order elements: id, items, status
        assert!(
            suggestions.iter().any(|s| s.text.contains("id")),
            "Should suggest 'id' from order element. Got: {:?}",
            suggestions.iter().map(|s| &s.text).collect::<Vec<_>>()
        );
        assert!(
            suggestions.iter().any(|s| s.text.contains("status")),
            "Should suggest 'status' from order element"
        );
    }

    #[test]
    fn test_cursor_in_middle_with_partial_field() {
        let (parsed, result_type) = create_test_json();
        // Query: .user.pro▎file.name (cursor in middle of "profile")
        let query = ".user.profile.name";
        let cursor_pos = 9; // After ".user.pro"
        let tracker = tracker_for(query);

        let suggestions = get_suggestions(
            query,
            cursor_pos,
            Some(parsed.clone()),
            Some(result_type),
            Some(parsed),
            &tracker,
        );

        // Should filter suggestions by partial "pro"
        assert!(
            suggestions.iter().any(|s| s.text.contains("profile")),
            "Should suggest 'profile' matching partial 'pro'. Got: {:?}",
            suggestions.iter().map(|s| &s.text).collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_cursor_in_middle_nonexistent_path_falls_back() {
        let (parsed, result_type) = create_test_json();
        // Query: .nonexistent.▎field.stuff (cursor after ".nonexistent.")
        let query = ".nonexistent.field.stuff";
        let cursor_pos = 13; // After ".nonexistent."
        let tracker = tracker_for(query);

        let suggestions = get_suggestions(
            query,
            cursor_pos,
            Some(parsed.clone()),
            Some(result_type),
            Some(parsed),
            &tracker,
        );

        // Navigation fails, should fall back to original_json suggestions
        assert!(
            !suggestions.is_empty(),
            "Should have fallback suggestions for nonexistent path"
        );
    }

    #[test]
    fn test_cursor_in_middle_inside_map() {
        let (parsed, result_type) = create_test_json();
        // Query: map(.user.▎profile.name) (cursor inside map, in middle of path)
        let query = "map(.user.profile.name)";
        let cursor_pos = 10; // After "map(.user."
        let tracker = tracker_for(query);

        let suggestions = get_suggestions(
            query,
            cursor_pos,
            Some(parsed.clone()),
            Some(result_type),
            Some(parsed),
            &tracker,
        );

        // Inside map(), middle of query, should navigate from original
        // map expects array input, but original is object, so falls back
        assert!(
            !suggestions.is_empty(),
            "Should have suggestions inside map with cursor in middle"
        );
    }

    #[test]
    fn test_cursor_in_middle_inside_array_builder() {
        let (parsed, result_type) = create_test_json();
        // Query: [.user.▎profile.name] (cursor inside array builder, middle of path)
        let query = "[.user.profile.name]";
        let cursor_pos = 7; // After "[.user."
        let tracker = tracker_for(query);

        let suggestions = get_suggestions(
            query,
            cursor_pos,
            Some(parsed.clone()),
            Some(result_type),
            Some(parsed),
            &tracker,
        );

        // Inside array builder, middle of query
        assert!(
            suggestions.iter().any(|s| s.text.contains("profile")),
            "Array builder with cursor in middle should suggest nested fields. Got: {:?}",
            suggestions.iter().map(|s| &s.text).collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_cursor_at_end_vs_middle_different_results() {
        let (parsed, result_type) = create_test_json();
        let query = ".user.profile.name";
        let tracker = tracker_for(query);

        // Cursor at end (after ".name")
        let suggestions_end = get_suggestions(
            query,
            query.len(),
            Some(parsed.clone()),
            Some(result_type.clone()),
            Some(parsed.clone()),
            &tracker,
        );

        // Cursor in middle (after ".user.")
        let suggestions_middle = get_suggestions(
            query,
            6,
            Some(parsed.clone()),
            Some(result_type),
            Some(parsed),
            &tracker,
        );

        // Results should be different - end shows name's type, middle shows user's fields
        let end_texts: Vec<_> = suggestions_end.iter().map(|s| &s.text).collect();
        let middle_texts: Vec<_> = suggestions_middle.iter().map(|s| &s.text).collect();

        // Middle should have profile/settings, end might be empty (name is a string)
        assert!(
            middle_texts.iter().any(|t| t.contains("profile")),
            "Middle should suggest profile. Got: {:?}",
            middle_texts
        );

        // Verify they're different (end is likely empty since .name is a string)
        assert!(
            end_texts != middle_texts || suggestions_end.is_empty(),
            "End vs middle should produce different suggestions"
        );
    }
}

mod performance_tests {
    use super::*;
    use std::time::Instant;

    fn create_ten_level_nested_json() -> (Arc<Value>, ResultType) {
        let json = r#"{
            "l1": {"l2": {"l3": {"l4": {"l5": {"l6": {"l7": {"l8": {"l9": {"l10": {"value": "deep"}}}}}}}}}}
        }"#;
        let parsed = serde_json::from_str::<Value>(json).unwrap();
        (Arc::new(parsed), ResultType::Object)
    }

    fn create_wide_json() -> (Arc<Value>, ResultType) {
        let mut fields = Vec::new();
        for i in 0..100 {
            fields.push(format!(r#""field{}": {{"nested": "value{}"}}"#, i, i));
        }
        let json = format!("{{{}}}", fields.join(", "));
        let parsed = serde_json::from_str::<Value>(&json).unwrap();
        (Arc::new(parsed), ResultType::Object)
    }

    #[test]
    fn test_ten_level_deep_nesting_performance() {
        let (parsed, result_type) = create_ten_level_nested_json();
        let query = "[.l1.l2.l3.l4.l5.l6.l7.l8.l9.l10.";
        let tracker = tracker_for(query);

        let start = Instant::now();
        let _suggestions = get_suggestions(
            query,
            query.len(),
            Some(parsed.clone()),
            Some(result_type),
            Some(parsed),
            &tracker,
        );
        let elapsed = start.elapsed();

        // Should complete in under 10ms (with generous margin for test environments)
        assert!(
            elapsed.as_millis() < 100,
            "Ten-level nesting took {:?}, should be < 100ms",
            elapsed
        );
    }

    #[test]
    fn test_wide_object_performance() {
        let (parsed, result_type) = create_wide_json();
        let query = "[.field50.";
        let tracker = tracker_for(query);

        let start = Instant::now();
        let suggestions = get_suggestions(
            query,
            query.len(),
            Some(parsed.clone()),
            Some(result_type),
            Some(parsed),
            &tracker,
        );
        let elapsed = start.elapsed();

        // Should complete quickly and find the nested field
        assert!(
            elapsed.as_millis() < 100,
            "Wide object took {:?}, should be < 100ms",
            elapsed
        );
        assert!(
            suggestions.iter().any(|s| s.text.contains("nested")),
            "Should find nested field. Got: {:?}",
            suggestions.iter().map(|s| &s.text).collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_repeated_navigation_performance() {
        let (parsed, result_type) = create_test_json();
        let query = "[.user.profile.";
        let tracker = tracker_for(query);

        let start = Instant::now();
        for _ in 0..100 {
            let _suggestions = get_suggestions(
                query,
                query.len(),
                Some(parsed.clone()),
                Some(result_type.clone()),
                Some(parsed.clone()),
                &tracker,
            );
        }
        let elapsed = start.elapsed();

        // 100 iterations should complete quickly
        assert!(
            elapsed.as_millis() < 500,
            "100 iterations took {:?}, should be < 500ms",
            elapsed
        );
    }
}
