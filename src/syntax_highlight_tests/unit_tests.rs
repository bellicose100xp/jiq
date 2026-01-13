use super::*;

#[test]
fn test_highlight_empty() {
    let spans = JqHighlighter::highlight("");
    assert_eq!(spans.len(), 0);
}

#[test]
fn test_highlight_simple_field() {
    let spans = JqHighlighter::highlight(".name");
    assert_eq!(spans.len(), 1);
    assert_eq!(spans[0].style.fg, None);
}

#[test]
fn test_highlight_keyword() {
    let spans = JqHighlighter::highlight("if");
    assert_eq!(spans.len(), 1);
    assert_eq!(spans[0].style.fg, Some(Color::Yellow));
}

#[test]
fn test_highlight_string() {
    let spans = JqHighlighter::highlight(r#""hello""#);
    assert_eq!(spans.len(), 1);
    assert_eq!(spans[0].style.fg, Some(Color::Green));
}

#[test]
fn test_highlight_number() {
    let spans = JqHighlighter::highlight("123");
    assert_eq!(spans.len(), 1);
    assert_eq!(spans[0].style.fg, Some(Color::Cyan));
}

#[test]
fn test_highlight_function() {
    let spans = JqHighlighter::highlight("map");
    assert_eq!(spans.len(), 1);
    assert_eq!(spans[0].style.fg, Some(Color::Blue));
}

#[test]
fn test_highlight_operator() {
    let spans = JqHighlighter::highlight("|");
    assert_eq!(spans.len(), 1);
    assert_eq!(spans[0].style.fg, Some(Color::Magenta));
}

#[test]
fn test_highlight_complex_query() {
    let spans = JqHighlighter::highlight(r#".users[] | select(.active == true) | .name"#);
    assert!(spans.len() > 5);
}

#[test]
fn test_highlight_with_whitespace() {
    let spans = JqHighlighter::highlight("  map  ");
    assert!(spans.len() >= 2);
}

#[test]
fn test_unterminated_string() {
    let spans = JqHighlighter::highlight(r#""unterminated"#);
    assert_eq!(spans.len(), 1);
    assert_eq!(spans[0].style.fg, Some(Color::Green));
    assert_eq!(spans[0].content, r#""unterminated"#);
}

#[test]
fn test_string_with_escapes() {
    let spans = JqHighlighter::highlight(r#""hello \"world\"""#);
    assert_eq!(spans.len(), 1);
    assert_eq!(spans[0].style.fg, Some(Color::Green));
}

#[test]
fn test_negative_number() {
    let spans = JqHighlighter::highlight("-123");
    assert_eq!(spans.len(), 1);
    assert_eq!(spans[0].style.fg, Some(Color::Cyan));
    assert_eq!(spans[0].content, "-123");
}

#[test]
fn test_decimal_number() {
    let spans = JqHighlighter::highlight("3.14");
    assert_eq!(spans.len(), 1);
    assert_eq!(spans[0].style.fg, Some(Color::Cyan));
    assert_eq!(spans[0].content, "3.14");
}

#[test]
fn test_two_char_operators() {
    // Test ==
    let spans = JqHighlighter::highlight("==");
    assert_eq!(spans.len(), 1);
    assert_eq!(spans[0].content, "==");
    assert_eq!(spans[0].style.fg, Some(Color::Magenta));

    // Test !=
    let spans = JqHighlighter::highlight("!=");
    assert_eq!(spans.len(), 1);
    assert_eq!(spans[0].content, "!=");

    // Test <=
    let spans = JqHighlighter::highlight("<=");
    assert_eq!(spans.len(), 1);
    assert_eq!(spans[0].content, "<=");

    // Test >=
    let spans = JqHighlighter::highlight(">=");
    assert_eq!(spans.len(), 1);
    assert_eq!(spans[0].content, ">=");

    // Test //
    let spans = JqHighlighter::highlight("//");
    assert_eq!(spans.len(), 1);
    assert_eq!(spans[0].content, "//");
}

#[test]
fn test_nested_field_path() {
    let spans = JqHighlighter::highlight(".foo.bar.baz");
    assert_eq!(spans.len(), 1);
    assert_eq!(spans[0].style.fg, None);
    assert_eq!(spans[0].content, ".foo.bar.baz");
}

#[test]
fn test_just_dot() {
    let spans = JqHighlighter::highlight(".");
    assert_eq!(spans.len(), 1);
    assert_eq!(spans[0].style.fg, None);
}

#[test]
fn test_variable_reference() {
    let spans = JqHighlighter::highlight("$foo");
    assert_eq!(spans.len(), 1);
    assert_eq!(spans[0].style.fg, Some(Color::Red));
}

#[test]
fn test_keywords_and_or() {
    let spans = JqHighlighter::highlight("and");
    assert_eq!(spans.len(), 1);
    assert_eq!(spans[0].style.fg, Some(Color::Yellow));

    let spans = JqHighlighter::highlight("or");
    assert_eq!(spans.len(), 1);
    assert_eq!(spans[0].style.fg, Some(Color::Yellow));
}

#[test]
fn test_comparison_in_context() {
    let spans = JqHighlighter::highlight(".age >= 18");
    assert!(spans.len() >= 5);
    let op_span = spans.iter().find(|s| s.content == ">=");
    assert!(op_span.is_some());
    assert_eq!(op_span.unwrap().style.fg, Some(Color::Magenta));
}

#[test]
fn test_empty_keyword() {
    let spans = JqHighlighter::highlight("empty");
    assert_eq!(spans.len(), 1);
    assert_eq!(spans[0].style.fg, Some(Color::Yellow));
}

#[test]
fn test_unicode_in_string() {
    let spans = JqHighlighter::highlight(r#""hello 世界""#);
    assert_eq!(spans.len(), 1);
    assert_eq!(spans[0].style.fg, Some(Color::Green));
}

#[test]
fn test_array_indexing() {
    let spans = JqHighlighter::highlight(".items[0]");
    assert!(spans.len() >= 3);
}

#[test]
fn test_keywords_inside_strings_not_highlighted() {
    let spans = JqHighlighter::highlight(r#""if then else""#);
    assert_eq!(spans.len(), 1);
    assert_eq!(spans[0].content, r#""if then else""#);
    assert_eq!(spans[0].style.fg, Some(Color::Green));
}

#[test]
fn test_query_with_string_containing_keywords() {
    let spans = JqHighlighter::highlight(r#"select(.status == "if")"#);

    let string_span = spans.iter().find(|s| s.content == r#""if""#);
    assert!(string_span.is_some());
    assert_eq!(string_span.unwrap().style.fg, Some(Color::Green));

    let select_span = spans.iter().find(|s| s.content == "select");
    assert!(select_span.is_some());
    assert_eq!(select_span.unwrap().style.fg, Some(Color::Blue));
}

#[test]
fn test_object_field_names_highlighted() {
    let spans = JqHighlighter::highlight("{name: .name}");

    let field_span = spans.iter().find(|s| s.content == "name");
    assert!(field_span.is_some());
    assert_eq!(field_span.unwrap().style.fg, Some(Color::Cyan));

    let accessor_span = spans.iter().find(|s| s.content == ".name");
    assert!(accessor_span.is_some());
    assert_eq!(accessor_span.unwrap().style.fg, None);
}

#[test]
fn test_object_with_multiple_fields() {
    let spans = JqHighlighter::highlight("{firstName: .first, lastName: .last, age: .age}");

    for field_name in ["firstName", "lastName", "age"] {
        let field_span = spans.iter().find(|s| s.content == field_name);
        assert!(field_span.is_some());
        assert_eq!(field_span.unwrap().style.fg, Some(Color::Cyan));
    }

    for accessor in [".first", ".last", ".age"] {
        let accessor_span = spans.iter().find(|s| s.content == accessor);
        assert!(accessor_span.is_some());
        assert_eq!(accessor_span.unwrap().style.fg, None);
    }
}

#[test]
fn test_object_field_with_whitespace_before_colon() {
    let spans = JqHighlighter::highlight("{name : .value}");

    let field_span = spans.iter().find(|s| s.content == "name");
    assert!(field_span.is_some());
    assert_eq!(field_span.unwrap().style.fg, Some(Color::Cyan));
}
