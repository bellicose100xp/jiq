use super::common::tracker_for;
use crate::autocomplete::*;
use crate::query::ResultType;
use proptest::prelude::*;
use serde_json::Value;
use std::sync::Arc;

proptest! {
    #[test]
    fn prop_object_key_context_suggestions_no_leading_dot(
        partial in "[a-z]{1,10}",
        field_names in prop::collection::vec("[a-z]{1,10}", 1..5),
    ) {
        let query = format!("{{{}", partial);
        let tracker = tracker_for(&query);

        let json_fields: Vec<String> = field_names
            .iter()
            .map(|name| format!("\"{}\": \"value\"", name))
            .collect();
        let json_result = format!("{{{}}}", json_fields.join(", "));

        let parsed = serde_json::from_str::<Value>(&json_result).ok().map(Arc::new);
        let suggestions = get_suggestions(
            &query,
            query.len(),
            parsed,
            Some(ResultType::Object),
            &tracker,
        );

        for suggestion in &suggestions {
            prop_assert!(
                !suggestion.text.starts_with('.'),
                "ObjectKeyContext suggestion '{}' should NOT start with '.', query: '{}'",
                suggestion.text,
                query
            );
        }
    }

    #[test]
    fn prop_object_key_context_after_comma_no_leading_dot(
        first_key in "[a-z]{1,8}",
        partial in "[a-z]{1,10}",
        field_names in prop::collection::vec("[a-z]{1,10}", 1..5),
    ) {
        let query = format!("{{{}: .{}, {}", first_key, first_key, partial);
        let tracker = tracker_for(&query);

        let json_fields: Vec<String> = field_names
            .iter()
            .map(|name| format!("\"{}\": \"value\"", name))
            .collect();
        let json_result = format!("{{{}}}", json_fields.join(", "));

        let parsed = serde_json::from_str::<Value>(&json_result).ok().map(Arc::new);
        let suggestions = get_suggestions(
            &query,
            query.len(),
            parsed,
            Some(ResultType::Object),
            &tracker,
        );

        for suggestion in &suggestions {
            prop_assert!(
                !suggestion.text.starts_with('.'),
                "ObjectKeyContext suggestion '{}' should NOT start with '.', query: '{}'",
                suggestion.text,
                query
            );
        }
    }

    #[test]
    fn prop_non_object_contexts_never_return_object_key_context(
        prefix in "[a-z.| ]*",
        partial in "[a-z]{1,10}",
        brace_type in prop_oneof![Just('['), Just('(')],
    ) {
        let query = format!("{}{}{}", prefix, brace_type, partial);

        let tracker = tracker_for(&query);
        let (ctx, _) = analyze_context(&query, &tracker);

        prop_assert_ne!(
            ctx,
            SuggestionContext::ObjectKeyContext,
            "Query '{}' with innermost brace '{}' should NOT return ObjectKeyContext, got {:?}",
            query,
            brace_type,
            ctx
        );
    }

    #[test]
    fn prop_comma_in_non_object_context_not_object_key(
        prefix in "[a-z.| ]*",
        inner in "[a-z0-9., ]{0,20}",
        partial in "[a-z]{1,10}",
        brace_type in prop_oneof![Just('['), Just('(')],
    ) {
        let query = format!("{}{}{}, {}", prefix, brace_type, inner, partial);

        let tracker = tracker_for(&query);
        let (ctx, _) = analyze_context(&query, &tracker);

        prop_assert_ne!(
            ctx,
            SuggestionContext::ObjectKeyContext,
            "Query '{}' with comma inside '{}' should NOT return ObjectKeyContext, got {:?}",
            query,
            brace_type,
            ctx
        );
    }

    #[test]
    fn prop_field_context_preserved_at_start(
        partial in "[a-z]{1,10}",
    ) {
        let query = format!(".{}", partial);
        let tracker = tracker_for(&query);
        let (ctx, returned_partial) = analyze_context(&query, &tracker);

        prop_assert_eq!(
            ctx,
            SuggestionContext::FieldContext,
            "Query '{}' starting with '.' should return FieldContext, got {:?}",
            query,
            ctx
        );

        prop_assert!(
            returned_partial == partial,
            "Query '{}' should return partial '{}', got '{}'",
            query,
            partial,
            returned_partial
        );
    }

    #[test]
    fn prop_field_context_preserved_after_pipe(
        field1 in "[a-z]{1,8}",
        partial in "[a-z]{1,10}",
    ) {
        let query = format!(".{} | .{}", field1, partial);
        let tracker = tracker_for(&query);
        let (ctx, returned_partial) = analyze_context(&query, &tracker);

        prop_assert_eq!(
            ctx,
            SuggestionContext::FieldContext,
            "Query '{}' with pipe and dot should return FieldContext, got {:?}",
            query,
            ctx
        );

        prop_assert!(
            returned_partial == partial,
            "Query '{}' should return partial '{}', got '{}'",
            query,
            partial,
            returned_partial
        );
    }

    #[test]
    fn prop_field_context_preserved_in_function_call(
        func in "(map|select|sort_by|group_by|unique_by|min_by|max_by)",
        partial in "[a-z]{1,10}",
    ) {
        let query = format!("{}(.{}", func, partial);
        let tracker = tracker_for(&query);
        let (ctx, returned_partial) = analyze_context(&query, &tracker);

        prop_assert_eq!(
            ctx,
            SuggestionContext::FieldContext,
            "Query '{}' with function call and dot should return FieldContext, got {:?}",
            query,
            ctx
        );

        prop_assert!(
            returned_partial == partial,
            "Query '{}' should return partial '{}', got '{}'",
            query,
            partial,
            returned_partial
        );
    }

    #[test]
    fn prop_function_context_preserved_at_start(
        partial in "[a-z]{1,10}",
    ) {
        let query = partial.clone();
        let tracker = tracker_for(&query);
        let (ctx, returned_partial) = analyze_context(&query, &tracker);

        prop_assert_eq!(
            ctx,
            SuggestionContext::FunctionContext,
            "Query '{}' (bare identifier) should return FunctionContext, got {:?}",
            query,
            ctx
        );

        prop_assert!(
            returned_partial == partial,
            "Query '{}' should return partial '{}', got '{}'",
            query,
            partial,
            returned_partial
        );
    }

    #[test]
    fn prop_function_context_preserved_after_pipe(
        field in "[a-z]{1,8}",
        partial in "[a-z]{1,10}",
    ) {
        let query = format!(".{} | {}", field, partial);
        let tracker = tracker_for(&query);
        let (ctx, returned_partial) = analyze_context(&query, &tracker);

        prop_assert_eq!(
            ctx,
            SuggestionContext::FunctionContext,
            "Query '{}' with pipe and bare identifier should return FunctionContext, got {:?}",
            query,
            ctx
        );

        prop_assert!(
            returned_partial == partial,
            "Query '{}' should return partial '{}', got '{}'",
            query,
            partial,
            returned_partial
        );
    }

    #[test]
    fn prop_element_context_suggestions_no_brackets(
        field_names in prop::collection::vec("[a-z]{1,8}", 1..5),
    ) {
        let query = "map(.";
        let tracker = tracker_for(query);

        let json_arr: Vec<String> = field_names
            .iter()
            .map(|name| format!("{{\"{}\": \"value\"}}", name))
            .collect();
        let json_result = format!("[{}]", json_arr.first().unwrap_or(&"{}".to_string()));

        let parsed = serde_json::from_str::<Value>(&json_result).ok().map(Arc::new);
        let suggestions = get_suggestions(
            query,
            query.len(),
            parsed,
            Some(ResultType::ArrayOfObjects),
            &tracker,
        );

        for suggestion in &suggestions {
            if suggestion.text != ".[]" {
                prop_assert!(
                    !suggestion.text.contains("[]."),
                    "Element context suggestion '{}' should NOT contain '[].'",
                    suggestion.text
                );
            }
        }
    }

    #[test]
    fn prop_outside_context_suggestions_have_brackets(
        field_names in prop::collection::vec("[a-z]{1,8}", 1..5),
    ) {
        let query = ".";
        let tracker = tracker_for(query);

        let json_arr: Vec<String> = field_names
            .iter()
            .map(|name| format!("{{\"{}\": \"value\"}}", name))
            .collect();
        let json_result = format!("[{}]", json_arr.first().unwrap_or(&"{}".to_string()));

        let parsed = serde_json::from_str::<Value>(&json_result).ok().map(Arc::new);
        let suggestions = get_suggestions(
            query,
            query.len(),
            parsed,
            Some(ResultType::ArrayOfObjects),
            &tracker,
        );

        let field_suggestions: Vec<_> = suggestions
            .iter()
            .filter(|s| s.text != ".[]" && s.text.len() > 1)
            .collect();

        for suggestion in &field_suggestions {
            prop_assert!(
                suggestion.text.contains("[]."),
                "Outside element context, suggestion '{}' should contain '[].'",
                suggestion.text
            );
        }
    }
}
