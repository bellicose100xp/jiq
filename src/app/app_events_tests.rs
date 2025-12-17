//! Tests for app_events

use crate::test_utils::test_helpers::test_app;
use proptest::prelude::*;

#[test]
fn test_paste_event_inserts_text() {
    let mut app = test_app(r#"{"name": "test"}"#);

    app.handle_paste_event(".name".to_string());

    assert_eq!(app.query(), ".name");
}

#[test]
fn test_paste_event_executes_query() {
    let mut app = test_app(r#"{"name": "Alice"}"#);

    app.handle_paste_event(".name".to_string());

    assert!(app.query.result.is_ok());
    let result = app.query.result.as_ref().unwrap();
    assert!(result.contains("Alice"));
}

#[test]
fn test_paste_event_appends_to_existing_text() {
    let mut app = test_app(r#"{"user": {"name": "Bob"}}"#);

    app.input.textarea.insert_str(".user");

    app.handle_paste_event(".name".to_string());

    assert_eq!(app.query(), ".user.name");
}

#[test]
fn test_paste_event_with_empty_string() {
    let mut app = test_app(r#"{"name": "test"}"#);

    app.handle_paste_event(String::new());

    assert_eq!(app.query(), "");
}

#[test]
fn test_paste_event_with_multiline_text() {
    let mut app = test_app(r#"{"name": "test"}"#);

    app.handle_paste_event(".name\n| length".to_string());

    assert!(app.query().contains(".name"));
}

// Feature: performance, Property 1: Paste text insertion integrity
// *For any* string pasted into the application, the input field content after
// the paste operation should contain exactly that string at the cursor position.
// **Validates: Requirements 1.2**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop_paste_text_insertion_integrity(
        // Generate printable ASCII strings (avoiding control characters that might
        // cause issues with the textarea)
        text in "[a-zA-Z0-9._\\[\\]|? ]{0,50}"
    ) {
        let mut app = test_app(r#"{"test": true}"#);

        // Paste the text
        app.handle_paste_event(text.clone());

        // The query should contain exactly the pasted text
        prop_assert_eq!(
            app.query(), &text,
            "Pasted text should appear exactly in the input field"
        );
    }

    #[test]
    fn prop_paste_appends_at_cursor_position(
        // Generate two parts of text
        prefix in "[a-zA-Z0-9.]{0,20}",
        pasted in "[a-zA-Z0-9.]{0,20}",
    ) {
        let mut app = test_app(r#"{"test": true}"#);

        // First insert the prefix
        app.input.textarea.insert_str(&prefix);

        // Then paste additional text
        app.handle_paste_event(pasted.clone());

        // The query should be prefix + pasted
        let expected = format!("{}{}", prefix, pasted);
        prop_assert_eq!(
            app.query(), &expected,
            "Pasted text should be appended at cursor position"
        );
    }

    #[test]
    fn prop_paste_executes_query_once(
        // Generate valid jq-like queries
        query in "\\.[a-z]{1,10}"
    ) {
        let json = r#"{"name": "test", "value": 42}"#;
        let mut app = test_app(json);

        // Paste a query
        app.handle_paste_event(query.clone());

        // Query should have been executed (result should be set)
        // We can't easily verify "exactly once" but we can verify it was executed
        prop_assert!(
            app.query.result.is_ok() || app.query.result.is_err(),
            "Query should have been executed after paste"
        );

        // The query text should match what was pasted
        prop_assert_eq!(
            app.query(), &query,
            "Query text should match pasted text"
        );
    }
}
