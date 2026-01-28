//! Tests for search_state

use super::*;
use proptest::prelude::*;

#[test]
fn test_new_state() {
    let state = SearchState::new();
    assert!(!state.is_visible());
    assert!(state.query().is_empty());
    assert!(state.matches().is_empty());
    assert_eq!(state.current_index(), 0);
}

#[test]
fn test_open_sets_visible() {
    let mut state = SearchState::new();
    state.open();
    assert!(state.is_visible());
}

#[test]
fn test_close_resets_state() {
    let mut state = SearchState::new();
    state.open();
    state.search_textarea_mut().insert_str("test");
    state.matches = vec![
        Match {
            line: 0,
            col: 0,
            len: 4,
        },
        Match {
            line: 1,
            col: 5,
            len: 4,
        },
    ];
    state.current_index = 1;
    state.last_query = "test".to_string();

    state.matches_by_line.insert(0, vec![0]);
    state.matches_by_line.insert(1, vec![1]);

    state.close();

    assert!(!state.is_visible());
    assert!(state.query().is_empty());
    assert!(state.matches().is_empty());
    assert_eq!(state.current_index(), 0);
    assert!(state.last_query.is_empty());
    assert!(
        state.matches_by_line.is_empty(),
        "matches_by_line should be cleared"
    );
}

#[test]
fn test_match_count_display_empty() {
    let state = SearchState::new();
    assert_eq!(state.match_count_display(), "0/0");
}

#[test]
fn test_match_count_display_with_matches() {
    let mut state = SearchState::new();
    state.matches = vec![
        Match {
            line: 0,
            col: 0,
            len: 4,
        },
        Match {
            line: 1,
            col: 5,
            len: 4,
        },
        Match {
            line: 2,
            col: 10,
            len: 4,
        },
    ];
    state.current_index = 0;
    assert_eq!(state.match_count_display(), "1/3");

    state.current_index = 2;
    assert_eq!(state.match_count_display(), "3/3");
}

#[test]
fn test_next_match_empty() {
    let mut state = SearchState::new();
    assert_eq!(state.next_match(), None);
}

#[test]
fn test_next_match_wraps() {
    let mut state = SearchState::new();
    state.matches = vec![
        Match {
            line: 0,
            col: 0,
            len: 4,
        },
        Match {
            line: 5,
            col: 0,
            len: 4,
        },
        Match {
            line: 10,
            col: 0,
            len: 4,
        },
    ];
    state.current_index = 0;

    assert_eq!(state.next_match(), Some(5));
    assert_eq!(state.current_index(), 1);

    assert_eq!(state.next_match(), Some(10));
    assert_eq!(state.current_index(), 2);

    // Wrap around
    assert_eq!(state.next_match(), Some(0));
    assert_eq!(state.current_index(), 0);
}

#[test]
fn test_prev_match_empty() {
    let mut state = SearchState::new();
    assert_eq!(state.prev_match(), None);
}

#[test]
fn test_prev_match_wraps() {
    let mut state = SearchState::new();
    state.matches = vec![
        Match {
            line: 0,
            col: 0,
            len: 4,
        },
        Match {
            line: 5,
            col: 0,
            len: 4,
        },
        Match {
            line: 10,
            col: 0,
            len: 4,
        },
    ];
    state.current_index = 0;

    // Wrap to end
    assert_eq!(state.prev_match(), Some(10));
    assert_eq!(state.current_index(), 2);

    assert_eq!(state.prev_match(), Some(5));
    assert_eq!(state.current_index(), 1);

    assert_eq!(state.prev_match(), Some(0));
    assert_eq!(state.current_index(), 0);
}

#[test]
fn test_current_match() {
    let mut state = SearchState::new();
    assert!(state.current_match().is_none());

    state.matches = vec![
        Match {
            line: 0,
            col: 5,
            len: 3,
        },
        Match {
            line: 2,
            col: 10,
            len: 3,
        },
    ];
    state.current_index = 0;

    let m = state.current_match().unwrap();
    assert_eq!(m.line, 0);
    assert_eq!(m.col, 5);
    assert_eq!(m.len, 3);

    state.current_index = 1;
    let m = state.current_match().unwrap();
    assert_eq!(m.line, 2);
}

#[test]
fn test_textarea_input() {
    let mut state = SearchState::new();
    state.search_textarea_mut().insert_str("hello");
    assert_eq!(state.query(), "hello");
}

#[test]
fn test_update_matches_finds_matches() {
    let mut state = SearchState::new();
    state.search_textarea_mut().insert_str("hello");

    let content = "hello world\nsay hello\ngoodbye";
    state.update_matches(content);

    assert_eq!(state.matches().len(), 2);
    assert_eq!(state.matches()[0].line, 0);
    assert_eq!(state.matches()[0].col, 0);
    assert_eq!(state.matches()[1].line, 1);
    assert_eq!(state.matches()[1].col, 4);
    assert_eq!(state.current_index(), 0);
}

#[test]
fn test_update_matches_resets_index() {
    let mut state = SearchState::new();
    state.search_textarea_mut().insert_str("test");
    state.matches = vec![
        Match {
            line: 0,
            col: 0,
            len: 4,
        },
        Match {
            line: 1,
            col: 0,
            len: 4,
        },
    ];
    state.current_index = 1;
    state.last_query = "old".to_string();

    let content = "test one\ntest two\ntest three";
    state.update_matches(content);

    // Index should be reset to 0 when query changes
    assert_eq!(state.current_index(), 0);
    assert_eq!(state.matches().len(), 3);
}

#[test]
fn test_update_matches_skips_if_query_unchanged() {
    let mut state = SearchState::new();
    state.search_textarea_mut().insert_str("test");
    state.last_query = "test".to_string();
    state.matches = vec![Match {
        line: 99,
        col: 0,
        len: 4,
    }];
    state.current_index = 0;

    let content = "test one\ntest two";
    state.update_matches(content);

    // Should not update since query hasn't changed
    assert_eq!(state.matches().len(), 1);
    assert_eq!(state.matches()[0].line, 99);
}

// Feature: search-in-results, Property 3: Match count accuracy
// *For any* search query and results content, the displayed match count
// (current/total) should accurately reflect the actual number of matches
// found and the current position.
// **Validates: Requirements 1.4**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop_match_count_accuracy(
        num_matches in 0usize..50,
        current_index in 0usize..50,
    ) {
        let mut state = SearchState::new();

        // Generate matches
        for i in 0..num_matches {
            state.matches.push(Match {
                line: i as u32,
                col: 0,
                len: 3,
            });
        }

        // Set current index (clamped to valid range if matches exist)
        if !state.matches.is_empty() {
            state.current_index = current_index % state.matches.len();
        }

        let display = state.match_count_display();

        if num_matches == 0 {
            // When no matches, should show "0/0"
            prop_assert_eq!(
                display, "0/0",
                "Empty matches should display 0/0"
            );
        } else {
            // Parse the display string to verify accuracy
            let expected_current = (current_index % num_matches) + 1; // 1-indexed
            let expected = format!("{}/{}", expected_current, num_matches);
            prop_assert_eq!(
                display, expected,
                "Match count display should be accurate"
            );
        }
    }
}

// Feature: search-in-results, Property 4: Escape clears search state
// *For any* search state (visible, with matches, at any current index),
// pressing Esc should result in search being hidden, matches being empty,
// and current index being 0.
// **Validates: Requirements 1.5, 5.3**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop_escape_clears_search_state(
        // Generate arbitrary search state
        visible in any::<bool>(),
        num_matches in 0usize..100,
        current_index in 0usize..100,
        query in "[a-zA-Z0-9 ]{0,50}",
    ) {
        let mut state = SearchState::new();

        // Set up arbitrary state
        state.visible = visible;
        state.last_query = query.clone();

        // Generate matches
        for i in 0..num_matches {
            state.matches.push(Match {
                line: i as u32,
                col: 0,
                len: 3,
            });
        }

        // Set current index (clamped to valid range if matches exist)
        if !state.matches.is_empty() {
            state.current_index = current_index % state.matches.len();
        }

        // Insert query text
        if !query.is_empty() {
            state.search_textarea_mut().insert_str(&query);
        }

        // Simulate Escape by calling close()
        state.close();

        // Verify all state is cleared
        prop_assert!(
            !state.is_visible(),
            "Search should not be visible after close"
        );
        prop_assert!(
            state.matches().is_empty(),
            "Matches should be empty after close"
        );
        prop_assert_eq!(
            state.current_index(), 0,
            "Current index should be 0 after close"
        );
        prop_assert!(
            state.query().is_empty(),
            "Query should be empty after close"
        );
        prop_assert!(
            state.last_query.is_empty(),
            "Last query should be empty after close"
        );
    }
}

// Feature: search-in-results, Property 6: Next match advances index with wrap
// *For any* search state with N matches (N > 0), calling next_match should
// advance current_index by 1, wrapping from N-1 to 0.
// **Validates: Requirements 3.1, 3.4**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop_next_match_advances_with_wrap(
        num_matches in 1usize..100,
        initial_index in 0usize..100,
    ) {
        let mut state = SearchState::new();

        // Generate matches with distinct line numbers
        for i in 0..num_matches {
            state.matches.push(Match {
                line: i as u32,
                col: 0,
                len: 3,
            });
        }

        // Set initial index (clamped to valid range)
        let clamped_initial = initial_index % num_matches;
        state.current_index = clamped_initial;

        // Call next_match
        let result = state.next_match();

        // Verify result is Some (since we have matches)
        prop_assert!(result.is_some(), "next_match should return Some when matches exist");

        // Verify index advanced with wrap
        let expected_index = (clamped_initial + 1) % num_matches;
        prop_assert_eq!(
            state.current_index(), expected_index,
            "next_match should advance index by 1 with wrap"
        );

        // Verify returned line matches the new current match
        let expected_line = expected_index as u32;
        prop_assert_eq!(
            result.unwrap(), expected_line,
            "next_match should return the line of the new current match"
        );
    }
}

// Feature: search-in-results, Property 7: Previous match decrements index with wrap
// *For any* search state with N matches (N > 0), calling prev_match should
// decrement current_index by 1, wrapping from 0 to N-1.
// **Validates: Requirements 3.2, 3.5**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop_prev_match_decrements_with_wrap(
        num_matches in 1usize..100,
        initial_index in 0usize..100,
    ) {
        let mut state = SearchState::new();

        // Generate matches with distinct line numbers
        for i in 0..num_matches {
            state.matches.push(Match {
                line: i as u32,
                col: 0,
                len: 3,
            });
        }

        // Set initial index (clamped to valid range)
        let clamped_initial = initial_index % num_matches;
        state.current_index = clamped_initial;

        // Call prev_match
        let result = state.prev_match();

        // Verify result is Some (since we have matches)
        prop_assert!(result.is_some(), "prev_match should return Some when matches exist");

        // Verify index decremented with wrap
        let expected_index = if clamped_initial == 0 {
            num_matches - 1
        } else {
            clamped_initial - 1
        };
        prop_assert_eq!(
            state.current_index(), expected_index,
            "prev_match should decrement index by 1 with wrap"
        );

        // Verify returned line matches the new current match
        let expected_line = expected_index as u32;
        prop_assert_eq!(
            result.unwrap(), expected_line,
            "prev_match should return the line of the new current match"
        );
    }
}

#[cfg(test)]
#[path = "search_state_tests/matches_by_line_tests.rs"]
mod matches_by_line_tests;
