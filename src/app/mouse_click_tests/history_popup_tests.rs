//! Tests for clicks inside the history popup.

use super::*;

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
