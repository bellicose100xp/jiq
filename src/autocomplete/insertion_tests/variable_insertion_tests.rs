use super::*;

fn variable_suggestion(text: &str) -> Suggestion {
    Suggestion::new(text, SuggestionType::Variable)
}

mod basic_insertion {
    use super::*;

    #[test]
    fn inserts_variable_at_end() {
        let (mut textarea, mut query_state) = setup_insertion_test(". as $x | $");
        let suggestion = variable_suggestion("$x");

        insert_suggestion(&mut textarea, &mut query_state, &suggestion);

        assert_eq!(textarea.lines()[0], ". as $x | $x");
    }

    #[test]
    fn inserts_variable_replacing_partial() {
        let (mut textarea, mut query_state) = setup_insertion_test(". as $item | $it");
        let suggestion = variable_suggestion("$item");

        insert_suggestion(&mut textarea, &mut query_state, &suggestion);

        assert_eq!(textarea.lines()[0], ". as $item | $item");
    }

    #[test]
    fn inserts_env_variable() {
        let (mut textarea, mut query_state) = setup_insertion_test("$E");
        let suggestion = variable_suggestion("$ENV");

        insert_suggestion(&mut textarea, &mut query_state, &suggestion);

        assert_eq!(textarea.lines()[0], "$ENV");
    }

    #[test]
    fn inserts_loc_variable() {
        let (mut textarea, mut query_state) = setup_insertion_test("$__");
        let suggestion = variable_suggestion("$__loc__");

        insert_suggestion(&mut textarea, &mut query_state, &suggestion);

        assert_eq!(textarea.lines()[0], "$__loc__");
    }
}

mod mid_query_insertion {
    use super::*;

    #[test]
    fn inserts_variable_in_middle() {
        let (mut textarea, mut query_state) = setup_insertion_test(". as $x | $ | .foo");
        move_cursor_to_column(&mut textarea, 11);
        let suggestion = variable_suggestion("$x");

        insert_suggestion(&mut textarea, &mut query_state, &suggestion);

        assert_eq!(textarea.lines()[0], ". as $x | $x | .foo");
    }

    #[test]
    fn inserts_variable_replacing_partial_in_middle() {
        let (mut textarea, mut query_state) = setup_insertion_test(". as $data | $da + .value");
        move_cursor_to_column(&mut textarea, ". as $data | $da".len());
        let suggestion = variable_suggestion("$data");

        insert_suggestion(&mut textarea, &mut query_state, &suggestion);

        assert_eq!(textarea.lines()[0], ". as $data | $data + .value");
    }
}

mod cursor_positioning {
    use super::*;

    #[test]
    fn cursor_at_end_after_insertion() {
        let (mut textarea, mut query_state) = setup_insertion_test("$");
        let suggestion = variable_suggestion("$ENV");

        insert_suggestion(&mut textarea, &mut query_state, &suggestion);

        assert_eq!(textarea.cursor().1, 4);
    }

    #[test]
    fn cursor_positioned_correctly_mid_query() {
        let (mut textarea, mut query_state) = setup_insertion_test(". as $x | $ | .bar");
        move_cursor_to_column(&mut textarea, 11);
        let suggestion = variable_suggestion("$x");

        insert_suggestion(&mut textarea, &mut query_state, &suggestion);

        assert_eq!(textarea.cursor().1, 12);
    }
}

mod complex_scenarios {
    use super::*;

    #[test]
    fn inserts_variable_inside_map() {
        let (mut textarea, mut query_state) = setup_insertion_test(".data as $d | map(. + $)");
        move_cursor_to_column(&mut textarea, 23);
        let suggestion = variable_suggestion("$d");

        insert_suggestion(&mut textarea, &mut query_state, &suggestion);

        assert_eq!(textarea.lines()[0], ".data as $d | map(. + $d)");
    }

    #[test]
    fn inserts_variable_in_reduce() {
        let (mut textarea, mut query_state) =
            setup_insertion_test("reduce .[] as $item (0; . + $)");
        move_cursor_to_column(&mut textarea, 29);
        let suggestion = variable_suggestion("$item");

        insert_suggestion(&mut textarea, &mut query_state, &suggestion);

        assert_eq!(textarea.lines()[0], "reduce .[] as $item (0; . + $item)");
    }

    #[test]
    fn inserts_variable_with_underscore() {
        let (mut textarea, mut query_state) = setup_insertion_test(". as $my_var | $my");
        let suggestion = variable_suggestion("$my_var");

        insert_suggestion(&mut textarea, &mut query_state, &suggestion);

        assert_eq!(textarea.lines()[0], ". as $my_var | $my_var");
    }
}
