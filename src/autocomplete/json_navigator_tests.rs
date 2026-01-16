use super::json_navigator::navigate;
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
