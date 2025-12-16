use ratatui::text::Span;
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

        let visible = extract_visible_spans(&spans, 5, 10);

        assert_eq!(visible.len(), 2);
        assert_eq!(visible[0].content, "56789");
        assert_eq!(visible[1].content, "ABCDE");
    }

    #[test]
    fn test_extract_visible_spans_beyond_text() {
        let spans = vec![Span::styled("Short", Style::default())];

        let visible = extract_visible_spans(&spans, 10, 20);

        assert_eq!(visible.len(), 0);
    }

    #[test]
    fn test_insert_cursor_at_start() {
        let spans = vec![Span::styled("Hello", Style::default().fg(Color::Red))];

        let result = insert_cursor_into_spans(spans, 0);

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].content, "H");
        assert!(result[0].style.add_modifier.contains(Modifier::REVERSED));
        assert_eq!(result[1].content, "ello");
    }

    #[test]
    fn test_insert_cursor_in_middle() {
        let spans = vec![Span::styled("Hello", Style::default().fg(Color::Red))];

        let result = insert_cursor_into_spans(spans, 2);

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

        assert!(result.len() >= 2);
        assert_eq!(result[0].content, "Hello");
        assert_eq!(result[1].content, "W");
        assert!(result[1].style.add_modifier.contains(Modifier::REVERSED));
    }

    #[test]
    fn test_insert_cursor_empty_spans() {
        let spans = vec![];

        let result = insert_cursor_into_spans(spans, 0);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].content, " ");
        assert!(result[0].style.add_modifier.contains(Modifier::REVERSED));
    }

    #[test]
    fn test_extract_visible_spans_unicode() {
        let spans = vec![Span::styled("Hello👋World", Style::default())];

        let visible = extract_visible_spans(&spans, 3, 5);

        assert_eq!(visible.len(), 1);
        assert_eq!(visible[0].content, "lo👋Wo");
    }
}

#[cfg(test)]
mod snapshot_tests {
    use super::*;
    use insta::assert_yaml_snapshot;
    use ratatui::style::{Color, Style};

    use super::super::snapshot_helpers::serialize_spans;

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
