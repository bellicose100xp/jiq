use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tui_textarea::CursorMove;

use crate::app::App;
use crate::clipboard;
use crate::editor::EditorMode;

pub fn handle_insert_mode_key(app: &mut App, key: KeyEvent) {
    let content_changed = app.input.textarea.input(key);

    if content_changed {
        app.history.reset_cycling();
        app.debouncer.schedule_execution();
        app.results_scroll.reset();
        app.error_overlay_visible = false;
        app.input
            .brace_tracker
            .rebuild(app.input.textarea.lines()[0].as_ref());
    }

    app.update_autocomplete();
    app.update_tooltip();
}

pub fn handle_normal_mode_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('?') => {
            app.help.visible = !app.help.visible;
        }

        KeyCode::Char('h') | KeyCode::Left => {
            app.input.textarea.move_cursor(CursorMove::Back);
        }
        KeyCode::Char('l') | KeyCode::Right => {
            app.input.textarea.move_cursor(CursorMove::Forward);
        }

        KeyCode::Char('0') | KeyCode::Char('^') | KeyCode::Home => {
            app.input.textarea.move_cursor(CursorMove::Head);
        }
        KeyCode::Char('$') | KeyCode::End => {
            app.input.textarea.move_cursor(CursorMove::End);
        }

        KeyCode::Char('w') => {
            app.input.textarea.move_cursor(CursorMove::WordForward);
        }
        KeyCode::Char('b') => {
            app.input.textarea.move_cursor(CursorMove::WordBack);
        }
        KeyCode::Char('e') => {
            app.input.textarea.move_cursor(CursorMove::WordEnd);
        }

        KeyCode::Char('i') => {
            app.input.editor_mode = EditorMode::Insert;
        }
        KeyCode::Char('a') => {
            app.input.textarea.move_cursor(CursorMove::Forward);
            app.input.editor_mode = EditorMode::Insert;
        }
        KeyCode::Char('I') => {
            app.input.textarea.move_cursor(CursorMove::Head);
            app.input.editor_mode = EditorMode::Insert;
        }
        KeyCode::Char('A') => {
            app.input.textarea.move_cursor(CursorMove::End);
            app.input.editor_mode = EditorMode::Insert;
        }

        KeyCode::Char('x') => {
            app.input.textarea.delete_next_char();
            execute_query(app);
        }
        KeyCode::Char('X') => {
            app.input.textarea.delete_char();
            execute_query(app);
        }

        KeyCode::Char('D') => {
            app.input.textarea.delete_line_by_end();
            execute_query(app);
        }
        KeyCode::Char('C') => {
            app.input.textarea.delete_line_by_end();
            app.input.textarea.cancel_selection();
            app.input.editor_mode = EditorMode::Insert;
            execute_query(app);
        }

        KeyCode::Char('d') => {
            app.input.editor_mode = EditorMode::Operator('d');
            app.input.textarea.start_selection();
        }
        KeyCode::Char('c') => {
            app.input.editor_mode = EditorMode::Operator('c');
            app.input.textarea.start_selection();
        }
        KeyCode::Char('y') => {
            app.input.editor_mode = EditorMode::Operator('y');
        }

        KeyCode::Char('u') => {
            app.input.textarea.undo();
            execute_query(app);
        }
        KeyCode::Char('r') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.input.textarea.redo();
            execute_query(app);
        }

        _ => {}
    }

    app.update_tooltip();
}

pub fn handle_operator_mode_key(app: &mut App, key: KeyEvent) {
    let operator = match app.input.editor_mode {
        EditorMode::Operator(op) => op,
        _ => return,
    };

    if key.code == KeyCode::Char(operator) {
        match operator {
            'y' => {
                clipboard::clipboard_events::handle_yank_key(app, app.clipboard_backend);
                app.input.editor_mode = EditorMode::Normal;
            }
            'd' | 'c' => {
                app.input.textarea.delete_line_by_head();
                app.input.textarea.delete_line_by_end();
                app.input.editor_mode = if operator == 'c' {
                    EditorMode::Insert
                } else {
                    EditorMode::Normal
                };
                execute_query(app);
            }
            _ => {
                app.input.editor_mode = EditorMode::Normal;
            }
        }
        return;
    }

    let motion_applied = match key.code {
        KeyCode::Char('w') => {
            app.input.textarea.move_cursor(CursorMove::WordForward);
            true
        }
        KeyCode::Char('b') => {
            app.input.textarea.move_cursor(CursorMove::WordBack);
            true
        }
        KeyCode::Char('e') => {
            app.input.textarea.move_cursor(CursorMove::WordEnd);
            app.input.textarea.move_cursor(CursorMove::Forward);
            true
        }

        KeyCode::Char('0') | KeyCode::Char('^') | KeyCode::Home => {
            app.input.textarea.move_cursor(CursorMove::Head);
            true
        }
        KeyCode::Char('$') | KeyCode::End => {
            app.input.textarea.move_cursor(CursorMove::End);
            true
        }

        KeyCode::Char('h') | KeyCode::Left => {
            app.input.textarea.move_cursor(CursorMove::Back);
            true
        }
        KeyCode::Char('l') | KeyCode::Right => {
            app.input.textarea.move_cursor(CursorMove::Forward);
            true
        }

        _ => false,
    };

    if motion_applied {
        match operator {
            'd' => {
                app.input.textarea.cut();
                app.input.editor_mode = EditorMode::Normal;
            }
            'c' => {
                app.input.textarea.cut();
                app.input.editor_mode = EditorMode::Insert;
            }
            _ => {
                app.input.textarea.cancel_selection();
                app.input.editor_mode = EditorMode::Normal;
            }
        }
        execute_query(app);
    } else {
        app.input.textarea.cancel_selection();
        app.input.editor_mode = EditorMode::Normal;
    }

    app.update_tooltip();
}

pub fn execute_query(app: &mut App) {
    execute_query_with_auto_show(app);
}

pub fn execute_query_with_auto_show(app: &mut App) {
    let query = app.input.textarea.lines()[0].as_ref();

    app.input.brace_tracker.rebuild(query);
    app.query.execute(query);
    app.results_scroll.reset();
    app.error_overlay_visible = false;

    let cursor_pos = app.input.textarea.cursor().1;
    crate::ai::ai_events::handle_query_result(
        &mut app.ai,
        &app.query.result,
        query,
        cursor_pos,
        app.query.executor.json_input(),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::Focus;
    use crate::autocomplete::{Suggestion, SuggestionType};
    use crate::test_utils::test_helpers::{app_with_query, key, key_with_mods};
    use tui_textarea::CursorMove;

    fn move_cursor_to_position(app: &mut App, target_pos: usize) {
        app.input.textarea.move_cursor(CursorMove::Head);
        for _ in 0..target_pos {
            app.input.textarea.move_cursor(CursorMove::Forward);
        }
    }

    #[test]
    fn test_operator_dw_deletes_word_from_start() {
        let mut app = app_with_query(".name.first");
        app.input.textarea.move_cursor(CursorMove::Head);
        app.input.editor_mode = EditorMode::Normal;

        // Type 'd' to enter Operator mode
        app.handle_key_event(key(KeyCode::Char('d')));
        assert!(matches!(app.input.editor_mode, EditorMode::Operator('d')));

        // Type 'w' to delete word
        app.handle_key_event(key(KeyCode::Char('w')));
        // The selection behavior deletes from cursor to end of word motion
        assert!(app.query().contains("first"));
        assert_eq!(app.input.editor_mode, EditorMode::Normal);
    }

    #[test]
    fn test_operator_dw_deletes_word_from_middle() {
        let mut app = app_with_query(".name.first");
        // Move to position 5 (at the dot before "first")
        move_cursor_to_position(&mut app, 5);
        app.input.editor_mode = EditorMode::Normal;

        app.handle_key_event(key(KeyCode::Char('d')));
        app.handle_key_event(key(KeyCode::Char('w')));
        // Verify something was deleted
        assert!(app.query().len() < ".name.first".len());
        assert!(app.query().starts_with(".name"));
    }

    #[test]
    fn test_operator_db_deletes_word_backward() {
        let mut app = app_with_query(".name.first");
        app.input.textarea.move_cursor(CursorMove::End);
        app.input.editor_mode = EditorMode::Normal;

        app.handle_key_event(key(KeyCode::Char('d')));
        app.handle_key_event(key(KeyCode::Char('b')));

        // Should delete ".first" backwards
        assert!(app.query().starts_with(".name"));
    }

    #[test]
    fn test_operator_de_deletes_to_word_end() {
        let mut app = app_with_query(".name.first");
        app.input.textarea.move_cursor(CursorMove::Head);
        app.input.editor_mode = EditorMode::Normal;

        app.handle_key_event(key(KeyCode::Char('d')));
        app.handle_key_event(key(KeyCode::Char('e')));

        // Should delete to end of first word (including the character at cursor)
        assert!(app.query().contains("first"));
    }

    #[test]
    fn test_operator_d_dollar_deletes_to_end_of_line() {
        let mut app = app_with_query(".name.first");
        move_cursor_to_position(&mut app, 5);
        app.input.editor_mode = EditorMode::Normal;

        app.handle_key_event(key(KeyCode::Char('d')));
        app.handle_key_event(key(KeyCode::Char('$')));

        assert_eq!(app.query(), ".name");
    }

    #[test]
    fn test_operator_d0_deletes_to_start_of_line() {
        let mut app = app_with_query(".name.first");
        move_cursor_to_position(&mut app, 6);
        app.input.editor_mode = EditorMode::Normal;

        app.handle_key_event(key(KeyCode::Char('d')));
        app.handle_key_event(key(KeyCode::Char('0')));

        assert!(app.query().ends_with("first"));
    }

    #[test]
    fn test_operator_d_caret_deletes_to_start_of_line() {
        let mut app = app_with_query(".name.first");
        move_cursor_to_position(&mut app, 6);
        app.input.editor_mode = EditorMode::Normal;

        app.handle_key_event(key(KeyCode::Char('d')));
        app.handle_key_event(key(KeyCode::Char('^')));

        assert!(app.query().ends_with("first"));
    }

    #[test]
    fn test_operator_dd_deletes_entire_line() {
        let mut app = app_with_query(".name.first");
        app.input.editor_mode = EditorMode::Normal;

        app.handle_key_event(key(KeyCode::Char('d')));
        app.handle_key_event(key(KeyCode::Char('d')));

        assert_eq!(app.query(), "");
        assert_eq!(app.input.editor_mode, EditorMode::Normal);
    }

    #[test]
    fn test_operator_cw_changes_word() {
        let mut app = app_with_query(".name.first");
        app.input.textarea.move_cursor(CursorMove::Head);
        app.input.editor_mode = EditorMode::Normal;

        app.handle_key_event(key(KeyCode::Char('c')));
        app.handle_key_event(key(KeyCode::Char('w')));

        assert!(app.query().contains("first"));
        assert_eq!(app.input.editor_mode, EditorMode::Insert);
    }

    #[test]
    fn test_operator_cc_changes_entire_line() {
        let mut app = app_with_query(".name.first");
        app.input.editor_mode = EditorMode::Normal;

        app.handle_key_event(key(KeyCode::Char('c')));
        app.handle_key_event(key(KeyCode::Char('c')));

        assert_eq!(app.query(), "");
        assert_eq!(app.input.editor_mode, EditorMode::Insert);
    }

    #[test]
    fn test_operator_invalid_motion_cancels() {
        let mut app = app_with_query(".name");
        app.input.editor_mode = EditorMode::Normal;
        let original_query = app.query().to_string();

        app.handle_key_event(key(KeyCode::Char('d')));
        assert!(matches!(app.input.editor_mode, EditorMode::Operator('d')));

        app.handle_key_event(key(KeyCode::Char('z')));

        assert_eq!(app.input.editor_mode, EditorMode::Normal);
        assert_eq!(app.query(), original_query);
    }

    #[test]
    fn test_escape_in_operator_mode_cancels_operator() {
        let mut app = app_with_query(".name");
        app.input.editor_mode = EditorMode::Normal;
        let original_query = app.query().to_string();

        app.handle_key_event(key(KeyCode::Char('d')));
        assert!(matches!(app.input.editor_mode, EditorMode::Operator('d')));

        app.handle_key_event(key(KeyCode::Esc));

        assert_eq!(app.input.editor_mode, EditorMode::Normal);
        assert_eq!(app.query(), original_query);
    }

    #[test]
    fn test_operator_dh_deletes_character_backward() {
        let mut app = app_with_query(".name");
        app.input.textarea.move_cursor(CursorMove::End);
        app.input.editor_mode = EditorMode::Normal;

        app.handle_key_event(key(KeyCode::Char('d')));
        app.handle_key_event(key(KeyCode::Char('h')));

        assert!(app.query().len() < 5);
        assert_eq!(app.input.editor_mode, EditorMode::Normal);
    }

    #[test]
    fn test_operator_dl_deletes_character_forward() {
        let mut app = app_with_query(".name");
        app.input.textarea.move_cursor(CursorMove::Head);
        app.input.editor_mode = EditorMode::Normal;

        app.handle_key_event(key(KeyCode::Char('d')));
        app.handle_key_event(key(KeyCode::Char('l')));

        assert!(app.query().len() < 5);
        assert_eq!(app.input.editor_mode, EditorMode::Normal);
    }

    #[test]
    fn test_escape_from_insert_to_normal() {
        let mut app = app_with_query(".name");
        app.input.editor_mode = EditorMode::Insert;

        app.handle_key_event(key(KeyCode::Esc));

        assert_eq!(app.input.editor_mode, EditorMode::Normal);
    }

    #[test]
    fn test_i_enters_insert_mode_at_cursor() {
        let mut app = app_with_query(".name");
        app.input.editor_mode = EditorMode::Normal;
        app.input.textarea.move_cursor(CursorMove::Head);
        let cursor_before = app.input.textarea.cursor();

        app.handle_key_event(key(KeyCode::Char('i')));

        assert_eq!(app.input.editor_mode, EditorMode::Insert);
        assert_eq!(app.input.textarea.cursor(), cursor_before);
    }

    #[test]
    fn test_a_enters_insert_mode_after_cursor() {
        let mut app = app_with_query(".name");
        app.input.editor_mode = EditorMode::Normal;
        app.input.textarea.move_cursor(CursorMove::Head);
        let cursor_col_before = app.input.textarea.cursor().1;

        app.handle_key_event(key(KeyCode::Char('a')));

        assert_eq!(app.input.editor_mode, EditorMode::Insert);
        assert_eq!(app.input.textarea.cursor().1, cursor_col_before + 1);
    }

    #[test]
    fn test_capital_i_enters_insert_at_line_start() {
        let mut app = app_with_query(".name");
        app.input.editor_mode = EditorMode::Normal;
        app.input.textarea.move_cursor(CursorMove::End);

        app.handle_key_event(key(KeyCode::Char('I')));

        assert_eq!(app.input.editor_mode, EditorMode::Insert);
        assert_eq!(app.input.textarea.cursor().1, 0);
    }

    #[test]
    fn test_capital_a_enters_insert_at_line_end() {
        let mut app = app_with_query(".name");
        app.input.editor_mode = EditorMode::Normal;
        app.input.textarea.move_cursor(CursorMove::Head);

        app.handle_key_event(key(KeyCode::Char('A')));

        assert_eq!(app.input.editor_mode, EditorMode::Insert);
        assert_eq!(app.input.textarea.cursor().1, 5);
    }

    #[test]
    fn test_d_enters_operator_mode() {
        let mut app = app_with_query(".name");
        app.input.editor_mode = EditorMode::Normal;

        app.handle_key_event(key(KeyCode::Char('d')));

        assert!(matches!(app.input.editor_mode, EditorMode::Operator('d')));
    }

    #[test]
    fn test_c_enters_operator_mode() {
        let mut app = app_with_query(".name");
        app.input.editor_mode = EditorMode::Normal;

        app.handle_key_event(key(KeyCode::Char('c')));

        assert!(matches!(app.input.editor_mode, EditorMode::Operator('c')));
    }

    #[test]
    fn test_x_deletes_character_at_cursor() {
        let mut app = app_with_query(".name");
        app.input.textarea.move_cursor(CursorMove::Head);
        app.input.editor_mode = EditorMode::Normal;

        app.handle_key_event(key(KeyCode::Char('x')));

        assert_eq!(app.query(), "name");
    }

    #[test]
    fn test_capital_x_deletes_character_before_cursor() {
        let mut app = app_with_query(".name");
        app.input.textarea.move_cursor(CursorMove::Head);
        app.input.textarea.move_cursor(CursorMove::Forward); // Move to 'n'
        app.input.editor_mode = EditorMode::Normal;

        app.handle_key_event(key(KeyCode::Char('X')));

        assert_eq!(app.query(), "name");
    }

    #[test]
    fn test_capital_d_deletes_to_end_of_line() {
        let mut app = app_with_query(".name.first");
        move_cursor_to_position(&mut app, 5);
        app.input.editor_mode = EditorMode::Normal;

        app.handle_key_event(key(KeyCode::Char('D')));

        assert_eq!(app.query(), ".name");
    }

    #[test]
    fn test_capital_c_changes_to_end_of_line() {
        let mut app = app_with_query(".name.first");
        move_cursor_to_position(&mut app, 5);
        app.input.editor_mode = EditorMode::Normal;

        app.handle_key_event(key(KeyCode::Char('C')));

        assert_eq!(app.query(), ".name");
        assert_eq!(app.input.editor_mode, EditorMode::Insert);
    }

    #[test]
    fn test_u_triggers_undo() {
        let mut app = app_with_query("");
        app.input.editor_mode = EditorMode::Insert;
        app.input.textarea.insert_str(".name");

        app.input.editor_mode = EditorMode::Normal;
        app.handle_key_event(key(KeyCode::Char('u')));

        // After undo, query should be empty
        assert_eq!(app.query(), "");
    }

    #[test]
    fn test_ctrl_r_triggers_redo() {
        let mut app = app_with_query("");
        app.input.editor_mode = EditorMode::Insert;
        app.input.textarea.insert_str(".name");

        app.input.editor_mode = EditorMode::Normal;
        app.input.textarea.undo();
        assert_eq!(app.query(), "");

        app.handle_key_event(key_with_mods(KeyCode::Char('r'), KeyModifiers::CONTROL));

        assert_eq!(app.query(), ".name");
    }

    #[test]
    fn test_h_moves_cursor_left() {
        let mut app = app_with_query(".name");
        app.input.textarea.move_cursor(CursorMove::End);
        app.input.editor_mode = EditorMode::Normal;
        let cursor_before = app.input.textarea.cursor().1;

        app.handle_key_event(key(KeyCode::Char('h')));

        assert_eq!(app.input.textarea.cursor().1, cursor_before - 1);
    }

    #[test]
    fn test_l_moves_cursor_right() {
        let mut app = app_with_query(".name");
        app.input.textarea.move_cursor(CursorMove::Head);
        app.input.editor_mode = EditorMode::Normal;

        app.handle_key_event(key(KeyCode::Char('l')));

        assert_eq!(app.input.textarea.cursor().1, 1);
    }

    #[test]
    fn test_0_moves_to_line_start() {
        let mut app = app_with_query(".name");
        app.input.textarea.move_cursor(CursorMove::End);
        app.input.editor_mode = EditorMode::Normal;

        app.handle_key_event(key(KeyCode::Char('0')));

        assert_eq!(app.input.textarea.cursor().1, 0);
    }

    #[test]
    fn test_caret_moves_to_line_start() {
        let mut app = app_with_query(".name");
        app.input.textarea.move_cursor(CursorMove::End);
        app.input.editor_mode = EditorMode::Normal;

        app.handle_key_event(key(KeyCode::Char('^')));

        assert_eq!(app.input.textarea.cursor().1, 0);
    }

    #[test]
    fn test_dollar_moves_to_line_end() {
        let mut app = app_with_query(".name");
        app.input.textarea.move_cursor(CursorMove::Head);
        app.input.editor_mode = EditorMode::Normal;

        app.handle_key_event(key(KeyCode::Char('$')));

        assert_eq!(app.input.textarea.cursor().1, 5);
    }

    #[test]
    fn test_w_moves_word_forward() {
        let mut app = app_with_query(".name.first");
        app.input.textarea.move_cursor(CursorMove::Head);
        app.input.editor_mode = EditorMode::Normal;
        let cursor_before = app.input.textarea.cursor().1;

        app.handle_key_event(key(KeyCode::Char('w')));

        assert!(app.input.textarea.cursor().1 > cursor_before);
    }

    #[test]
    fn test_b_moves_word_backward() {
        let mut app = app_with_query(".name.first");
        app.input.textarea.move_cursor(CursorMove::End);
        app.input.editor_mode = EditorMode::Normal;
        let cursor_before = app.input.textarea.cursor().1;

        app.handle_key_event(key(KeyCode::Char('b')));

        assert!(app.input.textarea.cursor().1 < cursor_before);
    }

    #[test]
    fn test_e_moves_to_word_end() {
        let mut app = app_with_query(".name.first");
        app.input.textarea.move_cursor(CursorMove::Head);
        app.input.editor_mode = EditorMode::Normal;
        let cursor_before = app.input.textarea.cursor().1;

        app.handle_key_event(key(KeyCode::Char('e')));

        assert!(app.input.textarea.cursor().1 > cursor_before);
    }

    #[test]
    fn test_escape_closes_autocomplete() {
        let mut app = app_with_query(".na");
        app.input.editor_mode = EditorMode::Insert;

        let suggestions = vec![Suggestion::new(".name", SuggestionType::Field)];
        app.autocomplete.update_suggestions(suggestions);
        assert!(app.autocomplete.is_visible());

        app.handle_key_event(key(KeyCode::Esc));

        assert!(!app.autocomplete.is_visible());
        assert_eq!(app.query(), ".na");
        assert_eq!(app.input.editor_mode, EditorMode::Normal);
    }

    #[test]
    fn test_escape_without_autocomplete_switches_to_normal() {
        let mut app = app_with_query(".name");
        app.input.editor_mode = EditorMode::Insert;
        assert!(!app.autocomplete.is_visible());

        app.handle_key_event(key(KeyCode::Esc));

        assert_eq!(app.input.editor_mode, EditorMode::Normal);
    }

    #[test]
    fn test_down_arrow_selects_next_suggestion() {
        let mut app = app_with_query(".na");
        app.input.editor_mode = EditorMode::Insert;

        let suggestions = vec![
            Suggestion::new(".name", SuggestionType::Field),
            Suggestion::new(".nested", SuggestionType::Field),
        ];
        app.autocomplete.update_suggestions(suggestions);

        app.handle_key_event(key(KeyCode::Down));

        assert_eq!(app.autocomplete.selected().unwrap().text, ".nested");
    }

    #[test]
    fn test_up_arrow_selects_previous_suggestion() {
        let mut app = app_with_query(".na");
        app.input.editor_mode = EditorMode::Insert;

        let suggestions = vec![
            Suggestion::new(".name", SuggestionType::Field),
            Suggestion::new(".nested", SuggestionType::Field),
        ];
        app.autocomplete.update_suggestions(suggestions);

        app.autocomplete.select_next();

        app.handle_key_event(key(KeyCode::Up));

        assert_eq!(app.autocomplete.selected().unwrap().text, ".name");
    }

    #[test]
    fn test_tab_accepts_autocomplete_suggestion() {
        let mut app = app_with_query(".na");
        app.input.editor_mode = EditorMode::Insert;
        app.focus = Focus::InputField;

        use crate::query::ResultType;
        assert_eq!(
            app.query.base_query_for_suggestions,
            Some(".".to_string()),
            "base_query should remain '.' since .na returns null"
        );
        assert_eq!(
            app.query.base_type_for_suggestions,
            Some(ResultType::Object),
            "base_type should be Object (root object)"
        );

        let suggestions = vec![Suggestion::new("name", SuggestionType::Field)];
        app.autocomplete.update_suggestions(suggestions);

        app.handle_key_event(key(KeyCode::Tab));

        assert_eq!(app.query(), ".name");
        assert!(!app.autocomplete.is_visible());
    }

    #[test]
    fn test_tab_without_autocomplete_stays_in_consistent_state() {
        let mut app = app_with_query("x");
        app.input.editor_mode = EditorMode::Insert;
        app.focus = Focus::InputField;

        app.autocomplete.hide();
        assert!(!app.autocomplete.is_visible());

        app.handle_key_event(key(KeyCode::Tab));

        assert_eq!(app.input.editor_mode, EditorMode::Insert);
        assert_eq!(app.focus, Focus::InputField);
    }

    #[test]
    fn test_autocomplete_navigation_only_works_in_insert_mode() {
        let mut app = app_with_query(".na");
        app.input.editor_mode = EditorMode::Normal;
        app.focus = Focus::InputField;

        let suggestions = vec![Suggestion::new(".name", SuggestionType::Field)];
        app.autocomplete.update_suggestions(suggestions);

        let selected_before = app.autocomplete.selected().unwrap().text.clone();
        app.handle_key_event(key(KeyCode::Down));
        let selected_after = app.autocomplete.selected().unwrap().text.clone();

        assert_eq!(selected_before, selected_after);
    }
}
