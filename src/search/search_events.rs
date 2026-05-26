use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::{App, Focus};
use crate::json_path::SiblingDir;
use crate::path_at_cursor_apply::{
    ApplyOutcome, PathSource, SiblingCursorOutcome, apply_iterate, apply_keep_kv,
    apply_sibling_cursor,
};
use crate::results::results_events::{drill_back, handle_results_pane_key, step_out};

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

        // Drill chords only fire after the search is confirmed. While
        // editing the query, these characters must reach the textarea so
        // the user can search for `>`, `<`, `*`, `^`, `}`, `[`, `]` as
        // literal text — common when grepping JSON that embeds operator
        // characters or HTML/XML.
        KeyCode::Char('>') if app.search.is_confirmed() => {
            drill_in_from_search(app);
            true
        }

        KeyCode::Char('<') if app.search.is_confirmed() => {
            drill_back(app);
            true
        }

        KeyCode::Char('*') if app.search.is_confirmed() => {
            iterate_from_search(app);
            true
        }

        KeyCode::Char('^') if app.search.is_confirmed() => {
            step_out(app);
            true
        }

        KeyCode::Char('}') if app.search.is_confirmed() => {
            keep_kv_from_search(app);
            true
        }

        KeyCode::Char('[') if app.search.is_confirmed() => {
            sibling_from_search(app, SiblingDir::Prev);
            true
        }

        KeyCode::Char(']') if app.search.is_confirmed() => {
            sibling_from_search(app, SiblingDir::Next);
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
    use crate::path_at_cursor_apply::apply_path;

    let match_row = match resolve_match_row(app) {
        Some(r) => r,
        None => return,
    };
    match apply_path(app, PathSource::Row(match_row)) {
        ApplyOutcome::Applied(_) => close_search(app),
        ApplyOutcome::AtRoot => app.notification.show("Already at root"),
        ApplyOutcome::NoPath => app.notification.show("No path at cursor"),
        ApplyOutcome::NoArrayToIterate => unreachable!("apply_path never returns this"),
        ApplyOutcome::NoKeyToWrap => unreachable!("apply_path never returns this"),
    }
}

/// `*` from search — iterate-splat the current match's path. Like
/// `drill_in_from_search`, closes the overlay on success so the user
/// lands directly on the splat-drilled view.
fn iterate_from_search(app: &mut App) {
    let match_row = match resolve_match_row(app) {
        Some(r) => r,
        None => return,
    };
    match apply_iterate(app, PathSource::Row(match_row)) {
        ApplyOutcome::Applied(_) => close_search(app),
        ApplyOutcome::AtRoot => app.notification.show("Already at root"),
        ApplyOutcome::NoPath => app.notification.show("No path at cursor"),
        ApplyOutcome::NoArrayToIterate => app.notification.show("No array in path to iterate"),
        ApplyOutcome::NoKeyToWrap => unreachable!("apply_iterate never returns this"),
    }
}

/// `}` from search — wrap the match row's leaf as `<parent> | {key}`.
/// Closes the overlay on success; surfaces a notification if the match
/// row has no key to wrap (e.g. it's an array element row).
fn keep_kv_from_search(app: &mut App) {
    let match_row = match resolve_match_row(app) {
        Some(r) => r,
        None => return,
    };
    match apply_keep_kv(app, PathSource::Row(match_row)) {
        ApplyOutcome::Applied(_) => close_search(app),
        ApplyOutcome::AtRoot => app.notification.show("Already at root"),
        ApplyOutcome::NoPath => app.notification.show("No path at cursor"),
        ApplyOutcome::NoArrayToIterate => unreachable!("apply_keep_kv never returns this"),
        ApplyOutcome::NoKeyToWrap => app.notification.show("No key at cursor to wrap"),
    }
}

/// `[` / `]` from search — move the results-pane cursor to the prev /
/// next sibling row of the match row's path. Pure cursor movement: does
/// NOT close the search overlay or modify the query.
fn sibling_from_search(app: &mut App, dir: SiblingDir) {
    let match_row = match resolve_match_row(app) {
        Some(r) => r,
        None => return,
    };
    match apply_sibling_cursor(app, PathSource::Row(match_row), dir) {
        SiblingCursorOutcome::Moved(line) => {
            let total = app.results_line_count_u32();
            app.results_cursor.update_total_lines(total);
            app.results_cursor.move_to_line(line);
            app.results_scroll
                .ensure_cursor_visible(app.results_cursor.cursor_line());
        }
        SiblingCursorOutcome::AtRoot => app.notification.show("Already at root"),
        SiblingCursorOutcome::NoPath => app.notification.show("No path at cursor"),
        SiblingCursorOutcome::NoSibling => {
            app.notification.show("No sibling to navigate to");
        }
    }
}

/// Find the line of the current match, surfacing the "no match" notification
/// when there isn't one. Returns `None` when the caller should bail.
fn resolve_match_row(app: &mut App) -> Option<u32> {
    match app.search.current_match() {
        Some(m) => Some(m.line),
        None => {
            app.notification.show("No match to navigate to");
            None
        }
    }
}

#[cfg(test)]
#[path = "search_events_tests.rs"]
mod search_events_tests;
