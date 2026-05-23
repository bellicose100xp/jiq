//! Key routing while paste-recovery is active.
//!
//! We deliberately reuse `app.input.textarea` and `app.input.editor_mode`
//! during recovery, then forward keys through the existing input
//! handlers — that's what gives recovery every VIM binding the query
//! input has (operators, char-search, text objects, dd/cc/D/C/dw/ci"/...,
//! ;, ,, undo/redo) for free, with no duplicated logic.
//!
//! The only override is `Enter` (no modifier): in any non-Operator-style
//! mode it submits the textarea contents as JSON. Inside an in-progress
//! operator/char-search/text-object motion we don't intercept Enter
//! because the existing handlers don't bind it anyway and the user
//! is mid-command.

use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tui_textarea::CursorMove;

use crate::app::App;
use crate::editor::EditorMode;

/// Handle a key while paste recovery is active. Returns true after
/// handling — paste recovery consumes everything so `handle_events`
/// doesn't need a fallback.
pub fn handle_key(app: &mut App, key: KeyEvent) -> bool {
    // Ctrl+X: nuke the paste content. Useful when the user pasted the
    // wrong thing and would rather start over than back-out edit.
    if key.code == KeyCode::Char('x') && key.modifiers.contains(KeyModifiers::CONTROL) {
        clear_input_textarea(app);
        return true;
    }

    // `Ctrl+J` == LF byte (0x0A). When bracketed paste isn't forwarded
    // by the terminal/multiplexer (common on Cloud Desktop, plain tmux,
    // mosh, and many SSH setups), pasted multi-line content arrives as
    // a stream of `Char` events with `\n` decoded as `Ctrl+J`. The
    // shared input handler delegates to `tui-textarea::input()`, whose
    // default `Ctrl+J` mapping is "delete line by head" — that's what
    // was wiping each line back to column 0 on every paste-newline.
    //
    // Always insert a real newline here. Distinct from `KeyCode::Enter`
    // (the deliberate user keystroke), which keeps its submit semantics.
    // Logged because this is the diagnostic signal for that class of
    // terminal bug.
    if key.code == KeyCode::Char('j') && key.modifiers.contains(KeyModifiers::CONTROL) {
        log::debug!("paste-recovery: Ctrl+J (paste-newline) -> insert_newline");
        app.input.textarea.insert_newline();
        return true;
    }

    // Enter at "rest" (Insert or Normal) submits.
    if key.code == KeyCode::Enter
        && !key.modifiers.contains(KeyModifiers::CONTROL)
        && !key.modifiers.contains(KeyModifiers::SHIFT)
        && !key.modifiers.contains(KeyModifiers::ALT)
        && matches!(
            app.input.editor_mode,
            EditorMode::Insert | EditorMode::Normal
        )
    {
        try_submit(app);
        return true;
    }

    // The query input is single-line, so the shared handler doesn't bind
    // any vertical-motion keys (j/k/↑/↓/g/G). Recovery is multi-line, so
    // we layer those motions on *before* delegation. We deliberately
    // don't expose them on the query input — that block stays unchanged.
    if handle_recovery_only_motion(app, key) {
        return true;
    }

    // The query input maps `Up` (Insert mode) to "open history popup".
    // Recovery has no history to scroll through, so swallow Up/Down in
    // Insert mode to avoid triggering a nonsensical popup. (j/k/↑/↓ in
    // Normal mode are already handled above.)
    if app.input.editor_mode == EditorMode::Insert
        && matches!(key.code, KeyCode::Up | KeyCode::Down)
    {
        let cm = if key.code == KeyCode::Up {
            CursorMove::Up
        } else {
            CursorMove::Down
        };
        app.input.textarea.move_cursor(cm);
        return true;
    }

    // Everything else: route through the same handler the query input
    // uses. `execute_query` inside the editor handlers is a no-op when
    // `app.query` is None (which it is during recovery), so dd/cc/dw
    // just edit the textarea without trying to run jq.
    app.handle_input_field_key(key);
    true
}

/// Multi-line motions that the single-line query input doesn't expose.
/// Returns true if the key was a recovery-only motion and was handled.
fn handle_recovery_only_motion(app: &mut App, key: KeyEvent) -> bool {
    if app.input.editor_mode != EditorMode::Normal {
        return false;
    }
    if key
        .modifiers
        .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT)
    {
        return false;
    }
    let motion = match key.code {
        KeyCode::Char('j') => CursorMove::Down,
        KeyCode::Char('k') => CursorMove::Up,
        KeyCode::Char('g') => CursorMove::Top,
        KeyCode::Char('G') => CursorMove::Bottom,
        _ => return false,
    };
    app.input.textarea.move_cursor(motion);
    true
}

fn try_submit(app: &mut App) {
    let raw = app.input.textarea.lines().join("\n");
    let result = match &mut app.paste_recovery {
        Some(r) => r.try_submit(&raw),
        None => return,
    };
    match result {
        Ok(json) => {
            // Wipe the paste content so the user lands on an empty query
            // input as if jiq had launched normally.
            clear_input_textarea(app);
            app.accept_paste_recovery_json(json);
        }
        Err(msg) => {
            // Red toast nudges the user's eye to the top "No JSON
            // loaded" block, where the same message has been written.
            app.notification.show_error(&msg);
        }
    }
}

fn clear_input_textarea(app: &mut App) {
    use tui_textarea::TextArea;
    app.input.textarea = TextArea::default();
    app.input.editor_mode = EditorMode::Insert;
}
