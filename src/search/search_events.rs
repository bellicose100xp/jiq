//! Search event handling
//!
//! Handles keyboard events for the search feature including:
//! - Opening/closing search bar
//! - Text input to search query
//! - Navigation between matches (n/N, Enter/Shift+Enter)

use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[cfg(debug_assertions)]
use log::debug;

use crate::app::{App, Focus};
use crate::results::results_events::handle_results_pane_key;

#[path = "search_events/scroll.rs"]
mod scroll;

use scroll::scroll_to_line;

/// Handle search-related key events when search is visible
/// Returns true if event was consumed, false otherwise
pub fn handle_search_key(app: &mut App, key: KeyEvent) -> bool {
    if !app.search.is_visible() {
        return false;
    }

    match key.code {
        // Close search with Escape
        KeyCode::Esc => {
            close_search(app);
            true
        }

        // Enter confirms search (first press) or navigates to next match (subsequent presses)
        KeyCode::Enter if !key.modifiers.contains(KeyModifiers::SHIFT) => {
            if !app.search.is_confirmed() {
                // First Enter: just confirm and scroll to current match (index 0)
                app.search.confirm();

                if let Some(current_match) = app.search.current_match() {
                    #[cfg(debug_assertions)]
                    debug!(
                        "Search: confirmed on first match -> line {}, index {}/{}",
                        current_match.line,
                        app.search.current_index() + 1,
                        app.search.matches().len()
                    );
                    scroll_to_line(app, current_match.line);
                }
            } else {
                // Already confirmed: navigate to next match
                if let Some(line) = app.search.next_match() {
                    #[cfg(debug_assertions)]
                    debug!(
                        "Search: next match (Enter) -> line {}, index {}/{}",
                        line,
                        app.search.current_index() + 1,
                        app.search.matches().len()
                    );
                    scroll_to_line(app, line);
                }
            }
            true
        }

        // Shift+Enter confirms search (first press) or navigates to previous match (subsequent presses)
        KeyCode::Enter if key.modifiers.contains(KeyModifiers::SHIFT) => {
            if !app.search.is_confirmed() {
                // First Shift+Enter: just confirm and scroll to current match (index 0)
                app.search.confirm();

                if let Some(current_match) = app.search.current_match() {
                    #[cfg(debug_assertions)]
                    debug!(
                        "Search: confirmed on first match (Shift+Enter) -> line {}, index {}/{}",
                        current_match.line,
                        app.search.current_index() + 1,
                        app.search.matches().len()
                    );
                    scroll_to_line(app, current_match.line);
                }
            } else {
                // Already confirmed: navigate to previous match
                if let Some(line) = app.search.prev_match() {
                    #[cfg(debug_assertions)]
                    debug!(
                        "Search: prev match (Shift+Enter) -> line {}, index {}/{}",
                        line,
                        app.search.current_index() + 1,
                        app.search.matches().len()
                    );
                    scroll_to_line(app, line);
                }
            }
            true
        }

        // n/N only navigate when search is confirmed (after Enter)
        KeyCode::Char('n')
            if !key.modifiers.contains(KeyModifiers::SHIFT) && app.search.is_confirmed() =>
        {
            if let Some(line) = app.search.next_match() {
                #[cfg(debug_assertions)]
                debug!(
                    "Search: next match -> line {}, index {}/{}",
                    line,
                    app.search.current_index() + 1,
                    app.search.matches().len()
                );
                scroll_to_line(app, line);
            }
            true
        }
        KeyCode::Char('N') if app.search.is_confirmed() => {
            if let Some(line) = app.search.prev_match() {
                #[cfg(debug_assertions)]
                debug!(
                    "Search: prev match -> line {}, index {}/{}",
                    line,
                    app.search.current_index() + 1,
                    app.search.matches().len()
                );
                scroll_to_line(app, line);
            }
            true
        }
        KeyCode::Char('n')
            if key.modifiers.contains(KeyModifiers::SHIFT) && app.search.is_confirmed() =>
        {
            if let Some(line) = app.search.prev_match() {
                #[cfg(debug_assertions)]
                debug!(
                    "Search: prev match (Shift+n) -> line {}, index {}/{}",
                    line,
                    app.search.current_index() + 1,
                    app.search.matches().len()
                );
                scroll_to_line(app, line);
            }
            true
        }

        // Ctrl+F re-enters edit mode when search is confirmed
        KeyCode::Char('f')
            if key.modifiers.contains(KeyModifiers::CONTROL) && app.search.is_confirmed() =>
        {
            #[cfg(debug_assertions)]
            debug!("Search: re-entering edit mode via Ctrl+F");
            app.search.unconfirm();
            true
        }

        // '/' re-enters edit mode when search is confirmed
        KeyCode::Char('/') if app.search.is_confirmed() => {
            #[cfg(debug_assertions)]
            debug!("Search: re-entering edit mode via /");
            app.search.unconfirm();
            true
        }

        // When confirmed, delegate navigation keys to results pane handler
        // User must press Ctrl+F or / to re-enter edit mode
        _ if app.search.is_confirmed() => {
            #[cfg(debug_assertions)]
            debug!(
                "Search: delegating key {:?} to results pane handler",
                key.code
            );
            handle_results_pane_key(app, key);
            true
        }

        // When NOT confirmed, pass keys to the search textarea for text input
        _ => {
            // Forward key to textarea
            app.search.search_textarea_mut().input(key);

            // Update matches based on new query
            // Use unformatted result (without ANSI codes) so match positions align with rendered text
            if let Some(content) = &app.query.last_successful_result_unformatted {
                app.search.update_matches(content);

                #[cfg(debug_assertions)]
                debug!(
                    "Search: query changed to '{}', found {} matches",
                    app.search.query(),
                    app.search.matches().len()
                );
            }

            // Jump to first match if we have any
            if let Some(m) = app.search.current_match() {
                scroll_to_line(app, m.line);
            }

            true
        }
    }
}

/// Open search bar and focus results pane
pub fn open_search(app: &mut App) {
    #[cfg(debug_assertions)]
    debug!("Search: opened");

    app.search.open();
    app.focus = Focus::ResultsPane;
}

/// Close search bar and clear all state
pub fn close_search(app: &mut App) {
    #[cfg(debug_assertions)]
    debug!("Search: closed (query was '{}')", app.search.query());

    app.search.close();
}

#[cfg(test)]
#[path = "search_events_tests.rs"]
mod search_events_tests;
