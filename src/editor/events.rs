//! VIM-style editing mode handlers
//!
//! This module implements VIM-style text editing including Insert mode, Normal mode,
//! and Operator-pending mode with motions.

use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tui_textarea::CursorMove;

use crate::clipboard;
use crate::editor::EditorMode;
use crate::app::App;

/// Handle keys in Insert mode
pub fn handle_insert_mode_key(app: &mut App, key: KeyEvent) {
    // Use textarea's built-in input handling
    let content_changed = app.input.textarea.input(key);

    // Schedule debounced query execution on content change
    if content_changed {
        // Reset history cycling when user types
        app.history.reset_cycling();

        // Schedule debounced execution instead of immediate execution
        // The event loop will check should_execute() and run the query
        // after the debounce period (50ms) has elapsed
        app.debouncer.schedule_execution();

        // Reset scroll when query changes
        app.results_scroll.reset();
        app.error_overlay_visible = false; // Auto-hide error overlay on query change
    }

    // Update autocomplete suggestions after any input
    app.update_autocomplete();
    
    // Update tooltip based on cursor position
    app.update_tooltip();
}

/// Handle keys in Normal mode (VIM navigation and commands)
pub fn handle_normal_mode_key(app: &mut App, key: KeyEvent) {
    match key.code {
        // Toggle help popup
        KeyCode::Char('?') => {
            app.help.visible = !app.help.visible;
        }

        // Basic cursor movement (h/l)
        KeyCode::Char('h') | KeyCode::Left => {
            app.input.textarea.move_cursor(CursorMove::Back);
        }
        KeyCode::Char('l') | KeyCode::Right => {
            app.input.textarea.move_cursor(CursorMove::Forward);
        }

        // Line extent movement (0/^/$)
        KeyCode::Char('0') | KeyCode::Char('^') | KeyCode::Home => {
            app.input.textarea.move_cursor(CursorMove::Head);
        }
        KeyCode::Char('$') | KeyCode::End => {
            app.input.textarea.move_cursor(CursorMove::End);
        }

        // Word movement (w/b/e)
        KeyCode::Char('w') => {
            app.input.textarea.move_cursor(CursorMove::WordForward);
        }
        KeyCode::Char('b') => {
            app.input.textarea.move_cursor(CursorMove::WordBack);
        }
        KeyCode::Char('e') => {
            app.input.textarea.move_cursor(CursorMove::WordEnd);
        }

        // Enter Insert mode commands
        KeyCode::Char('i') => {
            // i - Insert at cursor
            app.input.editor_mode = EditorMode::Insert;
        }
        KeyCode::Char('a') => {
            // a - Append (insert after cursor)
            app.input.textarea.move_cursor(CursorMove::Forward);
            app.input.editor_mode = EditorMode::Insert;
        }
        KeyCode::Char('I') => {
            // I - Insert at line start
            app.input.textarea.move_cursor(CursorMove::Head);
            app.input.editor_mode = EditorMode::Insert;
        }
        KeyCode::Char('A') => {
            // A - Append at line end
            app.input.textarea.move_cursor(CursorMove::End);
            app.input.editor_mode = EditorMode::Insert;
        }

        // Simple delete operations
        KeyCode::Char('x') => {
            // x - Delete character at cursor
            app.input.textarea.delete_next_char();
            execute_query(app);
        }
        KeyCode::Char('X') => {
            // X - Delete character before cursor
            app.input.textarea.delete_char();
            execute_query(app);
        }

        // Delete/Change to end of line
        KeyCode::Char('D') => {
            // D - Delete to end of line (like d$)
            app.input.textarea.delete_line_by_end();
            execute_query(app);
        }
        KeyCode::Char('C') => {
            // C - Change to end of line (like c$)
            app.input.textarea.delete_line_by_end();
            app.input.textarea.cancel_selection();
            app.input.editor_mode = EditorMode::Insert;
            execute_query(app);
        }

        // Operators - enter Operator mode
        KeyCode::Char('d') => {
            // d - Delete operator (wait for motion)
            app.input.editor_mode = EditorMode::Operator('d');
            app.input.textarea.start_selection();
        }
        KeyCode::Char('c') => {
            // c - Change operator (delete then insert)
            app.input.editor_mode = EditorMode::Operator('c');
            app.input.textarea.start_selection();
        }
        KeyCode::Char('y') => {
            // y - Yank operator (wait for motion, yy copies entire line)
            app.input.editor_mode = EditorMode::Operator('y');
        }

        // Undo/Redo
        KeyCode::Char('u') => {
            // u - Undo
            app.input.textarea.undo();
            execute_query(app);
        }
        KeyCode::Char('r') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            // Ctrl+r - Redo
            app.input.textarea.redo();
            execute_query(app);
        }

        _ => {
            // Other VIM commands not yet implemented
        }
    }
    
    // Update tooltip based on cursor position after any cursor movement
    app.update_tooltip();
}


/// Handle keys in Operator mode (waiting for motion after d/c)
pub fn handle_operator_mode_key(app: &mut App, key: KeyEvent) {
    let operator = match app.input.editor_mode {
        EditorMode::Operator(op) => op,
        _ => return, // Should never happen
    };

    // Check for double operator (dd, cc, yy)
    if key.code == KeyCode::Char(operator) {
        match operator {
            'y' => {
                // yy - yank (copy) entire query to clipboard
                clipboard::events::handle_yank_key(app, app.clipboard_backend);
                app.input.editor_mode = EditorMode::Normal;
            }
            'd' | 'c' => {
                // dd or cc - delete entire line
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

    // Apply operator with motion
    let motion_applied = match key.code {
        // Word motions
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
            app.input.textarea.move_cursor(CursorMove::Forward); // Include char at cursor
            true
        }

        // Line extent motions
        KeyCode::Char('0') | KeyCode::Char('^') | KeyCode::Home => {
            app.input.textarea.move_cursor(CursorMove::Head);
            true
        }
        KeyCode::Char('$') | KeyCode::End => {
            app.input.textarea.move_cursor(CursorMove::End);
            true
        }

        // Character motions
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
        // Execute the operator
        match operator {
            'd' => {
                // Delete - cut and stay in Normal mode
                app.input.textarea.cut();
                app.input.editor_mode = EditorMode::Normal;
            }
            'c' => {
                // Change - cut and enter Insert mode
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
        // Invalid motion or ESC - cancel operator
        app.input.textarea.cancel_selection();
        app.input.editor_mode = EditorMode::Normal;
    }
    
    // Update tooltip based on cursor position after any operation
    app.update_tooltip();
}

/// Execute current query and update results
pub fn execute_query(app: &mut App) {
    let query = app.input.textarea.lines()[0].as_ref();
    // Use QueryState::execute() which handles non-null result caching
    app.query.execute(query);

    app.results_scroll.reset();
    app.error_overlay_visible = false; // Auto-hide error overlay on query change
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::Focus;
    use crate::autocomplete::{Suggestion, SuggestionType};
    use crate::config::ClipboardBackend;
    use crate::history::HistoryState;
    use tui_textarea::CursorMove;

    // Test fixture data
    const TEST_JSON: &str = r#"{"name": "test", "age": 30, "city": "NYC"}"#;

    /// Helper to create App with default clipboard backend for tests
    fn test_app(json: &str) -> App {
        App::new(json.to_string(), ClipboardBackend::Auto)
    }

    // Helper to create a KeyEvent without modifiers
    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::empty())
    }

    // Helper to create a KeyEvent with specific modifiers
    fn key_with_mods(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
        KeyEvent::new(code, modifiers)
    }

    // Helper to set up an app with text in the query field
    fn app_with_query(query: &str) -> App {
        let mut app = test_app(TEST_JSON);
        app.input.textarea.insert_str(query);
        // Use empty in-memory history for all tests to prevent disk writes
        app.history = HistoryState::empty();
        app
    }

    // Helper to move cursor to specific position by text content
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
        // Move to position 5 (after ".name")
        move_cursor_to_position(&mut app, 5);
        app.input.editor_mode = EditorMode::Normal;

        app.handle_key_event(key(KeyCode::Char('d')));
        app.handle_key_event(key(KeyCode::Char('$')));

        assert_eq!(app.query(), ".name");
    }

    #[test]
    fn test_operator_d0_deletes_to_start_of_line() {
        let mut app = app_with_query(".name.first");
        // Move to middle of text
        move_cursor_to_position(&mut app, 6);
        app.input.editor_mode = EditorMode::Normal;

        app.handle_key_event(key(KeyCode::Char('d')));
        app.handle_key_event(key(KeyCode::Char('0')));

        assert!(app.query().ends_with("first"));
    }

    #[test]
    fn test_operator_d_caret_deletes_to_start_of_line() {
        let mut app = app_with_query(".name.first");
        // Move to middle of text
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

        // Should delete word and enter Insert mode
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

        // Press invalid motion key (z is not a valid motion)
        app.handle_key_event(key(KeyCode::Char('z')));

        // Should cancel operator and return to Normal mode without changing text
        assert_eq!(app.input.editor_mode, EditorMode::Normal);
        assert_eq!(app.query(), original_query);
    }

    #[test]
    fn test_escape_in_operator_mode_cancels_operator() {
        let mut app = app_with_query(".name");
        app.input.editor_mode = EditorMode::Normal;
        let original_query = app.query().to_string();

        // Enter operator mode
        app.handle_key_event(key(KeyCode::Char('d')));
        assert!(matches!(app.input.editor_mode, EditorMode::Operator('d')));

        // Press Escape - should NOT go to Insert mode, should cancel operator
        app.handle_key_event(key(KeyCode::Esc));

        // Should return to Normal mode and preserve text
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

        // Should delete one character backward
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

        // Should delete one character forward
        assert!(app.query().len() < 5);
        assert_eq!(app.input.editor_mode, EditorMode::Normal);
    }


    // ========== Mode Transition Tests ==========

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
        // Cursor should remain at same position
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
        // Cursor should move forward by one
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
        assert_eq!(app.input.textarea.cursor().1, 5); // Should be at end of ".name"
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

    // ========== Simple VIM Commands ==========

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
        app.input.textarea.undo(); // Undo the insert
        assert_eq!(app.query(), "");

        app.handle_key_event(key_with_mods(KeyCode::Char('r'), KeyModifiers::CONTROL));

        // After redo, query should be back
        assert_eq!(app.query(), ".name");
    }


    // ========== VIM Navigation Tests ==========

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

        // Should move forward by at least one position
        assert!(app.input.textarea.cursor().1 > cursor_before);
    }

    #[test]
    fn test_b_moves_word_backward() {
        let mut app = app_with_query(".name.first");
        app.input.textarea.move_cursor(CursorMove::End);
        app.input.editor_mode = EditorMode::Normal;
        let cursor_before = app.input.textarea.cursor().1;

        app.handle_key_event(key(KeyCode::Char('b')));

        // Should move backward
        assert!(app.input.textarea.cursor().1 < cursor_before);
    }

    #[test]
    fn test_e_moves_to_word_end() {
        let mut app = app_with_query(".name.first");
        app.input.textarea.move_cursor(CursorMove::Head);
        app.input.editor_mode = EditorMode::Normal;
        let cursor_before = app.input.textarea.cursor().1;

        app.handle_key_event(key(KeyCode::Char('e')));

        // Should move forward
        assert!(app.input.textarea.cursor().1 > cursor_before);
    }

    // ========== Autocomplete Interaction Tests ==========

    #[test]
    fn test_escape_closes_autocomplete() {
        let mut app = app_with_query(".na");
        app.input.editor_mode = EditorMode::Insert;

        // Manually set autocomplete as visible with suggestions
        let suggestions = vec![
            Suggestion::new(".name", SuggestionType::Field),
        ];
        app.autocomplete.update_suggestions(suggestions);
        assert!(app.autocomplete.is_visible());

        app.handle_key_event(key(KeyCode::Esc));

        assert!(!app.autocomplete.is_visible());
        assert_eq!(app.query(), ".na"); // Query unchanged
        assert_eq!(app.input.editor_mode, EditorMode::Insert); // Still in insert mode
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

        // Should select second suggestion
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

        // Move to second suggestion
        app.autocomplete.select_next();

        app.handle_key_event(key(KeyCode::Up));

        // Should select first suggestion
        assert_eq!(app.autocomplete.selected().unwrap().text, ".name");
    }


    #[test]
    fn test_tab_accepts_autocomplete_suggestion() {
        // Test accepting field suggestion at root level
        let mut app = app_with_query(".na");
        app.input.editor_mode = EditorMode::Insert;
        app.focus = Focus::InputField;

        // Validate base state
        // .na returns null, so base_query stays at "." (from App::new())
        use crate::query::ResultType;
        assert_eq!(app.query.base_query_for_suggestions, Some(".".to_string()),
                   "base_query should remain '.' since .na returns null");
        assert_eq!(app.query.base_type_for_suggestions, Some(ResultType::Object),
                   "base_type should be Object (root object)");

        // Suggestion should be "name" (no leading dot) since after Dot (CharType::Dot)
        let suggestions = vec![
            Suggestion::new("name", SuggestionType::Field),
        ];
        app.autocomplete.update_suggestions(suggestions);

        app.handle_key_event(key(KeyCode::Tab));

        // Formula for Dot: base + suggestion = "." + "name" = ".name" ✅
        assert_eq!(app.query(), ".name");
        assert!(!app.autocomplete.is_visible());
    }

    #[test]
    fn test_tab_without_autocomplete_stays_in_consistent_state() {
        let mut app = app_with_query("x");  // Use a query that won't trigger autocomplete
        app.input.editor_mode = EditorMode::Insert;
        app.focus = Focus::InputField;

        // Ensure autocomplete is not visible
        app.autocomplete.hide();
        assert!(!app.autocomplete.is_visible());

        app.handle_key_event(key(KeyCode::Tab));

        // Tab without autocomplete gets passed through to textarea
        // Verify the app remains in a consistent state (doesn't crash, mode unchanged)
        assert_eq!(app.input.editor_mode, EditorMode::Insert);
        assert_eq!(app.focus, Focus::InputField);
    }

    #[test]
    fn test_autocomplete_navigation_only_works_in_insert_mode() {
        let mut app = app_with_query(".na");
        app.input.editor_mode = EditorMode::Normal;
        app.focus = Focus::InputField;

        let suggestions = vec![
            Suggestion::new(".name", SuggestionType::Field),
        ];
        app.autocomplete.update_suggestions(suggestions);

        // Down arrow in Normal mode should NOT navigate autocomplete (it's not handled)
        let selected_before = app.autocomplete.selected().unwrap().text.clone();
        app.handle_key_event(key(KeyCode::Down));
        let selected_after = app.autocomplete.selected().unwrap().text.clone();

        // Autocomplete selection should remain unchanged in Normal mode
        assert_eq!(selected_before, selected_after);
    }
}
