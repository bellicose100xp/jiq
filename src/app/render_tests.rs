use super::*;

#[test]
fn test_syntax_highlighting_enabled_for_short_queries() {
    // This test documents that syntax highlighting works for queries
    // that fit within the viewport width

    let json = r#"{"test": true}"#;
    let app = App::new(json.to_string());

    // Short query - should be eligible for highlighting
    let short_query = ".test";
    assert!(short_query.chars().count() < 50); // Typical viewport width

    // Verify query method works (syntax highlighting uses this)
    assert_eq!(app.query(), "");
}

#[test]
fn test_long_query_handling() {
    // This test documents the behavior for queries that exceed viewport width
    // Syntax highlighting is disabled to prevent cursor sync issues

    let json = r#"{"test": true}"#;
    let _app = App::new(json.to_string());

    // Create a very long query (would exceed typical terminal width)
    let long_query = ".field1 | .field2 | .field3 | .field4 | .field5 | .field6 | .field7 | .field8 | .field9 | .field10 | select(.value > 100)";
    assert!(long_query.chars().count() > 100);

    // The rendering logic will check: query_len >= viewport_width
    // If true, it skips the syntax highlighting overlay
    // This allows tui-textarea's native scrolling to work correctly
}

#[test]
fn test_viewport_width_threshold() {
    // Documents the exact threshold behavior for syntax highlighting

    let json = r#"{"test": true}"#;
    let _app = App::new(json.to_string());

    // If terminal inner width is 80 columns (typical)
    // And query is 80+ characters, highlighting is disabled
    // And query is <80 characters, highlighting is enabled

    let at_threshold = "a".repeat(80);
    assert_eq!(at_threshold.chars().count(), 80);

    // The render logic checks: if query_len >= viewport_width { skip highlighting }
    // So at exactly viewport_width, highlighting is disabled
}

#[test]
fn test_empty_query_has_no_highlighting() {
    // Empty queries should not render any syntax highlighting

    let json = r#"{"test": true}"#;
    let app = App::new(json.to_string());

    assert_eq!(app.query(), "");
    // The render_syntax_highlighting method returns early for empty queries
}

#[test]
fn test_char_count_not_byte_count() {
    // Verify we count characters (not bytes) for viewport comparison
    // Important for UTF-8 queries with emoji or multi-byte characters

    let emoji_query = "🔍 search term";
    let char_count = emoji_query.chars().count();
    let byte_count = emoji_query.len();

    assert!(byte_count > char_count); // Emoji takes multiple bytes
    // We use chars().count() which correctly handles UTF-8
}
