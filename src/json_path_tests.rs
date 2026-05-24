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

mod line_at_path_tests {
    use super::*;

    #[test]
    fn root_path_is_line_zero() {
        let v = json!({"a": 1, "b": 2});
        assert_eq!(line_at_path(&v, &JsonPath::new()), Some(0));
    }

    #[test]
    fn scalar_root_is_line_zero() {
        let v = json!(42);
        assert_eq!(line_at_path(&v, &JsonPath::new()), Some(0));
    }

    #[test]
    fn object_key_lines_are_walked_in_input_order() {
        let v: Value = serde_json::from_str(r#"{"z": 1, "a": 2, "m": 3}"#).unwrap();
        let mut p = JsonPath::new();
        p.push_key("z");
        assert_eq!(line_at_path(&v, &p), Some(1));
        let mut p = JsonPath::new();
        p.push_key("a");
        assert_eq!(line_at_path(&v, &p), Some(2));
        let mut p = JsonPath::new();
        p.push_key("m");
        assert_eq!(line_at_path(&v, &p), Some(3));
    }

    #[test]
    fn array_index_lines_are_walked() {
        let v = json!([10, 20, 30]);
        for i in 0..3 {
            let mut p = JsonPath::new();
            p.push_index(i);
            assert_eq!(line_at_path(&v, &p), Some(i + 1));
        }
    }

    #[test]
    fn nested_object_lines_account_for_prior_siblings_recursively() {
        let v = json!({"users": [{"name": "alice"}, {"name": "bob"}]});
        let mut p = JsonPath::new();
        p.push_key("users");
        assert_eq!(line_at_path(&v, &p), Some(1));
        p.push_index(0);
        assert_eq!(line_at_path(&v, &p), Some(2));
        p.push_key("name");
        assert_eq!(line_at_path(&v, &p), Some(3));
        let mut p2 = JsonPath::new();
        p2.push_key("users");
        p2.push_index(1);
        assert_eq!(line_at_path(&v, &p2), Some(5));
        p2.push_key("name");
        assert_eq!(line_at_path(&v, &p2), Some(6));
    }

    #[test]
    fn missing_key_returns_none() {
        let v = json!({"a": 1});
        let mut p = JsonPath::new();
        p.push_key("missing");
        assert_eq!(line_at_path(&v, &p), None);
    }

    #[test]
    fn out_of_bounds_index_returns_none() {
        let v = json!([1, 2]);
        let mut p = JsonPath::new();
        p.push_index(5);
        assert_eq!(line_at_path(&v, &p), None);
    }

    #[test]
    fn type_mismatch_returns_none() {
        let v = json!({"a": 1});
        let mut p = JsonPath::new();
        p.push_index(0);
        assert_eq!(line_at_path(&v, &p), None);
    }

    #[test]
    fn round_trip_with_path_at_line_for_every_row() {
        // For every line in the pretty-printed value, path_at_line(line)
        // must return a path whose line_at_path lands at the same line
        // (modulo closing-bracket lines, which map back to the parent).
        let v = json!({
            "users": [
                {"name": "alice", "age": 30},
                {"name": "bob", "age": 25},
            ],
            "meta": {"count": 2}
        });
        let pretty = serde_json::to_string_pretty(&v).unwrap();
        for (i, _) in pretty.lines().enumerate() {
            let path = path_at_line(&v, i).expect("path exists");
            let back = line_at_path(&v, &path).expect("line exists");
            // Closing brackets map back to the parent, so the round-trip
            // is allowed to land on or before the original line.
            assert!(
                back <= i,
                "line {} path {:?} mapped back to {}",
                i,
                path,
                back
            );
        }
    }

    #[test]
    fn special_keys_are_resolved() {
        let v: Value = serde_json::from_str(r#"{"foo-bar": {"nested": 1}}"#).unwrap();
        let mut p = JsonPath::new();
        p.push_key("foo-bar");
        assert_eq!(line_at_path(&v, &p), Some(1));
        p.push_key("nested");
        assert_eq!(line_at_path(&v, &p), Some(2));
    }

    #[test]
    fn array_of_objects_indices_land_on_open_brace_lines() {
        // Pretty layout for [{"a":1},{"b":2}]:
        // 0: [
        // 1:   {     ← .[0]
        // 2:     "a": 1
        // 3:   },
        // 4:   {     ← .[1]
        // 5:     "b": 2
        // 6:   }
        // 7: ]
        let v = json!([{"a": 1}, {"b": 2}]);
        let mut p0 = JsonPath::new();
        p0.push_index(0);
        assert_eq!(line_at_path(&v, &p0), Some(1));
        let mut p1 = JsonPath::new();
        p1.push_index(1);
        assert_eq!(line_at_path(&v, &p1), Some(4));
    }
}

mod sibling_at_tests {
    use super::*;

    fn key(k: &str) -> JsonPath {
        let mut p = JsonPath::new();
        p.push_key(k);
        p
    }

    fn keys(parts: &[&str]) -> JsonPath {
        let mut p = JsonPath::new();
        for k in parts {
            p.push_key(*k);
        }
        p
    }

    fn idx(i: usize) -> JsonPath {
        let mut p = JsonPath::new();
        p.push_index(i);
        p
    }

    fn key_then_idx(k: &str, i: usize) -> JsonPath {
        let mut p = JsonPath::new();
        p.push_key(k);
        p.push_index(i);
        p
    }

    fn assert_sibling(out: SiblingOutcome, expected: &str) {
        match out {
            SiblingOutcome::Sibling(p) => assert_eq!(p.to_jq(), expected),
            other => panic!("expected Sibling({}), got {:?}", expected, other),
        }
    }

    #[test]
    fn next_walks_object_keys_in_input_order() {
        let v: Value = serde_json::from_str(r#"{"z": 1, "a": 2, "m": 3}"#).unwrap();
        assert_sibling(sibling_at(&v, &key("z"), SiblingDir::Next), ".a");
        assert_sibling(sibling_at(&v, &key("a"), SiblingDir::Next), ".m");
    }

    #[test]
    fn prev_walks_object_keys_in_reverse_input_order() {
        let v: Value = serde_json::from_str(r#"{"z": 1, "a": 2, "m": 3}"#).unwrap();
        assert_sibling(sibling_at(&v, &key("m"), SiblingDir::Prev), ".a");
        assert_sibling(sibling_at(&v, &key("a"), SiblingDir::Prev), ".z");
    }

    #[test]
    fn next_wraps_at_last_object_key() {
        let v: Value = serde_json::from_str(r#"{"z": 1, "a": 2, "m": 3}"#).unwrap();
        assert_sibling(sibling_at(&v, &key("m"), SiblingDir::Next), ".z");
    }

    #[test]
    fn prev_wraps_at_first_object_key() {
        let v: Value = serde_json::from_str(r#"{"z": 1, "a": 2, "m": 3}"#).unwrap();
        assert_sibling(sibling_at(&v, &key("z"), SiblingDir::Prev), ".m");
    }

    #[test]
    fn next_walks_array_indices() {
        let v = json!([10, 20, 30]);
        assert_sibling(sibling_at(&v, &idx(0), SiblingDir::Next), ".[1]");
        assert_sibling(sibling_at(&v, &idx(1), SiblingDir::Next), ".[2]");
    }

    #[test]
    fn prev_walks_array_indices() {
        let v = json!([10, 20, 30]);
        assert_sibling(sibling_at(&v, &idx(2), SiblingDir::Prev), ".[1]");
        assert_sibling(sibling_at(&v, &idx(1), SiblingDir::Prev), ".[0]");
    }

    #[test]
    fn next_wraps_at_last_array_index() {
        let v = json!([10, 20, 30]);
        assert_sibling(sibling_at(&v, &idx(2), SiblingDir::Next), ".[0]");
    }

    #[test]
    fn prev_wraps_at_first_array_index() {
        let v = json!([10, 20, 30]);
        assert_sibling(sibling_at(&v, &idx(0), SiblingDir::Prev), ".[2]");
    }

    #[test]
    fn root_path_returns_no_parent() {
        let v = json!({"a": 1});
        assert_eq!(
            sibling_at(&v, &JsonPath::new(), SiblingDir::Next),
            SiblingOutcome::NoParent
        );
        assert_eq!(
            sibling_at(&v, &JsonPath::new(), SiblingDir::Prev),
            SiblingOutcome::NoParent
        );
    }

    #[test]
    fn single_object_child_returns_single() {
        let v = json!({"only": 1});
        assert_eq!(
            sibling_at(&v, &key("only"), SiblingDir::Next),
            SiblingOutcome::Single
        );
        assert_eq!(
            sibling_at(&v, &key("only"), SiblingDir::Prev),
            SiblingOutcome::Single
        );
    }

    #[test]
    fn single_array_element_returns_single() {
        let v = json!([42]);
        assert_eq!(
            sibling_at(&v, &idx(0), SiblingDir::Next),
            SiblingOutcome::Single
        );
        assert_eq!(
            sibling_at(&v, &idx(0), SiblingDir::Prev),
            SiblingOutcome::Single
        );
    }

    #[test]
    fn key_step_with_array_parent_is_invalid() {
        let v = json!([1, 2, 3]);
        assert_eq!(
            sibling_at(&v, &key("foo"), SiblingDir::Next),
            SiblingOutcome::Invalid
        );
    }

    #[test]
    fn index_step_with_object_parent_is_invalid() {
        let v = json!({"a": 1, "b": 2});
        assert_eq!(
            sibling_at(&v, &idx(0), SiblingDir::Next),
            SiblingOutcome::Invalid
        );
    }

    #[test]
    fn out_of_bounds_array_index_is_invalid() {
        let v = json!([1, 2]);
        assert_eq!(
            sibling_at(&v, &idx(5), SiblingDir::Next),
            SiblingOutcome::Invalid
        );
    }

    #[test]
    fn missing_object_key_is_invalid() {
        let v = json!({"a": 1, "b": 2});
        assert_eq!(
            sibling_at(&v, &key("missing"), SiblingDir::Next),
            SiblingOutcome::Invalid
        );
    }

    #[test]
    fn nested_array_in_object_walks_inner_siblings() {
        let v = json!({"users": [{"name": "alice"}, {"name": "bob"}, {"name": "carol"}]});
        assert_sibling(
            sibling_at(&v, &key_then_idx("users", 0), SiblingDir::Next),
            ".users[1]",
        );
        assert_sibling(
            sibling_at(&v, &key_then_idx("users", 2), SiblingDir::Next),
            ".users[0]",
        );
        assert_sibling(
            sibling_at(&v, &key_then_idx("users", 0), SiblingDir::Prev),
            ".users[2]",
        );
    }

    #[test]
    fn deeply_nested_object_keys_walk_correctly() {
        let v = json!({"outer": {"a": 1, "b": 2, "c": 3}});
        assert_sibling(
            sibling_at(&v, &keys(&["outer", "a"]), SiblingDir::Next),
            ".outer.b",
        );
        assert_sibling(
            sibling_at(&v, &keys(&["outer", "c"]), SiblingDir::Next),
            ".outer.a",
        );
    }

    #[test]
    fn special_chars_in_sibling_key_emit_bracket_notation() {
        let v: Value = serde_json::from_str(r#"{"normal": 1, "foo-bar": 2, "café": 3}"#).unwrap();
        assert_sibling(
            sibling_at(&v, &key("normal"), SiblingDir::Next),
            ".[\"foo-bar\"]",
        );
        assert_sibling(
            sibling_at(&v, &key("foo-bar"), SiblingDir::Next),
            ".[\"café\"]",
        );
        assert_sibling(sibling_at(&v, &key("café"), SiblingDir::Next), ".normal");
    }

    #[test]
    fn invalid_then_valid_step_in_path_is_invalid() {
        let v = json!({"users": [{"name": "alice"}]});
        let mut p = JsonPath::new();
        p.push_index(0);
        p.push_key("name");
        assert_eq!(
            sibling_at(&v, &p, SiblingDir::Next),
            SiblingOutcome::Invalid
        );
    }

    #[test]
    fn next_then_prev_round_trips_in_object() {
        let v: Value = serde_json::from_str(r#"{"a": 1, "b": 2, "c": 3}"#).unwrap();
        let next = match sibling_at(&v, &key("b"), SiblingDir::Next) {
            SiblingOutcome::Sibling(p) => p,
            other => panic!("{:?}", other),
        };
        assert_eq!(next.to_jq(), ".c");
        let back = match sibling_at(&v, &next, SiblingDir::Prev) {
            SiblingOutcome::Sibling(p) => p,
            other => panic!("{:?}", other),
        };
        assert_eq!(back.to_jq(), ".b");
    }

    #[test]
    fn next_then_prev_round_trips_with_wrap() {
        let v: Value = serde_json::from_str(r#"{"a": 1, "b": 2, "c": 3}"#).unwrap();
        let next = match sibling_at(&v, &key("c"), SiblingDir::Next) {
            SiblingOutcome::Sibling(p) => p,
            other => panic!("{:?}", other),
        };
        assert_eq!(next.to_jq(), ".a");
        let back = match sibling_at(&v, &next, SiblingDir::Prev) {
            SiblingOutcome::Sibling(p) => p,
            other => panic!("{:?}", other),
        };
        assert_eq!(back.to_jq(), ".c");
    }

    #[test]
    fn deep_nesting_does_not_blow_stack() {
        let mut v = json!(0i64);
        for _ in 0..200 {
            v = json!({ "n": v });
        }
        let mut p = JsonPath::new();
        for _ in 0..150 {
            p.push_key("n");
        }
        assert_eq!(sibling_at(&v, &p, SiblingDir::Next), SiblingOutcome::Single);
    }
}
