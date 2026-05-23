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

    // Enter submits — but only at "rest" (Insert or Normal). When the
    // user is mid-operator (`d…`, `c…`, `f…`, `ci…`, etc.) Enter falls
    // through to the existing handlers which currently ignore it,
    // matching the query input's behavior.
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

    // Everything else: route through the same handler the query input
    // uses. `execute_query` inside the editor handlers is a no-op when
    // `app.query` is None (which it is during recovery), so dd/cc/dw
    // just edit the textarea without trying to run jq.
    app.handle_input_field_key(key);
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
