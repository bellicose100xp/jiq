use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::{App, Focus};
use crate::clipboard;
use crate::editor::EditorMode;
use crate::help::HelpTab;

pub fn handle_results_pane_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Tab if !key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.focus = Focus::InputField;
            app.ai.visible = app.saved_ai_visibility_for_results;
            app.tooltip.enabled = app.saved_tooltip_visibility_for_results;
        }

        KeyCode::Char('i') => {
            app.focus = Focus::InputField;
            app.input.editor_mode = EditorMode::Insert;
            app.ai.visible = app.saved_ai_visibility_for_results;
            app.tooltip.enabled = app.saved_tooltip_visibility_for_results;
        }

        KeyCode::Char('/') => {
            crate::search::search_events::open_search(app);
        }

        KeyCode::Char('?') => {
            if app.help.visible {
                app.help.reset();
            } else {
                // Check if search is active (takes priority over Result tab)
                app.help.active_tab = if app.search.is_visible() {
                    HelpTab::Search
                } else {
                    HelpTab::Result
                };
                app.help.visible = true;
            }
        }

        KeyCode::Char('y') => {
            clipboard::clipboard_events::handle_yank_key(app, app.clipboard_backend);
        }

        KeyCode::Up | KeyCode::Char('k') => {
            app.results_scroll.scroll_up(1);
        }
        KeyCode::Down | KeyCode::Char('j') => {
            app.results_scroll.scroll_down(1);
        }

        KeyCode::Char('K') => {
            app.results_scroll.scroll_up(10);
        }
        KeyCode::Char('J') => {
            app.results_scroll.scroll_down(10);
        }

        KeyCode::Left | KeyCode::Char('h') => {
            app.results_scroll.scroll_left(1);
        }
        KeyCode::Right | KeyCode::Char('l') => {
            app.results_scroll.scroll_right(1);
        }

        KeyCode::Char('H') => {
            app.results_scroll.scroll_left(10);
        }
        KeyCode::Char('L') => {
            app.results_scroll.scroll_right(10);
        }

        KeyCode::Char('0') | KeyCode::Char('^') => {
            app.results_scroll.jump_to_left();
        }

        KeyCode::Char('$') => {
            app.results_scroll.jump_to_right();
        }

        KeyCode::Home | KeyCode::Char('g') => {
            app.results_scroll.jump_to_top();
        }

        KeyCode::End | KeyCode::Char('G') => {
            app.results_scroll.jump_to_bottom();
        }

        KeyCode::PageUp | KeyCode::Char('u')
            if key.code == KeyCode::PageUp || key.modifiers.contains(KeyModifiers::CONTROL) =>
        {
            app.results_scroll.page_up();
        }
        KeyCode::PageDown | KeyCode::Char('d')
            if key.code == KeyCode::PageDown || key.modifiers.contains(KeyModifiers::CONTROL) =>
        {
            app.results_scroll.page_down();
        }

        _ => {}
    }
}

#[cfg(test)]
#[path = "results_events_tests.rs"]
mod results_events_tests;
