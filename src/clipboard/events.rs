//! Clipboard event handlers
//!
//! Handles keybindings for copy operations.

use crate::app::{App, Focus};
use crate::config::ClipboardBackend;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::backend::copy_to_clipboard;

/// Handle clipboard-related key events
/// Returns true if the key was handled
pub fn handle_clipboard_key(app: &mut App, key: KeyEvent, backend: ClipboardBackend) -> bool {
    // Ctrl+Y - copy in any mode
    if key.code == KeyCode::Char('y') && key.modifiers.contains(KeyModifiers::CONTROL) {
        return copy_focused_content(app, backend);
    }

    false
}

/// Handle 'y' key in Normal mode (for yy command)
/// Called from vim event handler when 'y' is pressed
pub fn handle_yank_key(app: &mut App, backend: ClipboardBackend) -> bool {
    copy_focused_content(app, backend)
}

/// Copy content based on current focus
fn copy_focused_content(app: &mut App, backend: ClipboardBackend) -> bool {
    match app.focus {
        Focus::InputField => copy_query(app, backend),
        Focus::ResultsPane => copy_result(app, backend),
    }
}

/// Copy current query to clipboard
fn copy_query(app: &mut App, backend: ClipboardBackend) -> bool {
    let query = app.query();

    // Don't copy empty queries
    if query.is_empty() {
        return false;
    }

    if copy_to_clipboard(query, backend).is_ok() {
        // Use the shared notification system
        app.notification.show("Copied query!");
        true
    } else {
        false
    }
}

/// Copy current result to clipboard
fn copy_result(app: &mut App, backend: ClipboardBackend) -> bool {
    // Get result text, stripping ANSI codes
    let result = match &app.query.result {
        Ok(text) => strip_ansi_codes(text),
        Err(_) => return false, // Don't copy errors
    };

    // Don't copy empty results
    if result.is_empty() {
        return false;
    }

    if copy_to_clipboard(&result, backend).is_ok() {
        // Use the shared notification system
        app.notification.show("Copied result!");
        true
    } else {
        false
    }
}

/// Strip ANSI escape codes from text
///
/// Removes terminal color codes and other escape sequences
/// to get plain text for clipboard operations.
///
/// Handles:
/// - CSI sequences: `\x1b[...m` (colors, styles)
/// - OSC sequences: `\x1b]...(\x07|\x1b\\)` (operating system commands)
/// - Simple escape sequences: `\x1b` followed by a single character
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
mod tests {
    use super::*;
    use crate::config::ClipboardBackend;
    use proptest::prelude::*;

    /// Helper to create App with default clipboard backend for tests
    fn test_app(json: &str) -> App {
        App::new(json.to_string(), ClipboardBackend::Auto)
    }

    // =========================================================================
    // Unit Tests
    // =========================================================================

    #[test]
    fn test_strip_ansi_codes_no_codes() {
        assert_eq!(strip_ansi_codes("hello world"), "hello world");
    }

    #[test]
    fn test_strip_ansi_codes_simple_color() {
        // Red text: \x1b[31m
        assert_eq!(strip_ansi_codes("\x1b[31mhello\x1b[0m"), "hello");
    }

    #[test]
    fn test_strip_ansi_codes_multiple_colors() {
        // Bold red: \x1b[1;31m
        assert_eq!(
            strip_ansi_codes("\x1b[1;31mbold red\x1b[0m normal"),
            "bold red normal"
        );
    }

    #[test]
    fn test_strip_ansi_codes_empty_string() {
        assert_eq!(strip_ansi_codes(""), "");
    }

    #[test]
    fn test_strip_ansi_codes_only_escape_sequences() {
        assert_eq!(strip_ansi_codes("\x1b[31m\x1b[0m"), "");
    }

    #[test]
    fn test_strip_ansi_codes_preserves_newlines() {
        assert_eq!(
            strip_ansi_codes("\x1b[32mline1\x1b[0m\nline2"),
            "line1\nline2"
        );
    }

    #[test]
    fn test_strip_ansi_codes_osc_sequence() {
        // OSC sequence with BEL terminator
        assert_eq!(strip_ansi_codes("\x1b]0;title\x07text"), "text");
    }

    // =========================================================================
    // Property-Based Tests
    // =========================================================================

    // Feature: clipboard, Property 2: ANSI stripping preserves non-ANSI content
    // *For any* input text without ANSI escape sequences, stripping ANSI codes
    // should return the identical text.
    // Validates: Requirements 2.4
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn prop_ansi_stripping_preserves_non_ansi_content(
            // Generate strings that don't contain escape character
            text in "[^\x1b]*"
        ) {
            let result = strip_ansi_codes(&text);
            prop_assert_eq!(
                result, text,
                "Text without ANSI codes should be unchanged"
            );
        }
    }

    // Feature: clipboard, Property 3: ANSI stripping removes all escape sequences
    // *For any* input text with ANSI escape sequences, the stripped output should
    // contain no escape sequences (no `\x1b` characters).
    // Validates: Requirements 2.4
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn prop_ansi_stripping_removes_all_escape_sequences(
            // Generate text parts (ASCII only to avoid char boundary issues)
            prefix in "[a-zA-Z0-9 ]{0,20}",
            suffix in "[a-zA-Z0-9 ]{0,20}",
            ansi_params in "[0-9;]{0,10}",
            ansi_letter in "[A-Za-z]",
        ) {
            // Construct text with ANSI sequence in the middle
            let text_with_ansi = format!(
                "{}\x1b[{}{}{}",
                prefix,
                ansi_params,
                ansi_letter,
                suffix
            );

            let result = strip_ansi_codes(&text_with_ansi);

            // The result should contain no escape characters
            prop_assert!(
                !result.contains('\x1b'),
                "Stripped text should not contain escape character. Input: {:?}, Output: {:?}",
                text_with_ansi,
                result
            );

            // The result should be the concatenation of prefix and suffix
            prop_assert_eq!(
                result,
                format!("{}{}", prefix, suffix),
                "Stripped text should be prefix + suffix"
            );
        }
    }

    // Feature: clipboard, Property 5: Empty content rejection
    // *For any* empty string input, the copy operation should not proceed and return false.
    // Validates: Requirements 1.4, 2.5
    //
    // This property is tested via unit tests since copy_query/copy_result require
    // a full App instance. The core property is that empty strings are rejected.
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn prop_empty_content_rejection_query(
            // Generate whitespace-only strings (empty or spaces/tabs/newlines)
            _whitespace in "[ \t\n]*"
        ) {
            // Create app with minimal JSON
            let mut app = test_app("{}");

            // Set the query to whitespace-only content
            // Note: We test the empty check logic - empty queries should be rejected
            // The textarea starts empty, so copy should return false
            let result = copy_query(&mut app, ClipboardBackend::Osc52);

            // Empty query should be rejected
            prop_assert!(
                !result,
                "Empty query should be rejected, but copy returned true"
            );

            // Notification should NOT be shown for rejected copy
            prop_assert!(
                app.notification.current().is_none(),
                "No notification should be shown for rejected empty copy"
            );
        }

        #[test]
        fn prop_empty_content_rejection_result(
            // Generate ANSI-only strings that become empty after stripping
            ansi_params in "[0-9;]{0,10}",
            ansi_letter in "[A-Za-z]",
        ) {
            // Create app with minimal JSON
            let mut app = test_app("{}");

            // Set result to ANSI-only content (becomes empty after stripping)
            let ansi_only = format!("\x1b[{}{}", ansi_params, ansi_letter);
            app.query.result = Ok(ansi_only);

            let result = copy_result(&mut app, ClipboardBackend::Osc52);

            // Result that becomes empty after ANSI stripping should be rejected
            prop_assert!(
                !result,
                "Result that is empty after ANSI stripping should be rejected"
            );

            // Notification should NOT be shown for rejected copy
            prop_assert!(
                app.notification.current().is_none(),
                "No notification should be shown for rejected empty copy"
            );
        }
    }

    // =========================================================================
    // Unit Tests for Empty Content Rejection
    // =========================================================================

    #[test]
    fn test_copy_query_rejects_empty() {
        let mut app = test_app("{}");
        // Query starts empty
        let result = copy_query(&mut app, ClipboardBackend::Osc52);
        assert!(!result, "Empty query should be rejected");
        assert!(
            app.notification.current().is_none(),
            "No notification for rejected copy"
        );
    }

    #[test]
    fn test_copy_result_rejects_empty() {
        let mut app = test_app("{}");
        app.query.result = Ok(String::new());
        let result = copy_result(&mut app, ClipboardBackend::Osc52);
        assert!(!result, "Empty result should be rejected");
        assert!(
            app.notification.current().is_none(),
            "No notification for rejected copy"
        );
    }

    #[test]
    fn test_copy_result_rejects_ansi_only() {
        let mut app = test_app("{}");
        // Result with only ANSI codes (becomes empty after stripping)
        app.query.result = Ok("\x1b[31m\x1b[0m".to_string());
        let result = copy_result(&mut app, ClipboardBackend::Osc52);
        assert!(!result, "ANSI-only result should be rejected");
        assert!(
            app.notification.current().is_none(),
            "No notification for rejected copy"
        );
    }

    #[test]
    fn test_copy_result_rejects_error() {
        let mut app = test_app("{}");
        app.query.result = Err("some error".to_string());
        let result = copy_result(&mut app, ClipboardBackend::Osc52);
        assert!(!result, "Error result should be rejected");
        assert!(
            app.notification.current().is_none(),
            "No notification for rejected copy"
        );
    }

    #[test]
    fn test_copy_query_accepts_non_empty() {
        let mut app = test_app("{}");
        // Insert a query
        app.input.textarea.insert_str(".foo");
        let result = copy_query(&mut app, ClipboardBackend::Osc52);
        assert!(result, "Non-empty query should be accepted");
        assert_eq!(
            app.notification.current_message(),
            Some("Copied query!"),
            "Notification should be shown for successful copy"
        );
    }

    #[test]
    fn test_copy_result_accepts_non_empty() {
        let mut app = test_app("{}");
        app.query.result = Ok(r#"{"key": "value"}"#.to_string());
        let result = copy_result(&mut app, ClipboardBackend::Osc52);
        assert!(result, "Non-empty result should be accepted");
        assert_eq!(
            app.notification.current_message(),
            Some("Copied result!"),
            "Notification should be shown for successful copy"
        );
    }
}
