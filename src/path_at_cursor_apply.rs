//! Apply / undo helpers for the `>` (drill in) and `<` (back) chords on
//! the results pane.
//!
//! `>` resolves the jq path of the row pointed at by the chosen `PathSource`,
//! pipe-composes it onto the current input, and pushes a snapshot to
//! [`QueryUndoRing`] so a subsequent `<` can restore the prior state. `<`
//! pops the most recent snapshot, restoring the input verbatim — except
//! when the user manually edited the textarea between drill-ins, in which
//! case the ring is invalidated.
//!
//! These helpers ONLY touch the input textarea and the undo ring. Query
//! execution, results-pane refresh, AI re-prompt, etc. all flow through
//! the existing keystroke pipeline: rewriting the textarea schedules the
//! debouncer (just like a typed character), and the main event loop fires
//! the async query when the debouncer's quiet period elapses. That keeps
//! `>` / `<` from re-implementing any execution path and means the user
//! sees identical behavior whether they typed the new query themselves or
//! synthesized it via drill-in.

use crate::app::App;
use crate::query_undo::{PopOutcome, ViewportState};

/// Where to source the row whose path we apply on `>`.
#[derive(Debug, Clone, Copy)]
pub enum PathSource {
    /// Use the cursor row of the results pane.
    CursorRow,
    /// Use a specific row (e.g. the row of the current search match).
    Row(u32),
}

/// Outcome of `apply_path`. Caller maps to a notification.
#[derive(Debug, PartialEq, Eq)]
pub enum ApplyOutcome {
    /// Path applied; new query string is included for downstream wiring
    /// (e.g. status bar, logging) but is also already written to the input.
    Applied(String),
    /// The current row resolves to no path. The input was not changed.
    NoPath,
    /// The current row resolves to the root identity (`.`). Composing
    /// `<query> | .` is a no-op after normalization, so we skip it.
    AtRoot,
}

/// Outcome of `pop_undo`. Caller maps to a notification.
#[derive(Debug, PartialEq, Eq)]
pub enum UndoOutcome {
    /// Snapshot restored; new (== prior) query string is included.
    Restored(String),
    /// Ring was empty.
    Empty,
    /// Top snapshot was stale because the textarea was edited manually
    /// after the last drill-in. The ring has been cleared.
    Invalidated,
}

/// Resolve `source` to a jq path string and pipe-compose it onto the
/// current input. On success, snapshot the prior state into the ring and
/// schedule a debounced async re-execution.
pub fn apply_path(app: &mut App, source: PathSource) -> ApplyOutcome {
    let path = match resolve_path(app, source) {
        Some(p) => p,
        None => return ApplyOutcome::NoPath,
    };
    if path == "." {
        return ApplyOutcome::AtRoot;
    }

    let prev = app.input.query().to_string();
    let trimmed = prev.trim();
    let composed = if trimmed.is_empty() || trimmed == "." {
        path
    } else {
        format!("{} | {}", trimmed, path)
    };
    let viewport = capture_viewport(app);

    rewrite_input_like_keystroke(app, &composed);
    app.query_undo.push(prev, composed.clone(), viewport);
    ApplyOutcome::Applied(composed)
}

/// Pop the most recent `>`-snapshot. Restores the prior query verbatim
/// and *defers* the viewport restore to the moment the worker delivers
/// the prior result — see [`App::pending_viewport_restore`]. Restoring
/// synchronously would clamp the cursor against the drilled result's
/// line layout, which is still in place until the async query completes.
pub fn pop_undo(app: &mut App) -> UndoOutcome {
    let current = app.input.query().to_string();
    match app.query_undo.pop_if_matches(&current) {
        PopOutcome::Restored { query, viewport } => {
            rewrite_input_like_keystroke(app, &query);
            app.pending_viewport_restore = Some(viewport);
            UndoOutcome::Restored(query)
        }
        PopOutcome::Empty => UndoOutcome::Empty,
        PopOutcome::Invalidated => UndoOutcome::Invalidated,
    }
}

fn capture_viewport(app: &App) -> ViewportState {
    ViewportState {
        cursor_row: app.results_cursor.cursor_line(),
        scroll_offset: app.results_scroll.offset,
        h_offset: app.results_scroll.h_offset,
    }
}

/// Apply a deferred viewport restore against the now-current result.
/// The cursor row is clamped against the new total line count so a stale
/// snapshot (e.g. line numbers that no longer exist after the result was
/// rewritten by a concurrent edit) lands on the last visible row instead
/// of off-screen.
pub fn apply_pending_viewport_restore(app: &mut App) {
    let viewport = match app.pending_viewport_restore.take() {
        Some(v) => v,
        None => return,
    };
    let total = app.results_line_count_u32();
    app.results_cursor.update_total_lines(total);
    app.results_cursor.move_to_line(viewport.cursor_row);
    app.results_scroll.offset = viewport.scroll_offset;
    app.results_scroll.h_offset = viewport.h_offset;
}

fn resolve_path(app: &mut App, source: PathSource) -> Option<String> {
    match source {
        PathSource::CursorRow => app.current_cursor_path().map(|p| p.to_jq()),
        PathSource::Row(row) => {
            let parsed = app.query.as_ref()?.last_successful_result_parsed.clone()?;
            // Mirror the gating in `App::current_cursor_path`: skip when
            // the result is empty/erroring/synthetic-merge.
            let qs = app.query.as_ref()?;
            if qs.is_empty_result || qs.result.is_err() || qs.is_synthetic_merge {
                return None;
            }
            crate::json_path::path_at_line(&parsed, row as usize).map(|p| p.to_jq())
        }
    }
}

/// Replace the textarea contents and run the same side-effects the editor
/// runs after a content-changing keystroke (see `editor_events::handle_key`).
/// The debouncer pickup re-executes the query on the next tick, exactly as
/// if the user had typed the new query by hand.
fn rewrite_input_like_keystroke(app: &mut App, text: &str) {
    app.input.textarea.delete_line_by_head();
    app.input.textarea.delete_line_by_end();
    app.input.textarea.insert_str(text);

    app.input.reset_manual_scroll();
    app.history.reset_cycling();
    app.debouncer.schedule_execution();
    app.results_scroll.reset();
    app.results_cursor.reset();
    app.error_overlay_visible = false;
    app.input
        .brace_tracker
        .rebuild(app.input.textarea.lines()[0].as_ref());
}

#[cfg(test)]
#[path = "path_at_cursor_apply_tests.rs"]
mod path_at_cursor_apply_tests;
