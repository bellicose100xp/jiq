//! Overlay utilities for syntax-highlighted text in scrolled viewports.
//!
//! This module provides functions for:
//! - Extracting the visible portion of styled spans when horizontally scrolled
//! - Inserting a cursor indicator into styled spans

use ratatui::text::Span;

/// Extracts the visible portion of spans for a scrolled viewport.
///
/// When the user scrolls horizontally in the input field, this function
/// determines which parts of the styled text should be displayed.
///
/// # Parameters
/// - `spans`: Complete styled text spans
/// - `scroll_offset`: Horizontal scroll position (characters from left)
/// - `viewport_width`: Width of visible area in characters
///
/// # Returns
/// Vector of spans containing only text visible in the viewport.
///
/// # Example
/// For text "Hello World" with scroll_offset=3 and viewport_width=5,
/// returns spans for "lo Wo".
pub fn extract_visible_spans(
    spans: &[Span<'static>],
    scroll_offset: usize,
    viewport_width: usize,
) -> Vec<Span<'static>> {
    let mut result = Vec::new();
    let mut current_col = 0;
    let end_col = scroll_offset + viewport_width;

    for span in spans {
        let span_len = span.content.chars().count();
        let span_end = current_col + span_len;

        if span_end <= scroll_offset {
            current_col = span_end;
            continue;
        }

        if current_col >= end_col {
            break;
        }
        let start_in_span = scroll_offset.saturating_sub(current_col);
        let end_in_span = (end_col - current_col).min(span_len);

        if start_in_span < end_in_span {
            let visible_content: String = span
                .content
                .chars()
                .skip(start_in_span)
                .take(end_in_span - start_in_span)
                .collect();

            result.push(Span::styled(visible_content, span.style));
        }

        current_col = span_end;
    }

    result
}

/// Inserts a cursor indicator at the specified position within spans.
///
/// Splits the span containing the cursor and applies a REVERSED style modifier
/// to the cursor character for visibility. This makes the cursor character appear
/// with inverted colors.
///
/// # Parameters
/// - `spans`: Styled text spans
/// - `cursor_pos`: Character position for cursor (0-indexed)
///
/// # Returns
/// Vector of spans with cursor character styled with REVERSED modifier.
/// If cursor_pos is beyond text length, appends a reversed space at the end.
pub fn insert_cursor_into_spans(
    spans: Vec<Span<'static>>,
    cursor_pos: usize,
) -> Vec<Span<'static>> {
    use ratatui::style::Modifier;

    let mut result = Vec::new();
    let mut current_pos = 0;

    for span in &spans {
        let span_chars: Vec<char> = span.content.chars().collect();
        let span_len = span_chars.len();
        let span_end = current_pos + span_len;

        if cursor_pos < current_pos || cursor_pos >= span_end {
            result.push(span.clone());
            current_pos = span_end;
            continue;
        }

        let cursor_in_span = cursor_pos - current_pos;

        if cursor_in_span > 0 {
            let before: String = span_chars[..cursor_in_span].iter().collect();
            result.push(Span::styled(before, span.style));
        }

        let cursor_char = span_chars[cursor_in_span].to_string();
        result.push(Span::styled(
            cursor_char,
            span.style.add_modifier(Modifier::REVERSED),
        ));

        if cursor_in_span + 1 < span_len {
            let after: String = span_chars[cursor_in_span + 1..].iter().collect();
            result.push(Span::styled(after, span.style));
        }

        current_pos = span_end;
    }
    let total_len: usize = spans.iter().map(|s| s.content.chars().count()).sum();
    if cursor_pos >= total_len {
        use ratatui::style::Style;
        result.push(Span::styled(
            " ",
            Style::default().add_modifier(Modifier::REVERSED),
        ));
    }

    result
}

#[cfg(test)]
#[path = "overlay_tests.rs"]
mod overlay_tests;
