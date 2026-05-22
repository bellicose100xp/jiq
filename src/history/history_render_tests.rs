//! Tests for history/history_render hit testing helpers

use super::{delete_button_at, display_index_at};
use crate::test_utils::test_helpers::test_app;
use ratatui::layout::Rect;

fn app_with_open_history(entries: &[&str]) -> crate::app::App {
    let mut app = test_app(r#"{"test": "data"}"#);
    for entry in entries {
        app.history.add_entry_in_memory(entry);
    }
    app.history.open(None);
    app.layout_regions.history_popup = Some(Rect::new(0, 0, 80, 10));
    app
}

#[test]
fn test_display_index_at_returns_none_when_popup_not_tracked() {
    let mut app = test_app(r#"{}"#);
    app.history.add_entry_in_memory(".foo");
    app.history.open(None);

    assert_eq!(display_index_at(&app, 10, 5), None);
}

#[test]
fn test_display_index_at_newest_at_bottom_row() {
    let app = app_with_open_history(&[".oldest", ".middle", ".newest"]);

    // Row 4 is the bottom entry row (newest, display index 0).
    assert_eq!(display_index_at(&app, 10, 4), Some(0));
}

#[test]
fn test_display_index_at_oldest_at_top_entry_row() {
    let app = app_with_open_history(&[".oldest", ".middle", ".newest"]);

    // Row 2 is the topmost entry row (oldest, display index 2).
    assert_eq!(display_index_at(&app, 10, 2), Some(2));
}

#[test]
fn test_display_index_at_middle_row() {
    let app = app_with_open_history(&[".oldest", ".middle", ".newest"]);

    assert_eq!(display_index_at(&app, 10, 3), Some(1));
}

#[test]
fn test_display_index_at_top_padding_row_returns_none() {
    let app = app_with_open_history(&[".only"]);

    // Single-entry popup: top padding row is row 1.
    assert_eq!(display_index_at(&app, 10, 1), None);
}

#[test]
fn test_display_index_at_bottom_padding_row_returns_none() {
    let app = app_with_open_history(&[".oldest", ".middle", ".newest"]);

    // Bottom padding row is row 5 (entries fill rows 2..=4).
    assert_eq!(display_index_at(&app, 10, 5), None);
}

#[test]
fn test_display_index_at_outside_horizontal_bounds() {
    let app = app_with_open_history(&[".only"]);

    assert_eq!(display_index_at(&app, 200, 2), None);
}

#[test]
fn test_display_index_at_empty_filter_returns_none() {
    let mut app = app_with_open_history(&[".foo", ".bar"]);
    app.history.search_textarea_mut().insert_str("nomatch");
    app.history.on_search_input_changed();
    assert_eq!(app.history.filtered_count(), 0);

    assert_eq!(display_index_at(&app, 10, 2), None);
}

#[test]
fn test_delete_button_at_within_button_column() {
    let app = app_with_open_history(&[".oldest", ".middle", ".newest"]);

    // Button column is x ∈ [80 - 6, 80 - 1) = [74, 79).
    assert_eq!(delete_button_at(&app, 76, 4), Some(0));
}

#[test]
fn test_delete_button_at_just_inside_left_edge() {
    let app = app_with_open_history(&[".only"]);

    assert_eq!(delete_button_at(&app, 74, 2), Some(0));
}

#[test]
fn test_delete_button_at_left_of_column_returns_none() {
    let app = app_with_open_history(&[".only"]);

    assert_eq!(delete_button_at(&app, 73, 2), None);
}

#[test]
fn test_delete_button_at_right_border_returns_none() {
    let app = app_with_open_history(&[".only"]);

    // x = 79 is the right border column itself.
    assert_eq!(delete_button_at(&app, 79, 2), None);
}

#[test]
fn test_delete_button_at_padding_row_returns_none() {
    let app = app_with_open_history(&[".only"]);

    // Top padding row, even on the button column.
    assert_eq!(delete_button_at(&app, 76, 1), None);
}
