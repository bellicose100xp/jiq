//! Tests for mouse hover handling

use ratatui::crossterm::event::{MouseEvent, MouseEventKind};
use ratatui::layout::Rect;

use super::*;
use crate::ai::suggestion::{Suggestion, SuggestionType};
use crate::test_utils::test_helpers::test_app;

fn create_test_app() -> App {
    test_app(r#"{"test": "data"}"#)
}

fn create_mouse_event(column: u16, row: u16) -> MouseEvent {
    MouseEvent {
        kind: MouseEventKind::Moved,
        column,
        row,
        modifiers: ratatui::crossterm::event::KeyModifiers::NONE,
    }
}

#[test]
fn test_hover_outside_ai_window_clears_hover() {
    let mut app = create_test_app();
    app.ai.visible = true;
    app.ai.suggestions = vec![Suggestion {
        query: ".test".to_string(),
        suggestion_type: SuggestionType::Fix,
        description: String::new(),
    }];
    app.ai.selection.set_hovered(Some(0));

    let mouse = create_mouse_event(0, 0);
    handle_hover(&mut app, None, mouse);

    assert!(app.ai.selection.get_hovered().is_none());
}

#[test]
fn test_hover_ai_window_no_suggestions() {
    let mut app = create_test_app();
    app.ai.visible = true;
    app.ai.suggestions = vec![];
    app.layout_regions.ai_window = Some(Rect::new(10, 5, 30, 10));

    let mouse = create_mouse_event(15, 7);
    handle_hover(&mut app, Some(Region::AiWindow), mouse);

    assert!(app.ai.selection.get_hovered().is_none());
}

#[test]
fn test_hover_ai_window_not_visible() {
    let mut app = create_test_app();
    app.ai.visible = false;
    app.ai.suggestions = vec![Suggestion {
        query: ".test".to_string(),
        suggestion_type: SuggestionType::Fix,
        description: String::new(),
    }];
    app.layout_regions.ai_window = Some(Rect::new(10, 5, 30, 10));

    let mouse = create_mouse_event(15, 7);
    handle_hover(&mut app, Some(Region::AiWindow), mouse);

    assert!(app.ai.selection.get_hovered().is_none());
}

#[test]
fn test_hover_ai_window_no_region() {
    let mut app = create_test_app();
    app.ai.visible = true;
    app.ai.suggestions = vec![Suggestion {
        query: ".test".to_string(),
        suggestion_type: SuggestionType::Fix,
        description: String::new(),
    }];
    app.layout_regions.ai_window = None;

    let mouse = create_mouse_event(15, 7);
    handle_hover(&mut app, Some(Region::AiWindow), mouse);

    assert!(app.ai.selection.get_hovered().is_none());
}

#[test]
fn test_hover_on_border_clears_hover() {
    let mut app = create_test_app();
    app.ai.visible = true;
    app.ai.suggestions = vec![Suggestion {
        query: ".test".to_string(),
        suggestion_type: SuggestionType::Fix,
        description: String::new(),
    }];
    app.layout_regions.ai_window = Some(Rect::new(10, 5, 30, 10));
    app.ai.selection.update_layout(vec![3], 8);
    app.ai.selection.set_hovered(Some(0));

    let mouse = create_mouse_event(10, 5);
    handle_hover(&mut app, Some(Region::AiWindow), mouse);

    assert!(app.ai.selection.get_hovered().is_none());
}

#[test]
fn test_hover_results_pane_clears_ai_hover() {
    let mut app = create_test_app();
    app.ai.selection.set_hovered(Some(0));

    let mouse = create_mouse_event(5, 5);
    handle_hover(&mut app, Some(Region::ResultsPane), mouse);

    assert!(app.ai.selection.get_hovered().is_none());
}
