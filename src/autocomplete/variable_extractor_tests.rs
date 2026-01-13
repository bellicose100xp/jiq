use super::*;

fn assert_contains_all(result: &[String], expected: &[&str]) {
    for exp in expected {
        assert!(
            result.contains(&exp.to_string()),
            "Expected {:?} to contain {:?}",
            result,
            exp
        );
    }
}

fn assert_not_contains(result: &[String], not_expected: &[&str]) {
    for exp in not_expected {
        assert!(
            !result.contains(&exp.to_string()),
            "Expected {:?} to NOT contain {:?}",
            result,
            exp
        );
    }
}

mod extract_variables_tests {
    use super::*;

    #[test]
    fn includes_builtin_variables() {
        let result = extract_variables("");
        assert_contains_all(&result, &["$ENV", "$__loc__"]);
    }

    #[test]
    fn simple_as_binding() {
        let result = extract_variables(". as $x | $x");
        assert_contains_all(&result, &["$x", "$ENV", "$__loc__"]);
    }

    #[test]
    fn multiple_as_bindings() {
        let result = extract_variables(". as $a | . as $b | $a + $b");
        assert_contains_all(&result, &["$a", "$b"]);
    }

    #[test]
    fn reduce_binding() {
        let result = extract_variables("reduce .[] as $item (0; . + $item)");
        assert_contains_all(&result, &["$item"]);
    }

    #[test]
    fn foreach_binding() {
        let result = extract_variables("foreach .[] as $x (0; . + $x)");
        assert_contains_all(&result, &["$x"]);
    }

    #[test]
    fn label_binding() {
        let result = extract_variables(
            "label $out | foreach .[] as $x (0; . + $x; if . > 3 then ., break $out else . end)",
        );
        assert_contains_all(&result, &["$out", "$x"]);
    }

    #[test]
    fn array_destructuring() {
        let result = extract_variables(". as [$first, $second] | $first");
        assert_contains_all(&result, &["$first", "$second"]);
    }

    #[test]
    fn object_destructuring() {
        let result = extract_variables(". as {name: $n, age: $a} | $n");
        assert_contains_all(&result, &["$n", "$a"]);
    }

    #[test]
    fn nested_array_destructuring() {
        let result = extract_variables(". as [[$a, $b], $c] | $a");
        assert_contains_all(&result, &["$a", "$b", "$c"]);
    }

    #[test]
    fn deduplicates_repeated_definitions() {
        let result = extract_variables(". as $x | . as $x | $x");
        let count = result.iter().filter(|v| *v == "$x").count();
        assert_eq!(count, 1, "Variable $x should appear only once");
    }

    #[test]
    fn variable_with_underscore() {
        let result = extract_variables(". as $my_var | $my_var");
        assert_contains_all(&result, &["$my_var"]);
    }

    #[test]
    fn variable_with_numbers() {
        let result = extract_variables(". as $x123 | $x123");
        assert_contains_all(&result, &["$x123"]);
    }

    #[test]
    fn ignores_variables_in_strings() {
        let result = extract_variables("\"as $fake\" | .");
        assert_not_contains(&result, &["$fake"]);
    }

    #[test]
    fn real_variable_alongside_string() {
        let result = extract_variables(". as $real | \"$fake\" | $real");
        assert_contains_all(&result, &["$real"]);
        assert_not_contains(&result, &["$fake"]);
    }

    #[test]
    fn escaped_quote_in_string() {
        let result = extract_variables(r#""\\" as $x | $x"#);
        assert_contains_all(&result, &["$x"]);
    }

    #[test]
    fn as_within_identifier_not_matched() {
        let result = extract_variables("has($x)");
        assert_not_contains(&result, &["$x"]);
    }

    #[test]
    fn empty_query() {
        let result = extract_variables("");
        assert_contains_all(&result, &["$ENV", "$__loc__"]);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn only_builtins_no_user_vars() {
        let result = extract_variables(".foo | .bar");
        assert_eq!(result.len(), 2);
        assert_contains_all(&result, &["$ENV", "$__loc__"]);
    }

    #[test]
    fn whitespace_after_as() {
        let result = extract_variables(". as   $spaced | $spaced");
        assert_contains_all(&result, &["$spaced"]);
    }

    #[test]
    fn complex_nested_expression() {
        let result =
            extract_variables("reduce (. as $outer | .[]) as $inner (0; . + $inner + $outer)");
        assert_contains_all(&result, &["$outer", "$inner"]);
    }

    #[test]
    fn map_with_variable() {
        let result = extract_variables(".items as $data | map(. + $data)");
        assert_contains_all(&result, &["$data"]);
    }
}

mod extract_single_variable_tests {
    use super::*;

    #[test]
    fn extracts_simple_variable() {
        let chars: Vec<char> = "$foo".chars().collect();
        let result = extract_single_variable(&chars, 0);
        assert_eq!(result, Some(("$foo".to_string(), 4)));
    }

    #[test]
    fn extracts_variable_with_underscore() {
        let chars: Vec<char> = "$my_var".chars().collect();
        let result = extract_single_variable(&chars, 0);
        assert_eq!(result, Some(("$my_var".to_string(), 7)));
    }

    #[test]
    fn extracts_variable_with_numbers() {
        let chars: Vec<char> = "$x123".chars().collect();
        let result = extract_single_variable(&chars, 0);
        assert_eq!(result, Some(("$x123".to_string(), 5)));
    }

    #[test]
    fn returns_none_for_empty_variable() {
        let chars: Vec<char> = "$ ".chars().collect();
        let result = extract_single_variable(&chars, 0);
        assert_eq!(result, None);
    }

    #[test]
    fn stops_at_delimiter() {
        let chars: Vec<char> = "$foo | bar".chars().collect();
        let result = extract_single_variable(&chars, 0);
        assert_eq!(result, Some(("$foo".to_string(), 4)));
    }
}

mod is_keyword_at_tests {
    use super::*;

    #[test]
    fn matches_as_at_start() {
        let chars: Vec<char> = "as $x".chars().collect();
        assert!(is_keyword_at(&chars, 0, "as"));
    }

    #[test]
    fn matches_as_after_space() {
        let chars: Vec<char> = ". as $x".chars().collect();
        assert!(is_keyword_at(&chars, 2, "as"));
    }

    #[test]
    fn does_not_match_has() {
        let chars: Vec<char> = "has($x)".chars().collect();
        assert!(!is_keyword_at(&chars, 1, "as"));
    }

    #[test]
    fn does_not_match_ask() {
        let chars: Vec<char> = "ask".chars().collect();
        assert!(!is_keyword_at(&chars, 0, "as"));
    }

    #[test]
    fn matches_label() {
        let chars: Vec<char> = "label $out".chars().collect();
        assert!(is_keyword_at(&chars, 0, "label"));
    }
}

mod extract_array_destructure_tests {
    use super::*;

    #[test]
    fn extracts_simple_array() {
        let chars: Vec<char> = "[$a, $b]".chars().collect();
        let result = extract_array_destructure_variables(&chars, 0);
        assert!(result.is_some());
        let (vars, _) = result.unwrap();
        assert_eq!(vars, vec!["$a", "$b"]);
    }

    #[test]
    fn extracts_nested_array() {
        let chars: Vec<char> = "[[$a], $b]".chars().collect();
        let result = extract_array_destructure_variables(&chars, 0);
        assert!(result.is_some());
        let (vars, _) = result.unwrap();
        assert_eq!(vars, vec!["$a", "$b"]);
    }

    #[test]
    fn handles_whitespace() {
        let chars: Vec<char> = "[ $a , $b ]".chars().collect();
        let result = extract_array_destructure_variables(&chars, 0);
        assert!(result.is_some());
        let (vars, _) = result.unwrap();
        assert_eq!(vars, vec!["$a", "$b"]);
    }
}

mod extract_object_destructure_tests {
    use super::*;

    #[test]
    fn extracts_simple_object() {
        let chars: Vec<char> = "{name: $n, age: $a}".chars().collect();
        let result = extract_object_destructure_variables(&chars, 0);
        assert!(result.is_some());
        let (vars, _) = result.unwrap();
        assert_eq!(vars, vec!["$n", "$a"]);
    }

    #[test]
    fn extracts_nested_object() {
        let chars: Vec<char> = "{outer: {inner: $i}, other: $o}".chars().collect();
        let result = extract_object_destructure_variables(&chars, 0);
        assert!(result.is_some());
        let (vars, _) = result.unwrap();
        assert_eq!(vars, vec!["$i", "$o"]);
    }
}
