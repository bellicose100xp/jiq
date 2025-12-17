use super::*;
use insta::assert_yaml_snapshot;

#[test]
fn snapshot_empty_input() {
    let spans = JqHighlighter::highlight("");
    assert_yaml_snapshot!(serialize_spans(&spans));
}

#[test]
fn snapshot_identity_filter() {
    let spans = JqHighlighter::highlight(".");
    assert_yaml_snapshot!(serialize_spans(&spans));
}

#[test]
fn snapshot_simple_field() {
    let spans = JqHighlighter::highlight(".name");
    assert_yaml_snapshot!(serialize_spans(&spans));
}

#[test]
fn snapshot_nested_field() {
    let spans = JqHighlighter::highlight(".user.address.city");
    assert_yaml_snapshot!(serialize_spans(&spans));
}

#[test]
fn snapshot_keywords() {
    let keywords = vec![
        "if", "then", "else", "elif", "end", "and", "or", "not", "as", "def", "reduce", "foreach",
        "try", "catch", "empty", "null", "true", "false",
    ];

    let results: Vec<_> = keywords
        .iter()
        .map(|kw| {
            (
                kw.to_string(),
                serialize_spans(&JqHighlighter::highlight(kw)),
            )
        })
        .collect();

    assert_yaml_snapshot!(results);
}

#[test]
fn snapshot_common_functions() {
    let functions = vec![
        "map", "select", "sort", "keys", "values", "length", "type", "add", "first", "last", "has",
        "contains", "split", "join",
    ];

    let results: Vec<_> = functions
        .iter()
        .map(|f| (f.to_string(), serialize_spans(&JqHighlighter::highlight(f))))
        .collect();

    assert_yaml_snapshot!(results);
}

#[test]
fn snapshot_operators() {
    let operators = vec![
        "|", "==", "!=", "<=", ">=", "//", "+", "-", "*", "/", "%", "(", ")", "[", "]", "{", "}",
        ",", ";", ":", "?",
    ];

    let results: Vec<_> = operators
        .iter()
        .map(|op| {
            (
                op.to_string(),
                serialize_spans(&JqHighlighter::highlight(op)),
            )
        })
        .collect();

    assert_yaml_snapshot!(results);
}

#[test]
fn snapshot_string_literals() {
    let strings = [
        r#""hello""#,
        r#""hello world""#,
        r#""hello \"escaped\" world""#,
        r#""unicode: 世界""#,
    ];

    let results: Vec<_> = strings
        .iter()
        .map(|s| (s.to_string(), serialize_spans(&JqHighlighter::highlight(s))))
        .collect();

    assert_yaml_snapshot!(results);
}

#[test]
fn snapshot_number_literals() {
    let numbers = ["0", "42", "-123", "3.14", "-0.5"];

    let results: Vec<_> = numbers
        .iter()
        .map(|n| (n.to_string(), serialize_spans(&JqHighlighter::highlight(n))))
        .collect();

    assert_yaml_snapshot!(results);
}

#[test]
fn snapshot_array_iteration() {
    let spans = JqHighlighter::highlight(".items[] | select(.active)");
    assert_yaml_snapshot!(serialize_spans(&spans));
}

#[test]
fn snapshot_if_then_else() {
    let spans = JqHighlighter::highlight(r#"if .value > 10 then "high" else "low" end"#);
    assert_yaml_snapshot!(serialize_spans(&spans));
}

#[test]
fn snapshot_object_construction() {
    let spans = JqHighlighter::highlight("{name: .name, age: .age, active: true}");
    assert_yaml_snapshot!(serialize_spans(&spans));
}

#[test]
fn snapshot_pipe_chain() {
    let spans = JqHighlighter::highlight(".users | map(.name) | sort | unique");
    assert_yaml_snapshot!(serialize_spans(&spans));
}

#[test]
fn snapshot_reduce() {
    let spans = JqHighlighter::highlight("reduce .[] as $x (0; . + $x)");
    assert_yaml_snapshot!(serialize_spans(&spans));
}

#[test]
fn snapshot_unterminated_string() {
    let spans = JqHighlighter::highlight(r#""unterminated"#);
    assert_yaml_snapshot!(serialize_spans(&spans));
}

#[test]
fn snapshot_keywords_inside_string() {
    let spans = JqHighlighter::highlight(r#""if then else are keywords""#);
    assert_yaml_snapshot!(serialize_spans(&spans));
}

#[test]
fn snapshot_whitespace_handling() {
    let spans = JqHighlighter::highlight("  .name  |  .age  ");
    assert_yaml_snapshot!(serialize_spans(&spans));
}
