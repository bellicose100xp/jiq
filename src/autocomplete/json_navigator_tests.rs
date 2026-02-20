use super::json_navigator::{ARRAY_SAMPLE_SIZE, navigate, navigate_multi};
use super::path_parser::PathSegment;
use serde_json::{Value, json};

mod simple_field_tests {
    use super::*;

    #[test]
    fn test_navigate_single_field() {
        let json = json!({"name": "Alice", "age": 30});
        let segments = vec![PathSegment::Field("name".into())];

        let result = navigate(&json, &segments);

        assert_eq!(result, Some(&json!("Alice")));
    }

    #[test]
    fn test_navigate_different_field() {
        let json = json!({"name": "Alice", "age": 30});
        let segments = vec![PathSegment::Field("age".into())];

        let result = navigate(&json, &segments);

        assert_eq!(result, Some(&json!(30)));
    }

    #[test]
    fn test_navigate_to_object() {
        let json = json!({"user": {"name": "Alice"}});
        let segments = vec![PathSegment::Field("user".into())];

        let result = navigate(&json, &segments);

        assert_eq!(result, Some(&json!({"name": "Alice"})));
    }

    #[test]
    fn test_navigate_to_array() {
        let json = json!({"items": [1, 2, 3]});
        let segments = vec![PathSegment::Field("items".into())];

        let result = navigate(&json, &segments);

        assert_eq!(result, Some(&json!([1, 2, 3])));
    }
}

mod nested_field_tests {
    use super::*;

    #[test]
    fn test_navigate_two_levels() {
        let json = json!({"user": {"name": "Alice"}});
        let segments = vec![
            PathSegment::Field("user".into()),
            PathSegment::Field("name".into()),
        ];

        let result = navigate(&json, &segments);

        assert_eq!(result, Some(&json!("Alice")));
    }

    #[test]
    fn test_navigate_three_levels() {
        let json = json!({"a": {"b": {"c": "deep"}}});
        let segments = vec![
            PathSegment::Field("a".into()),
            PathSegment::Field("b".into()),
            PathSegment::Field("c".into()),
        ];

        let result = navigate(&json, &segments);

        assert_eq!(result, Some(&json!("deep")));
    }

    #[test]
    fn test_navigate_deep_nesting() {
        let json = json!({"l1": {"l2": {"l3": {"l4": {"l5": "value"}}}}});
        let segments = vec![
            PathSegment::Field("l1".into()),
            PathSegment::Field("l2".into()),
            PathSegment::Field("l3".into()),
            PathSegment::Field("l4".into()),
            PathSegment::Field("l5".into()),
        ];

        let result = navigate(&json, &segments);

        assert_eq!(result, Some(&json!("value")));
    }
}

mod array_iterator_tests {
    use super::*;

    #[test]
    fn test_navigate_array_iterator_returns_first() {
        let json = json!([{"id": 1}, {"id": 2}, {"id": 3}]);
        let segments = vec![PathSegment::ArrayIterator];

        let result = navigate(&json, &segments);

        assert_eq!(result, Some(&json!({"id": 1})));
    }

    #[test]
    fn test_navigate_array_iterator_then_field() {
        let json = json!([{"name": "Alice"}, {"name": "Bob"}]);
        let segments = vec![
            PathSegment::ArrayIterator,
            PathSegment::Field("name".into()),
        ];

        let result = navigate(&json, &segments);

        assert_eq!(result, Some(&json!("Alice")));
    }

    #[test]
    fn test_navigate_field_then_iterator() {
        let json = json!({"users": [{"id": 1}, {"id": 2}]});
        let segments = vec![
            PathSegment::Field("users".into()),
            PathSegment::ArrayIterator,
        ];

        let result = navigate(&json, &segments);

        assert_eq!(result, Some(&json!({"id": 1})));
    }

    #[test]
    fn test_navigate_field_iterator_field() {
        let json = json!({"users": [{"profile": {"name": "Alice"}}]});
        let segments = vec![
            PathSegment::Field("users".into()),
            PathSegment::ArrayIterator,
            PathSegment::Field("profile".into()),
            PathSegment::Field("name".into()),
        ];

        let result = navigate(&json, &segments);

        assert_eq!(result, Some(&json!("Alice")));
    }

    #[test]
    fn test_navigate_chained_iterators() {
        let json = json!([[1, 2], [3, 4]]);
        let segments = vec![PathSegment::ArrayIterator, PathSegment::ArrayIterator];

        let result = navigate(&json, &segments);

        assert_eq!(result, Some(&json!(1)));
    }
}

mod array_index_tests {
    use super::*;

    #[test]
    fn test_navigate_index_zero() {
        let json = json!([10, 20, 30]);
        let segments = vec![PathSegment::ArrayIndex(0)];

        let result = navigate(&json, &segments);

        assert_eq!(result, Some(&json!(10)));
    }

    #[test]
    fn test_navigate_index_middle() {
        let json = json!([10, 20, 30]);
        let segments = vec![PathSegment::ArrayIndex(1)];

        let result = navigate(&json, &segments);

        assert_eq!(result, Some(&json!(20)));
    }

    #[test]
    fn test_navigate_index_last() {
        let json = json!([10, 20, 30]);
        let segments = vec![PathSegment::ArrayIndex(2)];

        let result = navigate(&json, &segments);

        assert_eq!(result, Some(&json!(30)));
    }

    #[test]
    fn test_navigate_negative_index() {
        let json = json!([10, 20, 30]);
        let segments = vec![PathSegment::ArrayIndex(-1)];

        let result = navigate(&json, &segments);

        assert_eq!(result, Some(&json!(30)));
    }

    #[test]
    fn test_navigate_negative_index_second_last() {
        let json = json!([10, 20, 30]);
        let segments = vec![PathSegment::ArrayIndex(-2)];

        let result = navigate(&json, &segments);

        assert_eq!(result, Some(&json!(20)));
    }

    #[test]
    fn test_navigate_index_then_field() {
        let json = json!([{"name": "first"}, {"name": "second"}]);
        let segments = vec![
            PathSegment::ArrayIndex(1),
            PathSegment::Field("name".into()),
        ];

        let result = navigate(&json, &segments);

        assert_eq!(result, Some(&json!("second")));
    }
}

mod optional_field_tests {
    use super::*;

    #[test]
    fn test_navigate_optional_field_exists() {
        let json = json!({"name": "Alice"});
        let segments = vec![PathSegment::OptionalField("name".into())];

        let result = navigate(&json, &segments);

        assert_eq!(result, Some(&json!("Alice")));
    }

    #[test]
    fn test_navigate_optional_field_missing() {
        let json = json!({"name": "Alice"});
        let segments = vec![PathSegment::OptionalField("missing".into())];

        let result = navigate(&json, &segments);

        assert_eq!(result, None);
    }

    #[test]
    fn test_navigate_mixed_optional_and_regular() {
        let json = json!({"user": {"profile": {"name": "Alice"}}});
        let segments = vec![
            PathSegment::OptionalField("user".into()),
            PathSegment::Field("profile".into()),
            PathSegment::OptionalField("name".into()),
        ];

        let result = navigate(&json, &segments);

        assert_eq!(result, Some(&json!("Alice")));
    }
}

mod nonexistent_path_tests {
    use super::*;

    #[test]
    fn test_navigate_nonexistent_field() {
        let json = json!({"name": "Alice"});
        let segments = vec![PathSegment::Field("missing".into())];

        let result = navigate(&json, &segments);

        assert_eq!(result, None);
    }

    #[test]
    fn test_navigate_nonexistent_nested() {
        let json = json!({"user": {"name": "Alice"}});
        let segments = vec![
            PathSegment::Field("user".into()),
            PathSegment::Field("missing".into()),
        ];

        let result = navigate(&json, &segments);

        assert_eq!(result, None);
    }

    #[test]
    fn test_navigate_index_out_of_bounds() {
        let json = json!([1, 2, 3]);
        let segments = vec![PathSegment::ArrayIndex(10)];

        let result = navigate(&json, &segments);

        assert_eq!(result, None);
    }

    #[test]
    fn test_navigate_negative_index_out_of_bounds() {
        let json = json!([1, 2, 3]);
        let segments = vec![PathSegment::ArrayIndex(-10)];

        let result = navigate(&json, &segments);

        assert_eq!(result, None);
    }
}

mod type_mismatch_tests {
    use super::*;

    #[test]
    fn test_navigate_field_on_array() {
        let json = json!([1, 2, 3]);
        let segments = vec![PathSegment::Field("name".into())];

        let result = navigate(&json, &segments);

        assert_eq!(result, None);
    }

    #[test]
    fn test_navigate_field_on_string() {
        let json = json!("hello");
        let segments = vec![PathSegment::Field("name".into())];

        let result = navigate(&json, &segments);

        assert_eq!(result, None);
    }

    #[test]
    fn test_navigate_field_on_number() {
        let json = json!(42);
        let segments = vec![PathSegment::Field("name".into())];

        let result = navigate(&json, &segments);

        assert_eq!(result, None);
    }

    #[test]
    fn test_navigate_iterator_on_object() {
        let json = json!({"name": "Alice"});
        let segments = vec![PathSegment::ArrayIterator];

        let result = navigate(&json, &segments);

        assert_eq!(result, None);
    }

    #[test]
    fn test_navigate_iterator_on_string() {
        let json = json!("hello");
        let segments = vec![PathSegment::ArrayIterator];

        let result = navigate(&json, &segments);

        assert_eq!(result, None);
    }

    #[test]
    fn test_navigate_index_on_object() {
        let json = json!({"name": "Alice"});
        let segments = vec![PathSegment::ArrayIndex(0)];

        let result = navigate(&json, &segments);

        assert_eq!(result, None);
    }
}

mod empty_structure_tests {
    use super::*;

    #[test]
    fn test_navigate_empty_array_iterator() {
        let json = json!([]);
        let segments = vec![PathSegment::ArrayIterator];

        let result = navigate(&json, &segments);

        assert_eq!(result, None);
    }

    #[test]
    fn test_navigate_empty_array_index() {
        let json = json!([]);
        let segments = vec![PathSegment::ArrayIndex(0)];

        let result = navigate(&json, &segments);

        assert_eq!(result, None);
    }

    #[test]
    fn test_navigate_empty_object() {
        let json = json!({});
        let segments = vec![PathSegment::Field("name".into())];

        let result = navigate(&json, &segments);

        assert_eq!(result, None);
    }
}

mod edge_case_tests {
    use super::*;

    #[test]
    fn test_navigate_empty_segments() {
        let json = json!({"name": "Alice"});
        let segments: Vec<PathSegment> = vec![];

        let result = navigate(&json, &segments);

        // Empty path returns root
        assert_eq!(result, Some(&json));
    }

    #[test]
    fn test_navigate_null_value() {
        let json = json!({"value": null});
        let segments = vec![PathSegment::Field("value".into())];

        let result = navigate(&json, &segments);

        assert_eq!(result, Some(&Value::Null));
    }

    #[test]
    fn test_navigate_boolean_value() {
        let json = json!({"active": true});
        let segments = vec![PathSegment::Field("active".into())];

        let result = navigate(&json, &segments);

        assert_eq!(result, Some(&json!(true)));
    }

    #[test]
    fn test_navigate_returns_borrowed_reference() {
        let json = json!({"user": {"name": "Alice"}});
        let segments = vec![PathSegment::Field("user".into())];

        let result = navigate(&json, &segments);

        // Verify it's a reference into the original JSON
        assert!(result.is_some());
        let user = result.unwrap();
        assert_eq!(user.get("name"), Some(&json!("Alice")));
    }
}

mod complex_path_tests {
    use super::*;

    #[test]
    fn test_navigate_realistic_api_path() {
        let json = json!({
            "response": {
                "data": {
                    "users": [
                        {"addresses": [{"city": "NYC"}, {"city": "LA"}]}
                    ]
                }
            }
        });
        let segments = vec![
            PathSegment::Field("response".into()),
            PathSegment::Field("data".into()),
            PathSegment::Field("users".into()),
            PathSegment::ArrayIterator,
            PathSegment::Field("addresses".into()),
            PathSegment::ArrayIndex(0),
            PathSegment::Field("city".into()),
        ];

        let result = navigate(&json, &segments);

        assert_eq!(result, Some(&json!("NYC")));
    }

    #[test]
    fn test_navigate_mixed_iterators_and_indices() {
        let json = json!({
            "matrix": [[1, 2], [3, 4], [5, 6]]
        });
        let segments = vec![
            PathSegment::Field("matrix".into()),
            PathSegment::ArrayIndex(1),
            PathSegment::ArrayIterator,
        ];

        let result = navigate(&json, &segments);

        assert_eq!(result, Some(&json!(3)));
    }
}

mod navigate_multi_tests {
    use super::*;

    #[test]
    fn test_empty_segments_returns_root() {
        let json = json!({"name": "Alice"});
        let result = navigate_multi(&json, &[], ARRAY_SAMPLE_SIZE);
        assert_eq!(result, vec![&json]);
    }

    #[test]
    fn test_fan_out_returns_n_elements() {
        let json = json!([{"a": 1}, {"b": 2}, {"c": 3}]);
        let result = navigate_multi(&json, &[PathSegment::ArrayIterator], ARRAY_SAMPLE_SIZE);

        assert_eq!(result.len(), 3);
        assert_eq!(result[0], &json!({"a": 1}));
        assert_eq!(result[1], &json!({"b": 2}));
        assert_eq!(result[2], &json!({"c": 3}));
    }

    #[test]
    fn test_field_then_array_iterator() {
        let json = json!({"users": [{"id": 1}, {"id": 2}]});
        let segments = vec![
            PathSegment::Field("users".into()),
            PathSegment::ArrayIterator,
        ];
        let result = navigate_multi(&json, &segments, ARRAY_SAMPLE_SIZE);

        assert_eq!(result.len(), 2);
        assert_eq!(result[0], &json!({"id": 1}));
        assert_eq!(result[1], &json!({"id": 2}));
    }

    #[test]
    fn test_nested_iterators_fan_out() {
        let json = json!([[{"a": 1}, {"b": 2}], [{"c": 3}]]);
        let segments = vec![PathSegment::ArrayIterator, PathSegment::ArrayIterator];
        let result = navigate_multi(&json, &segments, ARRAY_SAMPLE_SIZE);

        assert_eq!(result.len(), 3);
        assert_eq!(result[0], &json!({"a": 1}));
        assert_eq!(result[1], &json!({"b": 2}));
        assert_eq!(result[2], &json!({"c": 3}));
    }

    #[test]
    fn test_sample_size_limits_elements() {
        let json = json!([{"a": 1}, {"b": 2}, {"c": 3}, {"d": 4}, {"e": 5}]);
        let result = navigate_multi(&json, &[PathSegment::ArrayIterator], 2);

        assert_eq!(result.len(), 2);
        assert_eq!(result[0], &json!({"a": 1}));
        assert_eq!(result[1], &json!({"b": 2}));
    }

    #[test]
    fn test_array_index_does_not_fan_out() {
        let json = json!([{"id": 1}, {"id": 2}]);
        let result = navigate_multi(&json, &[PathSegment::ArrayIndex(1)], ARRAY_SAMPLE_SIZE);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0], &json!({"id": 2}));
    }

    #[test]
    fn test_empty_array_returns_empty() {
        let json = json!([]);
        let result = navigate_multi(&json, &[PathSegment::ArrayIterator], ARRAY_SAMPLE_SIZE);

        assert!(result.is_empty());
    }

    #[test]
    fn test_type_mismatch_returns_empty() {
        let json = json!("not an object");
        let segments = vec![PathSegment::Field("name".into())];
        let result = navigate_multi(&json, &segments, ARRAY_SAMPLE_SIZE);

        assert!(result.is_empty());
    }

    #[test]
    fn test_field_after_fan_out_filters_to_matching() {
        let json = json!([
            {"profile": {"name": "Alice"}},
            {"profile": {"name": "Bob"}}
        ]);
        let segments = vec![
            PathSegment::ArrayIterator,
            PathSegment::Field("profile".into()),
        ];
        let result = navigate_multi(&json, &segments, ARRAY_SAMPLE_SIZE);

        assert_eq!(result.len(), 2);
        assert_eq!(result[0], &json!({"name": "Alice"}));
        assert_eq!(result[1], &json!({"name": "Bob"}));
    }

    #[test]
    fn test_max_navigated_values_caps_total() {
        // Build a large nested structure that would produce >100 values
        // 15 outer * 15 inner = 225 uncapped, should cap at 100
        let inner: Vec<Value> = (0..15).map(|i| json!({"v": i})).collect();
        let outer: Vec<Value> = (0..15).map(|_| Value::Array(inner.clone())).collect();
        let json = Value::Array(outer);

        let segments = vec![PathSegment::ArrayIterator, PathSegment::ArrayIterator];
        let result = navigate_multi(&json, &segments, 15);

        assert!(
            result.len() <= 100,
            "Should cap at MAX_NAVIGATED_VALUES, got {}",
            result.len()
        );
    }

    #[test]
    fn test_field_on_non_object_values_skipped() {
        // Mixed array: some objects, some scalars — Field should skip non-objects
        let json = json!([{"name": "Alice"}, "string", 42, {"name": "Bob"}]);
        let segments = vec![
            PathSegment::ArrayIterator,
            PathSegment::Field("name".into()),
        ];
        let result = navigate_multi(&json, &segments, ARRAY_SAMPLE_SIZE);

        assert_eq!(result.len(), 2);
        assert_eq!(result[0], &json!("Alice"));
        assert_eq!(result[1], &json!("Bob"));
    }

    #[test]
    fn test_negative_index_underflow_skipped() {
        let json = json!([[{"a": 1}], [{"b": 2}]]);
        let segments = vec![PathSegment::ArrayIterator, PathSegment::ArrayIndex(-1000)];
        let result = navigate_multi(&json, &segments, ARRAY_SAMPLE_SIZE);

        assert!(
            result.is_empty(),
            "Negative index underflow should produce empty results"
        );
    }

    #[test]
    fn test_iterator_on_non_array_skipped() {
        // ArrayIterator on non-array values should skip them
        let json = json!([{"items": [1, 2]}, {"items": "not_array"}]);
        let segments = vec![
            PathSegment::ArrayIterator,
            PathSegment::Field("items".into()),
            PathSegment::ArrayIterator,
        ];
        let result = navigate_multi(&json, &segments, ARRAY_SAMPLE_SIZE);

        // Only the first element's items array is iterated
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], &json!(1));
        assert_eq!(result[1], &json!(2));
    }

    #[test]
    fn test_intermediate_empty_returns_empty() {
        // If an intermediate segment produces no matches, result is empty
        let json = json!({"users": [{"name": "Alice"}]});
        let segments = vec![
            PathSegment::Field("users".into()),
            PathSegment::ArrayIterator,
            PathSegment::Field("nonexistent".into()),
            PathSegment::Field("deep".into()),
        ];
        let result = navigate_multi(&json, &segments, ARRAY_SAMPLE_SIZE);

        assert!(result.is_empty());
    }
}
