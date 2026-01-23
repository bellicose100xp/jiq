//! Tests for ScrollState

use super::*;

#[test]
fn test_new_scroll_state() {
    let scroll = ScrollState::new();
    assert_eq!(scroll.offset, 0);
    assert_eq!(scroll.max_offset, 0);
    assert_eq!(scroll.viewport_height, 0);
    assert_eq!(scroll.h_offset, 0);
    assert_eq!(scroll.max_h_offset, 0);
}

#[test]
fn test_update_bounds_small_content() {
    let mut scroll = ScrollState::new();

    // Content fits in viewport
    scroll.update_bounds(10, 20);
    assert_eq!(scroll.max_offset, 0);
    assert_eq!(scroll.viewport_height, 20);
    assert_eq!(scroll.offset, 0);
}

#[test]
fn test_update_bounds_large_content() {
    let mut scroll = ScrollState::new();

    // Content larger than viewport
    scroll.update_bounds(100, 20);
    assert_eq!(scroll.max_offset, 80);
    assert_eq!(scroll.viewport_height, 20);
}

#[test]
fn test_update_bounds_clamps_offset() {
    let mut scroll = ScrollState::new();
    scroll.update_bounds(100, 20);
    scroll.offset = 80; // Set to max

    // Reduce content size
    scroll.update_bounds(50, 20);
    assert_eq!(scroll.max_offset, 30);
    assert_eq!(scroll.offset, 30); // Clamped from 80 to 30
}

#[test]
fn test_update_bounds_very_large_content() {
    let mut scroll = ScrollState::new();

    // Content with >65K lines (exceeds u16::MAX)
    scroll.update_bounds(70000, 20);
    assert_eq!(scroll.max_offset, u16::MAX);
    assert_eq!(scroll.viewport_height, 20);
}

#[test]
fn test_scroll_down() {
    let mut scroll = ScrollState::new();
    scroll.update_bounds(100, 20);

    scroll.scroll_down(10);
    assert_eq!(scroll.offset, 10);

    scroll.scroll_down(5);
    assert_eq!(scroll.offset, 15);
}

#[test]
fn test_scroll_down_clamped() {
    let mut scroll = ScrollState::new();
    scroll.update_bounds(100, 20);

    scroll.scroll_down(100); // Try to scroll past end
    assert_eq!(scroll.offset, 80); // Clamped to max_offset
}

#[test]
fn test_scroll_up() {
    let mut scroll = ScrollState::new();
    scroll.update_bounds(100, 20);
    scroll.offset = 50;

    scroll.scroll_up(10);
    assert_eq!(scroll.offset, 40);

    scroll.scroll_up(5);
    assert_eq!(scroll.offset, 35);
}

#[test]
fn test_scroll_up_clamped() {
    let mut scroll = ScrollState::new();
    scroll.update_bounds(100, 20);
    scroll.offset = 10;

    scroll.scroll_up(20); // Try to scroll past top
    assert_eq!(scroll.offset, 0); // Clamped to 0
}

#[test]
fn test_page_down() {
    let mut scroll = ScrollState::new();
    scroll.update_bounds(100, 20);

    scroll.page_down();
    assert_eq!(scroll.offset, 10); // Half of viewport_height (20/2)

    scroll.page_down();
    assert_eq!(scroll.offset, 20);
}

#[test]
fn test_page_up() {
    let mut scroll = ScrollState::new();
    scroll.update_bounds(100, 20);
    scroll.offset = 50;

    scroll.page_up();
    assert_eq!(scroll.offset, 40); // Half of viewport_height (20/2)

    scroll.page_up();
    assert_eq!(scroll.offset, 30);
}

#[test]
fn test_jump_to_top() {
    let mut scroll = ScrollState::new();
    scroll.update_bounds(100, 20);
    scroll.offset = 50;

    scroll.jump_to_top();
    assert_eq!(scroll.offset, 0);
}

#[test]
fn test_jump_to_bottom() {
    let mut scroll = ScrollState::new();
    scroll.update_bounds(100, 20);

    scroll.jump_to_bottom();
    assert_eq!(scroll.offset, 80); // max_offset
}

#[test]
fn test_reset() {
    let mut scroll = ScrollState::new();
    scroll.update_bounds(100, 20);
    scroll.offset = 50;

    scroll.reset();
    assert_eq!(scroll.offset, 0);
}

#[test]
fn test_default() {
    let scroll = ScrollState::default();
    assert_eq!(scroll, ScrollState::new());
}
