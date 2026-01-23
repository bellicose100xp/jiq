//! Tests for mouse click handling

use ratatui::crossterm::event::{KeyModifiers, MouseButton, MouseEvent, MouseEventKind};

use crate::app::Focus;
use crate::editor::EditorMode;
use crate::layout::Region;
use crate::test_utils::test_helpers::test_app;

use super::handle_click;

fn setup_app() -> crate::app::App {
    test_app(r#"{"test": "data"}"#)
}

fn create_mouse_event(column: u16, row: u16) -> MouseEvent {
    MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column,
        row,
        modifiers: KeyModifiers::NONE,
    }
}

#[test]
fn test_click_results_pane_changes_focus_from_input() {
    let mut app = setup_app();
    app.focus = Focus::InputField;
    let mouse = create_mouse_event(10, 10);

    handle_click(&mut app, Some(Region::ResultsPane), mouse);

    assert_eq!(app.focus, Focus::ResultsPane);
}

#[test]
fn test_click_results_pane_when_already_focused() {
    let mut app = setup_app();
    app.focus = Focus::ResultsPane;
    let mouse = create_mouse_event(10, 10);

    handle_click(&mut app, Some(Region::ResultsPane), mouse);

    assert_eq!(app.focus, Focus::ResultsPane);
}

#[test]
fn test_click_input_field_changes_focus_from_results() {
    let mut app = setup_app();
    app.focus = Focus::ResultsPane;
    app.input.editor_mode = EditorMode::Normal;
    let mouse = create_mouse_event(10, 10);

    handle_click(&mut app, Some(Region::InputField), mouse);

    assert_eq!(app.focus, Focus::InputField);
    assert_eq!(app.input.editor_mode, EditorMode::Insert);
}

#[test]
fn test_click_input_field_when_already_focused_does_not_change() {
    let mut app = setup_app();
    app.focus = Focus::InputField;
    app.input.editor_mode = EditorMode::Normal;
    let mouse = create_mouse_event(10, 10);

    handle_click(&mut app, Some(Region::InputField), mouse);

    assert_eq!(app.focus, Focus::InputField);
    assert_eq!(
        app.input.editor_mode,
        EditorMode::Normal,
        "Should not change editor mode when already focused"
    );
}

#[test]
fn test_click_search_bar_unconfirms_when_confirmed() {
    let mut app = setup_app();
    app.search.open();
    app.search.confirm();
    assert!(app.search.is_confirmed());
    let mouse = create_mouse_event(10, 10);

    handle_click(&mut app, Some(Region::SearchBar), mouse);

    assert!(
        !app.search.is_confirmed(),
        "Search should be unconfirmed after click"
    );
    assert!(app.search.is_visible(), "Search should still be visible");
}

#[test]
fn test_click_search_bar_does_nothing_when_not_confirmed() {
    let mut app = setup_app();
    app.search.open();
    assert!(!app.search.is_confirmed());
    let mouse = create_mouse_event(10, 10);

    handle_click(&mut app, Some(Region::SearchBar), mouse);

    assert!(!app.search.is_confirmed());
    assert!(app.search.is_visible());
}

#[test]
fn test_click_search_bar_does_nothing_when_not_visible() {
    let mut app = setup_app();
    assert!(!app.search.is_visible());
    let mouse = create_mouse_event(10, 10);

    handle_click(&mut app, Some(Region::SearchBar), mouse);

    assert!(!app.search.is_visible());
}

#[test]
fn test_click_none_region_does_nothing() {
    let mut app = setup_app();
    app.focus = Focus::InputField;
    let original_focus = app.focus;
    let mouse = create_mouse_event(10, 10);

    handle_click(&mut app, None, mouse);

    assert_eq!(app.focus, original_focus);
}

#[test]
fn test_click_ai_window_no_suggestions() {
    let mut app = setup_app();
    app.ai.visible = true;
    app.ai.suggestions = vec![];
    app.focus = Focus::InputField;
    let original_focus = app.focus;
    let mouse = create_mouse_event(15, 7);

    handle_click(&mut app, Some(Region::AiWindow), mouse);

    assert_eq!(app.focus, original_focus);
}

#[test]
fn test_click_help_popup_does_nothing_for_focus() {
    let mut app = setup_app();
    app.focus = Focus::InputField;
    let original_focus = app.focus;
    let mouse = create_mouse_event(10, 10);

    handle_click(&mut app, Some(Region::HelpPopup), mouse);

    assert_eq!(
        app.focus, original_focus,
        "Help popup click should not change focus"
    );
}

#[test]
fn test_click_snippet_list_does_nothing_for_focus() {
    let mut app = setup_app();
    app.focus = Focus::InputField;
    let original_focus = app.focus;
    let mouse = create_mouse_event(10, 10);

    handle_click(&mut app, Some(Region::SnippetList), mouse);

    assert_eq!(
        app.focus, original_focus,
        "Snippet list click should not change focus (handled in Phase 6)"
    );
}
