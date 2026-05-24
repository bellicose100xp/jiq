use super::*;
use serde_json::json;

#[test]
fn simple_identifier_accepts_basic_names() {
    assert!(is_simple_jq_identifier("foo"));
    assert!(is_simple_jq_identifier("foo_bar"));
    assert!(is_simple_jq_identifier("user1"));
    assert!(is_simple_jq_identifier("_private"));
}

#[test]
fn simple_identifier_rejects_problem_names() {
    assert!(!is_simple_jq_identifier(""));
    assert!(!is_simple_jq_identifier("1abc"));
    assert!(!is_simple_jq_identifier("foo-bar"));
    assert!(!is_simple_jq_identifier("foo.bar"));
    assert!(!is_simple_jq_identifier("café"));
    assert!(!is_simple_jq_identifier("中文"));
    assert!(!is_simple_jq_identifier("with space"));
}

#[test]
fn bracket_access_escapes_special_chars() {
    assert_eq!(format_bracket_access("foo"), "[\"foo\"]");
    assert_eq!(format_bracket_access("foo-bar"), "[\"foo-bar\"]");
    assert_eq!(format_bracket_access("a\"b"), "[\"a\\\"b\"]");
    assert_eq!(format_bracket_access("a\\b"), "[\"a\\\\b\"]");
    assert_eq!(format_bracket_access("a\nb"), "[\"a\\nb\"]");
}

#[test]
fn bracket_access_handles_unicode() {
    assert_eq!(format_bracket_access("café"), "[\"café\"]");
    assert_eq!(format_bracket_access("中文"), "[\"中文\"]");
    assert_eq!(format_bracket_access("👋"), "[\"👋\"]");
}

#[test]
fn format_field_name_picks_dot_or_bracket() {
    assert_eq!(format_field_name(".", "foo"), ".foo");
    assert_eq!(format_field_name(".user.", "name"), ".user.name");
    assert_eq!(format_field_name(".", "foo-bar"), ".[\"foo-bar\"]");
    assert_eq!(format_field_name(".user", "café"), ".user[\"café\"]");
}

#[test]
fn empty_path_renders_as_root() {
    let p = JsonPath::new();
    assert_eq!(p.to_jq(), ".");
    assert!(p.is_empty());
}

#[test]
fn jq_path_simple_keys() {
    let mut p = JsonPath::new();
    p.push_key("users");
    p.push_index(2);
    p.push_key("name");
    assert_eq!(p.to_jq(), ".users[2].name");
}

#[test]
fn jq_path_mixed_simple_and_quoted() {
    let mut p = JsonPath::new();
    p.push_key("user-info");
    p.push_key("zip-code");
    assert_eq!(p.to_jq(), ".[\"user-info\"][\"zip-code\"]");
}

#[test]
fn jq_path_simple_then_quoted() {
    let mut p = JsonPath::new();
    p.push_key("users");
    p.push_index(0);
    p.push_key("zip-code");
    assert_eq!(p.to_jq(), ".users[0][\"zip-code\"]");
}

#[test]
fn jq_path_unicode_keys() {
    let mut p = JsonPath::new();
    p.push_key("café");
    p.push_index(0);
    p.push_key("名前");
    assert_eq!(p.to_jq(), ".[\"café\"][0][\"名前\"]");
}

#[test]
fn jq_path_steps_accessor() {
    let mut p = JsonPath::new();
    p.push_key("a");
    p.push_index(3);
    let steps = p.steps();
    assert_eq!(steps.len(), 2);
    assert_eq!(steps[0], JsonPathStep::Key("a".into()));
    assert_eq!(steps[1], JsonPathStep::Index(3));
}

#[test]
fn jq_path_pop() {
    let mut p = JsonPath::new();
    p.push_key("a");
    p.push_index(3);
    p.pop();
    assert_eq!(p.to_jq(), ".a");
}

#[test]
fn splat_replaces_rightmost_index() {
    let mut p = JsonPath::new();
    p.push_key("users");
    p.push_index(2);
    p.push_key("tags");
    p.push_index(1);
    assert!(p.splat_nearest_index());
    assert_eq!(p.to_jq(), ".users[2].tags[]");
}

#[test]
fn splat_on_simple_indexed_path() {
    let mut p = JsonPath::new();
    p.push_index(5);
    assert!(p.splat_nearest_index());
    assert_eq!(p.to_jq(), ".[]");
}

#[test]
fn splat_returns_false_with_no_index() {
    let mut p = JsonPath::new();
    p.push_key("a");
    p.push_key("b");
    assert!(!p.splat_nearest_index());
    assert_eq!(p.to_jq(), ".a.b");
}

#[test]
fn splat_on_empty_path_returns_false() {
    let mut p = JsonPath::new();
    assert!(!p.splat_nearest_index());
    assert_eq!(p.to_jq(), ".");
}

#[test]
fn parse_root_returns_empty_path() {
    let p = parse_jq_path(".").unwrap();
    assert!(p.is_empty());
    let p = parse_jq_path("").unwrap();
    assert!(p.is_empty());
    let p = parse_jq_path("   ").unwrap();
    assert!(p.is_empty());
}

#[test]
fn parse_simple_keys() {
    let p = parse_jq_path(".users").unwrap();
    assert_eq!(p.to_jq(), ".users");
    let p = parse_jq_path(".users.profile.email").unwrap();
    assert_eq!(p.to_jq(), ".users.profile.email");
}

#[test]
fn parse_indices() {
    let p = parse_jq_path(".[5]").unwrap();
    assert_eq!(p.to_jq(), ".[5]");
    let p = parse_jq_path(".users[2].name").unwrap();
    assert_eq!(p.to_jq(), ".users[2].name");
}

#[test]
fn parse_splat() {
    let p = parse_jq_path(".[]").unwrap();
    assert_eq!(p.to_jq(), ".[]");
    let p = parse_jq_path(".users[].name").unwrap();
    assert_eq!(p.to_jq(), ".users[].name");
}

#[test]
fn parse_quoted_keys() {
    let p = parse_jq_path(".[\"foo-bar\"]").unwrap();
    assert_eq!(p.to_jq(), ".[\"foo-bar\"]");
    let p = parse_jq_path(".users[2][\"zip-code\"]").unwrap();
    assert_eq!(p.to_jq(), ".users[2][\"zip-code\"]");
}

#[test]
fn parse_unicode_quoted_keys() {
    let p = parse_jq_path(".[\"café\"][0][\"名前\"]").unwrap();
    assert_eq!(p.to_jq(), ".[\"café\"][0][\"名前\"]");
}

#[test]
fn parse_rejects_pipes_and_filters() {
    assert!(parse_jq_path(".users | .[2]").is_none());
    assert!(parse_jq_path("map(.x)").is_none());
    assert!(parse_jq_path(".users | length").is_none());
}

#[test]
fn parse_rejects_trailing_dot_and_unbalanced_brackets() {
    assert!(parse_jq_path(".users.").is_none());
    assert!(parse_jq_path(".[5").is_none());
    assert!(parse_jq_path(".[\"foo").is_none());
    assert!(parse_jq_path(".[5]extra").is_none());
}

#[test]
fn parse_round_trips_to_jq_emitted_paths() {
    let cases = [
        ".",
        ".a",
        ".users[2].profile.email",
        ".[\"foo-bar\"]",
        ".users[].tags[0]",
        ".users[0][\"zip-code\"]",
        ".[\"a/b\"][0]",
    ];
    for case in cases {
        let parsed = parse_jq_path(case).unwrap_or_else(|| panic!("parse failed: {}", case));
        assert_eq!(parsed.to_jq(), case, "round-trip mismatch for {}", case);
    }
}

#[test]
fn splat_followed_by_key_renders_correctly() {
    let mut p = JsonPath::new();
    p.push_key("users");
    p.push_index(0);
    p.push_key("name");
    assert!(p.splat_nearest_index());
    assert_eq!(p.to_jq(), ".users[].name");
}

#[test]
fn pretty_line_count_scalar() {
    assert_eq!(pretty_line_count(&json!(42)), 1);
    assert_eq!(pretty_line_count(&json!("hello")), 1);
    assert_eq!(pretty_line_count(&json!(null)), 1);
    assert_eq!(pretty_line_count(&json!(true)), 1);
}

#[test]
fn pretty_line_count_empty_collections() {
    assert_eq!(pretty_line_count(&json!([])), 1);
    assert_eq!(pretty_line_count(&json!({})), 1);
}

#[test]
fn pretty_line_count_matches_serde_pretty() {
    let cases = vec![
        json!({"a": 1, "b": 2}),
        json!([1, 2, 3]),
        json!({"users": [{"name": "alice"}, {"name": "bob"}]}),
        json!({"nested": {"deep": {"deeper": "value"}}}),
        json!([[1, 2], [3, 4]]),
        json!({"a": [], "b": {}}),
        json!([1, "a", true, null]),
    ];
    for v in cases {
        let pretty = serde_json::to_string_pretty(&v).unwrap();
        let actual_lines = pretty.lines().count();
        assert_eq!(
            pretty_line_count(&v),
            actual_lines,
            "mismatch for {}",
            pretty
        );
    }
}

#[test]
fn path_at_line_scalar_root() {
    let v = json!(42);
    let p = path_at_line(&v, 0).unwrap();
    assert_eq!(p.to_jq(), ".");
    assert!(path_at_line(&v, 1).is_none());
}

#[test]
fn path_at_line_object_keys() {
    let v = json!({"a": 1, "b": 2});
    let pretty = serde_json::to_string_pretty(&v).unwrap();
    assert_eq!(pretty.lines().count(), 4);
    assert_eq!(path_at_line(&v, 0).unwrap().to_jq(), ".");
    assert_eq!(path_at_line(&v, 1).unwrap().to_jq(), ".a");
    assert_eq!(path_at_line(&v, 2).unwrap().to_jq(), ".b");
    assert_eq!(path_at_line(&v, 3).unwrap().to_jq(), ".");
}

#[test]
fn path_at_line_array_indices() {
    let v = json!([10, 20, 30]);
    assert_eq!(path_at_line(&v, 0).unwrap().to_jq(), ".");
    assert_eq!(path_at_line(&v, 1).unwrap().to_jq(), ".[0]");
    assert_eq!(path_at_line(&v, 2).unwrap().to_jq(), ".[1]");
    assert_eq!(path_at_line(&v, 3).unwrap().to_jq(), ".[2]");
    assert_eq!(path_at_line(&v, 4).unwrap().to_jq(), ".");
}

#[test]
fn path_at_line_mixed_types_array() {
    let v = json!([1, "a", true, null]);
    assert_eq!(path_at_line(&v, 0).unwrap().to_jq(), ".");
    assert_eq!(path_at_line(&v, 1).unwrap().to_jq(), ".[0]");
    assert_eq!(path_at_line(&v, 2).unwrap().to_jq(), ".[1]");
    assert_eq!(path_at_line(&v, 3).unwrap().to_jq(), ".[2]");
    assert_eq!(path_at_line(&v, 4).unwrap().to_jq(), ".[3]");
    assert_eq!(path_at_line(&v, 5).unwrap().to_jq(), ".");
}

#[test]
fn path_at_line_nested_object() {
    let v = json!({"users": [{"name": "alice"}, {"name": "bob"}]});
    assert_eq!(path_at_line(&v, 0).unwrap().to_jq(), ".");
    assert_eq!(path_at_line(&v, 1).unwrap().to_jq(), ".users");
    assert_eq!(path_at_line(&v, 2).unwrap().to_jq(), ".users[0]");
    assert_eq!(path_at_line(&v, 3).unwrap().to_jq(), ".users[0].name");
    assert_eq!(path_at_line(&v, 4).unwrap().to_jq(), ".users[0]");
    assert_eq!(path_at_line(&v, 5).unwrap().to_jq(), ".users[1]");
    assert_eq!(path_at_line(&v, 6).unwrap().to_jq(), ".users[1].name");
    assert_eq!(path_at_line(&v, 7).unwrap().to_jq(), ".users[1]");
    assert_eq!(path_at_line(&v, 8).unwrap().to_jq(), ".users");
    assert_eq!(path_at_line(&v, 9).unwrap().to_jq(), ".");
    assert_eq!(path_at_line(&v, 10), None);
    assert_eq!(path_at_line(&v, 100), None);
}

#[test]
fn path_at_line_handles_special_keys() {
    let v = json!({"foo-bar": {"nested": 1}});
    assert_eq!(path_at_line(&v, 0).unwrap().to_jq(), ".");
    assert_eq!(path_at_line(&v, 1).unwrap().to_jq(), ".[\"foo-bar\"]");
    assert_eq!(
        path_at_line(&v, 2).unwrap().to_jq(),
        ".[\"foo-bar\"].nested"
    );
}

#[test]
fn path_at_line_handles_unicode_keys() {
    let v = json!({"café": {"name": "value"}});
    assert_eq!(path_at_line(&v, 1).unwrap().to_jq(), ".[\"café\"]");
    assert_eq!(path_at_line(&v, 2).unwrap().to_jq(), ".[\"café\"].name");
}

#[test]
fn path_at_line_empty_object_and_array() {
    let v = json!({"a": [], "b": {}, "c": 1});
    assert_eq!(pretty_line_count(&v), 5);
    assert_eq!(path_at_line(&v, 0).unwrap().to_jq(), ".");
    assert_eq!(path_at_line(&v, 1).unwrap().to_jq(), ".a");
    assert_eq!(path_at_line(&v, 2).unwrap().to_jq(), ".b");
    assert_eq!(path_at_line(&v, 3).unwrap().to_jq(), ".c");
    assert_eq!(path_at_line(&v, 4).unwrap().to_jq(), ".");
}

#[test]
fn deep_nesting_does_not_blow_stack() {
    let mut v = json!(0i64);
    for _ in 0..200 {
        v = json!({ "n": v });
    }
    let _ = path_at_line(&v, 50);
}

#[test]
fn path_at_line_returns_path_for_every_line_in_pretty_print() {
    let v = json!({
        "users": [
            {"name": "alice", "age": 30},
            {"name": "bob", "age": 25},
        ],
        "meta": {"count": 2}
    });
    let pretty = serde_json::to_string_pretty(&v).unwrap();
    for (i, _) in pretty.lines().enumerate() {
        let p = path_at_line(&v, i);
        assert!(p.is_some(), "line {} should have a path", i);
    }
}

#[test]
fn preserve_order_is_enabled() {
    let raw = r#"{"z": 1, "a": 2, "m": 3}"#;
    let v: Value = serde_json::from_str(raw).unwrap();
    let pretty = serde_json::to_string_pretty(&v).unwrap();
    let z = pretty.find("\"z\"").unwrap();
    let a = pretty.find("\"a\"").unwrap();
    let m = pretty.find("\"m\"").unwrap();
    assert!(
        z < a && a < m,
        "preserve_order must be enabled — keys reordered: {}",
        pretty
    );
}

#[test]
fn path_at_line_respects_input_order() {
    let raw = r#"{"z": 1, "a": 2, "m": 3}"#;
    let v: Value = serde_json::from_str(raw).unwrap();
    assert_eq!(path_at_line(&v, 1).unwrap().to_jq(), ".z");
    assert_eq!(path_at_line(&v, 2).unwrap().to_jq(), ".a");
    assert_eq!(path_at_line(&v, 3).unwrap().to_jq(), ".m");
}
