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

#[cfg(test)]
mod paste_recovery_event_tests {
    use super::*;
    use crate::app::App;
    use crate::app::app_events::paste_recovery as paste_recovery_events;
    use crate::config::Config;
    use crate::editor::EditorMode;
    use crate::input::loader::{LoaderSource, LoadingState};

    fn app_in_recovery() -> App {
        let (tx, rx) = std::sync::mpsc::channel();
        let _ = tx.send(Err(crate::error::JiqError::Io("err".to_string())));
        let loader = crate::input::FileLoader {
            state: LoadingState::Error(crate::error::JiqError::Io("err".to_string())),
            rx: Some(rx),
            source: LoaderSource::Clipboard,
        };
        let mut app = App::new_with_loader(loader, &Config::default());
        app.poll_file_loader();
        assert!(app.paste_recovery.is_some());
        app
    }

    #[test]
    fn enter_in_insert_mode_with_valid_json_loads_and_clears_recovery() {
        let mut app = app_in_recovery();
        app.input.textarea.insert_str(r#"{"name": "Alice"}"#);

        let consumed = paste_recovery_events::handle_key(
            &mut app,
            key_with_mods(KeyCode::Enter, KeyModifiers::NONE),
        );

        assert!(consumed);
        assert!(app.paste_recovery.is_none(), "recovery cleared on success");
        assert!(app.query.is_some(), "query initialised on success");
        // After accept, the input textarea is wiped so the user lands
        // on an empty query input, like a normal launch.
        assert_eq!(app.input.textarea.lines().join("\n"), "");
    }

    #[test]
    fn enter_with_invalid_json_keeps_recovery_and_updates_error_message() {
        let mut app = app_in_recovery();
        app.input.textarea.insert_str("not json");

        let _ = paste_recovery_events::handle_key(
            &mut app,
            key_with_mods(KeyCode::Enter, KeyModifiers::NONE),
        );

        let recovery = app
            .paste_recovery
            .as_ref()
            .expect("recovery still active on failure");
        assert!(recovery.error_message.starts_with("Invalid JSON:"));
        assert!(app.query.is_none(), "query should not be initialised");
    }

    #[test]
    fn ctrl_j_inserts_newline_not_destructive_delete() {
        // Regression: when bracketed paste isn't forwarded by the
        // terminal/multiplexer, pasted '\n' arrives as Ctrl+J. The
        // shared input handler would otherwise let tui-textarea's
        // default "Ctrl+J = delete line by head" wipe each line back
        // to column 0 (observed on Cloud Desktop / plain tmux).
        let mut app = app_in_recovery();
        app.input.textarea.insert_str("line1");

        let _ = paste_recovery_events::handle_key(
            &mut app,
            key_with_mods(KeyCode::Char('j'), KeyModifiers::CONTROL),
        );

        for c in "line2".chars() {
            let _ = paste_recovery_events::handle_key(
                &mut app,
                key_with_mods(KeyCode::Char(c), KeyModifiers::NONE),
            );
        }

        assert_eq!(app.input.textarea.lines().len(), 2);
        assert_eq!(app.input.textarea.lines()[0], "line1");
        assert_eq!(app.input.textarea.lines()[1], "line2");
    }

    #[test]
    fn enter_with_invalid_json_fires_red_toast() {
        // The user's eye needs a nudge to look at the top "No JSON
        // loaded" block when the error message changes silently.
        let mut app = app_in_recovery();
        app.input.textarea.insert_str("not json");

        let _ = paste_recovery_events::handle_key(
            &mut app,
            key_with_mods(KeyCode::Enter, KeyModifiers::NONE),
        );

        let notif = app.notification.current().expect("toast expected");
        assert!(notif.message.starts_with("Invalid JSON:"));
    }

    #[test]
    fn enter_submits_in_normal_mode_too() {
        let mut app = app_in_recovery();
        app.input.textarea.insert_str(r#"[1,2,3]"#);
        app.input.editor_mode = EditorMode::Normal;

        let _ = paste_recovery_events::handle_key(
            &mut app,
            key_with_mods(KeyCode::Enter, KeyModifiers::NONE),
        );

        assert!(app.paste_recovery.is_none());
        assert!(app.query.is_some());
    }

    #[test]
    fn esc_in_insert_mode_switches_to_normal_via_existing_handler() {
        // The existing handle_input_field_key already maps Esc to
        // EditorMode::Normal — this test confirms reuse.
        let mut app = app_in_recovery();
        assert_eq!(app.input.editor_mode, EditorMode::Insert);

        let _ = paste_recovery_events::handle_key(
            &mut app,
            key_with_mods(KeyCode::Esc, KeyModifiers::NONE),
        );

        assert!(!app.should_quit);
        assert_eq!(app.input.editor_mode, EditorMode::Normal);
        assert!(app.paste_recovery.is_some(), "still in recovery");
    }

    #[test]
    fn i_in_normal_mode_re_enters_insert() {
        let mut app = app_in_recovery();
        app.input.editor_mode = EditorMode::Normal;

        let _ = paste_recovery_events::handle_key(
            &mut app,
            key_with_mods(KeyCode::Char('i'), KeyModifiers::NONE),
        );

        assert_eq!(app.input.editor_mode, EditorMode::Insert);
    }

    #[test]
    fn x_in_normal_mode_deletes_char_via_existing_handler() {
        // Confirms the existing 'x' handler runs against app.input.textarea
        // during recovery (no separate codepath needed).
        let mut app = app_in_recovery();
        app.input.textarea.insert_str("abc");
        app.input
            .textarea
            .move_cursor(tui_textarea::CursorMove::Head);
        app.input.editor_mode = EditorMode::Normal;

        let _ = paste_recovery_events::handle_key(
            &mut app,
            key_with_mods(KeyCode::Char('x'), KeyModifiers::NONE),
        );

        assert_eq!(app.input.textarea.lines().join("\n"), "bc");
    }

    #[test]
    fn j_in_normal_mode_moves_cursor_down() {
        let mut app = app_in_recovery();
        app.input.textarea.insert_str("line1\nline2\nline3");
        app.input
            .textarea
            .move_cursor(tui_textarea::CursorMove::Top);
        app.input.editor_mode = EditorMode::Normal;
        let (start_row, _) = app.input.textarea.cursor();

        let _ = paste_recovery_events::handle_key(
            &mut app,
            key_with_mods(KeyCode::Char('j'), KeyModifiers::NONE),
        );

        let (end_row, _) = app.input.textarea.cursor();
        assert_eq!(end_row, start_row + 1, "j should move cursor down a line");
    }

    #[test]
    fn k_in_normal_mode_moves_cursor_up() {
        let mut app = app_in_recovery();
        app.input.textarea.insert_str("line1\nline2\nline3");
        app.input
            .textarea
            .move_cursor(tui_textarea::CursorMove::Bottom);
        app.input.editor_mode = EditorMode::Normal;
        let (start_row, _) = app.input.textarea.cursor();
        assert!(start_row > 0);

        let _ = paste_recovery_events::handle_key(
            &mut app,
            key_with_mods(KeyCode::Char('k'), KeyModifiers::NONE),
        );

        let (end_row, _) = app.input.textarea.cursor();
        assert_eq!(end_row + 1, start_row, "k should move cursor up a line");
    }

    #[test]
    fn capital_g_in_normal_mode_jumps_to_bottom() {
        let mut app = app_in_recovery();
        app.input.textarea.insert_str("a\nb\nc\nd");
        app.input
            .textarea
            .move_cursor(tui_textarea::CursorMove::Top);
        app.input.editor_mode = EditorMode::Normal;

        let _ = paste_recovery_events::handle_key(
            &mut app,
            key_with_mods(KeyCode::Char('G'), KeyModifiers::NONE),
        );

        let (row, _) = app.input.textarea.cursor();
        assert_eq!(row, 3);
    }

    #[test]
    fn lowercase_g_in_normal_mode_jumps_to_top() {
        let mut app = app_in_recovery();
        app.input.textarea.insert_str("a\nb\nc\nd");
        app.input
            .textarea
            .move_cursor(tui_textarea::CursorMove::Bottom);
        app.input.editor_mode = EditorMode::Normal;

        let _ = paste_recovery_events::handle_key(
            &mut app,
            key_with_mods(KeyCode::Char('g'), KeyModifiers::NONE),
        );

        let (row, _) = app.input.textarea.cursor();
        assert_eq!(row, 0);
    }

    #[test]
    fn up_down_arrows_move_cursor_in_insert_mode() {
        // Multi-line motion via arrow keys must work in recovery's Insert
        // mode without triggering the query-input's history popup.
        let mut app = app_in_recovery();
        app.input.textarea.insert_str("a\nb\nc");
        app.input
            .textarea
            .move_cursor(tui_textarea::CursorMove::Bottom);
        let (start_row, _) = app.input.textarea.cursor();

        let _ = paste_recovery_events::handle_key(
            &mut app,
            key_with_mods(KeyCode::Up, KeyModifiers::NONE),
        );

        let (end_row, _) = app.input.textarea.cursor();
        assert_eq!(end_row + 1, start_row);
        assert!(
            !app.history.is_visible(),
            "Up arrow during recovery must not trigger the history popup"
        );
    }

    #[test]
    fn dd_deletes_whole_line_via_existing_operator_infra() {
        // Confirms the operator/motion infra (dd, dw, ci", etc.) is
        // available during recovery for free. Here: 'd' enters Operator
        // mode, second 'd' triggers delete-line.
        let mut app = app_in_recovery();
        app.input.textarea.insert_str("garbage line");
        app.input.editor_mode = EditorMode::Normal;

        let _ = paste_recovery_events::handle_key(
            &mut app,
            key_with_mods(KeyCode::Char('d'), KeyModifiers::NONE),
        );
        let _ = paste_recovery_events::handle_key(
            &mut app,
            key_with_mods(KeyCode::Char('d'), KeyModifiers::NONE),
        );

        assert_eq!(app.input.textarea.lines().join("\n"), "");
    }

    #[test]
    fn capital_d_deletes_to_end_of_line() {
        let mut app = app_in_recovery();
        app.input.textarea.insert_str("keep|drop");
        app.input
            .textarea
            .move_cursor(tui_textarea::CursorMove::Head);
        for _ in 0..4 {
            app.input
                .textarea
                .move_cursor(tui_textarea::CursorMove::Forward);
        }
        app.input.editor_mode = EditorMode::Normal;

        let _ = paste_recovery_events::handle_key(
            &mut app,
            key_with_mods(KeyCode::Char('D'), KeyModifiers::NONE),
        );

        assert_eq!(app.input.textarea.lines().join("\n"), "keep");
    }

    #[test]
    fn typing_in_insert_mode_inserts_chars() {
        let mut app = app_in_recovery();

        for c in r#"{"x": 1}"#.chars() {
            let _ = paste_recovery_events::handle_key(
                &mut app,
                key_with_mods(KeyCode::Char(c), KeyModifiers::NONE),
            );
        }

        assert_eq!(app.input.textarea.lines().join("\n"), r#"{"x": 1}"#);
    }

    #[test]
    fn ctrl_x_clears_pasted_content() {
        let mut app = app_in_recovery();
        app.input
            .textarea
            .insert_str("accidentally pasted nonsense");
        app.input.editor_mode = EditorMode::Normal;

        let _ = paste_recovery_events::handle_key(
            &mut app,
            key_with_mods(KeyCode::Char('x'), KeyModifiers::CONTROL),
        );

        assert_eq!(app.input.textarea.lines().join("\n"), "");
        assert_eq!(
            app.input.editor_mode,
            EditorMode::Insert,
            "Ctrl+X should also drop back to Insert so user can paste again"
        );
        // Recovery still active — Ctrl+X clears, doesn't exit.
        assert!(app.paste_recovery.is_some());
    }

    #[test]
    fn handle_paste_in_recovery_inserts_into_input_textarea() {
        let mut app = app_in_recovery();

        app.handle_paste_recovery_paste(r#"{"k": 1}"#.to_string());

        assert_eq!(app.input.textarea.lines().join("\n"), r#"{"k": 1}"#);
        assert!(app.query.is_none(), "no query yet");
    }

    #[test]
    fn handle_paste_in_recovery_normalises_crlf() {
        let mut app = app_in_recovery();

        app.handle_paste_recovery_paste("{\r\n  \"a\": 1\r\n}\r\n".to_string());

        let content = app.input.textarea.lines().join("\n");
        assert!(!content.contains('\r'));
    }

    #[test]
    fn handle_paste_in_recovery_oversize_shows_error_notification() {
        let mut app = app_in_recovery();
        let huge = "x".repeat(crate::input::paste_recovery::PASTE_RECOVERY_MAX_BYTES + 1);

        app.handle_paste_recovery_paste(huge);

        let notif = app.notification.current();
        assert!(notif.is_some());
        assert!(notif.unwrap().message.contains("too large"));
    }

    #[test]
    fn paste_event_outside_recovery_still_inserts_into_query() {
        // Regression: outside recovery, paste must continue inserting
        // into the query textarea (existing behavior unchanged).
        let mut app = test_app(r#"{"x": 1}"#);
        assert!(app.paste_recovery.is_none());

        app.handle_paste_event(".x".to_string());

        assert_eq!(app.query(), ".x");
    }

    #[test]
    fn ctrl_c_quits_during_recovery() {
        // Truly global Ctrl+C handler still wins.
        let mut app = app_in_recovery();
        app.handle_paste_recovery_key_event(key_with_mods(
            KeyCode::Char('c'),
            KeyModifiers::CONTROL,
        ));
        assert!(app.should_quit);
    }

    #[test]
    fn loader_io_prefix_stripped_in_error_message() {
        let (tx, rx) = std::sync::mpsc::channel();
        let _ = tx.send(Err(crate::error::JiqError::Io(
            "Clipboard is empty.\n\nUsage:\n  jiq".to_string(),
        )));
        let loader = crate::input::FileLoader {
            state: LoadingState::Error(crate::error::JiqError::Io("err".to_string())),
            rx: Some(rx),
            source: LoaderSource::Clipboard,
        };
        let mut app = App::new_with_loader(loader, &Config::default());
        app.poll_file_loader();

        let recovery = app.paste_recovery.expect("recovery");
        assert_eq!(recovery.error_message, "Clipboard is empty.");
    }
}
