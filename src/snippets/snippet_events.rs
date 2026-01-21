use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tui_textarea::Input;

use super::snippet_state::SnippetMode;
use crate::app::App;

pub fn handle_snippet_popup_key(app: &mut App, key: KeyEvent) {
    match app.snippets.mode() {
        SnippetMode::Browse => handle_browse_mode(app, key),
        SnippetMode::CreateName => handle_create_name_mode(app, key),
        SnippetMode::CreateQuery => handle_create_query_mode(app, key),
        SnippetMode::CreateDescription => handle_create_description_mode(app, key),
        SnippetMode::EditName { .. } => handle_edit_name_mode(app, key),
        SnippetMode::EditQuery { .. } => handle_edit_query_mode(app, key),
        SnippetMode::EditDescription { .. } => handle_edit_description_mode(app, key),
        SnippetMode::ConfirmDelete { .. } => handle_confirm_delete_mode(app, key),
        SnippetMode::ConfirmUpdate { .. } => handle_confirm_update_mode(app, key),
    }
}

fn handle_browse_mode(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            app.snippets.close();
        }
        KeyCode::Up => {
            app.snippets.select_prev();
        }
        KeyCode::Down => {
            app.snippets.select_next();
        }
        KeyCode::Enter => {
            if let Some(snippet) = app.snippets.selected_snippet() {
                let query = snippet.query.clone();
                apply_snippet(app, &query);
            }
            app.snippets.close();
        }
        KeyCode::Char('n') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            let current_query = app.input.query().to_string();
            app.snippets.enter_create_mode(&current_query);
        }
        KeyCode::Char('e') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            if app.snippets.selected_snippet().is_some() {
                app.snippets.enter_edit_mode();
            }
        }
        KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            if app.snippets.selected_snippet().is_some() {
                app.snippets.enter_delete_mode();
            }
        }
        KeyCode::Char('r') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            if app.snippets.selected_snippet().is_some() {
                let current_query = app.input.query().to_string();
                if let Err(e) = app.snippets.enter_update_confirmation(current_query) {
                    app.notification.show_warning(&e);
                }
            }
        }
        _ => {
            let input = Input::from(key);
            if app.snippets.search_textarea_mut().input(input) {
                app.snippets.on_search_input_changed();
            }
        }
    }
}

fn handle_create_name_mode(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            app.snippets.cancel_create();
        }
        KeyCode::Enter => {
            if let Err(e) = app.snippets.save_new_snippet() {
                app.notification.show_warning(&e);
            }
        }
        KeyCode::Tab => {
            app.snippets.next_field();
        }
        KeyCode::BackTab => {
            app.snippets.prev_field();
        }
        _ => {
            let input = Input::from(key);
            app.snippets.name_textarea_mut().input(input);
        }
    }
}

fn handle_create_query_mode(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            app.snippets.cancel_create();
        }
        KeyCode::Enter => {
            if let Err(e) = app.snippets.save_new_snippet() {
                app.notification.show_warning(&e);
            }
        }
        KeyCode::Tab => {
            app.snippets.next_field();
        }
        KeyCode::BackTab => {
            app.snippets.prev_field();
        }
        _ => {
            let input = Input::from(key);
            app.snippets.query_textarea_mut().input(input);
        }
    }
}

fn handle_create_description_mode(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            app.snippets.cancel_create();
        }
        KeyCode::Enter => {
            if let Err(e) = app.snippets.save_new_snippet() {
                app.notification.show_warning(&e);
            }
        }
        KeyCode::Tab => {
            app.snippets.next_field();
        }
        KeyCode::BackTab => {
            app.snippets.prev_field();
        }
        _ => {
            let input = Input::from(key);
            app.snippets.description_textarea_mut().input(input);
        }
    }
}

fn handle_edit_name_mode(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            app.snippets.cancel_edit();
        }
        KeyCode::Enter => {
            if let Err(e) = app.snippets.update_snippet_name() {
                app.notification.show_warning(&e);
            } else {
                app.snippets.cancel_edit();
            }
        }
        KeyCode::Tab => {
            if let Err(e) = app.snippets.update_snippet_name() {
                app.notification.show_warning(&e);
            } else {
                app.snippets.next_field();
            }
        }
        KeyCode::BackTab => {
            if let Err(e) = app.snippets.update_snippet_name() {
                app.notification.show_warning(&e);
            } else {
                app.snippets.prev_field();
            }
        }
        _ => {
            let input = Input::from(key);
            app.snippets.name_textarea_mut().input(input);
        }
    }
}

fn handle_edit_query_mode(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            app.snippets.cancel_edit();
        }
        KeyCode::Enter => {
            if let Err(e) = app.snippets.update_snippet_query() {
                app.notification.show_warning(&e);
            } else {
                app.snippets.cancel_edit();
            }
        }
        KeyCode::Tab => {
            if let Err(e) = app.snippets.update_snippet_query() {
                app.notification.show_warning(&e);
            } else {
                app.snippets.next_field();
            }
        }
        KeyCode::BackTab => {
            if let Err(e) = app.snippets.update_snippet_query() {
                app.notification.show_warning(&e);
            } else {
                app.snippets.prev_field();
            }
        }
        _ => {
            let input = Input::from(key);
            app.snippets.query_textarea_mut().input(input);
        }
    }
}

fn handle_edit_description_mode(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            app.snippets.cancel_edit();
        }
        KeyCode::Enter => {
            if let Err(e) = app.snippets.update_snippet_description() {
                app.notification.show_warning(&e);
            } else {
                app.snippets.cancel_edit();
            }
        }
        KeyCode::Tab => {
            if let Err(e) = app.snippets.update_snippet_description() {
                app.notification.show_warning(&e);
            } else {
                app.snippets.next_field();
            }
        }
        KeyCode::BackTab => {
            if let Err(e) = app.snippets.update_snippet_description() {
                app.notification.show_warning(&e);
            } else {
                app.snippets.prev_field();
            }
        }
        _ => {
            let input = Input::from(key);
            app.snippets.description_textarea_mut().input(input);
        }
    }
}

fn handle_confirm_delete_mode(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Enter => {
            if let Err(e) = app.snippets.confirm_delete() {
                app.notification.show_warning(&e);
            }
        }
        KeyCode::Esc => {
            app.snippets.cancel_delete();
        }
        _ => {}
    }
}

fn handle_confirm_update_mode(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Enter => {
            if let Err(e) = app.snippets.confirm_update() {
                app.notification.show_warning(&e);
            }
        }
        KeyCode::Esc => {
            app.snippets.cancel_update();
        }
        _ => {}
    }
}

fn apply_snippet(app: &mut App, query: &str) {
    app.input.textarea.delete_line_by_head();
    app.input.textarea.delete_line_by_end();
    app.input.textarea.insert_str(query);

    let query_text = app.input.textarea.lines()[0].as_ref();
    if let Some(query_state) = &mut app.query {
        query_state.execute(query_text);
    }

    app.results_scroll.reset();
    app.error_overlay_visible = false;
}

#[cfg(test)]
#[path = "snippet_events_tests.rs"]
mod snippet_events_tests;
