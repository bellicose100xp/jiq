use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[cfg(debug_assertions)]
use log::debug;

use crate::app::{App, Focus};
use crate::results::results_events::handle_results_pane_key;

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

        KeyCode::Enter if key.modifiers.contains(KeyModifiers::SHIFT) => {
            if !app.search.is_confirmed() {
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

        KeyCode::Char('f')
            if key.modifiers.contains(KeyModifiers::CONTROL) && app.search.is_confirmed() =>
        {
            #[cfg(debug_assertions)]
            debug!("Search: re-entering edit mode via Ctrl+F");
            app.search.unconfirm();
            true
        }

        KeyCode::Char('/') if app.search.is_confirmed() => {
            #[cfg(debug_assertions)]
            debug!("Search: re-entering edit mode via /");
            app.search.unconfirm();
            true
        }

        // Delegate navigation keys to results pane when confirmed
        _ if app.search.is_confirmed() => {
            #[cfg(debug_assertions)]
            debug!(
                "Search: delegating key {:?} to results pane handler",
                key.code
            );
            handle_results_pane_key(app, key);
            true
        }

        _ => {
            app.search.search_textarea_mut().input(key);

            if let Some(content) = &app.query.last_successful_result_unformatted {
                app.search.update_matches(content);

                #[cfg(debug_assertions)]
                debug!(
                    "Search: query changed to '{}', found {} matches",
                    app.search.query(),
                    app.search.matches().len()
                );
            }

            if let Some(m) = app.search.current_match() {
                scroll_to_line(app, m.line);
            }

            true
        }
    }
}

pub fn open_search(app: &mut App) {
    #[cfg(debug_assertions)]
    debug!("Search: opened");

    app.search.open();
    app.focus = Focus::ResultsPane;
}

pub fn close_search(app: &mut App) {
    #[cfg(debug_assertions)]
    debug!("Search: closed (query was '{}')", app.search.query());

    app.search.close();
}

#[cfg(test)]
#[path = "search_events_tests.rs"]
mod search_events_tests;
