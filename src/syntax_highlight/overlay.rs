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
#[path = "overlay_tests.rs"]
mod overlay_tests;
