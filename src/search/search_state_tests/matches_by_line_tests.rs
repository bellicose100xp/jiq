use super::super::*;
use proptest::prelude::*;

#[test]
fn test_matches_on_line_empty_when_no_matches() {
    let state = SearchState::new();
    let matches: Vec<_> = state.matches_on_line(0).collect();
    assert!(matches.is_empty(), "Should return empty for no matches");
}

#[test]
fn test_matches_on_line_returns_single_match() {
    let mut state = SearchState::new();
    state.search_textarea_mut().insert_str("test");
    let content = "test on line 0\nno match";
    state.update_matches(content);

    let line0_matches: Vec<_> = state.matches_on_line(0).collect();
    assert_eq!(line0_matches.len(), 1, "Should find 1 match on line 0");
    assert_eq!(
        line0_matches[0].0, 0,
        "Global index should be 0 for first match"
    );
    assert_eq!(line0_matches[0].1.line, 0, "Match should be on line 0");
}

#[test]
fn test_matches_on_line_returns_multiple_matches_same_line() {
    let mut state = SearchState::new();
    state.search_textarea_mut().insert_str("test");
    let content = "test test test\nno match";
    state.update_matches(content);

    let line0_matches: Vec<_> = state.matches_on_line(0).collect();
    assert_eq!(line0_matches.len(), 3, "Should find 3 matches on line 0");

    assert_eq!(line0_matches[0].0, 0, "First match global index");
    assert_eq!(line0_matches[1].0, 1, "Second match global index");
    assert_eq!(line0_matches[2].0, 2, "Third match global index");

    for (_, m) in &line0_matches {
        assert_eq!(m.line, 0, "All matches should be on line 0");
    }
}

#[test]
fn test_matches_on_line_returns_empty_for_unmatched_line() {
    let mut state = SearchState::new();
    state.search_textarea_mut().insert_str("test");
    let content = "test on line 0\nno match on line 1";
    state.update_matches(content);

    let line1_matches: Vec<_> = state.matches_on_line(1).collect();
    assert!(
        line1_matches.is_empty(),
        "Should return empty for line with no matches"
    );
}

#[test]
fn test_matches_on_line_returns_correct_global_indices() {
    let mut state = SearchState::new();
    state.search_textarea_mut().insert_str("test");
    let content = "test on line 0\ntest test on line 1\ntest on line 2";
    state.update_matches(content);

    let line0_matches: Vec<_> = state.matches_on_line(0).collect();
    let line1_matches: Vec<_> = state.matches_on_line(1).collect();
    let line2_matches: Vec<_> = state.matches_on_line(2).collect();

    assert_eq!(line0_matches.len(), 1);
    assert_eq!(line1_matches.len(), 2);
    assert_eq!(line2_matches.len(), 1);

    assert_eq!(line0_matches[0].0, 0, "Line 0 match at global index 0");
    assert_eq!(
        line1_matches[0].0, 1,
        "Line 1 first match at global index 1"
    );
    assert_eq!(
        line1_matches[1].0, 2,
        "Line 1 second match at global index 2"
    );
    assert_eq!(line2_matches[0].0, 3, "Line 2 match at global index 3");

    for (idx, m) in state.matches().iter().enumerate() {
        if m.line == 0 {
            assert_eq!(idx, 0);
        } else if m.line == 1 {
            assert!(idx == 1 || idx == 2);
        } else if m.line == 2 {
            assert_eq!(idx, 3);
        }
    }
}

#[test]
fn test_update_matches_populates_matches_by_line() {
    let mut state = SearchState::new();
    state.search_textarea_mut().insert_str("hello");
    let content = "hello world\nsay hello\nhello hello";
    state.update_matches(content);

    assert!(
        !state.matches_by_line.is_empty(),
        "matches_by_line should be populated"
    );

    let line0_indices = state.matches_by_line.get(&0);
    let line1_indices = state.matches_by_line.get(&1);
    let line2_indices = state.matches_by_line.get(&2);

    assert!(line0_indices.is_some(), "Line 0 should have indices");
    assert!(line1_indices.is_some(), "Line 1 should have indices");
    assert!(line2_indices.is_some(), "Line 2 should have indices");

    assert_eq!(line0_indices.unwrap().len(), 1, "Line 0 has 1 match");
    assert_eq!(line1_indices.unwrap().len(), 1, "Line 1 has 1 match");
    assert_eq!(line2_indices.unwrap().len(), 2, "Line 2 has 2 matches");
}

#[test]
fn test_close_clears_matches_by_line() {
    let mut state = SearchState::new();
    state.search_textarea_mut().insert_str("test");
    let content = "test on line 0\ntest on line 1";
    state.update_matches(content);

    assert!(
        !state.matches_by_line.is_empty(),
        "matches_by_line should be populated before close"
    );

    state.close();

    assert!(
        state.matches_by_line.is_empty(),
        "matches_by_line should be cleared after close"
    );
}

#[test]
fn test_matches_by_line_with_unicode_content() {
    let mut state = SearchState::new();
    state.search_textarea_mut().insert_str("日本");
    let content = "日本語\nHello 日本\n日本 日本";
    state.update_matches(content);

    let line0_matches: Vec<_> = state.matches_on_line(0).collect();
    let line1_matches: Vec<_> = state.matches_on_line(1).collect();
    let line2_matches: Vec<_> = state.matches_on_line(2).collect();

    assert_eq!(line0_matches.len(), 1, "Line 0 should have 1 unicode match");
    assert_eq!(line1_matches.len(), 1, "Line 1 should have 1 unicode match");
    assert_eq!(
        line2_matches.len(),
        2,
        "Line 2 should have 2 unicode matches"
    );
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop_matches_by_line_contains_all_matches(
        num_lines in 1usize..20,
        matches_per_line in prop::collection::vec(0usize..5, 1..20)
    ) {
        let mut state = SearchState::new();
        state.search_textarea_mut().insert_str("x");

        let num_lines = num_lines.min(matches_per_line.len());
        let mut content = String::new();
        let mut expected_total = 0;

        for &count in matches_per_line.iter().take(num_lines) {
            expected_total += count;
            for _ in 0..count {
                content.push_str("x ");
            }
            content.push('\n');
        }

        state.update_matches(&content);

        let mut total_from_hashmap = 0;
        for indices in state.matches_by_line.values() {
            total_from_hashmap += indices.len();
        }

        prop_assert_eq!(
            total_from_hashmap,
            expected_total,
            "matches_by_line should contain indices for all matches"
        );

        prop_assert_eq!(
            state.matches().len(),
            expected_total,
            "Total matches should equal expected"
        );
    }

    #[test]
    fn prop_matches_on_line_indices_valid(
        num_matches in 1usize..50
    ) {
        let mut state = SearchState::new();
        state.search_textarea_mut().insert_str("x");

        let mut content = String::new();
        for i in 0..num_matches {
            content.push('x');
            if i < num_matches - 1 {
                content.push('\n');
            }
        }

        state.update_matches(&content);

        for line_num in 0..num_matches as u32 {
            for (global_idx, m) in state.matches_on_line(line_num) {
                prop_assert!(
                    global_idx < state.matches().len(),
                    "Global index should be valid"
                );

                prop_assert_eq!(
                    m.line, line_num,
                    "Match line should match queried line"
                );

                prop_assert_eq!(
                    state.matches()[global_idx].line, m.line,
                    "Global index should point to correct match"
                );
            }
        }
    }

    #[test]
    fn prop_matches_by_line_cleared_on_close(
        num_matches in 1usize..50
    ) {
        let mut state = SearchState::new();
        state.search_textarea_mut().insert_str("x");

        let content: String = (0..num_matches).map(|_| "x\n").collect();
        state.update_matches(&content);

        prop_assert!(
            !state.matches_by_line.is_empty(),
            "matches_by_line should be populated before close"
        );

        state.close();

        prop_assert!(
            state.matches_by_line.is_empty(),
            "matches_by_line should be cleared after close"
        );
    }
}
