use super::*;

#[test]
fn ascii_identifier_unchanged() {
    assert_eq!(sanitize_jq_query(".foo"), ".foo");
    assert_eq!(sanitize_jq_query(".foo.bar"), ".foo.bar");
    assert_eq!(sanitize_jq_query(".items[].name"), ".items[].name");
}

#[test]
fn cjk_key_rewritten() {
    assert_eq!(sanitize_jq_query(".名前"), r#".["名前"]"#);
    assert_eq!(sanitize_jq_query(".中文.year"), r#".["中文"].year"#);
}

#[test]
fn emoji_key_rewritten() {
    assert_eq!(sanitize_jq_query(".👋"), r#".["👋"]"#);
}

#[test]
fn accented_key_rewritten() {
    assert_eq!(sanitize_jq_query(".café"), r#".["café"]"#);
    assert_eq!(
        sanitize_jq_query(".résumé.título"),
        r#".["résumé"].["título"]"#
    );
}

#[test]
fn multiple_non_ascii_segments() {
    assert_eq!(
        sanitize_jq_query(".名前, .年齢, .職業"),
        r#".["名前"], .["年齢"], .["職業"]"#
    );
}

#[test]
fn nested_non_ascii_composition() {
    assert_eq!(sanitize_jq_query(".会社.名前"), r#".["会社"].["名前"]"#);
}

#[test]
fn non_ascii_with_array_iteration() {
    assert_eq!(sanitize_jq_query(".趣味[]"), r#".["趣味"][]"#);
}

#[test]
fn non_ascii_with_pipe() {
    assert_eq!(sanitize_jq_query(".住所 | keys"), r#".["住所"] | keys"#);
}

#[test]
fn already_bracket_notation_unchanged() {
    assert_eq!(sanitize_jq_query(r#".["名前"]"#), r#".["名前"]"#);
    assert_eq!(
        sanitize_jq_query(r#".["会社"]["名前"]"#),
        r#".["会社"]["名前"]"#
    );
}

#[test]
fn string_literals_preserved() {
    // A non-ASCII char inside a string literal must not be touched
    assert_eq!(
        sanitize_jq_query(r#".field | select(. == "名前")"#),
        r#".field | select(. == "名前")"#
    );
}

#[test]
fn escaped_quotes_in_strings_preserved() {
    assert_eq!(
        sanitize_jq_query(r#".[\"名前\"] | join(\", \")"#),
        r#".[\"名前\"] | join(\", \")"#
    );
}

#[test]
fn empty_string_noop() {
    assert_eq!(sanitize_jq_query(""), "");
}

#[test]
fn identity_dot_unchanged() {
    assert_eq!(sanitize_jq_query("."), ".");
    assert_eq!(sanitize_jq_query(". | length"), ". | length");
}

#[test]
fn bracket_access_with_index_unchanged() {
    assert_eq!(sanitize_jq_query(".items[0]"), ".items[0]");
    assert_eq!(sanitize_jq_query(".[0]"), ".[0]");
}

#[test]
fn complex_mixed_query() {
    assert_eq!(
        sanitize_jq_query(".users[].名前 | select(. != null)"),
        r#".users[].["名前"] | select(. != null)"#
    );
}

#[test]
fn alternative_operator_preserved() {
    // `//` is jq's alternative operator; sanitizer must not disturb it
    assert_eq!(sanitize_jq_query(".a // .b"), ".a // .b");
    assert_eq!(
        sanitize_jq_query(".名前 // \"unknown\""),
        r#".["名前"] // "unknown""#
    );
}

#[test]
fn format_string_interpolation_preserved() {
    // jq's @text "\(...)" interpolation contains `.x` inside a string —
    // must be preserved untouched
    assert_eq!(sanitize_jq_query(r#"@text "\(.x)""#), r#"@text "\(.x)""#);
    assert_eq!(
        sanitize_jq_query(r#"@csv "\(.a),\(.b)""#),
        r#"@csv "\(.a),\(.b)""#
    );
}

#[test]
fn key_with_embedded_backslash_is_escaped() {
    // `.a\b` captures `a\b` as the key (backslash isn't a jq separator),
    // fails the ASCII-identifier check (backslash isn't alphanumeric),
    // so gets bracket-wrapped with the backslash doubled for escape safety.
    // (`"` is a separator, so a literal quote can't appear inside a captured
    // name — the escape logic for `"` is defensive coverage only.)
    assert_eq!(sanitize_jq_query(".a\\b"), ".[\"a\\\\b\"]");
}
