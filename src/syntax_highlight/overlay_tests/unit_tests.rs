use super::*;
use crate::theme;

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

#[test]
fn test_highlight_bracket_pairs_simple() {
    let spans = vec![Span::styled("map(.)", Style::default().fg(Color::Magenta))];

    let result = highlight_bracket_pairs(spans, (3, 5));

    assert_eq!(result[0].content, "map");
    assert_eq!(result[1].content, "(");
    assert!(result[1].style.add_modifier.contains(Modifier::UNDERLINED));
    assert_eq!(result[2].content, ".");
    assert_eq!(result[3].content, ")");
    assert!(result[3].style.add_modifier.contains(Modifier::UNDERLINED));
}

#[test]
fn test_highlight_bracket_pairs_across_spans() {
    let spans = vec![
        Span::styled("map", Style::default().fg(Color::Blue)),
        Span::styled("(", Style::default().fg(Color::Magenta)),
        Span::styled(".", Style::default()),
        Span::styled(")", Style::default().fg(Color::Magenta)),
    ];

    let result = highlight_bracket_pairs(spans, (3, 5));

    assert_eq!(result[0].content, "map");
    assert_eq!(result[1].content, "(");
    assert!(result[1].style.add_modifier.contains(Modifier::UNDERLINED));
    assert_eq!(result[2].content, ".");
    assert_eq!(result[3].content, ")");
    assert!(result[3].style.add_modifier.contains(Modifier::UNDERLINED));
}

#[test]
fn test_highlight_bracket_pairs_at_start() {
    let spans = vec![Span::styled("(.)", Style::default())];

    let result = highlight_bracket_pairs(spans, (0, 2));

    assert_eq!(result[0].content, "(");
    assert!(result[0].style.add_modifier.contains(Modifier::UNDERLINED));
    assert_eq!(result[1].content, ".");
    assert_eq!(result[2].content, ")");
    assert!(result[2].style.add_modifier.contains(Modifier::UNDERLINED));
}

#[test]
fn test_highlight_bracket_pairs_at_end() {
    let spans = vec![Span::styled("test()", Style::default())];

    let result = highlight_bracket_pairs(spans, (4, 5));

    assert_eq!(result[0].content, "test");
    assert_eq!(result[1].content, "(");
    assert!(result[1].style.add_modifier.contains(Modifier::UNDERLINED));
    assert_eq!(result[2].content, ")");
    assert!(result[2].style.add_modifier.contains(Modifier::UNDERLINED));
}

#[test]
fn test_highlight_bracket_pairs_nested() {
    let spans = vec![Span::styled("map(select(.))", Style::default())];

    let result = highlight_bracket_pairs(spans, (3, 13));

    assert_eq!(result[0].content, "map");
    assert_eq!(result[1].content, "(");
    assert!(result[1].style.add_modifier.contains(Modifier::UNDERLINED));
    assert_eq!(result[2].content, "select(.)");
    assert_eq!(result[3].content, ")");
    assert!(result[3].style.add_modifier.contains(Modifier::UNDERLINED));
}

#[test]
fn test_highlight_bracket_pairs_preserves_existing_style() {
    let spans = vec![Span::styled(
        "map(.)",
        Style::default()
            .fg(theme::syntax::FUNCTION)
            .add_modifier(Modifier::BOLD),
    )];

    let result = highlight_bracket_pairs(spans, (3, 5));

    assert!(result[1].style.add_modifier.contains(Modifier::UNDERLINED));
    assert!(result[1].style.add_modifier.contains(Modifier::BOLD));
    assert_eq!(
        result[1].style.fg,
        Some(theme::syntax::bracket_match::COLOR)
    );
}

#[test]
fn test_highlight_bracket_pairs_square_brackets() {
    let spans = vec![Span::styled(".items[]", Style::default())];

    let result = highlight_bracket_pairs(spans, (6, 7));

    assert_eq!(result[0].content, ".items");
    assert_eq!(result[1].content, "[");
    assert!(result[1].style.add_modifier.contains(Modifier::UNDERLINED));
    assert_eq!(result[2].content, "]");
    assert!(result[2].style.add_modifier.contains(Modifier::UNDERLINED));
}

#[test]
fn test_highlight_bracket_pairs_curly_braces() {
    let spans = vec![Span::styled("{name}", Style::default())];

    let result = highlight_bracket_pairs(spans, (0, 5));

    assert_eq!(result[0].content, "{");
    assert!(result[0].style.add_modifier.contains(Modifier::UNDERLINED));
    assert_eq!(result[1].content, "name");
    assert_eq!(result[2].content, "}");
    assert!(result[2].style.add_modifier.contains(Modifier::UNDERLINED));
}

#[test]
fn test_highlight_bracket_pairs_adjacent_characters() {
    let spans = vec![Span::styled("()", Style::default())];

    let result = highlight_bracket_pairs(spans, (0, 1));

    assert_eq!(result[0].content, "(");
    assert!(result[0].style.add_modifier.contains(Modifier::UNDERLINED));
    assert_eq!(result[1].content, ")");
    assert!(result[1].style.add_modifier.contains(Modifier::UNDERLINED));
}

#[test]
fn test_highlight_bracket_pairs_multiple_spans_complex() {
    let spans = vec![
        Span::styled("map", Style::default().fg(Color::Blue)),
        Span::styled("(", Style::default().fg(Color::Magenta)),
        Span::styled("select", Style::default().fg(Color::Blue)),
        Span::styled("(", Style::default().fg(Color::Magenta)),
        Span::styled(".", Style::default()),
        Span::styled(")", Style::default().fg(Color::Magenta)),
        Span::styled(")", Style::default().fg(Color::Magenta)),
    ];

    let result = highlight_bracket_pairs(spans, (3, 13));

    let mut found_first_underlined = false;
    let mut found_second_underlined = false;

    for span in &result {
        if span.content == "("
            && span.style.add_modifier.contains(Modifier::UNDERLINED)
            && !found_first_underlined
        {
            found_first_underlined = true;
        }
        if span.content == ")" {
            let underlined_closing_count = result
                .iter()
                .filter(|s| s.content == ")" && s.style.add_modifier.contains(Modifier::UNDERLINED))
                .count();
            if underlined_closing_count > 0 {
                found_second_underlined = true;
            }
        }
    }

    assert!(found_first_underlined);
    assert!(found_second_underlined);
}

#[test]
fn test_highlight_bracket_pairs_unicode_content() {
    let spans = vec![Span::styled("test(👋)", Style::default())];

    let result = highlight_bracket_pairs(spans, (4, 6));

    assert_eq!(result[0].content, "test");
    assert_eq!(result[1].content, "(");
    assert!(result[1].style.add_modifier.contains(Modifier::UNDERLINED));
    assert_eq!(result[2].content, "👋");
    assert_eq!(result[3].content, ")");
    assert!(result[3].style.add_modifier.contains(Modifier::UNDERLINED));
}

#[test]
fn test_apply_modifier_at_positions_empty_positions() {
    let spans = vec![Span::styled("test", Style::default())];

    let result = super::apply_modifier_at_positions(spans.clone(), &[], Modifier::UNDERLINED);

    assert_eq!(result, spans);
}

#[test]
fn test_apply_modifier_at_positions_single_position() {
    let spans = vec![Span::styled("test", Style::default())];

    let result = super::apply_modifier_at_positions(spans, &[1], Modifier::UNDERLINED);

    assert_eq!(result[0].content, "t");
    assert_eq!(result[1].content, "e");
    assert!(result[1].style.add_modifier.contains(Modifier::UNDERLINED));
    assert_eq!(result[2].content, "st");
}

#[test]
fn test_apply_modifier_at_positions_multiple_positions_same_span() {
    let spans = vec![Span::styled("test", Style::default())];

    let result = super::apply_modifier_at_positions(spans, &[0, 3], Modifier::UNDERLINED);

    assert_eq!(result[0].content, "t");
    assert!(result[0].style.add_modifier.contains(Modifier::UNDERLINED));
    assert_eq!(result[1].content, "es");
    assert_eq!(result[2].content, "t");
    assert!(result[2].style.add_modifier.contains(Modifier::UNDERLINED));
}

#[test]
fn test_apply_modifier_at_positions_across_multiple_spans() {
    let spans = vec![
        Span::styled("ab", Style::default()),
        Span::styled("cd", Style::default()),
    ];

    let result = super::apply_modifier_at_positions(spans, &[1, 2], Modifier::BOLD);

    assert_eq!(result[0].content, "a");
    assert_eq!(result[1].content, "b");
    assert!(result[1].style.add_modifier.contains(Modifier::BOLD));
    assert_eq!(result[2].content, "c");
    assert!(result[2].style.add_modifier.contains(Modifier::BOLD));
    assert_eq!(result[3].content, "d");
}

#[test]
fn test_apply_modifier_at_positions_out_of_range() {
    let spans = vec![Span::styled("test", Style::default())];

    let result = super::apply_modifier_at_positions(spans.clone(), &[10, 20], Modifier::UNDERLINED);

    assert_eq!(result, spans);
}

#[test]
fn test_apply_modifier_at_positions_unsorted_positions() {
    let spans = vec![Span::styled("abcd", Style::default())];

    let result = super::apply_modifier_at_positions(spans, &[3, 1, 2], Modifier::UNDERLINED);

    assert_eq!(result[0].content, "a");
    assert_eq!(result[1].content, "b");
    assert!(result[1].style.add_modifier.contains(Modifier::UNDERLINED));
    assert_eq!(result[2].content, "c");
    assert!(result[2].style.add_modifier.contains(Modifier::UNDERLINED));
    assert_eq!(result[3].content, "d");
    assert!(result[3].style.add_modifier.contains(Modifier::UNDERLINED));
}

#[test]
fn test_apply_modifier_at_positions_consecutive_positions() {
    let spans = vec![Span::styled("abcd", Style::default())];

    let result = super::apply_modifier_at_positions(spans, &[1, 2], Modifier::UNDERLINED);

    assert_eq!(result[0].content, "a");
    assert_eq!(result[1].content, "b");
    assert!(result[1].style.add_modifier.contains(Modifier::UNDERLINED));
    assert_eq!(result[2].content, "c");
    assert!(result[2].style.add_modifier.contains(Modifier::UNDERLINED));
    assert_eq!(result[3].content, "d");
}
