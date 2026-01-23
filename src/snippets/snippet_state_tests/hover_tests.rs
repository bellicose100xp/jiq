use super::*;

fn create_test_state_with_snippets() -> SnippetState {
    let mut state = SnippetState::new_without_persistence();
    state.set_snippets(vec![
        Snippet {
            name: "test1".to_string(),
            query: ".test1".to_string(),
            description: None,
        },
        Snippet {
            name: "test2".to_string(),
            query: ".test2".to_string(),
            description: None,
        },
        Snippet {
            name: "test3".to_string(),
            query: ".test3".to_string(),
            description: None,
        },
    ]);
    state
}

#[test]
fn test_hover_initially_none() {
    let state = SnippetState::new();
    assert!(state.get_hovered().is_none());
}

#[test]
fn test_set_hovered() {
    let mut state = create_test_state_with_snippets();
    assert!(state.get_hovered().is_none());

    state.set_hovered(Some(0));
    assert_eq!(state.get_hovered(), Some(0));

    state.set_hovered(Some(1));
    assert_eq!(state.get_hovered(), Some(1));
}

#[test]
fn test_clear_hover() {
    let mut state = create_test_state_with_snippets();
    state.set_hovered(Some(0));
    assert_eq!(state.get_hovered(), Some(0));

    state.clear_hover();
    assert!(state.get_hovered().is_none());
}

#[test]
fn test_set_hovered_to_none() {
    let mut state = create_test_state_with_snippets();
    state.set_hovered(Some(0));
    assert_eq!(state.get_hovered(), Some(0));

    state.set_hovered(None);
    assert!(state.get_hovered().is_none());
}

#[test]
fn test_close_clears_hover() {
    let mut state = create_test_state_with_snippets();
    state.open();
    state.set_hovered(Some(1));
    assert_eq!(state.get_hovered(), Some(1));

    state.close();
    assert!(state.get_hovered().is_none());
}

#[test]
fn test_snippet_at_y_first_item() {
    let mut state = create_test_state_with_snippets();
    state.set_visible_count(10);

    let index = state.snippet_at_y(0);
    assert_eq!(index, Some(0));
}

#[test]
fn test_snippet_at_y_second_item() {
    let mut state = create_test_state_with_snippets();
    state.set_visible_count(10);

    let index = state.snippet_at_y(1);
    assert_eq!(index, Some(1));
}

#[test]
fn test_snippet_at_y_with_scroll_offset() {
    let mut state = create_test_state_with_snippets();
    state.set_visible_count(2);
    state.set_selected_index(2);

    let index = state.snippet_at_y(0);
    assert_eq!(index, Some(1));

    let index = state.snippet_at_y(1);
    assert_eq!(index, Some(2));
}

#[test]
fn test_snippet_at_y_out_of_bounds() {
    let mut state = create_test_state_with_snippets();
    state.set_visible_count(10);

    let index = state.snippet_at_y(10);
    assert!(index.is_none());
}

#[test]
fn test_snippet_at_y_empty_list() {
    let state = SnippetState::new_without_persistence();

    let index = state.snippet_at_y(0);
    assert!(index.is_none());
}

#[test]
fn test_select_at_valid_index() {
    let mut state = create_test_state_with_snippets();
    assert_eq!(state.selected_index(), 0);

    state.select_at(1);
    assert_eq!(state.selected_index(), 1);

    state.select_at(2);
    assert_eq!(state.selected_index(), 2);
}

#[test]
fn test_select_at_invalid_index() {
    let mut state = create_test_state_with_snippets();
    assert_eq!(state.selected_index(), 0);

    state.select_at(10);
    assert_eq!(state.selected_index(), 0);
}

#[test]
fn test_select_at_empty_list() {
    let mut state = SnippetState::new_without_persistence();
    assert_eq!(state.selected_index(), 0);

    state.select_at(0);
    assert_eq!(state.selected_index(), 0);
}
