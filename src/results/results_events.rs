use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::App;
use crate::clipboard;
use crate::editor::EditorMode;
use crate::help::HelpTab;

pub fn handle_results_pane_key(app: &mut App, key: KeyEvent) {
    if app.results_cursor.is_visual_mode() && handle_visual_mode_key(app, key) {
        return;
    }

    match key.code {
        KeyCode::Tab if !key.modifiers.contains(KeyModifiers::CONTROL) => {
            exit_results_pane(app);
        }

        KeyCode::BackTab => {
            exit_results_pane(app);
        }

        KeyCode::Char('i') => {
            exit_results_pane(app);
            app.input.editor_mode = EditorMode::Insert;
        }

        KeyCode::Char('/') => {
            crate::search::search_events::open_search(app);
        }

        KeyCode::Char('?') => {
            if app.help.visible {
                app.help.reset();
            } else {
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

        KeyCode::Char('v') | KeyCode::Char('V') => {
            app.results_cursor.enter_visual_mode();
        }

        KeyCode::Up | KeyCode::Char('k') => {
            move_cursor_up(app, 1);
        }
        KeyCode::Down | KeyCode::Char('j') => {
            move_cursor_down(app, 1);
        }

        KeyCode::Char('K') => {
            move_cursor_up(app, 10);
        }
        KeyCode::Char('J') => {
            move_cursor_down(app, 10);
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
            let width = app.results_cursor.get_cursor_line_width();
            let viewport_width = app.results_scroll.viewport_width;
            app.results_scroll.h_offset = width.saturating_sub(viewport_width);
        }

        KeyCode::Home | KeyCode::Char('g') => {
            app.results_cursor.move_to_first();
            app.results_scroll
                .ensure_cursor_visible(app.results_cursor.cursor_line());
        }

        KeyCode::End | KeyCode::Char('G') => {
            app.results_cursor.move_to_last();
            app.results_scroll
                .ensure_cursor_visible(app.results_cursor.cursor_line());
        }

        KeyCode::PageUp | KeyCode::Char('u')
            if key.code == KeyCode::PageUp || key.modifiers.contains(KeyModifiers::CONTROL) =>
        {
            let half_page = app.results_scroll.viewport_height / 2;
            move_cursor_up(app, half_page as u32);
        }
        KeyCode::PageDown | KeyCode::Char('d')
            if key.code == KeyCode::PageDown || key.modifiers.contains(KeyModifiers::CONTROL) =>
        {
            let half_page = app.results_scroll.viewport_height / 2;
            move_cursor_down(app, half_page as u32);
        }

        _ => {}
    }
}

fn handle_visual_mode_key(app: &mut App, key: KeyEvent) -> bool {
    match key.code {
        KeyCode::Esc => {
            app.results_cursor.exit_visual_mode();
            true
        }

        KeyCode::Char('v') | KeyCode::Char('V') => {
            app.results_cursor.exit_visual_mode();
            true
        }

        KeyCode::Char('y') => {
            clipboard::clipboard_events::handle_yank_key(app, app.clipboard_backend);
            app.results_cursor.exit_visual_mode();
            true
        }

        KeyCode::Char('$') => {
            let width = app.results_cursor.get_max_selected_line_width();
            let viewport_width = app.results_scroll.viewport_width;
            app.results_scroll.h_offset = width.saturating_sub(viewport_width);
            true
        }

        _ => false,
    }
}

fn move_cursor_up(app: &mut App, lines: u32) {
    app.results_cursor.move_up(lines);
    app.results_scroll
        .ensure_cursor_visible(app.results_cursor.cursor_line());
}

fn move_cursor_down(app: &mut App, lines: u32) {
    app.results_cursor.move_down(lines);
    app.results_scroll
        .ensure_cursor_visible(app.results_cursor.cursor_line());
}

fn exit_results_pane(app: &mut App) {
    app.focus_input_field();
    app.results_cursor.exit_visual_mode();
}

#[cfg(test)]
#[path = "results_events_tests.rs"]
mod results_events_tests;
