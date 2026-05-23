use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::{App, Focus};
use crate::path_at_cursor_apply::PathSource;
use crate::results::results_events::{drill_back, handle_results_pane_key};

#[path = "search_events/scroll.rs"]
mod scroll;

use scroll::scroll_to_line;

pub fn handle_search_key(app: &mut App, key: KeyEvent) -> bool {
    if !app.search.is_visible() {
        return false;
    }

    match key.code {
        KeyCode::Esc => {
            close_search(app);
            true
        }

        KeyCode::Enter if !key.modifiers.contains(KeyModifiers::SHIFT) => {
            if !app.search.is_confirmed() {
                app.search.confirm();

                if let Some(current_match) = app.search.current_match() {
                    scroll_to_line(app, current_match.line);
                }
            } else if let Some(line) = app.search.next_match() {
                scroll_to_line(app, line);
            }
            true
        }

        KeyCode::Enter if key.modifiers.contains(KeyModifiers::SHIFT) => {
            if !app.search.is_confirmed() {
                app.search.confirm();

                if let Some(current_match) = app.search.current_match() {
                    scroll_to_line(app, current_match.line);
                }
            } else if let Some(line) = app.search.prev_match() {
                scroll_to_line(app, line);
            }
            true
        }

        KeyCode::Char('n')
            if !key.modifiers.contains(KeyModifiers::SHIFT) && app.search.is_confirmed() =>
        {
            if let Some(line) = app.search.next_match() {
                scroll_to_line(app, line);
            }
            true
        }
        KeyCode::Char('N') if app.search.is_confirmed() => {
            if let Some(line) = app.search.prev_match() {
                scroll_to_line(app, line);
            }
            true
        }
        KeyCode::Char('n')
            if key.modifiers.contains(KeyModifiers::SHIFT) && app.search.is_confirmed() =>
        {
            if let Some(line) = app.search.prev_match() {
                scroll_to_line(app, line);
            }
            true
        }

        KeyCode::Char('f')
            if key.modifiers.contains(KeyModifiers::CONTROL) && app.search.is_confirmed() =>
        {
            app.search.unconfirm();
            true
        }

        KeyCode::Char('/') if app.search.is_confirmed() => {
            app.search.unconfirm();
            true
        }

        KeyCode::Tab
            if !key.modifiers.contains(KeyModifiers::CONTROL) && !app.search.is_confirmed() =>
        {
            app.search.confirm();

            if let Some(current_match) = app.search.current_match() {
                scroll_to_line(app, current_match.line);
            }
            true
        }

        KeyCode::Tab
            if !key.modifiers.contains(KeyModifiers::CONTROL) && app.search.is_confirmed() =>
        {
            app.search.unconfirm();
            true
        }

        KeyCode::Char('>') => {
            drill_in_from_search(app);
            true
        }

        KeyCode::Char('<') => {
            drill_back(app);
            true
        }

        // Delegate navigation keys to results pane when confirmed
        _ if app.search.is_confirmed() => {
            handle_results_pane_key(app, key);
            true
        }

        _ => {
            app.search.search_textarea_mut().input(key);

            // Only update matches if query state is available
            if let Some(query_state) = &app.query
                && let Some(content) = &query_state.last_successful_result_unformatted
            {
                app.search.update_matches(content);
            }

            if let Some(m) = app.search.current_match() {
                scroll_to_line(app, m.line);
            } else if !app.search.query().is_empty() {
                app.results_scroll.offset = 0;
                app.results_scroll.h_offset = 0;
            }

            true
        }
    }
}

pub fn open_search(app: &mut App) {
    app.saved_ai_visibility_for_search = app.ai.visible;
    app.ai.visible = false;
    app.saved_tooltip_visibility_for_search = app.tooltip.enabled;
    app.tooltip.enabled = false;
    app.saved_focus_for_search = app.focus;
    app.search.open();
    app.focus = Focus::ResultsPane;
}

pub fn close_search(app: &mut App) {
    app.search.close();
    app.ai.visible = app.saved_ai_visibility_for_search;
    app.tooltip.enabled = app.saved_tooltip_visibility_for_search;
    app.focus = app.saved_focus_for_search;
}

/// Apply the path of the row holding the *current* search match, then close
/// the search overlay so the user lands directly on the drilled result.
/// When no match exists, surface the user notification without touching
/// either the query or the search state. When the matched row is the root
/// (no meaningful drill-in), the notification fires and the search overlay
/// stays open so the user can pick a different match.
fn drill_in_from_search(app: &mut App) {
    use crate::path_at_cursor_apply::{ApplyOutcome, apply_path};

    let match_row = match app.search.current_match() {
        Some(m) => m.line,
        None => {
            app.notification.show("No match to navigate to");
            return;
        }
    };
    match apply_path(app, PathSource::Row(match_row)) {
        ApplyOutcome::Applied(_) => close_search(app),
        ApplyOutcome::AtRoot => app.notification.show("Already at root"),
        ApplyOutcome::NoPath => app.notification.show("No path at cursor"),
    }
}

#[cfg(test)]
#[path = "search_events_tests.rs"]
mod search_events_tests;
