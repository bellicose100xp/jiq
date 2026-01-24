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

#[test]
fn test_hover_snippet_list_updates_hovered_index() {
    let mut app = create_test_app();
    app.snippets.open();
    app.snippets.set_snippets(vec![
        crate::snippets::Snippet {
            name: "test1".to_string(),
            query: ".test1".to_string(),
            description: None,
        },
        crate::snippets::Snippet {
            name: "test2".to_string(),
            query: ".test2".to_string(),
            description: None,
        },
    ]);
    app.layout_regions.snippet_list = Some(Rect::new(0, 0, 50, 10));

    let mouse = create_mouse_event(5, 2);
    handle_hover(&mut app, Some(Region::SnippetList), mouse);

    assert_eq!(app.snippets.get_hovered(), Some(1));
}

#[test]
fn test_hover_snippet_list_on_border_clears_hover() {
    let mut app = create_test_app();
    app.snippets.open();
    app.snippets.set_snippets(vec![crate::snippets::Snippet {
        name: "test1".to_string(),
        query: ".test1".to_string(),
        description: None,
    }]);
    app.snippets.set_hovered(Some(0));
    app.layout_regions.snippet_list = Some(Rect::new(10, 5, 30, 10));

    let mouse = create_mouse_event(10, 5);
    handle_hover(&mut app, Some(Region::SnippetList), mouse);

    assert!(app.snippets.get_hovered().is_none());
}

#[test]
fn test_leaving_snippet_list_clears_hover() {
    let mut app = create_test_app();
    app.snippets.open();
    app.snippets.set_snippets(vec![crate::snippets::Snippet {
        name: "test1".to_string(),
        query: ".test1".to_string(),
        description: None,
    }]);
    app.snippets.set_hovered(Some(0));

    let mouse = create_mouse_event(5, 5);
    handle_hover(&mut app, Some(Region::ResultsPane), mouse);

    assert!(app.snippets.get_hovered().is_none());
}

#[test]
fn test_hover_snippet_list_when_not_visible() {
    let mut app = create_test_app();
    app.snippets.set_snippets(vec![crate::snippets::Snippet {
        name: "test1".to_string(),
        query: ".test1".to_string(),
        description: None,
    }]);
    app.layout_regions.snippet_list = Some(Rect::new(0, 0, 50, 10));

    let mouse = create_mouse_event(5, 2);
    handle_hover(&mut app, Some(Region::SnippetList), mouse);

    assert!(app.snippets.get_hovered().is_none());
}

#[test]
fn test_hover_snippet_list_no_region() {
    let mut app = create_test_app();
    app.snippets.open();
    app.snippets.set_snippets(vec![crate::snippets::Snippet {
        name: "test1".to_string(),
        query: ".test1".to_string(),
        description: None,
    }]);
    app.layout_regions.snippet_list = None;

    let mouse = create_mouse_event(5, 2);
    handle_hover(&mut app, Some(Region::SnippetList), mouse);

    assert!(app.snippets.get_hovered().is_none());
}

#[test]
fn test_hover_help_popup_tab_bar() {
    use crate::help::HelpTab;

    let mut app = create_test_app();
    app.help.visible = true;
    // Popup at (10, 5), width 70, height 20
    app.layout_regions.help_popup = Some(Rect::new(10, 5, 70, 20));

    // Tab bar is at y = 6 (popup y + 1 for border)
    // With Global active: [1:Global] = 10 chars at x=11
    let mouse = create_mouse_event(15, 6);
    handle_hover(&mut app, Some(Region::HelpPopup), mouse);

    assert_eq!(app.help.get_hovered_tab(), Some(HelpTab::Global));
}

#[test]
fn test_hover_help_popup_second_tab() {
    use crate::help::HelpTab;

    let mut app = create_test_app();
    app.help.visible = true;
    app.layout_regions.help_popup = Some(Rect::new(10, 5, 70, 20));

    // With Global active: [1:Global] = 10 chars, divider = 3, so 2:Input starts at inner_x = 13
    // inner_x starts at popup_x + 1 = 11, so Input starts at column 24
    let mouse = create_mouse_event(24, 6);
    handle_hover(&mut app, Some(Region::HelpPopup), mouse);

    assert_eq!(app.help.get_hovered_tab(), Some(HelpTab::Input));
}

#[test]
fn test_hover_help_popup_on_divider_clears_hover() {
    let mut app = create_test_app();
    app.help.visible = true;
    app.layout_regions.help_popup = Some(Rect::new(10, 5, 70, 20));
    app.help.set_hovered_tab(Some(crate::help::HelpTab::Input));

    // Position 21 (inner x=10) is the divider after [1:Global] (10-12 are divider)
    let mouse = create_mouse_event(21, 6);
    handle_hover(&mut app, Some(Region::HelpPopup), mouse);

    assert_eq!(app.help.get_hovered_tab(), None);
}

#[test]
fn test_hover_help_popup_below_tab_bar_clears_hover() {
    use crate::help::HelpTab;

    let mut app = create_test_app();
    app.help.visible = true;
    app.layout_regions.help_popup = Some(Rect::new(10, 5, 70, 20));
    app.help.set_hovered_tab(Some(HelpTab::Input));

    // Hover on content area (y = 8, below tab bar at y = 6)
    let mouse = create_mouse_event(15, 8);
    handle_hover(&mut app, Some(Region::HelpPopup), mouse);

    assert_eq!(app.help.get_hovered_tab(), None);
}

#[test]
fn test_hover_help_popup_not_visible() {
    let mut app = create_test_app();
    app.help.visible = false;
    app.layout_regions.help_popup = Some(Rect::new(10, 5, 70, 20));

    let mouse = create_mouse_event(15, 6);
    handle_hover(&mut app, Some(Region::HelpPopup), mouse);

    assert_eq!(app.help.get_hovered_tab(), None);
}

#[test]
fn test_hover_help_popup_no_region() {
    let mut app = create_test_app();
    app.help.visible = true;
    app.layout_regions.help_popup = None;

    let mouse = create_mouse_event(15, 6);
    handle_hover(&mut app, Some(Region::HelpPopup), mouse);

    assert_eq!(app.help.get_hovered_tab(), None);
}

#[test]
fn test_leaving_help_popup_clears_hover() {
    use crate::help::HelpTab;

    let mut app = create_test_app();
    app.help.visible = true;
    app.help.set_hovered_tab(Some(HelpTab::Input));

    let mouse = create_mouse_event(5, 5);
    handle_hover(&mut app, Some(Region::ResultsPane), mouse);

    assert_eq!(app.help.get_hovered_tab(), None);
}
