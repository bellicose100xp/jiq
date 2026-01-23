use insta::assert_snapshot;
use ratatui::{Terminal, backend::TestBackend, layout::Rect};

use super::{render_vertical_scrollbar, render_vertical_scrollbar_styled};
use ratatui::style::Color;

fn create_test_terminal(width: u16, height: u16) -> Terminal<TestBackend> {
    let backend = TestBackend::new(width, height);
    Terminal::new(backend).unwrap()
}

fn render_scrollbar_to_string(
    total: usize,
    viewport: usize,
    scroll_offset: usize,
    track_height: u16,
) -> String {
    let mut terminal = create_test_terminal(5, track_height);
    terminal
        .draw(|frame| {
            let area = Rect::new(0, 0, 5, track_height);
            render_vertical_scrollbar_styled(
                frame,
                area,
                total,
                viewport,
                scroll_offset,
                Color::White,
            );
        })
        .unwrap();
    terminal.backend().to_string()
}

#[test]
fn test_scrollbar_not_rendered_when_content_fits() {
    let mut terminal = create_test_terminal(20, 10);
    terminal
        .draw(|frame| {
            let area = Rect::new(0, 0, 20, 10);
            // Content fits in viewport - no scrollbar should render
            render_vertical_scrollbar(frame, area, 5, 10, 0);
        })
        .unwrap();
}

#[test]
fn test_scrollbar_rendered_when_content_exceeds_viewport() {
    let mut terminal = create_test_terminal(20, 10);
    terminal
        .draw(|frame| {
            let area = Rect::new(0, 0, 20, 10);
            // Content exceeds viewport - scrollbar should render
            render_vertical_scrollbar(frame, area, 50, 10, 0);
        })
        .unwrap();
}

#[test]
fn test_scrollbar_with_scroll_offset() {
    let mut terminal = create_test_terminal(20, 10);
    terminal
        .draw(|frame| {
            let area = Rect::new(0, 0, 20, 10);
            // Scrolled to middle of content
            render_vertical_scrollbar(frame, area, 100, 10, 45);
        })
        .unwrap();
}

#[test]
fn test_scrollbar_at_end() {
    let mut terminal = create_test_terminal(20, 10);
    terminal
        .draw(|frame| {
            let area = Rect::new(0, 0, 20, 10);
            // Scrolled to end of content
            render_vertical_scrollbar(frame, area, 100, 10, 90);
        })
        .unwrap();
}

#[test]
fn test_scrollbar_with_zero_items() {
    let mut terminal = create_test_terminal(20, 10);
    terminal
        .draw(|frame| {
            let area = Rect::new(0, 0, 20, 10);
            // No content - should not render
            render_vertical_scrollbar(frame, area, 0, 10, 0);
        })
        .unwrap();
}

#[test]
fn test_scrollbar_with_exact_viewport_size() {
    let mut terminal = create_test_terminal(20, 10);
    terminal
        .draw(|frame| {
            let area = Rect::new(0, 0, 20, 10);
            // Content exactly fits viewport - no scrollbar needed
            render_vertical_scrollbar(frame, area, 10, 10, 0);
        })
        .unwrap();
}

#[test]
fn test_scrollbar_with_one_extra_item() {
    let mut terminal = create_test_terminal(20, 10);
    terminal
        .draw(|frame| {
            let area = Rect::new(0, 0, 20, 10);
            // Just one item more than viewport - scrollbar should render
            render_vertical_scrollbar(frame, area, 11, 10, 0);
        })
        .unwrap();
}

// Snapshot tests for scrollbar position verification
// These tests verify the scrollbar renders correctly at different scroll positions

#[test]
fn snapshot_scrollbar_position_at_top() {
    // total=30, viewport=12, scroll=0 (at top)
    // max_scroll = 30 - 12 = 18
    // When scroll=0, thumb should be at the very top
    let output = render_scrollbar_to_string(30, 12, 0, 12);
    assert_snapshot!(output);
}

#[test]
fn snapshot_scrollbar_position_at_middle() {
    // total=30, viewport=12, scroll=9 (middle of 0-18 range)
    // thumb should be roughly in the middle
    let output = render_scrollbar_to_string(30, 12, 9, 12);
    assert_snapshot!(output);
}

#[test]
fn snapshot_scrollbar_position_at_bottom() {
    // total=30, viewport=12, scroll=18 (at max scroll)
    // thumb should be at the very bottom
    let output = render_scrollbar_to_string(30, 12, 18, 12);
    assert_snapshot!(output);
}

#[test]
fn snapshot_scrollbar_position_at_bottom_with_high_scroll() {
    // Try passing a higher scroll value to see if thumb moves to bottom
    // max_scroll = 30 - 12 = 18, but let's pass 30
    let output = render_scrollbar_to_string(30, 12, 30, 12);
    assert_snapshot!(output);
}

#[test]
fn snapshot_scrollbar_position_simple_case() {
    // Simpler case: total=20, viewport=10, scroll=10 (max scroll)
    // This should clearly show thumb at bottom
    let output = render_scrollbar_to_string(20, 10, 10, 10);
    assert_snapshot!(output);
}
