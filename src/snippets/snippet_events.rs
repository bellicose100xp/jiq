use ratatui::crossterm::event::{KeyCode, KeyEvent};

use crate::app::App;

pub fn handle_snippet_popup_key(app: &mut App, key: KeyEvent) {
    if key.code == KeyCode::Esc {
        app.snippets.close();
    }
}

#[cfg(test)]
#[path = "snippet_events_tests.rs"]
mod snippet_events_tests;
