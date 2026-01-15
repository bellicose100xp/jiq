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

#[test]
fn snapshot_highlight_bracket_pairs_simple() {
    let spans = vec![Span::styled("map(.)", Style::default().fg(Color::Magenta))];
    let result = highlight_bracket_pairs(spans, (3, 5));
    assert_yaml_snapshot!(serialize_spans(&result));
}

#[test]
fn snapshot_highlight_bracket_pairs_nested() {
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
    assert_yaml_snapshot!(serialize_spans(&result));
}

#[test]
fn snapshot_highlight_bracket_pairs_complex_query() {
    let spans = vec![
        Span::styled(".items", Style::default().fg(Color::Cyan)),
        Span::styled("[", Style::default().fg(Color::Magenta)),
        Span::styled("]", Style::default().fg(Color::Magenta)),
        Span::raw(" | "),
        Span::styled("select", Style::default().fg(Color::Blue)),
        Span::styled("(", Style::default().fg(Color::Magenta)),
        Span::styled(".price", Style::default().fg(Color::Cyan)),
        Span::raw(" > "),
        Span::styled("100", Style::default().fg(Color::Cyan)),
        Span::styled(")", Style::default().fg(Color::Magenta)),
        Span::raw(" | "),
        Span::styled("{", Style::default().fg(Color::Magenta)),
        Span::styled("name", Style::default().fg(Color::Cyan)),
        Span::raw(", "),
        Span::styled("price", Style::default().fg(Color::Cyan)),
        Span::styled("}", Style::default().fg(Color::Magenta)),
    ];
    let result = highlight_bracket_pairs(spans, (6, 7));
    assert_yaml_snapshot!(serialize_spans(&result));
}

#[test]
fn snapshot_highlight_bracket_pairs_deeply_nested() {
    let spans = vec![
        Span::styled("map", Style::default().fg(Color::Blue)),
        Span::styled("(", Style::default().fg(Color::Magenta)),
        Span::styled("select", Style::default().fg(Color::Blue)),
        Span::styled("(", Style::default().fg(Color::Magenta)),
        Span::styled("has", Style::default().fg(Color::Blue)),
        Span::styled("(", Style::default().fg(Color::Magenta)),
        Span::styled("{", Style::default().fg(Color::Magenta)),
        Span::styled("a", Style::default().fg(Color::Cyan)),
        Span::raw(": "),
        Span::styled("{", Style::default().fg(Color::Magenta)),
        Span::styled("b", Style::default().fg(Color::Cyan)),
        Span::raw(": "),
        Span::styled(".x", Style::default().fg(Color::Cyan)),
        Span::styled("}", Style::default().fg(Color::Magenta)),
        Span::styled("}", Style::default().fg(Color::Magenta)),
        Span::styled(")", Style::default().fg(Color::Magenta)),
        Span::styled(")", Style::default().fg(Color::Magenta)),
        Span::styled(")", Style::default().fg(Color::Magenta)),
    ];
    let result = highlight_bracket_pairs(spans, (3, 29));
    assert_yaml_snapshot!(serialize_spans(&result));
}

#[test]
fn snapshot_highlight_bracket_pairs_preserves_style() {
    let spans = vec![Span::styled(
        "map(.)",
        Style::default()
            .fg(Color::Blue)
            .add_modifier(Modifier::BOLD),
    )];
    let result = highlight_bracket_pairs(spans, (3, 5));
    assert_yaml_snapshot!(serialize_spans(&result));
}

#[test]
fn snapshot_highlight_bracket_pairs_square_brackets() {
    let spans = vec![
        Span::styled(".items", Style::default().fg(Color::Cyan)),
        Span::styled("[", Style::default().fg(Color::Magenta)),
        Span::styled("0", Style::default().fg(Color::Cyan)),
        Span::styled("]", Style::default().fg(Color::Magenta)),
        Span::styled("[", Style::default().fg(Color::Magenta)),
        Span::styled("1", Style::default().fg(Color::Cyan)),
        Span::styled("]", Style::default().fg(Color::Magenta)),
    ];
    let result = highlight_bracket_pairs(spans, (6, 10));
    assert_yaml_snapshot!(serialize_spans(&result));
}

#[test]
fn snapshot_highlight_bracket_pairs_curly_braces() {
    let spans = vec![
        Span::styled("{", Style::default().fg(Color::Magenta)),
        Span::styled("name", Style::default().fg(Color::Cyan)),
        Span::raw(": "),
        Span::styled(".x", Style::default().fg(Color::Cyan)),
        Span::raw(", "),
        Span::styled("count", Style::default().fg(Color::Cyan)),
        Span::raw(": "),
        Span::styled(".y", Style::default().fg(Color::Cyan)),
        Span::styled("}", Style::default().fg(Color::Magenta)),
    ];
    let result = highlight_bracket_pairs(spans, (0, 15));
    assert_yaml_snapshot!(serialize_spans(&result));
}
