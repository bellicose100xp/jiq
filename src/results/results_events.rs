use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::App;
use crate::clipboard;
use crate::editor::EditorMode;
use crate::help::HelpTab;
use crate::json_path::SiblingDir;
use crate::path_at_cursor_apply::{
    ApplyOutcome, PathSource, SiblingCursorOutcome, StepOutOutcome, UndoOutcome, apply_iterate,
    apply_keep_kv, apply_path, apply_sibling_cursor, apply_step_out, pop_undo,
};

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

        KeyCode::Char('0') => {
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

        KeyCode::Char('>') => {
            drill_in(app, PathSource::CursorRow);
        }

        KeyCode::Char('<') => {
            drill_back(app);
        }

        KeyCode::Char('*') => {
            iterate(app, PathSource::CursorRow);
        }

        KeyCode::Char('^') => {
            step_out(app);
        }

        KeyCode::Char('}') => {
            keep_kv(app, PathSource::CursorRow);
        }

        KeyCode::Char('[') => {
            sibling(app, PathSource::CursorRow, SiblingDir::Prev);
        }

        KeyCode::Char(']') => {
            sibling(app, PathSource::CursorRow, SiblingDir::Next);
        }

        _ => {}
    }
}

/// Map an `ApplyOutcome` to a notification. Used by all chords that go
/// through `apply_*` helpers (`>`, `*`, `}`) so the user-facing strings
/// stay in one place.
fn report_apply_outcome(app: &mut App, outcome: ApplyOutcome) {
    match outcome {
        ApplyOutcome::Applied(_) => {}
        ApplyOutcome::AtRoot => app.notification.show("Already at root"),
        ApplyOutcome::NoPath => app.notification.show("No path at cursor"),
        ApplyOutcome::NoArrayToIterate => app.notification.show("No array in path to iterate"),
        ApplyOutcome::NoKeyToWrap => app.notification.show("No key at cursor to wrap"),
    }
}

/// `>` — pipe-compose the cursor row's path onto the query and snapshot
/// for `<`. Public so the search-mode dispatcher can call it directly.
pub(crate) fn drill_in(app: &mut App, source: PathSource) {
    let outcome = apply_path(app, source);
    report_apply_outcome(app, outcome);
}

/// `*` — pipe-compose the cursor row's path with the rightmost array
/// index replaced by `[]`. Snapshots for `<`.
pub(crate) fn iterate(app: &mut App, source: PathSource) {
    let outcome = apply_iterate(app, source);
    report_apply_outcome(app, outcome);
}

/// `}` — wrap the cursor's leaf as a single-entry object literal,
/// `<parent> | {key}`. Snapshots like `>` and `*`.
pub(crate) fn keep_kv(app: &mut App, source: PathSource) {
    let outcome = apply_keep_kv(app, source);
    report_apply_outcome(app, outcome);
}

/// `[` / `]` — move the results-pane cursor to the previous / next
/// sibling row inside the parent container of `source`'s path. Pure
/// cursor movement: does NOT touch the query, the textarea, or the undo
/// ring. Wraps at container boundaries.
pub(crate) fn sibling(app: &mut App, source: PathSource, dir: SiblingDir) {
    match apply_sibling_cursor(app, source, dir) {
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

/// `^` — drop one step from the trailing path segment of the current
/// typed query. Ring-free, pipe-aware. Operates on the textarea
/// contents, not the cursor row, so repeated presses walk the query
/// independently of where the result-pane cursor is.
pub(crate) fn step_out(app: &mut App) {
    match apply_step_out(app) {
        StepOutOutcome::Stepped(_) => {}
        StepOutOutcome::AtRoot => app.notification.show("Already at root"),
        StepOutOutcome::Unparseable => app.notification.show("Can't step out of complex query"),
    }
}

/// `<` — pop the most recent drill-in snapshot, restoring the prior
/// query and viewport.
pub(crate) fn drill_back(app: &mut App) {
    match pop_undo(app) {
        UndoOutcome::Restored(_) => {}
        UndoOutcome::Empty => app.notification.show("Nothing to go back to"),
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
