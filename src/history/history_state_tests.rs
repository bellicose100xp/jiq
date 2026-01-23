//! Tests for history/history_state

use super::*;

fn create_test_state(entries: Vec<&str>) -> HistoryState {
    HistoryState {
        entries: entries.into_iter().map(String::from).collect(),
        filtered_indices: vec![0, 1, 2],
        search_textarea: create_search_textarea(),
        selected_index: 0,
        scroll_offset: 0,
        visible: false,
        matcher: HistoryMatcher::new(),
        persist_to_disk: false,
        cycling_index: None,
    }
}

#[test]
fn test_open_sets_visible() {
    let mut state = create_test_state(vec![".foo", ".bar", ".baz"]);
    state.open(None);
    assert!(state.is_visible());
}

#[test]
fn test_open_with_initial_query() {
    let mut state = create_test_state(vec![".foo", ".bar", ".baz"]);
    state.open(Some(".foo"));
    assert_eq!(state.search_query(), ".foo");
}

#[test]
fn test_close_resets_state() {
    let mut state = create_test_state(vec![".foo", ".bar", ".baz"]);
    state.open(Some("test"));
    state.select_next();
    state.close();

    assert!(!state.is_visible());
    assert!(state.search_query().is_empty());
    assert_eq!(state.selected_index(), 0);
}

#[test]
fn test_navigation_stops_at_edges() {
    let mut state = create_test_state(vec![".foo", ".bar", ".baz"]);
    state.filtered_indices = vec![0, 1, 2];

    state.select_previous();
    assert_eq!(state.selected_index(), 0);

    state.selected_index = 2;
    state.select_next();
    assert_eq!(state.selected_index(), 2);
}

#[test]
fn test_selected_entry() {
    let mut state = create_test_state(vec![".foo", ".bar", ".baz"]);
    state.filtered_indices = vec![0, 1, 2];

    assert_eq!(state.selected_entry(), Some(".foo"));

    state.select_next();
    assert_eq!(state.selected_entry(), Some(".bar"));
}

#[test]
fn test_textarea_search_input() {
    let mut state = create_test_state(vec![".foo", ".bar", ".baz"]);

    state.search_textarea_mut().insert_str("fo");
    assert_eq!(state.search_query(), "fo");

    state.search_textarea_mut().select_all();
    state.search_textarea_mut().cut();
    assert_eq!(state.search_query(), "");
}

#[test]
fn test_visible_entries_limited() {
    let entries: Vec<&str> = (0..20).map(|_| ".test").collect();
    let mut state = create_test_state(entries);
    state.filtered_indices = (0..20).collect();

    let visible: Vec<_> = state.visible_entries().collect();
    assert_eq!(visible.len(), MAX_VISIBLE_HISTORY);
}

#[test]
fn test_empty_navigation() {
    let mut state = create_test_state(vec![]);
    state.filtered_indices = vec![];

    state.select_next();
    state.select_previous();
    assert_eq!(state.selected_index(), 0);
}

#[test]
fn test_single_entry_navigation() {
    let mut state = create_test_state(vec![".only"]);
    state.filtered_indices = vec![0];

    // Should stay on the same entry
    state.select_next();
    assert_eq!(state.selected_index(), 0);
    assert_eq!(state.selected_entry(), Some(".only"));

    state.select_previous();
    assert_eq!(state.selected_index(), 0);
    assert_eq!(state.selected_entry(), Some(".only"));
}

#[test]
fn test_filter_updates_reset_selection() {
    let mut state = create_test_state(vec![".apple", ".banana", ".apricot"]);
    state.filtered_indices = vec![0, 1, 2];
    state.selected_index = 2;

    // Input change resets selection to 0
    state.search_textarea_mut().insert_char('a');
    state.on_search_input_changed();
    assert_eq!(state.selected_index(), 0);
}

#[test]
fn test_selected_entry_with_out_of_bounds_index() {
    let mut state = create_test_state(vec![".foo", ".bar"]);
    state.filtered_indices = vec![0, 1];
    state.selected_index = 5; // Out of bounds

    // Should return None gracefully
    assert_eq!(state.selected_entry(), None);
}

#[test]
fn test_cycling_at_boundaries() {
    let mut state = create_test_state(vec![".first", ".second", ".third"]);

    // Cycle to end
    let e1 = state.cycle_previous();
    let e2 = state.cycle_previous();
    let e3 = state.cycle_previous();
    assert_eq!(e1, Some(".first".to_string()));
    assert_eq!(e2, Some(".second".to_string()));
    assert_eq!(e3, Some(".third".to_string()));

    // Spam Ctrl+P at end - should stay at .third
    let e4 = state.cycle_previous();
    let e5 = state.cycle_previous();
    assert_eq!(e4, Some(".third".to_string()));
    assert_eq!(e5, Some(".third".to_string()));
}

#[test]
fn test_cycling_forward_to_none() {
    let mut state = create_test_state(vec![".first", ".second"]);

    // Cycle back
    state.cycle_previous();
    state.cycle_previous();

    // Cycle forward
    let e1 = state.cycle_next();
    assert_eq!(e1, Some(".first".to_string()));

    // Cycle forward to most recent
    let e2 = state.cycle_next();
    assert_eq!(e2, None); // At most recent, should reset
}

#[test]
fn test_reset_cycling() {
    let mut state = create_test_state(vec![".first", ".second"]);

    state.cycle_previous();
    state.cycle_previous();
    assert_eq!(state.cycling_index, Some(1));

    state.reset_cycling();
    assert_eq!(state.cycling_index, None);

    // Next cycle should start fresh
    let entry = state.cycle_previous();
    assert_eq!(entry, Some(".first".to_string()));
}

#[test]
fn test_default_creates_new_instance() {
    let state = HistoryState::default();
    assert!(!state.is_visible());
}

#[test]
fn test_add_entry_in_memory_ignores_empty() {
    let mut state = HistoryState::empty();
    state.add_entry_in_memory("");
    assert_eq!(state.total_count(), 0);

    state.add_entry_in_memory("   ");
    assert_eq!(state.total_count(), 0);
}

#[test]
fn test_add_entry_ignores_empty() {
    let mut state = HistoryState::empty();
    state.add_entry("");
    assert_eq!(state.total_count(), 0);

    state.add_entry("  \t\n  ");
    assert_eq!(state.total_count(), 0);
}

#[test]
fn test_cycle_next_when_not_cycling() {
    let mut state = create_test_state(vec![".first", ".second"]);

    let result = state.cycle_next();
    assert_eq!(result, None);
}

#[test]
fn test_scroll_offset_follows_selection_down() {
    let entries: Vec<&str> = (0..20)
        .map(|i| {
            if i == 0 {
                ".first"
            } else if i == 19 {
                ".last"
            } else {
                ".test"
            }
        })
        .collect();
    let mut state = create_test_state(entries);
    state.filtered_indices = (0..20).collect();

    assert_eq!(state.scroll_offset, 0);
    assert_eq!(state.selected_index(), 0);

    for _ in 0..15 {
        state.select_next();
    }

    assert_eq!(state.selected_index(), 15);
    assert_eq!(state.scroll_offset, 1);

    state.select_next();
    assert_eq!(state.selected_index(), 16);
    assert_eq!(state.scroll_offset, 2);
}

#[test]
fn test_scroll_offset_follows_selection_up() {
    let entries: Vec<&str> = (0..20).map(|_| ".test").collect();
    let mut state = create_test_state(entries);
    state.filtered_indices = (0..20).collect();
    state.selected_index = 16;
    state.scroll_offset = 5;

    state.select_previous();
    assert_eq!(state.selected_index(), 15);
    assert_eq!(state.scroll_offset, 5);

    for _ in 0..11 {
        state.select_previous();
    }

    assert_eq!(state.selected_index(), 4);
    assert_eq!(state.scroll_offset, 4);

    state.select_previous();
    assert_eq!(state.selected_index(), 3);
    assert_eq!(state.scroll_offset, 3);
}

#[test]
fn test_scroll_offset_resets_on_open() {
    let entries: Vec<&str> = (0..20).map(|_| ".test").collect();
    let mut state = create_test_state(entries);
    state.filtered_indices = (0..20).collect();
    state.selected_index = 10;
    state.scroll_offset = 5;

    state.open(None);

    assert_eq!(state.selected_index(), 0);
    assert_eq!(state.scroll_offset, 0);
}

#[test]
fn test_scroll_offset_resets_on_close() {
    let entries: Vec<&str> = (0..20).map(|_| ".test").collect();
    let mut state = create_test_state(entries);
    state.filtered_indices = (0..20).collect();
    state.selected_index = 10;
    state.scroll_offset = 5;

    state.close();

    assert_eq!(state.selected_index(), 0);
    assert_eq!(state.scroll_offset, 0);
}

#[test]
fn test_scroll_offset_resets_on_filter_change() {
    let entries: Vec<&str> = (0..20).map(|_| ".test").collect();
    let mut state = create_test_state(entries);
    state.filtered_indices = (0..20).collect();
    state.selected_index = 10;
    state.scroll_offset = 5;

    state.on_search_input_changed();

    assert_eq!(state.selected_index(), 0);
    assert_eq!(state.scroll_offset, 0);
}

#[test]
fn test_scroll_stops_at_bottom() {
    let entries: Vec<&str> = (0..20).map(|_| ".test").collect();
    let mut state = create_test_state(entries);
    state.filtered_indices = (0..20).collect();
    state.selected_index = 19;
    state.scroll_offset = 5;

    state.select_next();

    assert_eq!(state.selected_index(), 19);
    assert_eq!(state.scroll_offset, 5);
}

#[test]
fn test_scroll_stops_at_top() {
    let entries: Vec<&str> = (0..20).map(|_| ".test").collect();
    let mut state = create_test_state(entries);
    state.filtered_indices = (0..20).collect();
    state.selected_index = 0;
    state.scroll_offset = 0;

    state.select_previous();

    assert_eq!(state.selected_index(), 0);
    assert_eq!(state.scroll_offset, 0);
}

// Tests for Scrollable trait implementation

use crate::scroll::Scrollable;

fn create_scrollable_test_state(entry_count: usize) -> HistoryState {
    HistoryState {
        entries: (0..entry_count).map(|i| format!(".test{}", i)).collect(),
        filtered_indices: (0..entry_count).collect(),
        search_textarea: create_search_textarea(),
        selected_index: 0,
        scroll_offset: 0,
        visible: false,
        matcher: HistoryMatcher::new(),
        persist_to_disk: false,
        cycling_index: None,
    }
}

#[test]
fn test_scrollable_scroll_view_down() {
    let mut state = create_scrollable_test_state(30);

    state.scroll_view_down(3);
    assert_eq!(Scrollable::scroll_offset(&state), 3);

    state.scroll_view_down(5);
    assert_eq!(Scrollable::scroll_offset(&state), 8);
}

#[test]
fn test_scrollable_scroll_view_down_clamped() {
    let mut state = create_scrollable_test_state(30);

    // max_scroll = 30 - 15 = 15
    state.scroll_view_down(100);
    assert_eq!(Scrollable::scroll_offset(&state), 15);
}

#[test]
fn test_scrollable_scroll_view_up() {
    let mut state = create_scrollable_test_state(30);
    state.scroll_offset = 10;

    state.scroll_view_up(3);
    assert_eq!(Scrollable::scroll_offset(&state), 7);

    state.scroll_view_up(4);
    assert_eq!(Scrollable::scroll_offset(&state), 3);
}

#[test]
fn test_scrollable_scroll_view_up_clamped() {
    let mut state = create_scrollable_test_state(30);
    state.scroll_offset = 5;

    state.scroll_view_up(10);
    assert_eq!(Scrollable::scroll_offset(&state), 0);
}

#[test]
fn test_scrollable_max_scroll() {
    let state = create_scrollable_test_state(30);
    // max_scroll = 30 - 15 = 15
    assert_eq!(state.max_scroll(), 15);
}

#[test]
fn test_scrollable_max_scroll_content_fits() {
    let state = create_scrollable_test_state(10);
    // max_scroll = 10 - 15 = 0 (saturating)
    assert_eq!(state.max_scroll(), 0);
}

#[test]
fn test_scrollable_viewport_size() {
    let state = create_scrollable_test_state(30);
    assert_eq!(state.viewport_size(), MAX_VISIBLE_HISTORY);
    assert_eq!(state.viewport_size(), 15);
}

#[test]
fn test_scrollable_content_fits_in_viewport() {
    let mut state = create_scrollable_test_state(10);
    assert_eq!(state.max_scroll(), 0);

    state.scroll_view_down(5);
    assert_eq!(Scrollable::scroll_offset(&state), 0); // Can't scroll when content fits
}
