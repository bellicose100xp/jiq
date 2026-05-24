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
use crate::json_path::{JsonPath, JsonPathStep, format_field_name, is_simple_jq_identifier};
use crate::query_undo::ViewportState;

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
    /// Iterate-only: the cursor row's path has no `Index` step to splat.
    /// Treated as a soft no-op with its own user-facing notification.
    NoArrayToIterate,
    /// Keep-kv (`}`): the cursor row's path doesn't end in a Key, so
    /// there's no field name to wrap. Soft no-op with notification.
    NoKeyToWrap,
}

/// Outcome of `pop_undo`. Caller maps to a notification.
#[derive(Debug, PartialEq, Eq)]
pub enum UndoOutcome {
    /// Snapshot restored; new (== prior) query string is included.
    Restored(String),
    /// Ring was empty.
    Empty,
}

/// Outcome of `apply_step_out`. Caller maps to a notification.
#[derive(Debug, PartialEq, Eq)]
pub enum StepOutOutcome {
    /// One step popped from the trailing path segment of the current query.
    Stepped(String),
    /// Current query is `.` (or empty) — nothing to step out of.
    AtRoot,
    /// Trailing pipe segment isn't a recognizable jq path expression that
    /// our own renderer would emit. Don't touch the input.
    Unparseable,
}

/// Resolve `source` to a jq path string and pipe-compose it onto the
/// current input. On success, snapshot the prior state into the ring and
/// schedule a debounced async re-execution.
pub fn apply_path(app: &mut App, source: PathSource) -> ApplyOutcome {
    let path = match resolve_path(app, source) {
        Some(p) => p,
        None => return ApplyOutcome::NoPath,
    };
    compose_and_apply(app, path.to_jq(), /* push_to_ring */ true)
}

/// Resolve `source`'s path, replace the rightmost `Index` step with a
/// splat (`[]`), and pipe-compose. Snapshots prior state to the ring like
/// `apply_path`. Used by the `*` chord to "iterate over all elements at
/// the nearest array level the cursor is inside."
pub fn apply_iterate(app: &mut App, source: PathSource) -> ApplyOutcome {
    let mut path = match resolve_path(app, source) {
        Some(p) => p,
        None => return ApplyOutcome::NoPath,
    };
    if !path.splat_nearest_index() {
        return ApplyOutcome::NoArrayToIterate;
    }
    compose_and_apply(app, path.to_jq(), /* push_to_ring */ true)
}

/// `}` — wrap the cursor row's leaf as a single-entry object, parented
/// to the row's container. So `.users[0].name` becomes
/// `.users[0] | {name}` (simple key) or `.users[0] | {"foo-bar": .["foo-bar"]}`
/// (key requiring bracket notation). Pushes to the undo ring like `>` and
/// `*`, since the produced query is a meaningful drill the user may want
/// `<` to reverse.
///
/// Returns [`ApplyOutcome::NoKeyToWrap`] when the cursor row's path
/// doesn't end in a [`JsonPathStep::Key`] — array-element rows and the
/// root row both lack a key name to use in the object literal.
pub fn apply_keep_kv(app: &mut App, source: PathSource) -> ApplyOutcome {
    let mut path = match resolve_path(app, source) {
        Some(p) => p,
        None => return ApplyOutcome::NoPath,
    };
    let key = match path.steps().last() {
        Some(JsonPathStep::Key(k)) => k.clone(),
        _ => return ApplyOutcome::NoKeyToWrap,
    };
    path.pop();
    let parent = path.to_jq();
    let wrap = if is_simple_jq_identifier(&key) {
        format!("{{{}}}", key)
    } else {
        // Use `format_field_name` to render the bracket access for the
        // value side, matching how the rest of the renderer escapes
        // non-simple keys. The key on the LHS is JSON-escaped via
        // `serde_json::to_string`.
        let json_key = serde_json::to_string(&key).unwrap_or_else(|_| format!("\"{}\"", key));
        let access = format_field_name(".", &key);
        format!("{{{}: {}}}", json_key, access)
    };
    // When the parent is the root (a top-level key), drop the redundant
    // `. |` so the suffix reads cleanly as just `{key}` rather than
    // `. | {key}`.
    let suffix = if parent == "." {
        wrap
    } else {
        format!("{} | {}", parent, wrap)
    };
    compose_and_apply(app, suffix, /* push_to_ring */ true)
}

/// Shared compose+rewrite logic. Returns `AtRoot` when the path collapses
/// to `.` (composing `<query> | .` is a no-op); otherwise rewrites the
/// textarea, optionally pushes to the undo ring, and reports `Applied`.
fn compose_and_apply(app: &mut App, path: String, push_to_ring: bool) -> ApplyOutcome {
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
    if push_to_ring {
        app.query_undo.push(prev, viewport);
    }
    ApplyOutcome::Applied(composed)
}

/// `[` — drop one step from the trailing path segment of the current
/// query. Ring-free: doesn't push to the undo ring. Operates on the
/// textarea contents, not the cursor row, so repeated presses walk the
/// query independently of where the result-pane cursor is.
///
/// Pipe-aware: if the input contains pipes, only the segment after the
/// last `|` is treated as the path. Composing `>`-then-`[` cleanly
/// reverses the drill since `>` produces `<prefix> | <path>`.
pub fn apply_step_out(app: &mut App) -> StepOutOutcome {
    use crate::json_path::parse_jq_path;

    let current = app.input.query().to_string();
    let trimmed = current.trim();
    if trimmed.is_empty() || trimmed == "." {
        return StepOutOutcome::AtRoot;
    }

    let (prefix, tail) = match trimmed.rfind('|') {
        Some(p) => (&trimmed[..p], trimmed[p + 1..].trim()),
        None => ("", trimmed),
    };
    let mut path = match parse_jq_path(tail) {
        Some(p) => p,
        None => return StepOutOutcome::Unparseable,
    };

    if path.is_empty() {
        // Tail was already root. Drop the trailing pipe segment entirely
        // (and the pipe itself) so we walk into the prefix.
        return rewrite_after_step_out(app, prefix.trim());
    }

    path.pop();
    let new_tail = path.to_jq();

    let new_query = if prefix.is_empty() {
        new_tail
    } else if new_tail == "." {
        // The tail collapsed to root after pop. Drop the trailing
        // ` | .` so the query stays clean instead of ending in `| .`.
        prefix.trim_end().to_string()
    } else {
        format!("{} | {}", prefix.trim_end(), new_tail)
    };
    rewrite_after_step_out(app, &new_query)
}

/// Shared rewrite path for `[`: writes the new query and returns the
/// outcome variant.
fn rewrite_after_step_out(app: &mut App, new_query: &str) -> StepOutOutcome {
    let trimmed = new_query.trim();
    if trimmed.is_empty() || trimmed == "." {
        rewrite_input_like_keystroke(app, ".");
        return StepOutOutcome::Stepped(".".to_string());
    }
    rewrite_input_like_keystroke(app, trimmed);
    StepOutOutcome::Stepped(trimmed.to_string())
}

/// Pop the most recent `>`-snapshot. Restores the prior query verbatim
/// and *defers* the viewport restore to the moment the worker delivers
/// the prior result — see [`App::pending_viewport_restore`]. Restoring
/// synchronously would clamp the cursor against the drilled result's
/// line layout, which is still in place until the async query completes.
///
/// Always pops regardless of whether the user manually edited the
/// textarea between drill-ins. The trade-off is a simpler mental model
/// (`<` always undoes a `>`) over the lossless-edit guarantee — the
/// user's intermediate edits are discarded when they press `<`.
pub fn pop_undo(app: &mut App) -> UndoOutcome {
    match app.query_undo.pop() {
        Some((query, viewport)) => {
            rewrite_input_like_keystroke(app, &query);
            app.pending_viewport_restore = Some(viewport);
            UndoOutcome::Restored(query)
        }
        None => UndoOutcome::Empty,
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

fn resolve_path(app: &mut App, source: PathSource) -> Option<JsonPath> {
    match source {
        PathSource::CursorRow => app.current_cursor_path(),
        PathSource::Row(row) => {
            let parsed = app.query.as_ref()?.last_successful_result_parsed.clone()?;
            // Mirror the gating in `App::current_cursor_path`: skip when
            // the result is empty/erroring/synthetic-merge.
            let qs = app.query.as_ref()?;
            if qs.is_empty_result || qs.result.is_err() || qs.is_synthetic_merge {
                return None;
            }
            crate::json_path::path_at_line(&parsed, row as usize)
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
