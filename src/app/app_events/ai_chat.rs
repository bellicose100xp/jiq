//! Key handling while the AI popup's chat input has focus.
//!
//! The chat input behaves like a single-line text field: printable keys and
//! editing chords go to the buffer, Enter sends the question, Esc hands
//! focus back to the query box. Suggestion selection (Alt+1-5, Alt+arrows)
//! keeps working so an answer's queries can be applied without leaving the
//! chat. A short list of app-wide chords is left unhandled so the global
//! handler still sees them.

use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::app_state::App;
use crate::scroll::Scrollable;

/// Handle a key while the chat input is focused. Returns false for the
/// chords that must reach the global handler (Ctrl+A, Ctrl+G, Ctrl+T,
/// Shift+Tab, Ctrl+Q, Shift+Enter, Alt+Enter).
pub fn handle_ai_chat_key(app: &mut App, key: KeyEvent) -> bool {
    if !app.ai.visible {
        app.focus_input_field();
        return false;
    }

    if let Some(query) = &mut app.query
        && crate::ai::ai_events::handle_suggestion_selection(
            key,
            &mut app.ai,
            &mut app.input,
            query,
            &mut app.autocomplete,
        )
    {
        return true;
    }

    if passes_through_to_global(key) {
        return false;
    }

    let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
    match key.code {
        KeyCode::Esc => {
            app.focus_input_field();
            true
        }
        KeyCode::Enter => {
            app.send_ai_chat_question();
            true
        }
        KeyCode::Char('l') if ctrl => {
            app.ai.clear_conversation();
            true
        }
        KeyCode::Up => {
            app.ai.selection.scroll_view_up(1);
            true
        }
        KeyCode::Down => {
            app.ai.selection.scroll_view_down(1);
            true
        }
        KeyCode::PageUp => {
            let page = app.ai.selection.viewport_size().max(1);
            app.ai.selection.scroll_view_up(page);
            true
        }
        KeyCode::PageDown => {
            let page = app.ai.selection.viewport_size().max(1);
            app.ai.selection.scroll_view_down(page);
            true
        }
        KeyCode::Tab => true,
        _ => {
            app.ai.chat_input.input(key);
            true
        }
    }
}

/// App-wide chords that keep their meaning while the chat input is focused.
fn passes_through_to_global(key: KeyEvent) -> bool {
    let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
    let shift = key.modifiers.contains(KeyModifiers::SHIFT);
    let alt = key.modifiers.contains(KeyModifiers::ALT);
    match key.code {
        KeyCode::Char('a') | KeyCode::Char('g') | KeyCode::Char('t') | KeyCode::Char('q')
            if ctrl =>
        {
            true
        }
        KeyCode::BackTab => true,
        KeyCode::Enter if shift || alt => true,
        _ => false,
    }
}

/// Insert pasted text into the chat input, collapsing newlines so the
/// single-line buffer stays single-line.
pub fn paste_into_chat(app: &mut App, text: &str) {
    let flattened = text.replace(['\r', '\n'], " ");
    app.ai.chat_input.insert_str(&flattened);
}

#[cfg(test)]
#[path = "ai_chat_tests.rs"]
mod ai_chat_tests;
