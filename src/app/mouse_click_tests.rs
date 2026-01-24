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
fn test_click_snippet_list_selects_snippet() {
    let mut app = setup_app();
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
    app.layout_regions.snippet_list = Some(ratatui::layout::Rect::new(0, 0, 50, 10));

    assert_eq!(app.snippets.selected_index(), 0);

    let mouse = create_mouse_event(5, 2);
    handle_click(&mut app, Some(Region::SnippetList), mouse);

    assert_eq!(app.snippets.selected_index(), 1);
}

#[test]
fn test_click_snippet_list_on_border_is_ignored() {
    let mut app = setup_app();
    app.snippets.open();
    app.snippets.set_snippets(vec![crate::snippets::Snippet {
        name: "test1".to_string(),
        query: ".test1".to_string(),
        description: None,
    }]);
    app.layout_regions.snippet_list = Some(ratatui::layout::Rect::new(10, 5, 30, 10));

    assert_eq!(app.snippets.selected_index(), 0);

    let mouse = create_mouse_event(10, 5);
    handle_click(&mut app, Some(Region::SnippetList), mouse);

    assert_eq!(app.snippets.selected_index(), 0);
}

#[test]
fn test_click_snippet_list_with_empty_list() {
    let mut app = setup_app();
    app.snippets.disable_persistence();
    app.snippets.open();
    app.layout_regions.snippet_list = Some(ratatui::layout::Rect::new(0, 0, 50, 10));

    let mouse = create_mouse_event(5, 2);
    handle_click(&mut app, Some(Region::SnippetList), mouse);

    assert_eq!(app.snippets.selected_index(), 0);
}

#[test]
fn test_click_snippet_list_in_non_browse_mode() {
    let mut app = setup_app();
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
    app.snippets.enter_create_mode(".test");
    app.layout_regions.snippet_list = Some(ratatui::layout::Rect::new(0, 0, 50, 10));

    assert_eq!(app.snippets.selected_index(), 0);

    let mouse = create_mouse_event(5, 2);
    handle_click(&mut app, Some(Region::SnippetList), mouse);

    assert_eq!(app.snippets.selected_index(), 0);
}

#[test]
fn test_click_snippet_list_when_not_visible() {
    let mut app = setup_app();
    app.snippets.set_snippets(vec![crate::snippets::Snippet {
        name: "test1".to_string(),
        query: ".test1".to_string(),
        description: None,
    }]);
    app.layout_regions.snippet_list = Some(ratatui::layout::Rect::new(0, 0, 50, 10));

    let mouse = create_mouse_event(5, 2);
    handle_click(&mut app, Some(Region::SnippetList), mouse);

    assert_eq!(app.snippets.selected_index(), 0);
}

#[test]
fn test_click_outside_help_popup_dismisses_it() {
    let mut app = setup_app();
    app.help.visible = true;
    let mouse = create_mouse_event(10, 10);

    handle_click(&mut app, Some(Region::ResultsPane), mouse);

    assert!(!app.help.visible, "Help popup should be dismissed");
}

#[test]
fn test_click_inside_help_popup_does_not_dismiss() {
    let mut app = setup_app();
    app.help.visible = true;
    let mouse = create_mouse_event(10, 10);

    handle_click(&mut app, Some(Region::HelpPopup), mouse);

    assert!(app.help.visible, "Help popup should remain visible");
}

#[test]
fn test_click_outside_error_overlay_dismisses_it() {
    let mut app = setup_app();
    app.error_overlay_visible = true;
    let mouse = create_mouse_event(10, 10);

    handle_click(&mut app, Some(Region::ResultsPane), mouse);

    assert!(
        !app.error_overlay_visible,
        "Error overlay should be dismissed"
    );
}

#[test]
fn test_click_inside_error_overlay_does_not_dismiss() {
    let mut app = setup_app();
    app.error_overlay_visible = true;
    let mouse = create_mouse_event(10, 10);

    handle_click(&mut app, Some(Region::ErrorOverlay), mouse);

    assert!(
        app.error_overlay_visible,
        "Error overlay should remain visible"
    );
}

#[test]
fn test_dismiss_help_consumes_click() {
    let mut app = setup_app();
    app.focus = Focus::InputField;
    app.help.visible = true;
    let mouse = create_mouse_event(10, 10);

    handle_click(&mut app, Some(Region::ResultsPane), mouse);

    assert_eq!(
        app.focus,
        Focus::InputField,
        "Focus should not change when dismissing help popup"
    );
}

#[test]
fn test_dismiss_error_overlay_consumes_click() {
    let mut app = setup_app();
    app.focus = Focus::InputField;
    app.error_overlay_visible = true;
    let mouse = create_mouse_event(10, 10);

    handle_click(&mut app, Some(Region::ResultsPane), mouse);

    assert_eq!(
        app.focus,
        Focus::InputField,
        "Focus should not change when dismissing error overlay"
    );
}

#[test]
fn test_click_help_popup_tab_changes_active_tab() {
    use crate::help::HelpTab;

    let mut app = setup_app();
    app.help.visible = true;
    app.help.active_tab = HelpTab::Global;
    app.layout_regions.help_popup = Some(ratatui::layout::Rect::new(10, 5, 70, 20));

    // With Global active: [1:Global] = 10 chars, divider = 3, so 2:Input starts at inner_x = 13
    // inner_x starts at popup_x + 1 = 11, so Input starts at column 24
    let mouse = create_mouse_event(24, 6);
    handle_click(&mut app, Some(Region::HelpPopup), mouse);

    assert_eq!(app.help.active_tab, HelpTab::Input);
}

#[test]
fn test_click_help_popup_same_tab_stays_active() {
    use crate::help::HelpTab;

    let mut app = setup_app();
    app.help.visible = true;
    app.help.active_tab = HelpTab::Global;
    app.layout_regions.help_popup = Some(ratatui::layout::Rect::new(10, 5, 70, 20));

    // Click on [1:Global] at column 15, y = 6
    let mouse = create_mouse_event(15, 6);
    handle_click(&mut app, Some(Region::HelpPopup), mouse);

    assert_eq!(app.help.active_tab, HelpTab::Global);
}

#[test]
fn test_click_help_popup_on_divider_no_change() {
    use crate::help::HelpTab;

    let mut app = setup_app();
    app.help.visible = true;
    app.help.active_tab = HelpTab::Global;
    app.layout_regions.help_popup = Some(ratatui::layout::Rect::new(10, 5, 70, 20));

    // Click on divider at column 21 (inner_x = 10 which is divider after [1:Global])
    let mouse = create_mouse_event(21, 6);
    handle_click(&mut app, Some(Region::HelpPopup), mouse);

    assert_eq!(app.help.active_tab, HelpTab::Global);
}

#[test]
fn test_click_help_popup_below_tab_bar_no_change() {
    use crate::help::HelpTab;

    let mut app = setup_app();
    app.help.visible = true;
    app.help.active_tab = HelpTab::Global;
    app.layout_regions.help_popup = Some(ratatui::layout::Rect::new(10, 5, 70, 20));

    // Click on content area (y = 8, below tab bar at y = 6)
    let mouse = create_mouse_event(22, 8);
    handle_click(&mut app, Some(Region::HelpPopup), mouse);

    assert_eq!(app.help.active_tab, HelpTab::Global);
}

#[test]
fn test_click_help_popup_not_visible_no_change() {
    use crate::help::HelpTab;

    let mut app = setup_app();
    app.help.visible = false;
    app.help.active_tab = HelpTab::Global;
    app.layout_regions.help_popup = Some(ratatui::layout::Rect::new(10, 5, 70, 20));

    let mouse = create_mouse_event(22, 6);
    handle_click(&mut app, Some(Region::HelpPopup), mouse);

    assert_eq!(app.help.active_tab, HelpTab::Global);
}

#[test]
fn test_click_help_popup_no_region_no_change() {
    use crate::help::HelpTab;

    let mut app = setup_app();
    app.help.visible = true;
    app.help.active_tab = HelpTab::Global;
    app.layout_regions.help_popup = None;

    let mouse = create_mouse_event(22, 6);
    handle_click(&mut app, Some(Region::HelpPopup), mouse);

    assert_eq!(app.help.active_tab, HelpTab::Global);
}
