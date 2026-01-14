use super::*;

mod text_object_target_tests {
    use super::*;

    #[test]
    fn from_char_word() {
        assert_eq!(
            TextObjectTarget::from_char('w'),
            Some(TextObjectTarget::Word)
        );
    }

    #[test]
    fn from_char_quotes() {
        assert_eq!(
            TextObjectTarget::from_char('"'),
            Some(TextObjectTarget::DoubleQuote)
        );
        assert_eq!(
            TextObjectTarget::from_char('\''),
            Some(TextObjectTarget::SingleQuote)
        );
        assert_eq!(
            TextObjectTarget::from_char('`'),
            Some(TextObjectTarget::Backtick)
        );
    }

    #[test]
    fn from_char_parentheses() {
        assert_eq!(
            TextObjectTarget::from_char('('),
            Some(TextObjectTarget::Parentheses)
        );
        assert_eq!(
            TextObjectTarget::from_char(')'),
            Some(TextObjectTarget::Parentheses)
        );
        assert_eq!(
            TextObjectTarget::from_char('b'),
            Some(TextObjectTarget::Parentheses)
        );
    }

    #[test]
    fn from_char_brackets() {
        assert_eq!(
            TextObjectTarget::from_char('['),
            Some(TextObjectTarget::Brackets)
        );
        assert_eq!(
            TextObjectTarget::from_char(']'),
            Some(TextObjectTarget::Brackets)
        );
    }

    #[test]
    fn from_char_braces() {
        assert_eq!(
            TextObjectTarget::from_char('{'),
            Some(TextObjectTarget::Braces)
        );
        assert_eq!(
            TextObjectTarget::from_char('}'),
            Some(TextObjectTarget::Braces)
        );
        assert_eq!(
            TextObjectTarget::from_char('B'),
            Some(TextObjectTarget::Braces)
        );
    }

    #[test]
    fn from_char_invalid() {
        assert_eq!(TextObjectTarget::from_char('x'), None);
        assert_eq!(TextObjectTarget::from_char('z'), None);
        assert_eq!(TextObjectTarget::from_char(' '), None);
    }
}

mod word_bounds_tests {
    use super::*;

    #[test]
    fn inner_word_at_start() {
        let text = "hello world";
        assert_eq!(
            find_word_bounds(text, 0, TextObjectScope::Inner),
            Some((0, 5))
        );
        assert_eq!(
            find_word_bounds(text, 2, TextObjectScope::Inner),
            Some((0, 5))
        );
        assert_eq!(
            find_word_bounds(text, 4, TextObjectScope::Inner),
            Some((0, 5))
        );
    }

    #[test]
    fn inner_word_at_end() {
        let text = "hello world";
        assert_eq!(
            find_word_bounds(text, 6, TextObjectScope::Inner),
            Some((6, 11))
        );
        assert_eq!(
            find_word_bounds(text, 8, TextObjectScope::Inner),
            Some((6, 11))
        );
        assert_eq!(
            find_word_bounds(text, 10, TextObjectScope::Inner),
            Some((6, 11))
        );
    }

    #[test]
    fn around_word_with_trailing_space() {
        let text = "hello world";
        assert_eq!(
            find_word_bounds(text, 0, TextObjectScope::Around),
            Some((0, 6))
        );
    }

    #[test]
    fn around_word_with_leading_space() {
        let text = "hello world";
        assert_eq!(
            find_word_bounds(text, 6, TextObjectScope::Around),
            Some((5, 11))
        );
    }

    #[test]
    fn word_with_underscore() {
        let text = "foo_bar baz";
        assert_eq!(
            find_word_bounds(text, 0, TextObjectScope::Inner),
            Some((0, 7))
        );
        assert_eq!(
            find_word_bounds(text, 4, TextObjectScope::Inner),
            Some((0, 7))
        );
    }

    #[test]
    fn cursor_on_non_word_char() {
        let text = ".foo.bar";
        assert_eq!(find_word_bounds(text, 0, TextObjectScope::Inner), None);
        assert_eq!(find_word_bounds(text, 4, TextObjectScope::Inner), None);
    }

    #[test]
    fn cursor_on_word_between_dots() {
        let text = ".foo.bar";
        assert_eq!(
            find_word_bounds(text, 1, TextObjectScope::Inner),
            Some((1, 4))
        );
        assert_eq!(
            find_word_bounds(text, 5, TextObjectScope::Inner),
            Some((5, 8))
        );
    }

    #[test]
    fn empty_text() {
        assert_eq!(find_word_bounds("", 0, TextObjectScope::Inner), None);
    }

    #[test]
    fn cursor_beyond_text() {
        let text = "hello";
        assert_eq!(find_word_bounds(text, 10, TextObjectScope::Inner), None);
    }

    #[test]
    fn single_word_no_spaces() {
        let text = "hello";
        assert_eq!(
            find_word_bounds(text, 2, TextObjectScope::Inner),
            Some((0, 5))
        );
        assert_eq!(
            find_word_bounds(text, 2, TextObjectScope::Around),
            Some((0, 5))
        );
    }

    #[test]
    fn multiple_spaces_around() {
        let text = "foo   bar";
        assert_eq!(
            find_word_bounds(text, 0, TextObjectScope::Around),
            Some((0, 6))
        );
    }

    #[test]
    fn jq_field_access() {
        let text = ".name.first";
        assert_eq!(
            find_word_bounds(text, 1, TextObjectScope::Inner),
            Some((1, 5))
        );
        assert_eq!(
            find_word_bounds(text, 6, TextObjectScope::Inner),
            Some((6, 11))
        );
    }
}

mod quote_bounds_tests {
    use super::*;

    #[test]
    fn inner_double_quotes() {
        let text = r#""hello""#;
        assert_eq!(
            find_quote_bounds(text, 1, '"', TextObjectScope::Inner),
            Some((1, 6))
        );
        assert_eq!(
            find_quote_bounds(text, 3, '"', TextObjectScope::Inner),
            Some((1, 6))
        );
    }

    #[test]
    fn around_double_quotes() {
        let text = r#""hello""#;
        assert_eq!(
            find_quote_bounds(text, 3, '"', TextObjectScope::Around),
            Some((0, 7))
        );
    }

    #[test]
    fn cursor_on_opening_quote() {
        let text = r#""hello""#;
        assert_eq!(
            find_quote_bounds(text, 0, '"', TextObjectScope::Inner),
            Some((1, 6))
        );
    }

    #[test]
    fn cursor_on_closing_quote() {
        let text = r#""hello""#;
        assert_eq!(
            find_quote_bounds(text, 6, '"', TextObjectScope::Inner),
            Some((1, 6))
        );
    }

    #[test]
    fn single_quotes() {
        let text = "'hello'";
        assert_eq!(
            find_quote_bounds(text, 3, '\'', TextObjectScope::Inner),
            Some((1, 6))
        );
    }

    #[test]
    fn cursor_on_closing_single_quote() {
        let text = "'hello'";
        assert_eq!(
            find_quote_bounds(text, 6, '\'', TextObjectScope::Inner),
            Some((1, 6))
        );
    }

    #[test]
    fn backticks() {
        let text = "`hello`";
        assert_eq!(
            find_quote_bounds(text, 3, '`', TextObjectScope::Inner),
            Some((1, 6))
        );
    }

    #[test]
    fn cursor_on_closing_backtick() {
        let text = "`hello`";
        assert_eq!(
            find_quote_bounds(text, 6, '`', TextObjectScope::Inner),
            Some((1, 6))
        );
    }

    #[test]
    fn empty_quotes() {
        let text = r#""""#;
        assert_eq!(
            find_quote_bounds(text, 0, '"', TextObjectScope::Inner),
            Some((1, 1))
        );
        assert_eq!(
            find_quote_bounds(text, 0, '"', TextObjectScope::Around),
            Some((0, 2))
        );
    }

    #[test]
    fn quotes_in_jq_query() {
        let text = r#".name | select(. == "foo")"#;
        assert_eq!(
            find_quote_bounds(text, 21, '"', TextObjectScope::Inner),
            Some((21, 24))
        );
    }

    #[test]
    fn no_matching_quote() {
        let text = r#""hello"#;
        assert_eq!(
            find_quote_bounds(text, 3, '"', TextObjectScope::Inner),
            None
        );
    }

    #[test]
    fn cursor_outside_quotes() {
        let text = r#"foo "bar" baz"#;
        assert_eq!(
            find_quote_bounds(text, 0, '"', TextObjectScope::Inner),
            None
        );
    }

    #[test]
    fn multiple_quoted_strings() {
        let text = r#""foo" "bar""#;
        assert_eq!(
            find_quote_bounds(text, 2, '"', TextObjectScope::Inner),
            Some((1, 4))
        );
        assert_eq!(
            find_quote_bounds(text, 8, '"', TextObjectScope::Inner),
            Some((7, 10))
        );
    }
}

mod bracket_bounds_tests {
    use super::*;

    #[test]
    fn inner_parentheses() {
        let text = "(hello)";
        assert_eq!(
            find_bracket_bounds(text, 3, '(', ')', TextObjectScope::Inner),
            Some((1, 6))
        );
    }

    #[test]
    fn around_parentheses() {
        let text = "(hello)";
        assert_eq!(
            find_bracket_bounds(text, 3, '(', ')', TextObjectScope::Around),
            Some((0, 7))
        );
    }

    #[test]
    fn nested_parentheses_inner() {
        let text = "(foo (bar) baz)";
        assert_eq!(
            find_bracket_bounds(text, 7, '(', ')', TextObjectScope::Inner),
            Some((6, 9))
        );
    }

    #[test]
    fn nested_parentheses_outer() {
        let text = "(foo (bar) baz)";
        assert_eq!(
            find_bracket_bounds(text, 2, '(', ')', TextObjectScope::Inner),
            Some((1, 14))
        );
    }

    #[test]
    fn brackets() {
        let text = "[1, 2, 3]";
        assert_eq!(
            find_bracket_bounds(text, 4, '[', ']', TextObjectScope::Inner),
            Some((1, 8))
        );
    }

    #[test]
    fn braces() {
        let text = "{foo: bar}";
        assert_eq!(
            find_bracket_bounds(text, 5, '{', '}', TextObjectScope::Inner),
            Some((1, 9))
        );
    }

    #[test]
    fn cursor_on_opening_bracket() {
        let text = "(hello)";
        assert_eq!(
            find_bracket_bounds(text, 0, '(', ')', TextObjectScope::Inner),
            Some((1, 6))
        );
    }

    #[test]
    fn cursor_on_closing_bracket() {
        let text = "(hello)";
        assert_eq!(
            find_bracket_bounds(text, 6, '(', ')', TextObjectScope::Inner),
            Some((1, 6))
        );
    }

    #[test]
    fn empty_brackets() {
        let text = "()";
        assert_eq!(
            find_bracket_bounds(text, 0, '(', ')', TextObjectScope::Inner),
            Some((1, 1))
        );
        assert_eq!(
            find_bracket_bounds(text, 0, '(', ')', TextObjectScope::Around),
            Some((0, 2))
        );
    }

    #[test]
    fn no_matching_bracket() {
        let text = "(hello";
        assert_eq!(
            find_bracket_bounds(text, 3, '(', ')', TextObjectScope::Inner),
            None
        );
    }

    #[test]
    fn cursor_outside_brackets() {
        let text = "foo (bar) baz";
        assert_eq!(
            find_bracket_bounds(text, 0, '(', ')', TextObjectScope::Inner),
            None
        );
    }

    #[test]
    fn jq_array_access() {
        let text = ".items[0].name";
        assert_eq!(
            find_bracket_bounds(text, 7, '[', ']', TextObjectScope::Inner),
            Some((7, 8))
        );
    }

    #[test]
    fn jq_filter_expression() {
        let text = "map(select(.x > 0))";
        assert_eq!(
            find_bracket_bounds(text, 10, '(', ')', TextObjectScope::Inner),
            Some((11, 17))
        );
        assert_eq!(
            find_bracket_bounds(text, 5, '(', ')', TextObjectScope::Inner),
            Some((4, 18))
        );
    }

    #[test]
    fn deeply_nested() {
        let text = "((((x))))";
        assert_eq!(
            find_bracket_bounds(text, 4, '(', ')', TextObjectScope::Inner),
            Some((4, 5))
        );
        assert_eq!(
            find_bracket_bounds(text, 3, '(', ')', TextObjectScope::Inner),
            Some((4, 5))
        );
    }
}

mod find_text_object_bounds_tests {
    use super::*;

    #[test]
    fn delegates_to_word() {
        let text = "hello world";
        assert_eq!(
            find_text_object_bounds(text, 2, TextObjectTarget::Word, TextObjectScope::Inner),
            find_word_bounds(text, 2, TextObjectScope::Inner)
        );
    }

    #[test]
    fn delegates_to_quotes() {
        let text = r#""hello""#;
        assert_eq!(
            find_text_object_bounds(
                text,
                3,
                TextObjectTarget::DoubleQuote,
                TextObjectScope::Inner
            ),
            find_quote_bounds(text, 3, '"', TextObjectScope::Inner)
        );
    }

    #[test]
    fn delegates_to_brackets() {
        let text = "(hello)";
        assert_eq!(
            find_text_object_bounds(
                text,
                3,
                TextObjectTarget::Parentheses,
                TextObjectScope::Inner
            ),
            find_bracket_bounds(text, 3, '(', ')', TextObjectScope::Inner)
        );
    }
}

mod execute_text_object_tests {
    use super::*;
    use tui_textarea::TextArea;

    fn textarea_with(content: &str) -> TextArea<'static> {
        let mut ta = TextArea::default();
        ta.insert_str(content);
        ta.move_cursor(tui_textarea::CursorMove::Head);
        ta
    }

    fn move_to(ta: &mut TextArea, col: usize) {
        ta.move_cursor(tui_textarea::CursorMove::Head);
        for _ in 0..col {
            ta.move_cursor(tui_textarea::CursorMove::Forward);
        }
    }

    fn content(ta: &TextArea) -> String {
        ta.lines().first().cloned().unwrap_or_default()
    }

    #[test]
    fn delete_inner_word() {
        let mut ta = textarea_with("hello world");
        move_to(&mut ta, 2);

        let result = execute_text_object(&mut ta, TextObjectTarget::Word, TextObjectScope::Inner);

        assert!(result);
        assert_eq!(content(&ta), " world");
    }

    #[test]
    fn delete_around_word() {
        let mut ta = textarea_with("hello world");
        move_to(&mut ta, 2);

        let result = execute_text_object(&mut ta, TextObjectTarget::Word, TextObjectScope::Around);

        assert!(result);
        assert_eq!(content(&ta), "world");
    }

    #[test]
    fn delete_inner_quotes() {
        let mut ta = textarea_with(r#""hello""#);
        move_to(&mut ta, 3);

        let result = execute_text_object(
            &mut ta,
            TextObjectTarget::DoubleQuote,
            TextObjectScope::Inner,
        );

        assert!(result);
        assert_eq!(content(&ta), r#""""#);
    }

    #[test]
    fn delete_around_quotes() {
        let mut ta = textarea_with(r#"foo "bar" baz"#);
        move_to(&mut ta, 6);

        let result = execute_text_object(
            &mut ta,
            TextObjectTarget::DoubleQuote,
            TextObjectScope::Around,
        );

        assert!(result);
        assert_eq!(content(&ta), "foo  baz");
    }

    #[test]
    fn delete_inner_parentheses() {
        // Cursor at position 11 is on `.` inside inner parens (.x)
        let mut ta = textarea_with("map(select(.x))");
        move_to(&mut ta, 11);

        let result = execute_text_object(
            &mut ta,
            TextObjectTarget::Parentheses,
            TextObjectScope::Inner,
        );

        assert!(result);
        assert_eq!(content(&ta), "map(select())");
    }

    #[test]
    fn delete_around_parentheses() {
        let mut ta = textarea_with("foo (bar) baz");
        move_to(&mut ta, 6);

        let result = execute_text_object(
            &mut ta,
            TextObjectTarget::Parentheses,
            TextObjectScope::Around,
        );

        assert!(result);
        assert_eq!(content(&ta), "foo  baz");
    }

    #[test]
    fn delete_inner_brackets() {
        let mut ta = textarea_with(".items[0]");
        move_to(&mut ta, 7);

        let result =
            execute_text_object(&mut ta, TextObjectTarget::Brackets, TextObjectScope::Inner);

        assert!(result);
        assert_eq!(content(&ta), ".items[]");
    }

    #[test]
    fn delete_inner_braces() {
        let mut ta = textarea_with("{foo: bar}");
        move_to(&mut ta, 5);

        let result = execute_text_object(&mut ta, TextObjectTarget::Braces, TextObjectScope::Inner);

        assert!(result);
        assert_eq!(content(&ta), "{}");
    }

    #[test]
    fn no_match_returns_false() {
        let mut ta = textarea_with("hello");
        move_to(&mut ta, 2);

        let result = execute_text_object(
            &mut ta,
            TextObjectTarget::DoubleQuote,
            TextObjectScope::Inner,
        );

        assert!(!result);
        assert_eq!(content(&ta), "hello");
    }

    #[test]
    fn cursor_on_non_word_returns_false() {
        let mut ta = textarea_with(".foo");
        move_to(&mut ta, 0);

        let result = execute_text_object(&mut ta, TextObjectTarget::Word, TextObjectScope::Inner);

        assert!(!result);
        assert_eq!(content(&ta), ".foo");
    }

    #[test]
    fn jq_query_change_field_name() {
        let mut ta = textarea_with(".name.first");
        move_to(&mut ta, 2);

        let result = execute_text_object(&mut ta, TextObjectTarget::Word, TextObjectScope::Inner);

        assert!(result);
        assert_eq!(content(&ta), "..first");
    }

    #[test]
    fn jq_query_change_quoted_string() {
        let mut ta = textarea_with(r#"select(.name == "foo")"#);
        move_to(&mut ta, 18);

        let result = execute_text_object(
            &mut ta,
            TextObjectTarget::DoubleQuote,
            TextObjectScope::Inner,
        );

        assert!(result);
        assert_eq!(content(&ta), r#"select(.name == "")"#);
    }
}
