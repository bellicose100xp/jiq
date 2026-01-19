use ratatui::crossterm::event::{KeyCode, KeyEvent};

use crate::app::App;

pub fn handle_snippet_popup_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            app.snippets.close();
        }
        KeyCode::Up | KeyCode::Char('k') => {
            app.snippets.select_prev();
        }
        KeyCode::Down | KeyCode::Char('j') => {
            app.snippets.select_next();
        }
        KeyCode::Enter => {
            if let Some(snippet) = app.snippets.selected_snippet() {
                let query = snippet.query.clone();
                apply_snippet(app, &query);
            }
            app.snippets.close();
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
