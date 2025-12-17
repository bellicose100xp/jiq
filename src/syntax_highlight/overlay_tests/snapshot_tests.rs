use super::*;
use insta::assert_yaml_snapshot;

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
