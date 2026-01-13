use super::common::tracker_for;
use crate::autocomplete::BraceTracker;
use crate::autocomplete::context::{SuggestionContext, analyze_context};
use crate::autocomplete::get_suggestions;

fn get_var_suggestions(query: &str, cursor_pos: usize) -> Vec<String> {
    let tracker = tracker_for(query);
    get_suggestions(query, cursor_pos, None, None, &tracker)
        .into_iter()
        .map(|s| s.text)
        .collect()
}

fn assert_context_is_variable(before_cursor: &str) {
    let tracker = tracker_for(before_cursor);
    let (context, _) = analyze_context(before_cursor, &tracker);
    assert_eq!(
        context,
        SuggestionContext::VariableContext,
        "Expected VariableContext for '{}'",
        before_cursor
    );
}

fn assert_context_is_not_variable(before_cursor: &str) {
    let tracker = tracker_for(before_cursor);
    let (context, _) = analyze_context(before_cursor, &tracker);
    assert_ne!(
        context,
        SuggestionContext::VariableContext,
        "Expected non-VariableContext for '{}'",
        before_cursor
    );
}

mod basic_variable_usage {
    use super::*;

    #[test]
    fn typing_dollar_triggers_variable_context() {
        assert_context_is_variable("$");
    }

    #[test]
    fn typing_variable_name_triggers_context() {
        assert_context_is_variable("$x");
        assert_context_is_variable("$foo");
        assert_context_is_variable("$my_var");
    }

    #[test]
    fn variable_after_pipe() {
        assert_context_is_variable(". | $");
        assert_context_is_variable(". | $x");
    }

    #[test]
    fn variable_inside_parens() {
        assert_context_is_variable("map($");
        assert_context_is_variable("select($x");
    }

    #[test]
    fn variable_after_operator() {
        assert_context_is_variable(". + $");
        assert_context_is_variable(".x == $");
    }
}

mod variable_definition_no_suggestions {
    use super::*;

    #[test]
    fn after_as_keyword() {
        assert_context_is_not_variable(". as $");
        assert_context_is_not_variable(". as $x");
    }

    #[test]
    fn after_as_with_whitespace() {
        assert_context_is_not_variable(". as  $");
        assert_context_is_not_variable(".as $");
    }

    #[test]
    fn in_reduce_definition() {
        assert_context_is_not_variable("reduce .[] as $");
        assert_context_is_not_variable("reduce .[] as $item");
    }

    #[test]
    fn in_foreach_definition() {
        assert_context_is_not_variable("foreach .[] as $");
        assert_context_is_not_variable("foreach .[] as $x");
    }

    #[test]
    fn after_label_keyword() {
        assert_context_is_not_variable("label $");
        assert_context_is_not_variable("label $out");
    }

    #[test]
    fn in_array_destructuring() {
        assert_context_is_not_variable(". as [$");
        assert_context_is_not_variable(". as [$a, $");
        assert_context_is_not_variable(". as [$first, $second");
    }

    #[test]
    fn in_object_destructuring() {
        assert_context_is_not_variable(". as {name: $");
        assert_context_is_not_variable(". as {name: $n, age: $");
    }
}

mod suggestion_generation {
    use super::*;

    #[test]
    fn includes_builtin_variables() {
        let suggestions = get_var_suggestions("$", 1);
        assert!(suggestions.contains(&"$ENV".to_string()));
        assert!(suggestions.contains(&"$__loc__".to_string()));
    }

    #[test]
    fn includes_user_defined_variable() {
        let query = ". as $x | $";
        let suggestions = get_var_suggestions(query, query.len());
        assert!(suggestions.contains(&"$x".to_string()));
    }

    #[test]
    fn includes_variable_from_reduce() {
        let query = "reduce .[] as $item (0; $";
        let suggestions = get_var_suggestions(query, query.len());
        assert!(suggestions.contains(&"$item".to_string()));
    }

    #[test]
    fn includes_multiple_variables() {
        let query = ". as $a | . as $b | $";
        let suggestions = get_var_suggestions(query, query.len());
        assert!(suggestions.contains(&"$a".to_string()));
        assert!(suggestions.contains(&"$b".to_string()));
    }

    #[test]
    fn deduplicates_repeated_definitions() {
        let query = ". as $x | . as $x | $";
        let suggestions = get_var_suggestions(query, query.len());
        let count = suggestions.iter().filter(|s| *s == "$x").count();
        assert_eq!(count, 1, "Variable $x should appear only once");
    }

    #[test]
    fn includes_variables_from_entire_query() {
        let query = ". as $x | $ | . as $y";
        let cursor_pos = query.find("$ |").unwrap() + 1;
        let suggestions = get_var_suggestions(query, cursor_pos);
        assert!(suggestions.contains(&"$x".to_string()));
        assert!(suggestions.contains(&"$y".to_string()));
    }
}

mod case_sensitive_filtering {
    use super::*;

    #[test]
    fn filters_case_sensitively() {
        let query = ". as $Item | $it";
        let suggestions = get_var_suggestions(query, query.len());
        assert!(!suggestions.contains(&"$Item".to_string()));
    }

    #[test]
    fn matches_case_sensitive_prefix() {
        let query = ". as $item | $it";
        let suggestions = get_var_suggestions(query, query.len());
        assert!(suggestions.contains(&"$item".to_string()));
    }

    #[test]
    fn matches_uppercase_variable() {
        let query = ". as $Item | $It";
        let suggestions = get_var_suggestions(query, query.len());
        assert!(suggestions.contains(&"$Item".to_string()));
    }

    #[test]
    fn filters_env_by_prefix() {
        let suggestions = get_var_suggestions("$E", 2);
        assert!(suggestions.contains(&"$ENV".to_string()));
        assert!(!suggestions.contains(&"$__loc__".to_string()));
    }

    #[test]
    fn filters_loc_by_prefix() {
        let suggestions = get_var_suggestions("$__", 3);
        assert!(suggestions.contains(&"$__loc__".to_string()));
        assert!(!suggestions.contains(&"$ENV".to_string()));
    }
}

mod destructuring_variables {
    use super::*;

    #[test]
    fn extracts_from_array_destructuring() {
        let query = ". as [$first, $second] | $";
        let suggestions = get_var_suggestions(query, query.len());
        assert!(suggestions.contains(&"$first".to_string()));
        assert!(suggestions.contains(&"$second".to_string()));
    }

    #[test]
    fn extracts_from_object_destructuring() {
        let query = ". as {name: $n, age: $a} | $";
        let suggestions = get_var_suggestions(query, query.len());
        assert!(suggestions.contains(&"$n".to_string()));
        assert!(suggestions.contains(&"$a".to_string()));
    }
}

mod edge_cases {
    use super::*;

    #[test]
    fn empty_query_with_dollar() {
        let suggestions = get_var_suggestions("$", 1);
        assert!(suggestions.contains(&"$ENV".to_string()));
        assert!(suggestions.contains(&"$__loc__".to_string()));
        assert_eq!(suggestions.len(), 2);
    }

    #[test]
    fn variable_with_underscore() {
        let query = ". as $my_var | $my";
        let suggestions = get_var_suggestions(query, query.len());
        assert!(suggestions.contains(&"$my_var".to_string()));
    }

    #[test]
    fn variable_with_numbers() {
        let query = ". as $x123 | $x";
        let suggestions = get_var_suggestions(query, query.len());
        assert!(suggestions.contains(&"$x123".to_string()));
    }

    #[test]
    fn has_function_not_matched_as_keyword() {
        let tracker = BraceTracker::new();
        let (context, _) = analyze_context("has($", &tracker);
        assert_eq!(context, SuggestionContext::VariableContext);
    }

    #[test]
    fn variable_in_nested_map() {
        let query = ".items as $data | map(. + $";
        let suggestions = get_var_suggestions(query, query.len());
        assert!(suggestions.contains(&"$data".to_string()));
    }
}

mod mid_query_editing {
    use super::*;

    #[test]
    fn editing_at_start() {
        let query = "$ | . as $z";
        let suggestions = get_var_suggestions(query, 1);
        assert!(suggestions.contains(&"$z".to_string()));
    }

    #[test]
    fn editing_in_middle() {
        let query = ". as $x | $ | . as $y";
        let cursor_pos = query.find("$ |").unwrap() + 1;
        let suggestions = get_var_suggestions(query, cursor_pos);
        assert!(suggestions.contains(&"$x".to_string()));
        assert!(suggestions.contains(&"$y".to_string()));
    }
}
