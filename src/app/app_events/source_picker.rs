//! Key routing while the source picker is on screen.
//!
//! Bindings:
//! * `Enter`              — confirm whichever option is highlighted.
//! * `↑` / `k` / `Left`   — move highlight to previous option.
//! * `BackTab`            — move highlight to previous option.
//! * `↓` / `j` / `Right`  — move highlight to next option.
//! * `Tab`                — move highlight to next option.
//! * `Esc`                — quit jiq (no JSON has been loaded).
//!
//! Anything else is swallowed so a stray key cannot fall through to
//! the main app handlers, which assume `app.query` is loaded.

use ratatui::crossterm::event::{KeyCode, KeyEvent};

use crate::app::App;
use crate::input::SourceChoice;

pub fn handle_key(app: &mut App, key: KeyEvent) -> bool {
    match key.code {
        KeyCode::Esc => {
            app.should_quit = true;
            true
        }
        KeyCode::Enter => {
            // Guard against confirming Clipboard with no usable bytes
            // — ignore the keystroke so the user must toggle to Paste
            // first. (Pre-selection already does this; this guard
            // catches the case where the user toggled to Clipboard
            // manually.)
            if can_confirm(app) {
                app.confirm_source_picker();
            }
            true
        }
        KeyCode::Up
        | KeyCode::Left
        | KeyCode::Char('k')
        | KeyCode::Char('h')
        | KeyCode::BackTab => {
            if let Some(state) = app.source_picker.as_mut() {
                state.select_previous();
            }
            true
        }
        KeyCode::Down | KeyCode::Right | KeyCode::Char('j') | KeyCode::Char('l') | KeyCode::Tab => {
            if let Some(state) = app.source_picker.as_mut() {
                state.select_next();
            }
            true
        }
        _ => true,
    }
}

/// True when the highlighted option can actually be confirmed:
/// Clipboard requires the cached bytes to be present, Paste is always
/// confirmable.
fn can_confirm(app: &App) -> bool {
    let Some(state) = app.source_picker.as_ref() else {
        return false;
    };
    match state.selection {
        SourceChoice::Clipboard => state.clipboard_cache.is_some(),
        SourceChoice::Paste => true,
    }
}
