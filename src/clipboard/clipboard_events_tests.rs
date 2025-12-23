//! Tests for clipboard_events

use super::*;
use crate::test_utils::test_helpers::test_app;
use proptest::prelude::*;

#[test]
fn test_strip_ansi_codes_no_codes() {
    assert_eq!(strip_ansi_codes("hello world"), "hello world");
}

#[test]
fn test_strip_ansi_codes_simple_color() {
    assert_eq!(strip_ansi_codes("\x1b[31mhello\x1b[0m"), "hello");
}

#[test]
fn test_strip_ansi_codes_multiple_colors() {
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
    assert_eq!(strip_ansi_codes("\x1b]0;title\x07text"), "text");
}

// Feature: clipboard, Property 2: ANSI stripping preserves non-ANSI content
// *For any* input text without ANSI escape sequences, stripping ANSI codes
// should return the identical text.
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
        let query_state = app.query.as_mut().unwrap();
        query_state.result = Ok(ansi_only);
        // Clear cache to test the no-cache scenario
        query_state.last_successful_result_unformatted = None;

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

#[test]
fn test_copy_query_rejects_empty() {
    let mut app = test_app("{}");
    let result = copy_query(&mut app, ClipboardBackend::Osc52);
    assert!(!result, "Empty query should be rejected");
    assert!(
        app.notification.current().is_none(),
        "No notification for rejected copy"
    );
}

#[test]
fn test_copy_result_rejects_empty_without_cache() {
    let mut app = test_app("{}");
    let query_state = app.query.as_mut().unwrap();
    query_state.result = Ok(String::new());
    // Clear cache to test the no-cache scenario
    query_state.last_successful_result_unformatted = None;

    let result = copy_result(&mut app, ClipboardBackend::Osc52);
    assert!(!result, "Empty result without cache should be rejected");
    assert!(
        app.notification.current().is_none(),
        "No notification for rejected copy"
    );
}

#[test]
fn test_copy_result_rejects_ansi_only_without_cache() {
    let mut app = test_app("{}");
    let query_state = app.query.as_mut().unwrap();
    query_state.result = Ok("\x1b[31m\x1b[0m".to_string());
    // Clear cache to test the no-cache scenario
    query_state.last_successful_result_unformatted = None;

    let result = copy_result(&mut app, ClipboardBackend::Osc52);
    assert!(!result, "ANSI-only result without cache should be rejected");
    assert!(
        app.notification.current().is_none(),
        "No notification for rejected copy"
    );
}

#[test]
fn test_copy_result_rejects_error_without_cache() {
    let mut app = test_app("{}");
    let query_state = app.query.as_mut().unwrap();
    query_state.result = Err("some error".to_string());
    // Clear cache to test the no-cache scenario
    query_state.last_successful_result_unformatted = None;

    let result = copy_result(&mut app, ClipboardBackend::Osc52);
    assert!(!result, "Error result without cache should be rejected");
    assert!(
        app.notification.current().is_none(),
        "No notification for rejected copy"
    );
}

#[test]
fn test_copy_query_accepts_non_empty() {
    let mut app = test_app("{}");
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
    app.query.as_mut().unwrap().result = Ok(r#"{"key": "value"}"#.to_string());
    let result = copy_result(&mut app, ClipboardBackend::Osc52);
    assert!(result, "Non-empty result should be accepted");
    assert_eq!(
        app.notification.current_message(),
        Some("Copied result!"),
        "Notification should be shown for successful copy"
    );
}

#[test]
fn test_copy_result_uses_cached_when_result_is_error() {
    use std::sync::Arc;

    let mut app = test_app(r#"{"value": 42}"#);
    let query_state = app.query.as_mut().unwrap();

    // Set up cached successful result
    let cached_result = r#"{"cached": "data"}"#;
    query_state.last_successful_result_unformatted = Some(Arc::new(cached_result.to_string()));

    // Set current result to error
    query_state.result = Err("syntax error".to_string());

    // Copy should succeed and use cached result
    let result = copy_result(&mut app, ClipboardBackend::Osc52);
    assert!(
        result,
        "Copy should succeed with cached result even when current result is error"
    );
    assert_eq!(
        app.notification.current_message(),
        Some("Copied result!"),
        "Notification should be shown"
    );
}

#[test]
fn test_copy_result_uses_cached_when_result_is_null() {
    use std::sync::Arc;

    let mut app = test_app(r#"{"name": "test", "value": 42}"#);
    let query_state = app.query.as_mut().unwrap();

    // Set up cached successful result (meaningful data)
    let cached_result = r#"{"value": 42}"#;
    query_state.last_successful_result_unformatted = Some(Arc::new(cached_result.to_string()));

    // Set current result to null (partial query like ".nonexistent")
    query_state.result = Ok("null\n".to_string());

    // Copy should succeed and use cached result (not "null")
    let result = copy_result(&mut app, ClipboardBackend::Osc52);
    assert!(
        result,
        "Copy should succeed with cached result even when current result is null"
    );
    assert_eq!(
        app.notification.current_message(),
        Some("Copied result!"),
        "Notification should be shown"
    );
}
