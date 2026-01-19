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
        _ => {}
    }
}

#[cfg(test)]
#[path = "snippet_events_tests.rs"]
mod snippet_events_tests;
