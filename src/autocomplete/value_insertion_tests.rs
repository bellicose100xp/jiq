use super::*;
use crate::autocomplete::value_trigger::{TriggerKind, ValueTrigger};

fn trigger_at_quote(query: &str) -> ValueTrigger {
    let qb = query.rfind('"').expect("quote present");
    let partial = query[qb + 1..].to_string();
    ValueTrigger {
        kind: TriggerKind::Eq,
        lhs_path: Some(".x".to_string()),
        partial,
        quote_open_byte: qb,
    }
}

#[test]
fn appends_close_quote_when_missing() {
    let q = "select(.x == \"ab";
    let trig = trigger_at_quote(q);
    let (new_q, cursor) = build_inserted(q, q.len(), &trig, "abc");
    assert_eq!(new_q, "select(.x == \"abc\"");
    assert_eq!(cursor, new_q.len());
}

#[test]
fn replaces_existing_close_quote() {
    let q = "select(.x == \"ab\")";
    let cursor = q.find("ab").unwrap() + 2;
    let trig = ValueTrigger {
        kind: TriggerKind::Eq,
        lhs_path: Some(".x".to_string()),
        partial: "ab".to_string(),
        quote_open_byte: q.find('"').unwrap(),
    };
    let (new_q, cursor_after) = build_inserted(q, cursor, &trig, "abc");
    assert_eq!(new_q, "select(.x == \"abc\")");
    let close_quote_byte = new_q.rfind('"').unwrap();
    assert_eq!(cursor_after, close_quote_byte + 1);
}

#[test]
fn escapes_embedded_double_quote() {
    let q = "select(.x == \"";
    let trig = trigger_at_quote(q);
    let (new_q, _) = build_inserted(q, q.len(), &trig, "say \"hi\"");
    assert_eq!(new_q, "select(.x == \"say \\\"hi\\\"\"");
}

#[test]
fn escapes_backslash() {
    let q = "select(.x == \"";
    let trig = trigger_at_quote(q);
    let (new_q, _) = build_inserted(q, q.len(), &trig, r"a\b");
    assert_eq!(new_q, "select(.x == \"a\\\\b\"");
}

#[test]
fn escapes_control_chars() {
    let q = "select(.x == \"";
    let trig = trigger_at_quote(q);
    let (new_q, _) = build_inserted(q, q.len(), &trig, "line1\nline2");
    assert_eq!(new_q, "select(.x == \"line1\\nline2\"");
}

#[test]
fn preserves_utf8_in_value() {
    let q = "select(.name == \"";
    let trig = trigger_at_quote(q);
    let (new_q, _) = build_inserted(q, q.len(), &trig, "üñ");
    assert_eq!(new_q, "select(.name == \"üñ\"");
}

#[test]
fn preserves_text_after_cursor() {
    let q = "select(.x == \"\") | length";
    // cursor is between "" — the empty quoted string
    let cursor = q.find("\"\"").unwrap() + 1;
    let trig = ValueTrigger {
        kind: TriggerKind::Eq,
        lhs_path: Some(".x".to_string()),
        partial: String::new(),
        quote_open_byte: q.find('"').unwrap(),
    };
    let (new_q, _) = build_inserted(q, cursor, &trig, "yes");
    assert_eq!(new_q, "select(.x == \"yes\") | length");
}

#[test]
fn cursor_lands_after_close_quote() {
    let q = "select(.x == \"";
    let trig = trigger_at_quote(q);
    let (new_q, cursor) = build_inserted(q, q.len(), &trig, "abc");
    assert_eq!(&new_q[cursor.saturating_sub(1)..cursor], "\"");
}

#[test]
fn empty_value_just_closes_quote() {
    let q = "select(.x == \"";
    let trig = trigger_at_quote(q);
    let (new_q, _) = build_inserted(q, q.len(), &trig, "");
    assert_eq!(new_q, "select(.x == \"\"");
}

#[test]
fn does_not_double_escape() {
    // Roundtrip: the escaped output should never contain unescaped \" or \\.
    let q = "x == \"";
    let trig = trigger_at_quote(q);
    let (new_q, _) = build_inserted(q, q.len(), &trig, r#"a"b\c"#);
    assert!(new_q.contains(r#"\"b"#));
    assert!(new_q.contains(r#"\\c"#));
}

#[test]
fn cursor_at_close_quote_with_existing_string() {
    // User typed select(.x == "ab"|) | length where | is cursor — popup shows.
    // Selecting "abc" should produce select(.x == "abc") | length
    let q = "select(.x == \"ab\") | length";
    let cursor = q.find("ab\"").unwrap() + 2;
    let trig = ValueTrigger {
        kind: TriggerKind::Eq,
        lhs_path: Some(".x".to_string()),
        partial: "ab".to_string(),
        quote_open_byte: q.find('"').unwrap(),
    };
    let (new_q, _) = build_inserted(q, cursor, &trig, "abc");
    assert_eq!(new_q, "select(.x == \"abc\") | length");
}

#[test]
fn no_double_quote_when_user_already_typed_close() {
    let q = "select(.x == \"\") | length";
    let cursor = q.find("\"\"").unwrap() + 1;
    let trig = ValueTrigger {
        kind: TriggerKind::Eq,
        lhs_path: Some(".x".to_string()),
        partial: String::new(),
        quote_open_byte: q.find('"').unwrap(),
    };
    let (new_q, _) = build_inserted(q, cursor, &trig, "yes");
    let count = new_q.matches('"').count();
    assert_eq!(count, 2, "expected exactly 2 quote chars in {new_q:?}");
}

#[test]
fn cursor_inside_partial_with_existing_close_quote() {
    // Cursor sits between 'a' and 'b' in "ab" — partial = "a", but the
    // string already has a close quote later. Insertion must NOT leave a
    // dangling 'b' between value and close quote.
    let q = "select(.x == \"ab\")";
    let cursor = q.find("ab").unwrap() + 1;
    let trig = ValueTrigger {
        kind: TriggerKind::Eq,
        lhs_path: Some(".x".to_string()),
        partial: "a".to_string(),
        quote_open_byte: q.find('"').unwrap(),
    };
    let (new_q, _) = build_inserted(q, cursor, &trig, "abc");
    assert_eq!(new_q, "select(.x == \"abc\")");
    let quote_count = new_q.matches('"').count();
    assert_eq!(quote_count, 2, "expected exactly 2 quotes in {new_q:?}");
}

#[test]
fn cursor_anywhere_in_string_body_replaces_whole_body() {
    // The new behavior: regardless of cursor position inside the string,
    // the whole body up to the close quote is replaced by the value.
    let q = "select(.x == \"abcdef\")";
    let trig = ValueTrigger {
        kind: TriggerKind::Eq,
        lhs_path: Some(".x".to_string()),
        partial: "abc".to_string(),
        quote_open_byte: q.find('"').unwrap(),
    };
    // Try cursors at multiple points inside "abcdef"
    for pos in 0..="abcdef".len() {
        let cursor = q.find('"').unwrap() + 1 + pos;
        let (new_q, _) = build_inserted(q, cursor, &trig, "X");
        assert_eq!(new_q, "select(.x == \"X\")");
    }
}

#[test]
fn jq_round_trip_property() {
    // For a small set of representative values, confirm that the inserted
    // string parses back to the original via a manual unescape that matches
    // jq's rules. (Avoids spawning jq for unit tests; the manual unescape is
    // narrow but covers the escapes we emit.)
    let cases = [
        "plain",
        "with space",
        "üñ",
        r#"a"b"#,
        r"a\b",
        "tab\there",
        "newline\nhere",
        "",
    ];
    for value in cases {
        let escaped = escape_jq_string(value);
        let parsed: String = serde_json::from_str(&format!("\"{}\"", escaped))
            .expect("escaped string is valid JSON");
        assert_eq!(parsed, value, "round trip failed for {value:?}");
    }
}
