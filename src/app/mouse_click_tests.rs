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

// Tests for history popup click handling

/// Layout: popup origin (0, 0), width 80, list height = 3 entries + 4 = 7,
/// search height 3, total 10. Entries occupy rows 2..5 with the newest entry
/// at row 4 (display index 0) and the oldest at row 2 (display index 2).
fn setup_history_popup(app: &mut crate::app::App) {
    use ratatui::layout::Rect;

    app.history.add_entry_in_memory(".oldest");
    app.history.add_entry_in_memory(".middle");
    app.history.add_entry_in_memory(".newest");
    app.history.open(None);

    app.layout_regions.history_popup = Some(Rect::new(0, 0, 80, 10));
}

#[test]
fn test_click_history_popup_x_button_deletes_entry() {
    let mut app = setup_app();
    setup_history_popup(&mut app);

    // Newest (.newest, display index 0) is rendered on row 4.
    // The [✕] button column occupies the last 5 cells of the inner area:
    // x ∈ [80 - 6, 80 - 1) = [74, 79).
    let mouse = create_mouse_event(76, 4);
    handle_click(&mut app, Some(Region::HistoryPopup), mouse);

    assert!(app.history.is_visible());
    assert_eq!(app.history.total_count(), 2);
    assert_eq!(app.history.entry_at_display_index(0), Some(".middle"));
}

#[test]
fn test_click_history_popup_x_button_on_oldest_row() {
    let mut app = setup_app();
    setup_history_popup(&mut app);

    // Oldest (.oldest, display index 2) is rendered on row 2.
    let mouse = create_mouse_event(76, 2);
    handle_click(&mut app, Some(Region::HistoryPopup), mouse);

    assert_eq!(app.history.total_count(), 2);
    assert_eq!(app.history.entry_at_display_index(0), Some(".newest"));
    assert_eq!(app.history.entry_at_display_index(1), Some(".middle"));
}

#[test]
fn test_click_history_popup_row_selects_entry() {
    let mut app = setup_app();
    setup_history_popup(&mut app);

    // Row 3 holds .middle (display index 1). Click well to the left of the
    // [✕] column so the row-select branch handles it.
    let mouse = create_mouse_event(10, 3);
    handle_click(&mut app, Some(Region::HistoryPopup), mouse);

    assert!(!app.history.is_visible());
    assert_eq!(app.query(), ".middle");
}

#[test]
fn test_click_history_popup_top_padding_row_does_nothing() {
    let mut app = setup_app();
    setup_history_popup(&mut app);

    // Row 1 inside the popup is the top padding row (no entry).
    let mouse = create_mouse_event(10, 1);
    handle_click(&mut app, Some(Region::HistoryPopup), mouse);

    assert!(app.history.is_visible());
    assert_eq!(app.history.total_count(), 3);
}

#[test]
fn test_click_history_popup_closes_when_last_entry_deleted() {
    let mut app = setup_app();
    app.history.add_entry_in_memory(".only");
    app.history.open(None);
    app.layout_regions.history_popup = Some(ratatui::layout::Rect::new(0, 0, 80, 8));

    // Single-entry popup: list height = 1 + 4 = 5, total 8.
    // The only entry (display index 0) is rendered on row 2.
    let mouse = create_mouse_event(76, 2);
    handle_click(&mut app, Some(Region::HistoryPopup), mouse);

    assert!(!app.history.is_visible());
    assert_eq!(app.history.total_count(), 0);
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
