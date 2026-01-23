//! Tests for Scrollable trait implementations

use super::*;

struct TestScrollable {
    offset: usize,
    content_size: usize,
    viewport: usize,
}

impl TestScrollable {
    fn new(content_size: usize, viewport: usize) -> Self {
        Self {
            offset: 0,
            content_size,
            viewport,
        }
    }
}

impl Scrollable for TestScrollable {
    fn scroll_view_up(&mut self, lines: usize) {
        self.offset = self.offset.saturating_sub(lines);
    }

    fn scroll_view_down(&mut self, lines: usize) {
        let max = self.max_scroll();
        self.offset = (self.offset + lines).min(max);
    }

    fn scroll_offset(&self) -> usize {
        self.offset
    }

    fn max_scroll(&self) -> usize {
        self.content_size.saturating_sub(self.viewport)
    }

    fn viewport_size(&self) -> usize {
        self.viewport
    }
}

#[test]
fn test_scroll_view_down() {
    let mut scrollable = TestScrollable::new(20, 5);

    scrollable.scroll_view_down(3);
    assert_eq!(scrollable.scroll_offset(), 3);

    scrollable.scroll_view_down(2);
    assert_eq!(scrollable.scroll_offset(), 5);
}

#[test]
fn test_scroll_view_down_clamped() {
    let mut scrollable = TestScrollable::new(20, 5);

    scrollable.scroll_view_down(100);
    assert_eq!(scrollable.scroll_offset(), 15); // max_scroll = 20 - 5
}

#[test]
fn test_scroll_view_up() {
    let mut scrollable = TestScrollable::new(20, 5);
    scrollable.offset = 10;

    scrollable.scroll_view_up(3);
    assert_eq!(scrollable.scroll_offset(), 7);

    scrollable.scroll_view_up(2);
    assert_eq!(scrollable.scroll_offset(), 5);
}

#[test]
fn test_scroll_view_up_clamped() {
    let mut scrollable = TestScrollable::new(20, 5);
    scrollable.offset = 5;

    scrollable.scroll_view_up(10);
    assert_eq!(scrollable.scroll_offset(), 0);
}

#[test]
fn test_max_scroll() {
    let scrollable = TestScrollable::new(20, 5);
    assert_eq!(scrollable.max_scroll(), 15);

    // Content fits in viewport
    let small = TestScrollable::new(3, 5);
    assert_eq!(small.max_scroll(), 0);
}

#[test]
fn test_viewport_size() {
    let scrollable = TestScrollable::new(20, 5);
    assert_eq!(scrollable.viewport_size(), 5);
}

#[test]
fn test_content_fits_in_viewport() {
    let mut scrollable = TestScrollable::new(3, 10);
    assert_eq!(scrollable.max_scroll(), 0);

    scrollable.scroll_view_down(5);
    assert_eq!(scrollable.scroll_offset(), 0); // Can't scroll when content fits
}
