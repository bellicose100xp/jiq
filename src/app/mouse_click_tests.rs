//! Tests for mouse click handling

use ratatui::crossterm::event::{KeyModifiers, MouseButton, MouseEvent, MouseEventKind};

use crate::app::Focus;
use crate::editor::EditorMode;
use crate::layout::Region;
use crate::test_utils::test_helpers::test_app;

use super::handle_click;

#[path = "mouse_click_tests/ai_window_tests.rs"]
mod ai_window_tests;
#[path = "mouse_click_tests/history_popup_tests.rs"]
mod history_popup_tests;

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
fn test_click_results_pane_confirms_search_when_unconfirmed() {
    let mut app = setup_app();
    app.search.open();
    assert!(!app.search.is_confirmed());
    let mouse = create_mouse_event(10, 10);

    handle_click(&mut app, Some(Region::ResultsPane), mouse);

    assert!(app.search.is_confirmed());
    assert!(app.search.is_visible());
}

#[test]
fn test_click_results_pane_does_not_unconfirm_when_already_confirmed() {
    let mut app = setup_app();
    app.search.open();
    app.search.confirm();
    assert!(app.search.is_confirmed());
    let mouse = create_mouse_event(10, 10);

    handle_click(&mut app, Some(Region::ResultsPane), mouse);

    assert!(app.search.is_confirmed());
    assert!(app.search.is_visible());
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
fn test_click_input_field_unfocused_does_not_move_cursor() {
    let mut app = setup_app();
    app.focus = Focus::ResultsPane;
    app.input.textarea.insert_str("abcdefghijklmnop");
    app.input
        .textarea
        .move_cursor(tui_textarea::CursorMove::Head);
    app.input.scroll_offset = 0;
    app.layout_regions.input_field = Some(ratatui::layout::Rect::new(0, 0, 30, 3));
    let initial_cursor = app.input.textarea.cursor().1;

    // Click at column 10 - but since unfocused, cursor should NOT move
    let mouse = create_mouse_event(10, 1);
    handle_click(&mut app, Some(Region::InputField), mouse);

    assert_eq!(app.focus, Focus::InputField, "Should focus input field");
    assert_eq!(
        app.input.textarea.cursor().1,
        initial_cursor,
        "Cursor should NOT move when focusing from unfocused state"
    );
}

#[test]
fn test_click_input_field_when_already_focused_does_not_change_mode() {
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
fn test_click_input_field_positions_cursor() {
    let mut app = setup_app();
    app.focus = Focus::InputField;
    app.input.textarea.insert_str("abcdefghijklmnop");
    app.input
        .textarea
        .move_cursor(tui_textarea::CursorMove::Head);
    app.input.scroll_offset = 0;
    app.layout_regions.input_field = Some(ratatui::layout::Rect::new(0, 0, 30, 3));

    // Click at column 6 (inner x starts at 1, so column 6 means position 5)
    let mouse = create_mouse_event(6, 1);
    handle_click(&mut app, Some(Region::InputField), mouse);

    assert_eq!(
        app.input.textarea.cursor().1,
        5,
        "Cursor should be at position 5"
    );
}

#[test]
fn test_click_input_field_with_scroll_offset() {
    let mut app = setup_app();
    app.focus = Focus::InputField;
    app.input.textarea.insert_str("abcdefghijklmnopqrstuvwxyz");
    app.input
        .textarea
        .move_cursor(tui_textarea::CursorMove::Head);
    app.input.scroll_offset = 10;
    app.layout_regions.input_field = Some(ratatui::layout::Rect::new(0, 0, 20, 3));

    // Click at column 6 (inner x starts at 1, so relative position is 5)
    // With scroll offset 10, target should be 10 + 5 = 15
    let mouse = create_mouse_event(6, 1);
    handle_click(&mut app, Some(Region::InputField), mouse);

    assert_eq!(
        app.input.textarea.cursor().1,
        15,
        "Cursor should be at position 15 (scroll offset 10 + relative position 5)"
    );
}

#[test]
fn test_click_input_field_clamps_to_text_length() {
    let mut app = setup_app();
    app.focus = Focus::InputField;
    app.input.textarea.insert_str("short");
    app.input
        .textarea
        .move_cursor(tui_textarea::CursorMove::Head);
    app.input.scroll_offset = 0;
    app.layout_regions.input_field = Some(ratatui::layout::Rect::new(0, 0, 30, 3));

    // Click at position beyond text length
    let mouse = create_mouse_event(20, 1);
    handle_click(&mut app, Some(Region::InputField), mouse);

    assert_eq!(
        app.input.textarea.cursor().1,
        5,
        "Cursor should be clamped to text length (5)"
    );
}

#[test]
fn test_click_input_field_on_border_ignored() {
    let mut app = setup_app();
    app.focus = Focus::InputField;
    app.input.textarea.insert_str("abcdefghij");
    app.input
        .textarea
        .move_cursor(tui_textarea::CursorMove::Head);
    app.input.scroll_offset = 0;
    app.layout_regions.input_field = Some(ratatui::layout::Rect::new(5, 2, 20, 3));
    let initial_cursor = app.input.textarea.cursor().1;

    // Click on left border (column 5, which is the border x position)
    let mouse = create_mouse_event(5, 3);
    handle_click(&mut app, Some(Region::InputField), mouse);

    assert_eq!(
        app.input.textarea.cursor().1,
        initial_cursor,
        "Cursor should not change when clicking on border"
    );
}

#[test]
fn test_click_input_field_no_region_tracked() {
    let mut app = setup_app();
    app.focus = Focus::InputField;
    app.input.textarea.insert_str("abcdefghij");
    app.input
        .textarea
        .move_cursor(tui_textarea::CursorMove::Head);
    app.layout_regions.input_field = None;
    let initial_cursor = app.input.textarea.cursor().1;

    let mouse = create_mouse_event(10, 1);
    handle_click(&mut app, Some(Region::InputField), mouse);

    assert_eq!(
        app.input.textarea.cursor().1,
        initial_cursor,
        "Cursor should not change when input field region is not tracked"
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

#[test]
fn test_double_click_results_pane_drills_in() {
    use crate::test_utils::test_helpers::{execute_query_and_wait, test_app};

    let mut app = test_app(r#"{"a": {"b": 1}}"#);
    app.input.textarea.insert_str(".");
    execute_query_and_wait(&mut app);
    let total = app.results_line_count_u32();
    app.results_cursor.update_total_lines(total);
    app.layout_regions.results_pane = Some(ratatui::layout::Rect::new(0, 0, 40, 10));

    // First click positions the cursor on the inner-row 1 (the `"a"` row);
    // second click within the threshold is the double-click that drills in.
    let mouse = create_mouse_event(5, 2);
    handle_click(&mut app, Some(Region::ResultsPane), mouse);
    let query_before = app.input.query().to_string();
    handle_click(&mut app, Some(Region::ResultsPane), mouse);

    assert_ne!(
        app.input.query(),
        query_before,
        "double-click on results row should pipe-compose the row's path onto the query"
    );
}

#[test]
fn test_single_click_results_pane_does_not_drill() {
    use crate::test_utils::test_helpers::{execute_query_and_wait, test_app};

    let mut app = test_app(r#"{"a": {"b": 1}}"#);
    app.input.textarea.insert_str(".");
    execute_query_and_wait(&mut app);
    let total = app.results_line_count_u32();
    app.results_cursor.update_total_lines(total);
    app.layout_regions.results_pane = Some(ratatui::layout::Rect::new(0, 0, 40, 10));

    let original_query = app.input.query().to_string();
    let mouse = create_mouse_event(5, 2);
    handle_click(&mut app, Some(Region::ResultsPane), mouse);

    assert_eq!(
        app.input.query(),
        original_query,
        "a single click must only move the cursor — never drill"
    );
}

#[test]
fn test_double_click_autocomplete_accepts_suggestion() {
    use crate::autocomplete::autocomplete_state::{Suggestion, SuggestionType};

    let mut app = setup_app();
    app.focus = Focus::InputField;
    app.input.textarea.insert_str(".");
    app.autocomplete.update_suggestions(vec![
        Suggestion::new("name", SuggestionType::Field),
        Suggestion::new("age", SuggestionType::Field),
    ]);
    app.layout_regions.autocomplete = Some(ratatui::layout::Rect::new(0, 0, 30, 6));

    // Row 1 inside the popup border targets the first suggestion ("name").
    let mouse = create_mouse_event(5, 1);
    handle_click(&mut app, Some(Region::Autocomplete), mouse);
    handle_click(&mut app, Some(Region::Autocomplete), mouse);

    assert!(
        app.input.query().contains("name"),
        "double-click on a suggestion must insert it into the query, got `{}`",
        app.input.query()
    );
}

/// Helper: drill once via the `>` chord so the undo ring is non-empty.
/// Mirrors how `drill_back_round_trips` sets up state in
/// `results_events_tests.rs`.
fn push_one_drill(app: &mut crate::app::App) {
    use crate::test_utils::test_helpers::key;
    use ratatui::crossterm::event::KeyCode;
    app.focus = Focus::ResultsPane;
    app.input.textarea.insert_str(".");
    if let Some(qs) = app.query.as_mut() {
        qs.execute(".");
    }
    let total = app.results_line_count_u32();
    app.results_cursor.update_total_lines(total);
    app.results_cursor.move_to_line(1);
    app.handle_key_event(key(KeyCode::Char('>')));
}

#[test]
fn test_click_back_button_pops_undo_ring() {
    let mut app = test_app(r#"{"a": 1, "b": 2}"#);
    push_one_drill(&mut app);
    assert!(
        !app.query_undo.is_empty(),
        "precondition: ring is non-empty"
    );
    let drilled_query = app.input.query().to_string();
    assert_ne!(
        drilled_query, ".",
        "precondition: query was rewritten by `>`"
    );

    let mouse = create_mouse_event(5, 0);
    handle_click(&mut app, Some(Region::BackButton), mouse);

    assert_eq!(
        app.input.query(),
        ".",
        "back click should restore prior query"
    );
    assert!(app.query_undo.is_empty(), "back click should pop the ring");
}

#[test]
fn test_click_back_button_focuses_results_pane() {
    let mut app = test_app(r#"{"a": 1}"#);
    push_one_drill(&mut app);
    app.focus = Focus::InputField;

    let mouse = create_mouse_event(5, 0);
    handle_click(&mut app, Some(Region::BackButton), mouse);

    assert_eq!(app.focus, Focus::ResultsPane);
}

#[test]
fn test_click_back_button_with_empty_ring_notifies() {
    let mut app = setup_app();
    app.focus = Focus::ResultsPane;
    assert!(app.query_undo.is_empty());

    let mouse = create_mouse_event(5, 0);
    handle_click(&mut app, Some(Region::BackButton), mouse);

    assert_eq!(
        app.notification.current_message(),
        Some("Nothing to go back to"),
    );
}

#[test]
fn test_click_back_button_confirms_unconfirmed_search() {
    let mut app = test_app(r#"{"a": 1, "b": 2}"#);
    push_one_drill(&mut app);
    app.search.open();
    assert!(!app.search.is_confirmed());

    let mouse = create_mouse_event(5, 0);
    handle_click(&mut app, Some(Region::BackButton), mouse);

    assert!(app.search.is_confirmed());
    assert!(app.search.is_visible());
}

#[test]
fn test_single_click_autocomplete_only_highlights() {
    use crate::autocomplete::autocomplete_state::{Suggestion, SuggestionType};

    let mut app = setup_app();
    app.focus = Focus::InputField;
    app.input.textarea.insert_str(".");
    let original_query = app.input.query().to_string();
    app.autocomplete.update_suggestions(vec![
        Suggestion::new("name", SuggestionType::Field),
        Suggestion::new("age", SuggestionType::Field),
    ]);
    app.layout_regions.autocomplete = Some(ratatui::layout::Rect::new(0, 0, 30, 6));

    // Row 2 inside the popup border targets the second suggestion ("age").
    let mouse = create_mouse_event(5, 2);
    handle_click(&mut app, Some(Region::Autocomplete), mouse);

    assert_eq!(
        app.input.query(),
        original_query,
        "single click must not insert anything"
    );
    assert_eq!(
        app.autocomplete.selected_index(),
        1,
        "single click must highlight the clicked suggestion"
    );
}

// Tests for autocomplete click guard branches

#[test]
fn test_click_autocomplete_not_visible_does_nothing() {
    let mut app = setup_app();
    app.focus = Focus::InputField;
    // No suggestions pushed -> autocomplete is not visible.
    assert!(!app.autocomplete.is_visible());
    app.layout_regions.autocomplete = Some(ratatui::layout::Rect::new(0, 0, 30, 6));
    let before = app.autocomplete.selected_index();

    let mouse = create_mouse_event(5, 1);
    handle_click(&mut app, Some(Region::Autocomplete), mouse);

    assert_eq!(
        app.autocomplete.selected_index(),
        before,
        "an invisible autocomplete popup must ignore clicks"
    );
}

#[test]
fn test_click_autocomplete_no_layout_rect() {
    use crate::autocomplete::autocomplete_state::{Suggestion, SuggestionType};

    let mut app = setup_app();
    app.focus = Focus::InputField;
    app.autocomplete.update_suggestions(vec![
        Suggestion::new("name", SuggestionType::Field),
        Suggestion::new("age", SuggestionType::Field),
    ]);
    assert!(app.autocomplete.is_visible());
    app.layout_regions.autocomplete = None;
    let before = app.autocomplete.selected_index();

    let mouse = create_mouse_event(5, 1);
    handle_click(&mut app, Some(Region::Autocomplete), mouse);

    assert_eq!(
        app.autocomplete.selected_index(),
        before,
        "with no tracked autocomplete rect the click must not select"
    );
}

#[test]
fn test_click_autocomplete_out_of_vertical_bounds() {
    use crate::autocomplete::autocomplete_state::{Suggestion, SuggestionType};

    let mut app = setup_app();
    app.focus = Focus::InputField;
    app.autocomplete.update_suggestions(vec![
        Suggestion::new("name", SuggestionType::Field),
        Suggestion::new("age", SuggestionType::Field),
    ]);
    app.layout_regions.autocomplete = Some(ratatui::layout::Rect::new(0, 0, 30, 6));
    let before = app.autocomplete.selected_index();

    // Row 0 is the top border (< inner_y = 1).
    let mouse = create_mouse_event(5, 0);
    handle_click(&mut app, Some(Region::Autocomplete), mouse);

    assert_eq!(
        app.autocomplete.selected_index(),
        before,
        "a click above the inner row range must not select"
    );
}

#[test]
fn test_click_autocomplete_index_beyond_suggestions() {
    use crate::autocomplete::autocomplete_state::{Suggestion, SuggestionType};

    let mut app = setup_app();
    app.focus = Focus::InputField;
    app.autocomplete.update_suggestions(vec![
        Suggestion::new("name", SuggestionType::Field),
        Suggestion::new("age", SuggestionType::Field),
    ]);
    // Tall popup so row 3 is in-bounds but maps to visible_index 2 >= len 2.
    app.layout_regions.autocomplete = Some(ratatui::layout::Rect::new(0, 0, 30, 8));
    let before = app.autocomplete.selected_index();

    let mouse = create_mouse_event(5, 3);
    handle_click(&mut app, Some(Region::Autocomplete), mouse);

    assert_eq!(
        app.autocomplete.selected_index(),
        before,
        "a click on an in-bounds row past the last suggestion must not select"
    );
}

#[test]
fn test_click_results_pane_out_of_vertical_bounds_no_select() {
    use crate::results::cursor_state::SelectionMode;

    let mut app = setup_app();
    app.layout_regions.results_pane = Some(ratatui::layout::Rect::new(0, 0, 40, 10));

    // Row 0 is the top border (< inner_y = 1), so the cursor must not move.
    let mouse = create_mouse_event(5, 0);
    handle_click(&mut app, Some(Region::ResultsPane), mouse);

    assert_eq!(
        app.focus,
        Focus::ResultsPane,
        "clicking the results pane focuses it even when out of vertical bounds"
    );
    assert_eq!(
        app.results_cursor.cursor_line(),
        0,
        "an out-of-bounds row click must not move the cursor"
    );
    assert_eq!(
        app.results_cursor.mode(),
        SelectionMode::Normal,
        "an out-of-bounds row click must not trigger click_select (which enters Visual mode)"
    );
}

#[test]
fn test_click_history_popup_when_not_visible_does_nothing() {
    let mut app = setup_app();
    assert!(!app.history.is_visible());
    app.layout_regions.history_popup = Some(ratatui::layout::Rect::new(0, 0, 80, 10));
    let query_before = app.query().to_string();

    let mouse = create_mouse_event(10, 4);
    handle_click(&mut app, Some(Region::HistoryPopup), mouse);

    assert_eq!(
        app.history.total_count(),
        0,
        "an invisible history popup must not delete or alter entries"
    );
    assert_eq!(
        app.query(),
        query_before,
        "an invisible history popup click must not rewrite the query"
    );
}

#[test]
fn test_click_snippet_list_no_layout_rect() {
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
    app.layout_regions.snippet_list = None;
    assert_eq!(app.snippets.selected_index(), 0);

    let mouse = create_mouse_event(5, 2);
    handle_click(&mut app, Some(Region::SnippetList), mouse);

    assert_eq!(
        app.snippets.selected_index(),
        0,
        "with no tracked snippet_list rect the click must not change selection"
    );
}

#[test]
fn test_click_help_popup_column_out_of_horizontal_bounds() {
    use crate::help::HelpTab;

    let mut app = setup_app();
    app.help.visible = true;
    app.help.active_tab = HelpTab::Global;
    app.layout_regions.help_popup = Some(ratatui::layout::Rect::new(10, 5, 70, 20));

    // Tab-bar row is y = 6 (help_rect.y + 1). Column 9 is < inner_x = 11.
    let mouse = create_mouse_event(9, 6);
    handle_click(&mut app, Some(Region::HelpPopup), mouse);

    assert_eq!(
        app.help.active_tab,
        HelpTab::Global,
        "a tab-bar-row click left of the inner bounds must not change the active tab"
    );
}
