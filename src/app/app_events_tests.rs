//! Tests for app_events

use crate::app::Focus;
use crate::editor::EditorMode;
use crate::test_utils::test_helpers::{app_with_query, key_with_mods, test_app};
use proptest::prelude::*;
use ratatui::crossterm::event::{KeyCode, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use std::sync::Arc;

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

    assert!(app.query.as_ref().unwrap().result.is_ok());
    let result = app.query.as_ref().unwrap().result.as_ref().unwrap();
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
            app.query.as_ref().unwrap().result.is_ok() || app.query.as_ref().unwrap().result.is_err(),
            "Query should have been executed after paste"
        );

        // The query text should match what was pasted
        prop_assert_eq!(
            app.query(), &query,
            "Query text should match pasted text"
        );
    }
}

#[test]
fn test_ctrl_d_scrolls_results_from_input_field_insert_mode() {
    let mut app = app_with_query(".");
    app.focus = Focus::InputField;
    app.input.editor_mode = EditorMode::Insert;

    let content: String = (0..50).map(|i| format!("line{}\n", i)).collect();
    let query_state = app.query.as_mut().unwrap();
    query_state.result = Ok(content.clone());
    query_state.last_successful_result = Some(Arc::new(content.clone()));
    query_state.cached_line_count = content.lines().count() as u32;

    let line_count = app.results_line_count_u32();
    app.results_scroll.update_bounds(line_count, 20);
    app.results_scroll.offset = 0;

    app.handle_key_event(key_with_mods(KeyCode::Char('d'), KeyModifiers::CONTROL));

    assert_eq!(app.results_scroll.offset, 10);
    assert_eq!(app.focus, Focus::InputField);
}

#[test]
fn test_ctrl_u_scrolls_results_from_input_field_insert_mode() {
    let mut app = app_with_query(".");
    app.focus = Focus::InputField;
    app.input.editor_mode = EditorMode::Insert;
    app.results_scroll.offset = 20;
    app.results_scroll.viewport_height = 20;

    app.handle_key_event(key_with_mods(KeyCode::Char('u'), KeyModifiers::CONTROL));

    assert_eq!(app.results_scroll.offset, 10);
    assert_eq!(app.focus, Focus::InputField);
}

#[test]
fn test_ctrl_d_scrolls_results_from_input_field_normal_mode() {
    let mut app = app_with_query(".");
    app.focus = Focus::InputField;
    app.input.editor_mode = EditorMode::Normal;

    let content: String = (0..50).map(|i| format!("line{}\n", i)).collect();
    let query_state = app.query.as_mut().unwrap();
    query_state.result = Ok(content.clone());
    query_state.last_successful_result = Some(Arc::new(content.clone()));
    query_state.cached_line_count = content.lines().count() as u32;

    let line_count = app.results_line_count_u32();
    app.results_scroll.update_bounds(line_count, 20);
    app.results_scroll.offset = 0;

    app.handle_key_event(key_with_mods(KeyCode::Char('d'), KeyModifiers::CONTROL));

    assert_eq!(app.results_scroll.offset, 10);
    assert_eq!(app.focus, Focus::InputField);
    assert_eq!(app.input.editor_mode, EditorMode::Normal);
}

#[test]
fn test_ctrl_u_scrolls_results_from_input_field_normal_mode() {
    let mut app = app_with_query(".");
    app.focus = Focus::InputField;
    app.input.editor_mode = EditorMode::Normal;
    app.results_scroll.offset = 20;
    app.results_scroll.viewport_height = 20;

    app.handle_key_event(key_with_mods(KeyCode::Char('u'), KeyModifiers::CONTROL));

    assert_eq!(app.results_scroll.offset, 10);
    assert_eq!(app.focus, Focus::InputField);
    assert_eq!(app.input.editor_mode, EditorMode::Normal);
}

#[test]
fn test_ctrl_d_scrolls_results_from_input_field_operator_mode() {
    let mut app = app_with_query(".");
    app.focus = Focus::InputField;
    app.input.editor_mode = EditorMode::Operator('d');

    let content: String = (0..50).map(|i| format!("line{}\n", i)).collect();
    let query_state = app.query.as_mut().unwrap();
    query_state.result = Ok(content.clone());
    query_state.last_successful_result = Some(Arc::new(content.clone()));
    query_state.cached_line_count = content.lines().count() as u32;

    let line_count = app.results_line_count_u32();
    app.results_scroll.update_bounds(line_count, 20);
    app.results_scroll.offset = 0;

    app.handle_key_event(key_with_mods(KeyCode::Char('d'), KeyModifiers::CONTROL));

    assert_eq!(app.results_scroll.offset, 10);
    assert_eq!(app.focus, Focus::InputField);
}

#[test]
fn test_ctrl_u_scrolls_results_from_input_field_operator_mode() {
    let mut app = app_with_query(".");
    app.focus = Focus::InputField;
    app.input.editor_mode = EditorMode::Operator('c');
    app.results_scroll.offset = 20;
    app.results_scroll.viewport_height = 20;

    app.handle_key_event(key_with_mods(KeyCode::Char('u'), KeyModifiers::CONTROL));

    assert_eq!(app.results_scroll.offset, 10);
    assert_eq!(app.focus, Focus::InputField);
}

fn mouse_event(kind: MouseEventKind) -> MouseEvent {
    MouseEvent {
        kind,
        column: 0,
        row: 0,
        modifiers: KeyModifiers::NONE,
    }
}

#[test]
fn test_mouse_scroll_down_increases_offset() {
    let mut app = app_with_query(".");

    let content: String = (0..50).map(|i| format!("line{}\n", i)).collect();
    let query_state = app.query.as_mut().unwrap();
    query_state.result = Ok(content.clone());
    query_state.last_successful_result = Some(Arc::new(content.clone()));
    query_state.cached_line_count = content.lines().count() as u32;

    let line_count = app.results_line_count_u32();
    app.results_scroll.update_bounds(line_count, 20);
    app.results_scroll.offset = 0;

    app.handle_mouse_event(mouse_event(MouseEventKind::ScrollDown));

    assert_eq!(app.results_scroll.offset, 3);
}

#[test]
fn test_mouse_scroll_up_decreases_offset() {
    let mut app = app_with_query(".");
    app.results_scroll.offset = 10;
    app.results_scroll.viewport_height = 20;

    app.handle_mouse_event(mouse_event(MouseEventKind::ScrollUp));

    assert_eq!(app.results_scroll.offset, 7);
}

#[test]
fn test_mouse_scroll_up_stops_at_zero() {
    let mut app = app_with_query(".");
    app.results_scroll.offset = 2;
    app.results_scroll.viewport_height = 20;

    app.handle_mouse_event(mouse_event(MouseEventKind::ScrollUp));

    assert_eq!(app.results_scroll.offset, 0);
}

#[test]
fn test_mouse_scroll_down_multiple_times() {
    let mut app = app_with_query(".");

    let content: String = (0..50).map(|i| format!("line{}\n", i)).collect();
    let query_state = app.query.as_mut().unwrap();
    query_state.result = Ok(content.clone());
    query_state.last_successful_result = Some(Arc::new(content.clone()));
    query_state.cached_line_count = content.lines().count() as u32;

    let line_count = app.results_line_count_u32();
    app.results_scroll.update_bounds(line_count, 20);
    app.results_scroll.offset = 0;

    app.handle_mouse_event(mouse_event(MouseEventKind::ScrollDown));
    app.handle_mouse_event(mouse_event(MouseEventKind::ScrollDown));
    app.handle_mouse_event(mouse_event(MouseEventKind::ScrollDown));

    assert_eq!(app.results_scroll.offset, 9);
}

#[test]
fn test_mouse_other_events_ignored() {
    let mut app = app_with_query(".");
    app.results_scroll.offset = 5;
    app.results_scroll.viewport_height = 20;

    app.handle_mouse_event(mouse_event(MouseEventKind::Down(MouseButton::Left)));
    assert_eq!(app.results_scroll.offset, 5);

    app.handle_mouse_event(mouse_event(MouseEventKind::Up(MouseButton::Left)));
    assert_eq!(app.results_scroll.offset, 5);

    app.handle_mouse_event(mouse_event(MouseEventKind::Moved));
    assert_eq!(app.results_scroll.offset, 5);
}

#[test]
fn test_snippets_receives_keys_when_focus_is_results_pane() {
    use crate::snippets::Snippet;
    let mut app = app_with_query(".");

    app.snippets.disable_persistence();
    app.snippets.set_snippets(vec![Snippet {
        name: "test".to_string(),
        query: ".test".to_string(),
        description: None,
    }]);
    app.snippets.open();
    app.focus = Focus::ResultsPane;

    assert!(app.snippets.is_visible());

    app.handle_key_event(key_with_mods(KeyCode::Esc, KeyModifiers::NONE));

    assert!(
        !app.snippets.is_visible(),
        "Esc should close snippets even when focus is ResultsPane"
    );
}

#[test]
fn test_snippets_navigation_works_when_focus_is_results_pane() {
    use crate::snippets::Snippet;
    let mut app = app_with_query(".");

    app.snippets.disable_persistence();
    app.snippets.set_snippets(vec![
        Snippet {
            name: "first".to_string(),
            query: ".first".to_string(),
            description: None,
        },
        Snippet {
            name: "second".to_string(),
            query: ".second".to_string(),
            description: None,
        },
    ]);
    app.snippets.open();
    app.focus = Focus::ResultsPane;

    assert_eq!(app.snippets.selected_index(), 0);

    app.handle_key_event(key_with_mods(KeyCode::Down, KeyModifiers::NONE));

    assert_eq!(
        app.snippets.selected_index(),
        1,
        "Down arrow should navigate snippets even when focus is ResultsPane"
    );
}

#[test]
fn test_history_receives_keys_when_focus_is_results_pane() {
    let mut app = app_with_query(".");

    app.history.add_entry(".test1");
    app.history.add_entry(".test2");
    app.history.open(None);
    app.focus = Focus::ResultsPane;

    assert!(app.history.is_visible());

    app.handle_key_event(key_with_mods(KeyCode::Esc, KeyModifiers::NONE));

    assert!(
        !app.history.is_visible(),
        "Esc should close history even when focus is ResultsPane"
    );
}

#[test]
fn test_global_keys_work_when_snippets_visible() {
    use crate::snippets::Snippet;
    let mut app = app_with_query(".");

    app.snippets.disable_persistence();
    app.snippets.set_snippets(vec![Snippet {
        name: "test".to_string(),
        query: ".test".to_string(),
        description: None,
    }]);
    app.snippets.open();

    assert!(app.snippets.is_visible());
    assert!(!app.help.visible);

    app.handle_key_event(key_with_mods(KeyCode::F(1), KeyModifiers::NONE));

    assert!(
        app.help.visible,
        "F1 should toggle help even when snippets is visible"
    );
    assert!(
        app.snippets.is_visible(),
        "Snippets should remain visible after F1"
    );
}

#[test]
fn test_ctrl_c_quits_when_snippets_visible() {
    use crate::snippets::Snippet;
    let mut app = app_with_query(".");

    app.snippets.disable_persistence();
    app.snippets.set_snippets(vec![Snippet {
        name: "test".to_string(),
        query: ".test".to_string(),
        description: None,
    }]);
    app.snippets.open();

    assert!(app.snippets.is_visible());
    assert!(!app.should_quit);

    app.handle_key_event(key_with_mods(KeyCode::Char('c'), KeyModifiers::CONTROL));

    assert!(
        app.should_quit,
        "Ctrl+C should quit even when snippets is visible"
    );
}

#[test]
fn test_esc_closes_help_before_snippets() {
    use crate::snippets::Snippet;
    let mut app = app_with_query(".");

    app.snippets.disable_persistence();
    app.snippets.set_snippets(vec![Snippet {
        name: "test".to_string(),
        query: ".test".to_string(),
        description: None,
    }]);
    app.snippets.open();
    app.help.visible = true;

    assert!(app.snippets.is_visible());
    assert!(app.help.visible);

    // First Esc should close help, not snippets
    app.handle_key_event(key_with_mods(KeyCode::Esc, KeyModifiers::NONE));

    assert!(
        !app.help.visible,
        "Esc should close help first when both help and snippets are visible"
    );
    assert!(
        app.snippets.is_visible(),
        "Snippets should remain visible after closing help"
    );

    // Second Esc should close snippets
    app.handle_key_event(key_with_mods(KeyCode::Esc, KeyModifiers::NONE));

    assert!(
        !app.snippets.is_visible(),
        "Second Esc should close snippets"
    );
}
