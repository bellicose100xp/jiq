use super::*;
use std::sync::Arc;

#[test]
fn test_new_cursor_state() {
    let cursor = CursorState::new();
    assert_eq!(cursor.cursor_line(), 0);
    assert_eq!(cursor.mode(), SelectionMode::Normal);
    assert!(!cursor.is_visual_mode());
    assert_eq!(cursor.hovered_line(), None);
    assert_eq!(cursor.total_lines(), 0);
}

#[test]
fn test_move_up() {
    let mut cursor = CursorState::new();
    cursor.update_total_lines(100);
    cursor.move_to_line(50);

    cursor.move_up(1);
    assert_eq!(cursor.cursor_line(), 49);

    cursor.move_up(10);
    assert_eq!(cursor.cursor_line(), 39);

    cursor.move_up(100);
    assert_eq!(cursor.cursor_line(), 0);
}

#[test]
fn test_move_down() {
    let mut cursor = CursorState::new();
    cursor.update_total_lines(100);

    cursor.move_down(1);
    assert_eq!(cursor.cursor_line(), 1);

    cursor.move_down(10);
    assert_eq!(cursor.cursor_line(), 11);

    cursor.move_down(1000);
    assert_eq!(cursor.cursor_line(), 99);
}

#[test]
fn test_move_to_first_last() {
    let mut cursor = CursorState::new();
    cursor.update_total_lines(100);
    cursor.move_to_line(50);

    cursor.move_to_first();
    assert_eq!(cursor.cursor_line(), 0);

    cursor.move_to_last();
    assert_eq!(cursor.cursor_line(), 99);
}

#[test]
fn test_move_to_line() {
    let mut cursor = CursorState::new();
    cursor.update_total_lines(100);

    cursor.move_to_line(50);
    assert_eq!(cursor.cursor_line(), 50);

    cursor.move_to_line(200);
    assert_eq!(cursor.cursor_line(), 99);
}

#[test]
fn test_update_total_lines_clamps_cursor() {
    let mut cursor = CursorState::new();
    cursor.update_total_lines(100);
    cursor.move_to_line(90);

    cursor.update_total_lines(50);
    assert_eq!(cursor.cursor_line(), 49);
}

#[test]
fn test_visual_mode_enter_exit() {
    let mut cursor = CursorState::new();
    cursor.update_total_lines(100);
    cursor.move_to_line(25);

    cursor.enter_visual_mode();
    assert!(cursor.is_visual_mode());
    assert_eq!(cursor.selection_range(), (25, 25));

    cursor.move_down(10);
    assert_eq!(cursor.selection_range(), (25, 35));

    cursor.exit_visual_mode();
    assert!(!cursor.is_visual_mode());
}

#[test]
fn test_visual_mode_toggle() {
    let mut cursor = CursorState::new();

    cursor.toggle_visual_mode();
    assert!(cursor.is_visual_mode());

    cursor.toggle_visual_mode();
    assert!(!cursor.is_visual_mode());
}

#[test]
fn test_selection_range_normal_mode() {
    let mut cursor = CursorState::new();
    cursor.update_total_lines(100);
    cursor.move_to_line(50);

    let (start, end) = cursor.selection_range();
    assert_eq!(start, 50);
    assert_eq!(end, 50);
}

#[test]
fn test_selection_range_visual_mode_down() {
    let mut cursor = CursorState::new();
    cursor.update_total_lines(100);
    cursor.move_to_line(25);
    cursor.enter_visual_mode();
    cursor.move_down(10);

    let (start, end) = cursor.selection_range();
    assert_eq!(start, 25);
    assert_eq!(end, 35);
}

#[test]
fn test_selection_range_visual_mode_up() {
    let mut cursor = CursorState::new();
    cursor.update_total_lines(100);
    cursor.move_to_line(50);
    cursor.enter_visual_mode();
    cursor.move_up(10);

    let (start, end) = cursor.selection_range();
    assert_eq!(start, 40);
    assert_eq!(end, 50);
}

#[test]
fn test_is_line_selected() {
    let mut cursor = CursorState::new();
    cursor.update_total_lines(100);
    cursor.move_to_line(25);
    cursor.enter_visual_mode();
    cursor.move_down(10);

    assert!(cursor.is_line_selected(25));
    assert!(cursor.is_line_selected(30));
    assert!(cursor.is_line_selected(35));
    assert!(!cursor.is_line_selected(24));
    assert!(!cursor.is_line_selected(36));
}

#[test]
fn test_is_line_selected_normal_mode() {
    let mut cursor = CursorState::new();
    cursor.update_total_lines(100);
    cursor.move_to_line(50);

    assert!(!cursor.is_line_selected(50));
    assert!(!cursor.is_line_selected(49));
}

#[test]
fn test_is_cursor_line() {
    let mut cursor = CursorState::new();
    cursor.update_total_lines(100);
    cursor.move_to_line(50);

    assert!(cursor.is_cursor_line(50));
    assert!(!cursor.is_cursor_line(49));
    assert!(!cursor.is_cursor_line(51));
}

#[test]
fn test_hover() {
    let mut cursor = CursorState::new();

    cursor.set_hovered(Some(10));
    assert_eq!(cursor.hovered_line(), Some(10));

    cursor.clear_hover();
    assert_eq!(cursor.hovered_line(), None);
}

#[test]
fn test_reset() {
    let mut cursor = CursorState::new();
    cursor.update_total_lines(100);
    cursor.move_to_line(50);
    cursor.enter_visual_mode();
    cursor.move_down(10);
    cursor.set_hovered(Some(30));

    cursor.reset();
    assert_eq!(cursor.cursor_line(), 0);
    assert!(!cursor.is_visual_mode());
    assert_eq!(cursor.hovered_line(), None);
}

#[test]
fn test_click_select() {
    let mut cursor = CursorState::new();
    cursor.update_total_lines(100);

    cursor.click_select(50);
    assert_eq!(cursor.cursor_line(), 50);
    assert!(cursor.is_visual_mode());
    assert_eq!(cursor.selection_range(), (50, 50));
}

#[test]
fn test_click_select_clamps() {
    let mut cursor = CursorState::new();
    cursor.update_total_lines(100);

    cursor.click_select(200);
    assert_eq!(cursor.cursor_line(), 99);
}

#[test]
fn test_drag_extend() {
    let mut cursor = CursorState::new();
    cursor.update_total_lines(100);
    cursor.click_select(25);

    cursor.drag_extend(50);
    assert_eq!(cursor.cursor_line(), 50);
    assert_eq!(cursor.selection_range(), (25, 50));

    cursor.drag_extend(10);
    assert_eq!(cursor.cursor_line(), 10);
    assert_eq!(cursor.selection_range(), (10, 25));
}

#[test]
fn test_drag_extend_clamps() {
    let mut cursor = CursorState::new();
    cursor.update_total_lines(100);
    cursor.click_select(25);

    cursor.drag_extend(200);
    assert_eq!(cursor.cursor_line(), 99);
}

#[test]
fn test_drag_extend_ignored_in_normal_mode() {
    let mut cursor = CursorState::new();
    cursor.update_total_lines(100);
    cursor.move_to_line(25);

    cursor.drag_extend(50);
    assert_eq!(cursor.cursor_line(), 25);
}

#[test]
fn test_selected_line_count() {
    let mut cursor = CursorState::new();
    cursor.update_total_lines(100);

    assert_eq!(cursor.selected_line_count(), 1);

    cursor.enter_visual_mode();
    cursor.move_down(9);
    assert_eq!(cursor.selected_line_count(), 10);
}

#[test]
fn test_line_widths() {
    let mut cursor = CursorState::new();
    cursor.update_total_lines(5);
    let widths = Arc::new(vec![10, 20, 30, 40, 50]);
    cursor.update_line_widths(widths);

    cursor.move_to_line(2);
    assert_eq!(cursor.get_cursor_line_width(), 30);
    assert_eq!(cursor.get_line_width(0), 10);
    assert_eq!(cursor.get_line_width(4), 50);
    assert_eq!(cursor.get_line_width(100), 0);
}

#[test]
fn test_max_selected_line_width_normal_mode() {
    let mut cursor = CursorState::new();
    cursor.update_total_lines(5);
    let widths = Arc::new(vec![10, 20, 30, 40, 50]);
    cursor.update_line_widths(widths);

    cursor.move_to_line(2);
    assert_eq!(cursor.get_max_selected_line_width(), 30);
}

#[test]
fn test_max_selected_line_width_visual_mode() {
    let mut cursor = CursorState::new();
    cursor.update_total_lines(5);
    let widths = Arc::new(vec![10, 50, 30, 40, 20]);
    cursor.update_line_widths(widths);

    cursor.move_to_line(0);
    cursor.enter_visual_mode();
    cursor.move_down(2);

    assert_eq!(cursor.get_max_selected_line_width(), 50);
}

#[test]
fn test_compute_scroll_for_cursor_within_viewport() {
    let cursor = CursorState::new();
    let new_offset = cursor.compute_scroll_for_cursor(0, 20, 100);
    assert_eq!(new_offset, 0);
}

#[test]
fn test_compute_scroll_for_cursor_above_viewport() {
    let mut cursor = CursorState::new();
    cursor.update_total_lines(100);
    cursor.move_to_line(5);

    let new_offset = cursor.compute_scroll_for_cursor(20, 20, 100);
    assert_eq!(new_offset, 1);
}

#[test]
fn test_compute_scroll_for_cursor_below_viewport() {
    let mut cursor = CursorState::new();
    cursor.update_total_lines(100);
    cursor.move_to_line(30);

    let new_offset = cursor.compute_scroll_for_cursor(0, 20, 100);
    assert_eq!(new_offset, 15);
}

#[test]
fn test_compute_scroll_for_cursor_small_viewport() {
    let mut cursor = CursorState::new();
    cursor.update_total_lines(100);
    cursor.move_to_line(50);

    let new_offset = cursor.compute_scroll_for_cursor(0, 6, 100);
    assert!(new_offset > 0);
}

#[test]
fn test_compute_scroll_zero_viewport() {
    let mut cursor = CursorState::new();
    cursor.update_total_lines(100);
    cursor.move_to_line(50);

    let new_offset = cursor.compute_scroll_for_cursor(10, 0, 100);
    assert_eq!(new_offset, 10);
}

#[test]
fn test_scrolloff_constant() {
    assert_eq!(SCROLLOFF, 4);
}

#[test]
fn test_empty_document() {
    let mut cursor = CursorState::new();
    cursor.update_total_lines(0);

    cursor.move_down(10);
    assert_eq!(cursor.cursor_line(), 0);

    cursor.move_to_last();
    assert_eq!(cursor.cursor_line(), 0);

    cursor.move_to_line(50);
    assert_eq!(cursor.cursor_line(), 0);
}
