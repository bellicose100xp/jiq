use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tui_textarea::CursorMove;

use crate::app::App;
use crate::clipboard;
use crate::editor::EditorMode;
use crate::editor::char_search::{
    CharSearchState, SearchDirection, SearchType, execute_char_search,
};
use crate::editor::mode::TextObjectScope;
use crate::editor::text_objects::{TextObjectTarget, execute_text_object};

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

        KeyCode::Char('f') => {
            app.input.editor_mode =
                EditorMode::CharSearch(SearchDirection::Forward, SearchType::Find);
        }
        KeyCode::Char('F') => {
            app.input.editor_mode =
                EditorMode::CharSearch(SearchDirection::Backward, SearchType::Find);
        }
        KeyCode::Char('t') => {
            app.input.editor_mode =
                EditorMode::CharSearch(SearchDirection::Forward, SearchType::Till);
        }
        KeyCode::Char('T') => {
            app.input.editor_mode =
                EditorMode::CharSearch(SearchDirection::Backward, SearchType::Till);
        }

        KeyCode::Char(';') => {
            repeat_last_char_search(app, false);
        }
        KeyCode::Char(',') => {
            repeat_last_char_search(app, true);
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

        KeyCode::Char('i') => {
            app.input.textarea.cancel_selection();
            app.input.editor_mode = EditorMode::TextObject(operator, TextObjectScope::Inner);
            app.update_tooltip();
            return;
        }
        KeyCode::Char('a') => {
            app.input.textarea.cancel_selection();
            app.input.editor_mode = EditorMode::TextObject(operator, TextObjectScope::Around);
            app.update_tooltip();
            return;
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

pub fn handle_char_search_mode_key(app: &mut App, key: KeyEvent) {
    let (direction, search_type) = match app.input.editor_mode {
        EditorMode::CharSearch(dir, st) => (dir, st),
        _ => return,
    };

    if let KeyCode::Char(target) = key.code {
        let found = execute_char_search(&mut app.input.textarea, target, direction, search_type);

        if found {
            app.input.last_char_search = Some(CharSearchState {
                character: target,
                direction,
                search_type,
            });
        }
    }

    app.input.editor_mode = EditorMode::Normal;
    app.update_tooltip();
}

pub fn handle_text_object_mode_key(app: &mut App, key: KeyEvent) {
    let (operator, scope) = match app.input.editor_mode {
        EditorMode::TextObject(op, sc) => (op, sc),
        _ => return,
    };

    if let KeyCode::Char(target_char) = key.code {
        if let Some(target) = TextObjectTarget::from_char(target_char) {
            let success = execute_text_object(&mut app.input.textarea, target, scope);

            if success {
                app.input.editor_mode = if operator == 'c' {
                    EditorMode::Insert
                } else {
                    EditorMode::Normal
                };
                execute_query(app);
            } else {
                app.input.editor_mode = EditorMode::Normal;
            }
        } else {
            app.input.editor_mode = EditorMode::Normal;
        }
    } else {
        app.input.editor_mode = EditorMode::Normal;
    }

    app.update_tooltip();
}

fn repeat_last_char_search(app: &mut App, reverse: bool) {
    if let Some(search) = app.input.last_char_search {
        let direction = if reverse {
            search.direction.opposite()
        } else {
            search.direction
        };

        execute_char_search(
            &mut app.input.textarea,
            search.character,
            direction,
            search.search_type,
        );
    }
}

pub fn execute_query(app: &mut App) {
    execute_query_with_auto_show(app);
}

pub fn execute_query_with_auto_show(app: &mut App) {
    let query_state = match &mut app.query {
        Some(q) => q,
        None => return,
    };

    let query = app.input.textarea.lines()[0].as_ref();

    app.input.brace_tracker.rebuild(query);

    query_state.execute_async(query);

    app.results_scroll.reset();
    app.error_overlay_visible = false;

    // AI update happens in poll_query_response() when result arrives
}

#[cfg(test)]
#[path = "editor_events_tests.rs"]
mod editor_events_tests;
