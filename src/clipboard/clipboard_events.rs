use crate::app::{App, Focus};
use crate::config::ClipboardBackend;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::backend::copy_to_clipboard;

pub fn handle_clipboard_key(app: &mut App, key: KeyEvent, backend: ClipboardBackend) -> bool {
    if key.code == KeyCode::Char('y') && key.modifiers.contains(KeyModifiers::CONTROL) {
        return copy_focused_content(app, backend);
    }

    false
}

pub fn handle_yank_key(app: &mut App, backend: ClipboardBackend) -> bool {
    copy_focused_content(app, backend)
}

fn copy_focused_content(app: &mut App, backend: ClipboardBackend) -> bool {
    match app.focus {
        Focus::InputField => copy_query(app, backend),
        Focus::ResultsPane => copy_result(app, backend),
    }
}

fn copy_query(app: &mut App, backend: ClipboardBackend) -> bool {
    let query = app.query();

    if query.is_empty() {
        return false;
    }

    if copy_to_clipboard(query, backend).is_ok() {
        app.notification.show("Copied query!");
        true
    } else {
        false
    }
}

fn copy_result(app: &mut App, backend: ClipboardBackend) -> bool {
    // Only copy if query state is available
    let query_state = match &app.query {
        Some(q) => q,
        None => return false,
    };

    // Copy what's displayed: last_successful_result_unformatted
    let result = match &query_state.last_successful_result_unformatted {
        Some(text) => text.as_ref().to_string(),
        None => return false,
    };

    if result.is_empty() {
        return false;
    }

    if copy_to_clipboard(&result, backend).is_ok() {
        app.notification.show("Copied result!");
        true
    } else {
        false
    }
}

#[cfg(test)]
pub fn strip_ansi_codes(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\x1b' {
            // Handle escape sequence
            match chars.peek() {
                Some(&'[') => {
                    // CSI sequence: \x1b[...letter
                    chars.next(); // consume '['
                    // Skip until we hit a letter (end of sequence)
                    while let Some(&next) = chars.peek() {
                        chars.next();
                        if next.is_ascii_alphabetic() {
                            break;
                        }
                    }
                }
                Some(&']') => {
                    // OSC sequence: \x1b]...(\x07|\x1b\\)
                    chars.next(); // consume ']'
                    // Skip until we hit BEL (\x07) or ST (\x1b\\)
                    while let Some(&next) = chars.peek() {
                        if next == '\x07' {
                            chars.next();
                            break;
                        }
                        if next == '\x1b' {
                            chars.next();
                            // Check for backslash (ST terminator)
                            if chars.peek() == Some(&'\\') {
                                chars.next();
                            }
                            break;
                        }
                        chars.next();
                    }
                }
                Some(_) => {
                    // Simple escape sequence: skip the next character
                    chars.next();
                }
                None => {
                    // Lone escape at end of string - skip it
                }
            }
        } else {
            result.push(c);
        }
    }

    result
}

#[cfg(test)]
#[path = "clipboard_events_tests.rs"]
mod clipboard_events_tests;
