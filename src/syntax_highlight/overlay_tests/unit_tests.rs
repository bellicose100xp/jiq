use super::*;

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
