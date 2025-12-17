use ratatui::crossterm::event::{KeyCode, KeyEvent};
use tui_textarea::Input;

use crate::app::App;

pub fn handle_history_popup_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Up => {
            app.history.select_next();
        }
        KeyCode::Down => {
            app.history.select_previous();
        }

        KeyCode::Enter | KeyCode::Tab => {
            if let Some(entry) = app.history.selected_entry() {
                let entry = entry.to_string();
                replace_query_with(app, &entry);
            }
            app.history.close();
        }

        KeyCode::Esc => {
            app.history.close();
        }

        _ => {
            let input = Input::from(key);
            if app.history.search_textarea_mut().input(input) {
                app.history.on_search_input_changed();
            }
        }
    }
}

fn replace_query_with(app: &mut App, text: &str) {
    app.input.textarea.delete_line_by_head();
    app.input.textarea.delete_line_by_end();
    app.input.textarea.insert_str(text);

    let query = app.input.textarea.lines()[0].as_ref();
    app.query.execute(query);

    app.results_scroll.reset();
    app.error_overlay_visible = false;
}

#[cfg(test)]
#[path = "history_events_tests.rs"]
mod history_events_tests;
