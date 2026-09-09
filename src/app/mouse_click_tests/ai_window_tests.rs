//! Tests for clicks inside the AI popup: suggestion rows apply the clicked
//! suggestion, the chat rows move keyboard focus to the chat input.
//!
//! Popup geometry for a `Rect::new(0, 0, 40, 10)` popup: row 0 border,
//! row 1 padding, rows 2..=5 scrollable content, row 6 chat separator,
//! row 7 chat input, row 8 padding, row 9 border.

use ratatui::layout::Rect;

use super::*;
use crate::ai::{Suggestion, SuggestionType};

const POPUP: Rect = Rect {
    x: 0,
    y: 0,
    width: 40,
    height: 10,
};

/// Make the AI window visible with a single suggestion whose query is
/// `.picked`, layout populated so that content row 0 maps to suggestion 0,
/// and the `ai_window` layout rect tracked at the given origin/size.
fn setup_ai_window_one_suggestion(app: &mut crate::app::App, rect: Option<Rect>) {
    app.ai.visible = true;
    app.ai.suggestions = vec![Suggestion {
        query: ".picked".to_string(),
        description: String::new(),
        suggestion_type: SuggestionType::Query,
    }];
    // One suggestion of height 1, viewport 10: content row 0 -> suggestion 0.
    app.ai.selection.update_layout(vec![1], 10);
    app.layout_regions.ai_window = rect;
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
fn test_click_ai_window_applies_clicked_suggestion() {
    let mut app = setup_app();
    setup_ai_window_one_suggestion(&mut app, Some(POPUP));
    // Pre-select so the post-click assertion can prove clear_selection ran.
    app.ai.selection.select_index(0);
    assert_eq!(app.ai.selection.get_selected(), Some(0));

    // First content row (border + padding above it) -> suggestion 0.
    let mouse = create_mouse_event(1, 2);
    handle_click(&mut app, Some(Region::AiWindow), mouse);

    assert_eq!(
        app.input.query(),
        ".picked",
        "in-bounds click on a suggestion row should replace the query with that suggestion"
    );
    assert!(
        app.ai.selection.get_selected().is_none(),
        "applying a clicked suggestion should clear the selection"
    );
    assert_eq!(
        app.focus,
        Focus::InputField,
        "applying a suggestion does not move focus to the chat"
    );
}

#[test]
fn test_click_ai_window_border_and_padding_do_nothing() {
    let mut app = setup_app();
    setup_ai_window_one_suggestion(&mut app, Some(POPUP));
    let query_before = app.input.query().to_string();

    for row in [0, 1] {
        let mouse = create_mouse_event(1, row);
        handle_click(&mut app, Some(Region::AiWindow), mouse);
    }

    assert_eq!(
        app.input.query(),
        query_before,
        "clicks on the border or top padding must not apply any suggestion"
    );
    assert_eq!(app.focus, Focus::InputField);
}

#[test]
fn test_click_ai_window_below_suggestions_does_nothing() {
    let mut app = setup_app();
    setup_ai_window_one_suggestion(&mut app, Some(POPUP));
    let query_before = app.input.query().to_string();

    // Content row 1 (screen row 3) has no suggestion.
    let mouse = create_mouse_event(1, 3);
    handle_click(&mut app, Some(Region::AiWindow), mouse);

    assert_eq!(app.input.query(), query_before);
    assert_eq!(app.focus, Focus::InputField);
}

#[test]
fn test_click_ai_window_no_layout_rect() {
    let mut app = setup_app();
    setup_ai_window_one_suggestion(&mut app, None);
    let query_before = app.input.query().to_string();

    let mouse = create_mouse_event(5, 3);
    handle_click(&mut app, Some(Region::AiWindow), mouse);

    assert_eq!(
        app.input.query(),
        query_before,
        "with no tracked ai_window rect the click must return without applying"
    );
}

#[test]
fn test_click_ai_window_hidden_does_nothing() {
    let mut app = setup_app();
    setup_ai_window_one_suggestion(&mut app, Some(POPUP));
    app.ai.visible = false;
    let query_before = app.input.query().to_string();

    handle_click(&mut app, Some(Region::AiWindow), create_mouse_event(1, 2));
    handle_click(&mut app, Some(Region::AiWindow), create_mouse_event(1, 7));

    assert_eq!(app.input.query(), query_before);
    assert_eq!(app.focus, Focus::InputField);
}

#[test]
fn test_click_chat_input_row_focuses_chat() {
    let mut app = setup_app();
    setup_ai_window_one_suggestion(&mut app, Some(POPUP));
    app.focus = Focus::InputField;
    let query_before = app.input.query().to_string();

    let mouse = create_mouse_event(5, 7);
    handle_click(&mut app, Some(Region::AiWindow), mouse);

    assert_eq!(app.focus, Focus::AiChat);
    assert_eq!(
        app.input.query(),
        query_before,
        "clicking the chat input must not apply a suggestion"
    );
}

#[test]
fn test_click_chat_separator_row_focuses_chat() {
    let mut app = setup_app();
    setup_ai_window_one_suggestion(&mut app, Some(POPUP));
    app.focus = Focus::InputField;

    let mouse = create_mouse_event(5, 6);
    handle_click(&mut app, Some(Region::AiWindow), mouse);

    assert_eq!(app.focus, Focus::AiChat);
}

#[test]
fn test_click_chat_input_row_focuses_chat_without_suggestions() {
    let mut app = setup_app();
    app.ai.visible = true;
    app.ai.suggestions = vec![];
    app.layout_regions.ai_window = Some(POPUP);
    app.focus = Focus::InputField;

    handle_click(&mut app, Some(Region::AiWindow), create_mouse_event(5, 7));

    assert_eq!(
        app.focus,
        Focus::AiChat,
        "the chat input is clickable even when nothing has been suggested yet"
    );
}

#[test]
fn test_click_chat_input_row_from_results_pane_focuses_chat() {
    let mut app = setup_app();
    setup_ai_window_one_suggestion(&mut app, Some(POPUP));
    app.focus = Focus::ResultsPane;

    handle_click(&mut app, Some(Region::AiWindow), create_mouse_event(5, 7));

    assert_eq!(app.focus, Focus::AiChat);
    assert!(app.ai.visible);
}
