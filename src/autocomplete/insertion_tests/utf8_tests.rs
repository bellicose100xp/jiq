//! UTF-8 crash reproduction and regression tests for issue #153.
//!
//! Before the ingress normalization fix, typing non-ASCII characters in the
//! query caused panics at the boundary between tui_textarea (char indices)
//! and Rust string slicing (byte offsets). These tests lock in the fix.

use crate::autocomplete::autocomplete_state::update_suggestions_from_app;
use crate::test_utils::test_helpers::test_app;

mod update_suggestions_does_not_panic {
    use super::*;

    fn run_with_query(json: &str, query: &str) {
        let mut app = test_app(json);
        app.input.textarea.insert_str(query);
        update_suggestions_from_app(&mut app);
    }

    #[test]
    fn two_byte_accented_field() {
        run_with_query(r#"{"café": 1}"#, ".café");
    }

    #[test]
    fn three_byte_cjk_field() {
        run_with_query(r#"{"名前": "Alice"}"#, ".名前");
    }

    #[test]
    fn four_byte_emoji_field() {
        run_with_query(r#"{"👋": "hi"}"#, ".👋");
    }

    #[test]
    fn cjk_with_pipe_chain() {
        run_with_query(r#"{"中文": {"inner": 1}}"#, ".中文 | .inner");
    }

    #[test]
    fn emoji_in_object_construction() {
        run_with_query(r#"{"data": "x"}"#, "{🎉: .data}");
    }

    #[test]
    fn mixed_scripts_query() {
        run_with_query(r#"{"café": 1, "名前": "A", "👋": "hi"}"#, ".café | .名前");
    }

    #[test]
    fn cursor_mid_query_with_multibyte() {
        let mut app = test_app(r#"{"名前": "Alice", "name": "Bob"}"#);
        app.input.textarea.insert_str(".名前 | .name");
        use tui_textarea::CursorMove;
        for _ in 0..5 {
            app.input.textarea.move_cursor(CursorMove::Back);
        }
        update_suggestions_from_app(&mut app);
    }

    #[test]
    fn cursor_immediately_after_emoji() {
        let mut app = test_app(r#"{"x": 1}"#);
        app.input.textarea.insert_str("👋");
        update_suggestions_from_app(&mut app);
    }

    #[test]
    fn empty_multibyte_variations() {
        for q in ["é", "中", "👋", "a中", "a👋b", "中👋café"] {
            let mut app = test_app(r#"{"x": 1}"#);
            app.input.textarea.insert_str(q);
            update_suggestions_from_app(&mut app);
        }
    }
}

mod insert_suggestion_does_not_panic {
    use super::*;
    use crate::autocomplete::autocomplete_state::{Suggestion, SuggestionType};
    use crate::autocomplete::insertion::insert_suggestion_from_app;

    fn suggestion(text: &str) -> Suggestion {
        Suggestion::new(text, SuggestionType::Field)
    }

    #[test]
    fn insert_after_cjk_query() {
        let mut app = test_app(r#"{"名前": {"inner": 1}}"#);
        app.input.textarea.insert_str(".名前.");
        insert_suggestion_from_app(&mut app, &suggestion(".inner"));
    }

    #[test]
    fn insert_after_emoji_query() {
        let mut app = test_app(r#"{"👋": {"greeting": "hi"}}"#);
        app.input.textarea.insert_str(".👋.");
        insert_suggestion_from_app(&mut app, &suggestion(".greeting"));
    }

    #[test]
    fn insert_after_accented_query() {
        let mut app = test_app(r#"{"café": {"size": "large"}}"#);
        app.input.textarea.insert_str(".café.");
        insert_suggestion_from_app(&mut app, &suggestion(".size"));
    }

    #[test]
    fn insert_replaces_multibyte_trailing_separator() {
        let mut app = test_app(r#"{"名前": {"sub": 1}}"#);
        app.input.textarea.insert_str(".名前.");
        insert_suggestion_from_app(&mut app, &suggestion(".sub"));
        let result = app.input.query().to_string();
        assert_eq!(result, ".名前.sub");
    }

    #[test]
    fn insert_preserves_text_after_multibyte() {
        let mut app = test_app(r#"{"中文": 1, "name": "x"}"#);
        app.input.textarea.insert_str(".中文 | .");
        insert_suggestion_from_app(&mut app, &suggestion(".name"));
        let result = app.input.query().to_string();
        assert_eq!(result, ".中文 | .name");
    }
}

/// Regression tests for issue #153 — minimal reproducers that must never
/// panic again. Each is a specific failing scenario from the bug report.
mod regression_issue_153 {
    use super::*;
    use crate::autocomplete::autocomplete_state::{Suggestion, SuggestionType};
    use crate::autocomplete::insertion::insert_suggestion_from_app;

    fn suggestion(text: &str) -> Suggestion {
        Suggestion::new(text, SuggestionType::Field)
    }

    #[test]
    fn typing_cjk_character_in_query() {
        let mut app = test_app(r#"{"x": 1}"#);
        app.input.textarea.insert_str("中");
        update_suggestions_from_app(&mut app);
    }

    #[test]
    fn typing_emoji_in_query() {
        let mut app = test_app(r#"{"x": 1}"#);
        app.input.textarea.insert_str("👋");
        update_suggestions_from_app(&mut app);
    }

    #[test]
    fn typing_accented_character_in_query() {
        let mut app = test_app(r#"{"x": 1}"#);
        app.input.textarea.insert_str("é");
        update_suggestions_from_app(&mut app);
    }

    #[test]
    fn selecting_cjk_field_from_autocomplete() {
        let mut app = test_app(r#"{"名前": "Alice"}"#);
        app.input.textarea.insert_str(".");
        insert_suggestion_from_app(&mut app, &suggestion(".名前"));
    }

    #[test]
    fn selecting_emoji_field_from_autocomplete() {
        let mut app = test_app(r#"{"👋": "hi"}"#);
        app.input.textarea.insert_str(".");
        insert_suggestion_from_app(&mut app, &suggestion(".👋"));
    }
}

/// ASCII parity tests — ensure that after the byte/char refactor, ASCII
/// queries behave identically to pre-change behavior. These guard against
/// silent regressions in the most common code paths.
mod ascii_parity {
    use super::*;
    use crate::autocomplete::autocomplete_state::{Suggestion, SuggestionType};
    use crate::autocomplete::insertion::insert_suggestion_from_app;

    fn suggestion(text: &str) -> Suggestion {
        Suggestion::new(text, SuggestionType::Field)
    }

    #[test]
    fn simple_field_insertion() {
        let mut app = test_app(r#"{"name": "Alice", "age": 30}"#);
        app.input.textarea.insert_str(".");
        insert_suggestion_from_app(&mut app, &suggestion(".name"));
        assert_eq!(app.input.query(), ".name");
    }

    #[test]
    fn pipe_chain_insertion() {
        let mut app = test_app(r#"{"a": {"b": 1}}"#);
        app.input.textarea.insert_str(".a | .");
        insert_suggestion_from_app(&mut app, &suggestion(".b"));
        assert_eq!(app.input.query(), ".a | .b");
    }

    #[test]
    fn trailing_dot_replacement() {
        let mut app = test_app(r#"{"items": [1]}"#);
        app.input.textarea.insert_str(".items.");
        insert_suggestion_from_app(&mut app, &suggestion(".length"));
        assert_eq!(app.input.query(), ".items.length");
    }

    #[test]
    fn cursor_at_start_of_query() {
        let mut app = test_app(r#"{"x": 1}"#);
        app.input.textarea.insert_str("abc");
        use tui_textarea::CursorMove;
        app.input.textarea.move_cursor(CursorMove::Head);
        update_suggestions_from_app(&mut app);
    }

    #[test]
    fn cursor_at_end_of_query() {
        let mut app = test_app(r#"{"x": 1}"#);
        app.input.textarea.insert_str(".x");
        update_suggestions_from_app(&mut app);
    }

    #[test]
    fn function_call_context() {
        let mut app = test_app(r#"[{"x": 1}]"#);
        app.input.textarea.insert_str("map(.");
        update_suggestions_from_app(&mut app);
    }
}

/// End-to-end bracket-notation tests — verify that selecting a non-ASCII
/// field suggestion produces valid jq syntax (`.["名前"]`, not `.名前`).
/// These are the tests that would have caught the "autocomplete emits
/// invalid jq for CJK keys" bug reported after the crash fix.
mod bracket_notation_emission {
    use super::*;
    use crate::autocomplete::autocomplete_state::{Suggestion, SuggestionType};
    use crate::autocomplete::insertion::insert_suggestion_from_app;

    fn suggestion(text: &str) -> Suggestion {
        Suggestion::new(text, SuggestionType::Field)
    }

    #[test]
    fn cjk_field_inserts_bracket_notation() {
        let mut app = test_app(r#"{"名前": "Alice"}"#);
        app.input.textarea.insert_str(".");
        insert_suggestion_from_app(&mut app, &suggestion(r#".["名前"]"#));
        assert_eq!(app.input.query(), r#".["名前"]"#);
    }

    #[test]
    fn emoji_field_inserts_bracket_notation() {
        let mut app = test_app(r#"{"👋": "hi"}"#);
        app.input.textarea.insert_str(".");
        insert_suggestion_from_app(&mut app, &suggestion(r#".["👋"]"#));
        assert_eq!(app.input.query(), r#".["👋"]"#);
    }

    #[test]
    fn accented_field_inserts_bracket_notation() {
        let mut app = test_app(r#"{"café": 1}"#);
        app.input.textarea.insert_str(".");
        insert_suggestion_from_app(&mut app, &suggestion(r#".["café"]"#));
        assert_eq!(app.input.query(), r#".["café"]"#);
    }

    #[test]
    fn partial_cjk_match_replaces_correctly() {
        let mut app = test_app(r#"{"名前": "Alice"}"#);
        app.input.textarea.insert_str(".名");
        insert_suggestion_from_app(&mut app, &suggestion(r#".["名前"]"#));
        assert_eq!(app.input.query(), r#".["名前"]"#);
    }

    #[test]
    fn partial_accented_match_replaces_correctly() {
        let mut app = test_app(r#"{"café": 1}"#);
        app.input.textarea.insert_str(".ca");
        insert_suggestion_from_app(&mut app, &suggestion(r#".["café"]"#));
        assert_eq!(app.input.query(), r#".["café"]"#);
    }

    #[test]
    fn array_iteration_with_cjk_key() {
        let mut app = test_app(r#"[{"名前": "A"}]"#);
        app.input.textarea.insert_str(".");
        insert_suggestion_from_app(&mut app, &suggestion(r#".[]["名前"]"#));
        assert_eq!(app.input.query(), r#".[]["名前"]"#);
    }

    #[test]
    fn ascii_hyphenated_key_inserts_bracket_notation() {
        let mut app = test_app(r#"{"my-field": 1}"#);
        app.input.textarea.insert_str(".");
        insert_suggestion_from_app(&mut app, &suggestion(r#".["my-field"]"#));
        assert_eq!(app.input.query(), r#".["my-field"]"#);
    }

    #[test]
    fn ascii_identifier_still_inserts_dot_notation() {
        let mut app = test_app(r#"{"name": "Alice"}"#);
        app.input.textarea.insert_str(".");
        insert_suggestion_from_app(&mut app, &suggestion(".name"));
        assert_eq!(app.input.query(), ".name");
    }
}
