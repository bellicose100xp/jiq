use super::path_parser::{PathSegment, parse_path};

mod simple_field_tests {
    use super::*;

    #[test]
    fn test_single_field() {
        let result = parse_path(".name");
        assert_eq!(result.segments, vec![]);
        assert_eq!(result.partial, "name");
    }

    #[test]
    fn test_single_field_with_trailing_dot() {
        let result = parse_path(".name.");
        assert_eq!(result.segments, vec![PathSegment::Field("name".into())]);
        assert_eq!(result.partial, "");
    }

    #[test]
    fn test_field_with_underscore() {
        let result = parse_path(".user_name");
        assert_eq!(result.segments, vec![]);
        assert_eq!(result.partial, "user_name");
    }

    #[test]
    fn test_field_with_numbers() {
        let result = parse_path(".field123");
        assert_eq!(result.segments, vec![]);
        assert_eq!(result.partial, "field123");
    }
}

mod nested_field_tests {
    use super::*;

    #[test]
    fn test_two_fields() {
        let result = parse_path(".user.profile");
        assert_eq!(result.segments, vec![PathSegment::Field("user".into())]);
        assert_eq!(result.partial, "profile");
    }

    #[test]
    fn test_two_fields_trailing_dot() {
        let result = parse_path(".user.profile.");
        assert_eq!(
            result.segments,
            vec![
                PathSegment::Field("user".into()),
                PathSegment::Field("profile".into())
            ]
        );
        assert_eq!(result.partial, "");
    }

    #[test]
    fn test_three_fields() {
        let result = parse_path(".a.b.c");
        assert_eq!(
            result.segments,
            vec![
                PathSegment::Field("a".into()),
                PathSegment::Field("b".into())
            ]
        );
        assert_eq!(result.partial, "c");
    }

    #[test]
    fn test_deep_nesting() {
        let result = parse_path(".a.b.c.d.e.");
        assert_eq!(
            result.segments,
            vec![
                PathSegment::Field("a".into()),
                PathSegment::Field("b".into()),
                PathSegment::Field("c".into()),
                PathSegment::Field("d".into()),
                PathSegment::Field("e".into()),
            ]
        );
        assert_eq!(result.partial, "");
    }
}

mod array_iterator_tests {
    use super::*;

    #[test]
    fn test_array_iterator_only() {
        let result = parse_path(".[]");
        assert_eq!(result.segments, vec![PathSegment::ArrayIterator]);
        assert_eq!(result.partial, "");
    }

    #[test]
    fn test_array_iterator_with_trailing_dot() {
        let result = parse_path(".[].");
        assert_eq!(result.segments, vec![PathSegment::ArrayIterator]);
        assert_eq!(result.partial, "");
    }

    #[test]
    fn test_field_then_iterator() {
        let result = parse_path(".items[]");
        assert_eq!(
            result.segments,
            vec![
                PathSegment::Field("items".into()),
                PathSegment::ArrayIterator
            ]
        );
        assert_eq!(result.partial, "");
    }

    #[test]
    fn test_field_then_iterator_then_field() {
        let result = parse_path(".items[].name");
        assert_eq!(
            result.segments,
            vec![
                PathSegment::Field("items".into()),
                PathSegment::ArrayIterator
            ]
        );
        assert_eq!(result.partial, "name");
    }

    #[test]
    fn test_field_then_iterator_then_field_trailing_dot() {
        let result = parse_path(".items[].name.");
        assert_eq!(
            result.segments,
            vec![
                PathSegment::Field("items".into()),
                PathSegment::ArrayIterator,
                PathSegment::Field("name".into())
            ]
        );
        assert_eq!(result.partial, "");
    }

    #[test]
    fn test_chained_iterators() {
        let result = parse_path(".data[][].");
        assert_eq!(
            result.segments,
            vec![
                PathSegment::Field("data".into()),
                PathSegment::ArrayIterator,
                PathSegment::ArrayIterator
            ]
        );
        assert_eq!(result.partial, "");
    }

    #[test]
    fn test_optional_iterator() {
        let result = parse_path(".[]?.");
        assert_eq!(result.segments, vec![PathSegment::ArrayIterator]);
        assert_eq!(result.partial, "");
    }
}

mod array_index_tests {
    use super::*;

    #[test]
    fn test_index_zero() {
        let result = parse_path(".[0]");
        assert_eq!(result.segments, vec![PathSegment::ArrayIndex(0)]);
        assert_eq!(result.partial, "");
    }

    #[test]
    fn test_positive_index() {
        let result = parse_path(".[5]");
        assert_eq!(result.segments, vec![PathSegment::ArrayIndex(5)]);
        assert_eq!(result.partial, "");
    }

    #[test]
    fn test_negative_index() {
        let result = parse_path(".[-1]");
        assert_eq!(result.segments, vec![PathSegment::ArrayIndex(-1)]);
        assert_eq!(result.partial, "");
    }

    #[test]
    fn test_large_index() {
        let result = parse_path(".[999]");
        assert_eq!(result.segments, vec![PathSegment::ArrayIndex(999)]);
        assert_eq!(result.partial, "");
    }

    #[test]
    fn test_index_then_field() {
        let result = parse_path(".[0].name");
        assert_eq!(result.segments, vec![PathSegment::ArrayIndex(0)]);
        assert_eq!(result.partial, "name");
    }

    #[test]
    fn test_field_then_index() {
        let result = parse_path(".items[0]");
        assert_eq!(
            result.segments,
            vec![
                PathSegment::Field("items".into()),
                PathSegment::ArrayIndex(0)
            ]
        );
        assert_eq!(result.partial, "");
    }

    #[test]
    fn test_field_index_field() {
        let result = parse_path(".items[0].name.");
        assert_eq!(
            result.segments,
            vec![
                PathSegment::Field("items".into()),
                PathSegment::ArrayIndex(0),
                PathSegment::Field("name".into())
            ]
        );
        assert_eq!(result.partial, "");
    }
}

mod optional_field_tests {
    use super::*;

    #[test]
    fn test_optional_field() {
        let result = parse_path(".name?");
        assert_eq!(
            result.segments,
            vec![PathSegment::OptionalField("name".into())]
        );
        assert_eq!(result.partial, "");
    }

    #[test]
    fn test_optional_field_then_field() {
        let result = parse_path(".user?.profile");
        assert_eq!(
            result.segments,
            vec![PathSegment::OptionalField("user".into())]
        );
        assert_eq!(result.partial, "profile");
    }

    #[test]
    fn test_optional_field_trailing_dot() {
        let result = parse_path(".user?.profile?.");
        assert_eq!(
            result.segments,
            vec![
                PathSegment::OptionalField("user".into()),
                PathSegment::OptionalField("profile".into())
            ]
        );
        assert_eq!(result.partial, "");
    }

    #[test]
    fn test_mixed_optional_and_regular() {
        let result = parse_path(".user?.settings.theme?.");
        assert_eq!(
            result.segments,
            vec![
                PathSegment::OptionalField("user".into()),
                PathSegment::Field("settings".into()),
                PathSegment::OptionalField("theme".into())
            ]
        );
        assert_eq!(result.partial, "");
    }
}

mod bracket_notation_tests {
    use super::*;

    #[test]
    fn test_bracket_simple_field() {
        let result = parse_path(r#".["name"]"#);
        assert_eq!(result.segments, vec![PathSegment::Field("name".into())]);
        assert_eq!(result.partial, "");
    }

    #[test]
    fn test_bracket_field_with_hyphen() {
        let result = parse_path(r#".["foo-bar"]"#);
        assert_eq!(result.segments, vec![PathSegment::Field("foo-bar".into())]);
        assert_eq!(result.partial, "");
    }

    #[test]
    fn test_bracket_field_with_spaces() {
        let result = parse_path(r#".["field name"]"#);
        assert_eq!(
            result.segments,
            vec![PathSegment::Field("field name".into())]
        );
        assert_eq!(result.partial, "");
    }

    #[test]
    fn test_bracket_field_then_regular_field() {
        let result = parse_path(r#".["user"].profile"#);
        assert_eq!(result.segments, vec![PathSegment::Field("user".into())]);
        assert_eq!(result.partial, "profile");
    }

    #[test]
    fn test_mixed_bracket_and_dot_notation() {
        let result = parse_path(r#".user["settings"].theme."#);
        assert_eq!(
            result.segments,
            vec![
                PathSegment::Field("user".into()),
                PathSegment::Field("settings".into()),
                PathSegment::Field("theme".into())
            ]
        );
        assert_eq!(result.partial, "");
    }

    #[test]
    fn test_escaped_quote_in_bracket() {
        let result = parse_path(r#".["say\"hello"]"#);
        assert_eq!(
            result.segments,
            vec![PathSegment::Field("say\"hello".into())]
        );
        assert_eq!(result.partial, "");
    }
}

mod partial_field_tests {
    use super::*;

    #[test]
    fn test_partial_at_start() {
        let result = parse_path(".us");
        assert_eq!(result.segments, vec![]);
        assert_eq!(result.partial, "us");
    }

    #[test]
    fn test_partial_after_field() {
        let result = parse_path(".user.pro");
        assert_eq!(result.segments, vec![PathSegment::Field("user".into())]);
        assert_eq!(result.partial, "pro");
    }

    #[test]
    fn test_partial_after_iterator() {
        let result = parse_path(".[].na");
        assert_eq!(result.segments, vec![PathSegment::ArrayIterator]);
        assert_eq!(result.partial, "na");
    }

    #[test]
    fn test_partial_after_index() {
        let result = parse_path(".[0].val");
        assert_eq!(result.segments, vec![PathSegment::ArrayIndex(0)]);
        assert_eq!(result.partial, "val");
    }
}

mod edge_case_tests {
    use super::*;

    #[test]
    fn test_empty_input() {
        let result = parse_path("");
        assert_eq!(result.segments, vec![]);
        assert_eq!(result.partial, "");
    }

    #[test]
    fn test_dot_only() {
        let result = parse_path(".");
        assert_eq!(result.segments, vec![]);
        assert_eq!(result.partial, "");
    }

    #[test]
    fn test_identity() {
        // Just "." represents identity in jq
        let result = parse_path(".");
        assert!(result.segments.is_empty());
        assert!(result.partial.is_empty());
    }

    #[test]
    fn test_double_dot_skipped() {
        // ".." is recursive descent, not supported
        let result = parse_path("..name");
        assert_eq!(result.segments, vec![]);
        assert_eq!(result.partial, "name");
    }

    #[test]
    fn test_starts_without_dot() {
        // Edge case: input doesn't start with dot
        let result = parse_path("name");
        assert_eq!(result.segments, vec![]);
        assert_eq!(result.partial, "name");
    }
}

mod function_call_skipping_tests {
    use super::*;

    #[test]
    fn test_simple_function_call() {
        // select(...) should be skipped, then continue parsing [].name
        let result = parse_path(r#"select(.status=="ACTIVE")[].name"#);
        assert_eq!(result.segments, vec![PathSegment::ArrayIterator]);
        assert_eq!(result.partial, "name");
    }

    #[test]
    fn test_function_call_then_field_trailing_dot() {
        let result = parse_path(r#"select(.x)[].config."#);
        assert_eq!(
            result.segments,
            vec![
                PathSegment::ArrayIterator,
                PathSegment::Field("config".into())
            ]
        );
        assert_eq!(result.partial, "");
    }

    #[test]
    fn test_nested_function_calls() {
        // Nested parens: sort_by(.age | abs) has nested structure
        let result = parse_path(r#"sort_by(.age | tonumber)[].name."#);
        assert_eq!(
            result.segments,
            vec![
                PathSegment::ArrayIterator,
                PathSegment::Field("name".into())
            ]
        );
        assert_eq!(result.partial, "");
    }

    #[test]
    fn test_function_with_string_containing_parens() {
        // String inside function contains parens - should not confuse parser
        let result = parse_path(r#"select(.name == "foo(bar)")[].field"#);
        assert_eq!(result.segments, vec![PathSegment::ArrayIterator]);
        assert_eq!(result.partial, "field");
    }

    #[test]
    fn test_optional_function_call() {
        // Function call with ? after it
        let result = parse_path(r#"select(.x)?[].name"#);
        assert_eq!(result.segments, vec![PathSegment::ArrayIterator]);
        assert_eq!(result.partial, "name");
    }

    #[test]
    fn test_function_call_in_middle_of_path() {
        // Path with function in middle: .items | select(.active)[].config.
        let result = parse_path(r#"select(.active)[].config.setting."#);
        assert_eq!(
            result.segments,
            vec![
                PathSegment::ArrayIterator,
                PathSegment::Field("config".into()),
                PathSegment::Field("setting".into())
            ]
        );
        assert_eq!(result.partial, "");
    }

    #[test]
    fn test_map_function() {
        let result = parse_path(r#"map(.x + 1)[].value"#);
        assert_eq!(result.segments, vec![PathSegment::ArrayIterator]);
        assert_eq!(result.partial, "value");
    }

    #[test]
    fn test_multiple_function_calls_chained() {
        // Multiple functions: select(...) | sort_by(...)[].field
        // After first (, we skip to ), then continue. Pipe stops us, so we get remaining
        let result = parse_path(r#"select(.a) | sort_by(.b)[].name"#);
        // Parser stops at space after select(.a)
        assert_eq!(result.segments, vec![]);
        assert_eq!(result.partial, "");
    }

    #[test]
    fn test_function_at_start_then_deep_path() {
        let result = parse_path(r#"first(.items)[].nested.deep.field."#);
        assert_eq!(
            result.segments,
            vec![
                PathSegment::ArrayIterator,
                PathSegment::Field("nested".into()),
                PathSegment::Field("deep".into()),
                PathSegment::Field("field".into())
            ]
        );
        assert_eq!(result.partial, "");
    }

    #[test]
    fn test_group_by_function() {
        let result = parse_path(r#"group_by(.category)[].items[]."#);
        assert_eq!(
            result.segments,
            vec![
                PathSegment::ArrayIterator,
                PathSegment::Field("items".into()),
                PathSegment::ArrayIterator
            ]
        );
        assert_eq!(result.partial, "");
    }
}

mod complex_path_tests {
    use super::*;

    #[test]
    fn test_complex_mixed_path() {
        let result = parse_path(".users[].profile.settings[0].theme.");
        assert_eq!(
            result.segments,
            vec![
                PathSegment::Field("users".into()),
                PathSegment::ArrayIterator,
                PathSegment::Field("profile".into()),
                PathSegment::Field("settings".into()),
                PathSegment::ArrayIndex(0),
                PathSegment::Field("theme".into()),
            ]
        );
        assert_eq!(result.partial, "");
    }

    #[test]
    fn test_complex_with_optional() {
        let result = parse_path(".data?.items[]?.value?.");
        assert_eq!(
            result.segments,
            vec![
                PathSegment::OptionalField("data".into()),
                PathSegment::Field("items".into()),
                PathSegment::ArrayIterator,
                PathSegment::OptionalField("value".into()),
            ]
        );
        assert_eq!(result.partial, "");
    }

    #[test]
    fn test_complex_with_bracket_notation() {
        let result = parse_path(r#".["users"][0]["full-name"]."#);
        assert_eq!(
            result.segments,
            vec![
                PathSegment::Field("users".into()),
                PathSegment::ArrayIndex(0),
                PathSegment::Field("full-name".into()),
            ]
        );
        assert_eq!(result.partial, "");
    }

    #[test]
    fn test_realistic_api_response_path() {
        let result = parse_path(".response.data.users[].addresses[0].city.");
        assert_eq!(
            result.segments,
            vec![
                PathSegment::Field("response".into()),
                PathSegment::Field("data".into()),
                PathSegment::Field("users".into()),
                PathSegment::ArrayIterator,
                PathSegment::Field("addresses".into()),
                PathSegment::ArrayIndex(0),
                PathSegment::Field("city".into()),
            ]
        );
        assert_eq!(result.partial, "");
    }
}
