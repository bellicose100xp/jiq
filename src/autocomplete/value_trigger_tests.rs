use super::*;

fn classify_at_end(query: &str) -> Option<ValueTrigger> {
    classify(query, query.len())
}

#[test]
fn returns_none_outside_string() {
    assert!(classify_at_end("select(.x == ").is_none());
    assert!(classify_at_end(".items").is_none());
    assert!(classify_at_end("").is_none());
}

#[test]
fn returns_none_when_string_already_closed() {
    assert!(classify_at_end("select(.x == \"abc\")").is_none());
    assert!(classify_at_end("\"hello\"").is_none());
}

#[test]
fn returns_none_for_regex_functions() {
    assert!(classify_at_end("test(\"").is_none());
    assert!(classify_at_end("match(\"foo").is_none());
    assert!(classify_at_end("scan(\"x").is_none());
    assert!(classify_at_end("splits(\"a").is_none());
    assert!(classify_at_end("sub(\"a").is_none());
    assert!(classify_at_end("gsub(\"x").is_none());
}

#[test]
fn returns_none_inside_string_interpolation() {
    let q = "\"prefix\\(.x)";
    assert!(classify(q, q.len()).is_none());
}

#[test]
fn detects_eq_with_path() {
    let t = classify_at_end("select(.status == \"ac").expect("trigger");
    assert_eq!(t.kind, TriggerKind::Eq);
    assert_eq!(t.lhs_path.as_deref(), Some(".status"));
    assert_eq!(t.partial, "ac");
}

#[test]
fn detects_neq_with_path() {
    let t = classify_at_end(".user.role != \"ad").expect("trigger");
    assert_eq!(t.kind, TriggerKind::Neq);
    assert_eq!(t.lhs_path.as_deref(), Some(".user.role"));
    assert_eq!(t.partial, "ad");
}

#[test]
fn detects_eq_without_lhs_path() {
    let t = classify_at_end("== \"foo").expect("trigger");
    assert_eq!(t.kind, TriggerKind::Eq);
    assert!(t.lhs_path.is_none());
    assert_eq!(t.partial, "foo");
}

#[test]
fn detects_eq_with_no_space_before_op() {
    let t = classify_at_end(".status==\"ac").expect("trigger");
    assert_eq!(t.kind, TriggerKind::Eq);
    assert_eq!(t.lhs_path.as_deref(), Some(".status"));
    assert_eq!(t.partial, "ac");
}

#[test]
fn detects_contains_no_lhs() {
    let t = classify_at_end("contains(\"ab").expect("trigger");
    assert_eq!(t.kind, TriggerKind::Contains);
    assert!(t.lhs_path.is_none());
    assert_eq!(t.partial, "ab");
}

#[test]
fn detects_contains_with_pre_call_path() {
    let t = classify_at_end(".name | contains(\"x").expect("trigger");
    assert_eq!(t.kind, TriggerKind::Contains);
    assert_eq!(t.lhs_path.as_deref(), Some(".name"));
}

#[test]
fn detects_startswith() {
    let t = classify_at_end("startswith(\"pre").expect("trigger");
    assert_eq!(t.kind, TriggerKind::StartsWith);
    assert_eq!(t.partial, "pre");
}

#[test]
fn detects_endswith() {
    let t = classify_at_end("endswith(\".log").expect("trigger");
    assert_eq!(t.kind, TriggerKind::EndsWith);
    assert_eq!(t.partial, ".log");
}

#[test]
fn detects_inside() {
    let t = classify_at_end("inside(\"abc").expect("trigger");
    assert_eq!(t.kind, TriggerKind::Inside);
}

#[test]
fn detects_in_with_lhs_path() {
    let t = classify_at_end("IN(.role; \"ad").expect("trigger");
    assert_eq!(t.kind, TriggerKind::In);
    assert_eq!(t.lhs_path.as_deref(), Some(".role"));
    assert_eq!(t.partial, "ad");
}

#[test]
fn detects_in_without_lhs_path() {
    let t = classify_at_end("IN(\"ad").expect("trigger");
    assert_eq!(t.kind, TriggerKind::In);
    assert!(t.lhs_path.is_none());
}

#[test]
fn detects_has_or_in() {
    let t = classify_at_end("has(\"k").expect("trigger");
    assert_eq!(t.kind, TriggerKind::HasOrIn);
    let t = classify_at_end(".x | in(\"k").expect("trigger");
    assert_eq!(t.kind, TriggerKind::HasOrIn);
}

#[test]
fn handles_nested_call() {
    // `map(select(.role == "ad` — fold prepends `[]` for `map` and
    // identity-walks `select`, yielding `[].role`. Without an LHS to the
    // map, the absolute path can't be rooted — but the iterator step
    // is still meaningful as a top-level array iteration.
    let t = classify_at_end("map(select(.role == \"ad").expect("trigger");
    assert_eq!(t.kind, TriggerKind::Eq);
    assert_eq!(t.lhs_path.as_deref(), Some(".[].role"));
    assert_eq!(t.partial, "ad");
}

#[test]
fn cursor_at_opening_quote_returns_empty_partial() {
    let q = "select(.status == \"";
    let t = classify(q, q.len()).expect("trigger");
    assert_eq!(t.partial, "");
    assert_eq!(t.kind, TriggerKind::Eq);
}

#[test]
fn unmatched_earlier_close_quotes_dont_confuse() {
    // The LHS of the pipe is an array literal `["a", "b"]`, not a path.
    // Folding can't anchor `.x` to a root-relative path, so lhs_path is None
    // and the dispatcher falls through to the global string list. The
    // important guarantee is that the trigger STILL fires (kind + partial
    // are correct) — i.e. the unbalanced earlier quotes don't confuse the
    // active-string detector.
    let q = "[\"a\", \"b\"] | select(.x == \"y";
    let t = classify(q, q.len()).expect("trigger");
    assert_eq!(t.kind, TriggerKind::Eq);
    assert!(t.lhs_path.is_none());
    assert_eq!(t.partial, "y");
}

#[test]
fn utf8_partial_preserved() {
    let q = "select(.name == \"üñ";
    let t = classify(q, q.len()).expect("trigger");
    assert_eq!(t.partial, "üñ");
    assert!(matches!(t.kind, TriggerKind::Eq));
}

#[test]
fn cursor_in_middle_of_query() {
    let q = "select(.x == \"ab\") | length";
    let cursor = q.find("ab").unwrap() + 2;
    let t = classify(q, cursor).expect("trigger");
    assert_eq!(t.partial, "ab");
}

#[test]
fn cursor_outside_string_in_middle() {
    let q = "select(.x == \"ab\") | length";
    let cursor = q.len();
    assert!(classify(q, cursor).is_none());
}

#[test]
fn array_index_lhs_path_preserved() {
    let t = classify_at_end(".items[0].name == \"x").expect("trigger");
    assert_eq!(t.lhs_path.as_deref(), Some(".items[0].name"));
}

#[test]
fn array_iterator_lhs_path_preserved() {
    let t = classify_at_end(".items[].name == \"x").expect("trigger");
    assert_eq!(t.lhs_path.as_deref(), Some(".items[].name"));
}

#[test]
fn optional_field_lhs_path_preserved() {
    let t = classify_at_end(".user.role? == \"x").expect("trigger");
    assert_eq!(t.lhs_path.as_deref(), Some(".user.role?"));
}

#[test]
fn cursor_byte_clamped_to_char_boundary() {
    let q = "select(.x == \"ü";
    // Pass a cursor mid-codepoint; classify must clamp instead of panicking.
    let mid = q.len() - 1;
    let _ = classify(q, mid);
}

#[test]
fn pipe_chain_with_function_breaks_path_extraction() {
    // We deliberately don't fold paths through map(.x); the plan calls this out
    // as a known limitation. Verify None for the lhs_path here.
    let t = classify_at_end(".users | map(.role) | contains(\"a").expect("trigger");
    assert_eq!(t.kind, TriggerKind::Contains);
    assert!(t.lhs_path.is_none());
}

#[test]
fn classifier_returns_none_for_random_non_trigger_inputs() {
    let cases = [
        "",
        ".",
        ".a.b.c",
        "map(.x)",
        "select(.x)",
        "1 + 2",
        "if .x then .y else .z end",
        "[1, 2, 3]",
        "{a: .x}",
        "\"already closed\"",
    ];
    for q in cases {
        assert!(classify(q, q.len()).is_none(), "expected None for: {q}");
    }
}

#[test]
fn quote_open_byte_points_at_opening_quote() {
    let q = "select(.x == \"ab";
    let t = classify(q, q.len()).expect("trigger");
    assert_eq!(&q[t.quote_open_byte..t.quote_open_byte + 1], "\"");
}

#[test]
fn ignores_comparison_inside_already_closed_strings() {
    let q = "\"x == y\" | length";
    assert!(classify(q, q.len()).is_none());
}

#[test]
fn does_not_confuse_paren_inside_string() {
    let q = "\"(\" + .x == \"ab";
    let t = classify(q, q.len()).expect("trigger");
    assert_eq!(t.kind, TriggerKind::Eq);
    assert_eq!(t.lhs_path.as_deref(), Some(".x"));
}

// === Pipe-fold tests: produce ABSOLUTE paths rooted at the JSON root ===

#[test]
fn fold_pattern_a_top_level_field() {
    // Pattern A: `.status == "`
    let t = classify_at_end(".status == \"a").expect("trigger");
    assert_eq!(t.lhs_path.as_deref(), Some(".status"));
}

#[test]
fn fold_pattern_b_nested_field() {
    // Pattern B: `.user.role == "`
    let t = classify_at_end(".user.role == \"a").expect("trigger");
    assert_eq!(t.lhs_path.as_deref(), Some(".user.role"));
}

#[test]
fn fold_pattern_c_array_iteration_into_field() {
    // Pattern C: `.users[].role == "`
    let t = classify_at_end(".users[].role == \"a").expect("trigger");
    assert_eq!(t.lhs_path.as_deref(), Some(".users[].role"));
}

#[test]
fn fold_pattern_d_pipe_select_eq() {
    // Pattern D: `.users[] | select(.role == "`
    let t = classify_at_end(".users[] | select(.role == \"a").expect("trigger");
    assert_eq!(t.lhs_path.as_deref(), Some(".users[].role"));
}

#[test]
fn fold_pattern_e_pipe_select_contains() {
    // Pattern E: `.Output.attributes[] | select(.attr | contains("`
    let t = classify_at_end(".Output.attributes[] | select(.attr | contains(\"a").expect("trigger");
    assert_eq!(t.kind, TriggerKind::Contains);
    assert_eq!(t.lhs_path.as_deref(), Some(".Output.attributes[].attr"));
}

#[test]
fn fold_pattern_f_pipe_map_contains() {
    // Pattern F: `.users | map(.role) | contains("`
    // The classifier doesn't fold past `map(.role)` (the LHS of the second
    // pipe is a call expression, not a plain path chain). Falls back to None.
    let t = classify_at_end(".users | map(.role) | contains(\"a").expect("trigger");
    assert_eq!(t.kind, TriggerKind::Contains);
    assert!(t.lhs_path.is_none());
}

#[test]
fn fold_pattern_g_map_with_select_inside() {
    // Pattern G: `.users[] | map(select(.active == "`
    // Walks out: select (identity) → map (prepend []) → pipe → .users[]
    // Result: .users[][].active
    let t = classify_at_end(".users[] | map(select(.active == \"t").expect("trigger");
    assert_eq!(t.lhs_path.as_deref(), Some(".users[][].active"));
}

#[test]
fn fold_pattern_h_multi_pipe() {
    // Pattern H: `.a.b | .c | select(.d == "`
    let t = classify_at_end(".a.b | .c | select(.d == \"x").expect("trigger");
    assert_eq!(t.lhs_path.as_deref(), Some(".a.b.c.d"));
}

#[test]
fn fold_pattern_i_nested_select() {
    // Pattern I: `.users[] | select(.profile | select(.role == "`
    let t = classify_at_end(".users[] | select(.profile | select(.role == \"a").expect("trigger");
    assert_eq!(t.lhs_path.as_deref(), Some(".users[].profile.role"));
}

#[test]
fn fold_pattern_j_array_index_then_pipe() {
    // Pattern J: `.items[0] | select(.name == "`
    let t = classify_at_end(".items[0] | select(.name == \"a").expect("trigger");
    assert_eq!(t.lhs_path.as_deref(), Some(".items[0].name"));
}

#[test]
fn fold_pattern_k_top_level_no_lhs() {
    // Pattern K: `contains("` — no LHS at all
    let t = classify_at_end("contains(\"a").expect("trigger");
    assert_eq!(t.kind, TriggerKind::Contains);
    assert!(t.lhs_path.is_none());
}

#[test]
fn fold_pattern_l_eq_no_lhs() {
    // Pattern L: `== "` — no LHS at all
    let t = classify_at_end("== \"a").expect("trigger");
    assert_eq!(t.kind, TriggerKind::Eq);
    assert!(t.lhs_path.is_none());
}

#[test]
fn fold_pattern_m_filter_narrowing_does_not_propagate() {
    // Pattern M: `select(.year > 2020) | select(.title == "`
    // The first select's predicate narrows the stream but its body is
    // opaque to the fold (it's just a boolean expr, not a path-prefix).
    // The fold for the inner select walks out: select (identity) → pipe →
    // need a path on the left. The left of the pipe is a call expression,
    // so the fold gives up cleanly. Documented limitation.
    let t = classify_at_end("select(.year > 2020) | select(.title == \"a").expect("trigger");
    assert!(t.lhs_path.is_none());
}

#[test]
fn fold_pattern_n_reduce_gives_up() {
    // Pattern N: `reduce .items[] as $i (init; $i | select(.x == "`
    // The cursor is inside a reduce's body. Folding hits `reduce` (not in
    // identity/iterating wrapper sets) and gives up.
    let t = classify_at_end("reduce .items[] as $i (0; $i | select(.x == \"a").expect("trigger");
    assert!(t.lhs_path.is_none());
}

#[test]
fn fold_pattern_q_sort_by_breaks_fold() {
    // Pattern Q: `.items | sort_by(.date) | .[0] | select(.title == "`
    // sort_by isn't in our wrapper sets. The fold hits the call expression
    // on the left of the second pipe and gives up.
    let t =
        classify_at_end(".items | sort_by(.date) | .[0] | select(.title == \"a").expect("trigger");
    assert!(t.lhs_path.is_none());
}

#[test]
fn fold_through_pipe_with_bare_identifier() {
    // `.users[] | first | select(.role == "` — bare `first` between pipes.
    // The fold walks out of select (identity), hits a pipe, and
    // `extract_path_chain_ending_at` looks for path bytes ending at that
    // pipe. `first` is identifier chars but doesn't start with `.`, so the
    // chain extraction fails and the fold gives up cleanly. Documented
    // limitation — bare-identifier wrappers between pipes aren't folded.
    let t = classify_at_end(".users[] | first | select(.role == \"a").expect("trigger");
    assert!(t.lhs_path.is_none());
}

#[test]
fn fold_pipe_to_select_with_no_inner_path() {
    // `.users[] | select("` — partial inside select with no LHS path inside.
    // No EQ/NE, no STRING_PREDICATE → no trigger fires.
    assert!(classify_at_end(".users[] | select(\"a").is_none());
}

#[test]
fn fold_recursion_safety_bounded() {
    // Many nested selects shouldn't blow up. The fold has a 32-hop ceiling.
    let mut q = String::new();
    for _ in 0..50 {
        q.push_str("select(");
    }
    q.push_str(".x == \"a");
    // Whether it folds or returns None, it must not panic or hang.
    let _ = classify(&q, q.len());
}

// === Round 2: fold give-up branches, interpolation escape, IN-lhs guards ===

#[test]
fn fold_gives_up_on_unknown_named_call_wrapper() {
    // `foo(.x == "` — `foo` is not an identity/iterating wrapper. Folding from
    // the `.x` LHS hits the named-call Unknown arm and gives up, so the trigger
    // still fires (kind + partial correct) but lhs_path falls back to None.
    let t = classify_at_end("foo(.x == \"a").expect("trigger");
    assert_eq!(t.kind, TriggerKind::Eq);
    assert!(t.lhs_path.is_none());
    assert_eq!(t.partial, "a");
}

#[test]
fn fold_gives_up_across_logical_or() {
    // `.a || .x == "` — the `||` immediately before `.x` is a logical-OR
    // boundary, distinct from a `|` pipe. Folding classifies it as Unknown and
    // gives up, so lhs_path is None (must NOT be mis-anchored as a pipe).
    let t = classify_at_end(".a || .x == \"b").expect("trigger");
    assert_eq!(t.kind, TriggerKind::Eq);
    assert!(t.lhs_path.is_none());
    assert_eq!(t.partial, "b");
}

#[test]
fn fold_gives_up_on_bare_open_paren_before_path() {
    // `(.x == "` — a bare grouping paren (no function name) immediately before
    // the `.x` LHS. The empty-name guard classifies it as Unknown and folding
    // gives up, so lhs_path is None.
    let t = classify_at_end("(.x == \"a").expect("trigger");
    assert_eq!(t.kind, TriggerKind::Eq);
    assert!(t.lhs_path.is_none());
    assert_eq!(t.partial, "a");
}

#[test]
fn map_with_predicate_no_inner_path_yields_array_iter() {
    // `map(contains("` — folding out of `contains` (no inner LHS path) through
    // the iterating wrapper `map` prepends `[]` onto an empty accumulator,
    // yielding the bare array iterator `.[]` (right-empty concat short-circuit).
    let t = classify_at_end("map(contains(\"a").expect("trigger");
    assert_eq!(t.kind, TriggerKind::Contains);
    assert_eq!(t.lhs_path.as_deref(), Some(".[]"));
    assert_eq!(t.partial, "a");
}

#[test]
fn escaped_non_paren_does_not_trip_interpolation_guard() {
    // `select(.x == "a\nb` — the `\n` escape is NOT `\(` interpolation. The
    // interpolation detector must advance past the escape and return false, so
    // the trigger still fires with the literal backslash escape preserved.
    let t = classify_at_end("select(.x == \"a\\nb").expect("trigger");
    assert_eq!(t.kind, TriggerKind::Eq);
    assert_eq!(t.lhs_path.as_deref(), Some(".x"));
    assert!(t.partial.contains("\\n"));
}

#[test]
fn bare_open_paren_before_quote_no_function_name() {
    // `("` and `( "` — an unclosed grouping paren immediately before the quote
    // with no preceding identifier is not a function call. enclosing_function_call
    // hits its empty-name guard and classify returns None.
    assert!(classify_at_end("(\"a").is_none());
    assert!(classify_at_end("( \"a").is_none());
}

#[test]
fn in_with_empty_lhs_before_semicolon_no_path() {
    // `IN(; "a` — semicolon present but nothing before it. extract_in_lhs_path
    // slices an empty LHS and returns None at the empty-check, so the In trigger
    // fires with lhs_path None.
    let t = classify_at_end("IN(; \"a").expect("trigger");
    assert_eq!(t.kind, TriggerKind::In);
    assert!(t.lhs_path.is_none());
    assert_eq!(t.partial, "a");
}

#[test]
fn in_with_non_path_lhs_before_semicolon_no_path() {
    // `IN(foo; "a` — a bare identifier as the IN LHS. canonicalize_path rejects
    // it because it does not start with `.`, so the In trigger fires with
    // lhs_path None.
    let t = classify_at_end("IN(foo; \"a").expect("trigger");
    assert_eq!(t.kind, TriggerKind::In);
    assert!(t.lhs_path.is_none());
    assert_eq!(t.partial, "a");
}
