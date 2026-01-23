//! Reusable scrollbar rendering utility
//!
//! Provides a common function for rendering vertical scrollbars across all
//! scrollable components (AI window, snippets, history, help, autocomplete).

use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    widgets::{Scrollbar, ScrollbarOrientation, ScrollbarState},
};

/// Render a vertical scrollbar on the right border of the given area
///
/// The scrollbar is only rendered if the content exceeds the viewport size.
/// Uses ratatui's Scrollbar widget with minimal styling (no end symbols).
/// Renders on the border of the area to match the results pane style.
///
/// # Arguments
/// * `frame` - The frame to render to
/// * `area` - The full area including borders (scrollbar renders on right border)
/// * `total_items` - Total number of items/lines in the content
/// * `viewport_size` - Number of visible items/lines in the viewport
/// * `scroll_offset` - Current scroll position (0 = top)
/// * `color` - Color for the scrollbar thumb and track
pub fn render_vertical_scrollbar_styled(
    frame: &mut Frame,
    area: Rect,
    total_items: usize,
    viewport_size: usize,
    scroll_offset: usize,
    color: Color,
) {
    if total_items <= viewport_size || viewport_size == 0 {
        return;
    }

    let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
        .begin_symbol(None)
        .end_symbol(None)
        .thumb_style(Style::default().fg(color))
        .track_style(Style::default().fg(color));

    // Ratatui uses max_position = content_length - 1 for thumb positioning.
    // To make the thumb reach the bottom when at max scroll, we pass
    // content_length = max_scroll + 1, so max_position equals our max_scroll.
    let max_scroll = total_items.saturating_sub(viewport_size);
    let mut state = ScrollbarState::new(max_scroll + 1)
        .position(scroll_offset.min(max_scroll))
        .viewport_content_length(viewport_size);

    frame.render_stateful_widget(scrollbar, area, &mut state);
}

/// Render a vertical scrollbar with default styling (used by results pane)
pub fn render_vertical_scrollbar(
    frame: &mut Frame,
    area: Rect,
    total_items: usize,
    viewport_size: usize,
    scroll_offset: usize,
) {
    if total_items <= viewport_size || viewport_size == 0 {
        return;
    }

    let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
        .begin_symbol(None)
        .end_symbol(None);

    // Ratatui uses max_position = content_length - 1 for thumb positioning.
    // To make the thumb reach the bottom when at max scroll, we pass
    // content_length = max_scroll + 1, so max_position equals our max_scroll.
    let max_scroll = total_items.saturating_sub(viewport_size);
    let mut state = ScrollbarState::new(max_scroll + 1)
        .position(scroll_offset.min(max_scroll))
        .viewport_content_length(viewport_size);

    frame.render_stateful_widget(scrollbar, area, &mut state);
}

#[cfg(test)]
#[path = "scrollbar_tests.rs"]
mod scrollbar_tests;
