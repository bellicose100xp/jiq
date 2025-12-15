//! Error overlay tests

use super::*;
use tui_textarea::CursorMove;

#[test]
fn test_error_overlay_initializes_hidden() {
    let app = test_app(TEST_JSON);
    assert!(!app.error_overlay_visible);
}

#[test]
fn test_ctrl_e_toggles_error_overlay_when_error_exists() {
    let mut app = test_app(TEST_JSON);
    app.input.editor_mode = EditorMode::Insert;

    // Type an invalid query (| is invalid jq syntax)
    app.handle_key_event(key(KeyCode::Char('|')));
    // Flush debounced query to execute immediately (simulates debounce period passing)
    flush_debounced_query(&mut app);

    // Should have an error now
    assert!(app.query.result.is_err());
    assert!(!app.error_overlay_visible); // Initially hidden

    // Press Ctrl+E to show overlay
    app.handle_key_event(key_with_mods(KeyCode::Char('e'), KeyModifiers::CONTROL));
    assert!(app.error_overlay_visible);

    // Press Ctrl+E again to hide overlay
    app.handle_key_event(key_with_mods(KeyCode::Char('e'), KeyModifiers::CONTROL));
    assert!(!app.error_overlay_visible);
}

#[test]
fn test_ctrl_e_does_nothing_when_no_error() {
    let mut app = test_app(TEST_JSON);
    // Initial query "." should succeed
    assert!(app.query.result.is_ok());
    assert!(!app.error_overlay_visible);

    // Press Ctrl+E (should do nothing since no error)
    app.handle_key_event(key_with_mods(KeyCode::Char('e'), KeyModifiers::CONTROL));
    assert!(!app.error_overlay_visible); // Should remain hidden
}

#[test]
fn test_error_overlay_hides_on_query_change() {
    let mut app = test_app(TEST_JSON);
    app.input.editor_mode = EditorMode::Insert;

    // Type invalid query
    app.handle_key_event(key(KeyCode::Char('|')));
    // Flush debounced query to execute immediately
    flush_debounced_query(&mut app);
    assert!(app.query.result.is_err());

    // Show error overlay
    app.handle_key_event(key_with_mods(KeyCode::Char('e'), KeyModifiers::CONTROL));
    assert!(app.error_overlay_visible);

    // Change query by pressing backspace to delete the invalid character
    app.handle_key_event(key(KeyCode::Backspace));

    // Overlay should auto-hide after query change
    assert!(!app.error_overlay_visible);
}

#[test]
fn test_error_overlay_hides_on_query_change_in_normal_mode() {
    let mut app = test_app(TEST_JSON);
    app.input.editor_mode = EditorMode::Insert;

    // Type invalid query
    app.handle_key_event(key(KeyCode::Char('|')));
    // Flush debounced query to execute immediately
    flush_debounced_query(&mut app);
    assert!(app.query.result.is_err());

    // Show error overlay
    app.handle_key_event(key_with_mods(KeyCode::Char('e'), KeyModifiers::CONTROL));
    assert!(app.error_overlay_visible);

    // Switch to Normal mode and delete the character
    app.handle_key_event(key(KeyCode::Esc));
    app.input.textarea.move_cursor(CursorMove::Head);
    app.handle_key_event(key(KeyCode::Char('x')));

    // Overlay should auto-hide after query change
    assert!(!app.error_overlay_visible);
}

#[test]
fn test_ctrl_e_works_in_normal_mode() {
    let mut app = test_app(TEST_JSON);
    app.input.editor_mode = EditorMode::Insert;

    // Type invalid query
    app.handle_key_event(key(KeyCode::Char('|')));
    // Flush debounced query to execute immediately
    flush_debounced_query(&mut app);
    assert!(app.query.result.is_err());

    // Switch to Normal mode
    app.handle_key_event(key(KeyCode::Esc));
    assert_eq!(app.input.editor_mode, EditorMode::Normal);

    // Press Ctrl+E in Normal mode
    app.handle_key_event(key_with_mods(KeyCode::Char('e'), KeyModifiers::CONTROL));
    assert!(app.error_overlay_visible);
}

#[test]
fn test_ctrl_e_works_when_results_pane_focused() {
    let mut app = test_app(TEST_JSON);
    app.input.editor_mode = EditorMode::Insert;

    // Type invalid query
    app.handle_key_event(key(KeyCode::Char('|')));
    // Flush debounced query to execute immediately
    flush_debounced_query(&mut app);
    assert!(app.query.result.is_err());

    // Switch focus to results pane
    app.handle_key_event(key(KeyCode::BackTab));
    assert_eq!(app.focus, Focus::ResultsPane);

    // Press Ctrl+E while results pane is focused
    app.handle_key_event(key_with_mods(KeyCode::Char('e'), KeyModifiers::CONTROL));
    assert!(app.error_overlay_visible);
}
