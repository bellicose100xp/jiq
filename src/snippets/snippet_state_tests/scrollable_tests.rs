//! Tests for Scrollable trait implementation on SnippetState

use super::{Snippet, SnippetState};
use crate::scroll::Scrollable;

fn create_test_snippets(count: usize) -> Vec<Snippet> {
    (0..count)
        .map(|i| Snippet {
            name: format!("snippet{}", i),
            query: format!(".query{}", i),
            description: None,
        })
        .collect()
}

#[test]
fn test_scroll_view_down() {
    let mut state = SnippetState::new_without_persistence();
    state.set_snippets(create_test_snippets(20));
    state.set_visible_count(10);

    state.scroll_view_down(3);
    assert_eq!(Scrollable::scroll_offset(&state), 3);

    state.scroll_view_down(4);
    assert_eq!(Scrollable::scroll_offset(&state), 7);
}

#[test]
fn test_scroll_view_down_clamped() {
    let mut state = SnippetState::new_without_persistence();
    state.set_snippets(create_test_snippets(20));
    state.set_visible_count(10);

    // max_scroll = 20 - 10 = 10
    state.scroll_view_down(100);
    assert_eq!(Scrollable::scroll_offset(&state), 10);
}

#[test]
fn test_scroll_view_up() {
    let mut state = SnippetState::new_without_persistence();
    state.set_snippets(create_test_snippets(20));
    state.set_visible_count(10);
    state.scroll_view_down(10);

    state.scroll_view_up(3);
    assert_eq!(Scrollable::scroll_offset(&state), 7);

    state.scroll_view_up(4);
    assert_eq!(Scrollable::scroll_offset(&state), 3);
}

#[test]
fn test_scroll_view_up_clamped() {
    let mut state = SnippetState::new_without_persistence();
    state.set_snippets(create_test_snippets(20));
    state.set_visible_count(10);
    state.scroll_view_down(5);

    state.scroll_view_up(10);
    assert_eq!(Scrollable::scroll_offset(&state), 0);
}

#[test]
fn test_max_scroll() {
    let mut state = SnippetState::new_without_persistence();
    state.set_snippets(create_test_snippets(20));
    state.set_visible_count(10);

    // max_scroll = 20 - 10 = 10
    assert_eq!(state.max_scroll(), 10);
}

#[test]
fn test_max_scroll_content_fits() {
    let mut state = SnippetState::new_without_persistence();
    state.set_snippets(create_test_snippets(5));
    state.set_visible_count(10);

    // max_scroll = 5 - 10 = 0 (saturating)
    assert_eq!(state.max_scroll(), 0);
}

#[test]
fn test_viewport_size() {
    let mut state = SnippetState::new_without_persistence();
    state.set_visible_count(15);
    assert_eq!(state.viewport_size(), 15);
}

#[test]
fn test_content_fits_in_viewport() {
    let mut state = SnippetState::new_without_persistence();
    state.set_snippets(create_test_snippets(5));
    state.set_visible_count(10);

    assert_eq!(state.max_scroll(), 0);

    state.scroll_view_down(5);
    assert_eq!(Scrollable::scroll_offset(&state), 0); // Can't scroll when content fits
}

#[test]
fn test_scroll_with_filtered_content() {
    let mut state = SnippetState::new_without_persistence();
    state.set_snippets(create_test_snippets(20));
    state.set_visible_count(5);
    state.set_search_query("snippet1"); // Matches snippet1, snippet10-19 (11 items)

    // Filtered count should be 11 (snippet1 and snippet10-19)
    assert_eq!(state.filtered_count(), 11);
    // max_scroll = 11 - 5 = 6
    assert_eq!(state.max_scroll(), 6);

    state.scroll_view_down(10);
    assert_eq!(Scrollable::scroll_offset(&state), 6); // Clamped to max
}
