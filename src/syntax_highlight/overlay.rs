//! Syntax highlighting utilities
//!
//! Functions for rendering syntax-highlighted jq query text with cursor support.

use ratatui::text::Span;

/// Extract spans that are visible in the current viewport
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
            // Span is entirely before visible area - skip
            current_col = span_end;
            continue;
        }

        if current_col >= end_col {
            // Past visible area - done
            break;
        }

        // Span overlaps with visible area
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


/// Insert cursor into spans at the given position (relative to visible area)
/// The character at cursor_pos gets reversed style (like vim)
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
            // Cursor not in this span - keep as is
            result.push(span.clone());
            current_pos = span_end;
            continue;
        }

        // Cursor is within this span - split it
        let cursor_in_span = cursor_pos - current_pos;

        // Part before cursor
        if cursor_in_span > 0 {
            let before: String = span_chars[..cursor_in_span].iter().collect();
            result.push(Span::styled(before, span.style));
        }

        // Character at cursor (with reversed style)
        let cursor_char = span_chars[cursor_in_span].to_string();
        result.push(Span::styled(
            cursor_char,
            span.style.add_modifier(Modifier::REVERSED),
        ));

        // Part after cursor
        if cursor_in_span + 1 < span_len {
            let after: String = span_chars[cursor_in_span + 1..].iter().collect();
            result.push(Span::styled(after, span.style));
        }

        current_pos = span_end;
    }

    // If cursor is at end of all spans (no character under cursor), add reversed space
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
mod tests {
    use super::*;
    use ratatui::style::{Color, Modifier, Style};

    #[test]
    fn test_extract_visible_spans_no_scroll() {
        let spans = vec![
            Span::styled("Hello", Style::default().fg(Color::Red)),
            Span::styled(" ", Style::default()),
            Span::styled("World", Style::default().fg(Color::Blue)),
        ];

        let visible = extract_visible_spans(&spans, 0, 20);

        assert_eq!(visible.len(), 3);
        assert_eq!(visible[0].content, "Hello");
        assert_eq!(visible[2].content, "World");
    }

    #[test]
    fn test_extract_visible_spans_with_scroll() {
        let spans = vec![
            Span::styled("0123456789", Style::default().fg(Color::Red)),
            Span::styled("ABCDEFGHIJ", Style::default().fg(Color::Blue)),
        ];

        // Scroll offset 5, viewport 10 → should show "56789ABCDE"
        let visible = extract_visible_spans(&spans, 5, 10);

        assert_eq!(visible.len(), 2);
        assert_eq!(visible[0].content, "56789"); // Rest of first span
        assert_eq!(visible[1].content, "ABCDE"); // Part of second span
    }

    #[test]
    fn test_extract_visible_spans_beyond_text() {
        let spans = vec![Span::styled("Short", Style::default())];

        // Scroll past the text
        let visible = extract_visible_spans(&spans, 10, 20);

        assert_eq!(visible.len(), 0); // Nothing visible
    }

    #[test]
    fn test_insert_cursor_at_start() {
        let spans = vec![Span::styled("Hello", Style::default().fg(Color::Red))];

        let result = insert_cursor_into_spans(spans, 0);

        // Should be: ["H" (reversed), "ello"]
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].content, "H");
        assert!(result[0].style.add_modifier.contains(Modifier::REVERSED));
        assert_eq!(result[1].content, "ello");
    }

    #[test]
    fn test_insert_cursor_in_middle() {
        let spans = vec![Span::styled("Hello", Style::default().fg(Color::Red))];

        let result = insert_cursor_into_spans(spans, 2);

        // Should be: ["He", "l" (reversed), "lo"]
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].content, "He");
        assert_eq!(result[1].content, "l");
        assert!(result[1].style.add_modifier.contains(Modifier::REVERSED));
        assert_eq!(result[2].content, "lo");
    }

    #[test]
    fn test_insert_cursor_at_end() {
        let spans = vec![Span::styled("Hi", Style::default().fg(Color::Red))];

        let result = insert_cursor_into_spans(spans, 2);

        // Should be: ["Hi", " " (reversed space)]
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].content, "Hi");
        assert_eq!(result[1].content, " ");
        assert!(result[1].style.add_modifier.contains(Modifier::REVERSED));
    }

    #[test]
    fn test_insert_cursor_across_spans() {
        let spans = vec![
            Span::styled("Hello", Style::default().fg(Color::Red)),
            Span::styled("World", Style::default().fg(Color::Blue)),
        ];

        let result = insert_cursor_into_spans(spans, 5);

        // Cursor at position 5 is the 'W' in "World" (first char of second span)
        // Should be: ["Hello", "W" (reversed), "orld"]
        assert!(result.len() >= 2);
        assert_eq!(result[0].content, "Hello");
        assert_eq!(result[1].content, "W");
        assert!(result[1].style.add_modifier.contains(Modifier::REVERSED));
    }

    #[test]
    fn test_insert_cursor_empty_spans() {
        let spans = vec![];

        let result = insert_cursor_into_spans(spans, 0);

        // Should add a reversed space cursor
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].content, " ");
        assert!(result[0].style.add_modifier.contains(Modifier::REVERSED));
    }

    #[test]
    fn test_extract_visible_spans_unicode() {
        let spans = vec![Span::styled("Hello👋World", Style::default())];

        // "Hello👋World" is 11 chars (emoji is 1 char)
        // Extract middle portion
        let visible = extract_visible_spans(&spans, 3, 5);

        // Should get "lo👋Wo" (chars 3-7)
        assert_eq!(visible.len(), 1);
        assert_eq!(visible[0].content, "lo👋Wo");
    }
}


#[cfg(test)]
mod snapshot_tests {
    use super::*;
    use ratatui::style::{Color, Style};
    use insta::assert_yaml_snapshot;

    // Import serialize_spans helper from parent syntax_highlight module
    use super::super::snapshot_helpers::serialize_spans;

    // === Cursor Positioning Tests ===

    #[test]
    fn snapshot_cursor_at_start() {
        let spans = vec![Span::styled("Hello", Style::default().fg(Color::Red))];
        let result = insert_cursor_into_spans(spans, 0);
        assert_yaml_snapshot!(serialize_spans(&result));
    }

    #[test]
    fn snapshot_cursor_in_middle() {
        let spans = vec![Span::styled("Hello", Style::default().fg(Color::Red))];
        let result = insert_cursor_into_spans(spans, 2);
        assert_yaml_snapshot!(serialize_spans(&result));
    }

    #[test]
    fn snapshot_cursor_at_end() {
        let spans = vec![Span::styled("Hi", Style::default().fg(Color::Red))];
        let result = insert_cursor_into_spans(spans, 2);
        assert_yaml_snapshot!(serialize_spans(&result));
    }

    #[test]
    fn snapshot_cursor_across_spans() {
        let spans = vec![
            Span::styled("Hello", Style::default().fg(Color::Red)),
            Span::styled("World", Style::default().fg(Color::Blue)),
        ];
        let result = insert_cursor_into_spans(spans, 5);
        assert_yaml_snapshot!(serialize_spans(&result));
    }

    #[test]
    fn snapshot_cursor_empty_spans() {
        let spans = vec![];
        let result = insert_cursor_into_spans(spans, 0);
        assert_yaml_snapshot!(serialize_spans(&result));
    }

    // === Visible Span Extraction Tests ===

    #[test]
    fn snapshot_extract_visible_no_scroll() {
        let spans = vec![
            Span::styled("Hello", Style::default().fg(Color::Red)),
            Span::raw(" "),
            Span::styled("World", Style::default().fg(Color::Blue)),
        ];
        let visible = extract_visible_spans(&spans, 0, 20);
        assert_yaml_snapshot!(serialize_spans(&visible));
    }

    #[test]
    fn snapshot_extract_visible_with_scroll() {
        let spans = vec![
            Span::styled("0123456789", Style::default().fg(Color::Red)),
            Span::styled("ABCDEFGHIJ", Style::default().fg(Color::Blue)),
        ];
        let visible = extract_visible_spans(&spans, 5, 10);
        assert_yaml_snapshot!(serialize_spans(&visible));
    }

    #[test]
    fn snapshot_extract_visible_unicode() {
        let spans = vec![Span::styled("Hello👋World", Style::default())];
        let visible = extract_visible_spans(&spans, 3, 5);
        assert_yaml_snapshot!(serialize_spans(&visible));
    }

    #[test]
    fn snapshot_extract_visible_beyond_text() {
        let spans = vec![Span::styled("Short", Style::default())];
        let visible = extract_visible_spans(&spans, 10, 20);
        assert_yaml_snapshot!(serialize_spans(&visible));
    }
}
