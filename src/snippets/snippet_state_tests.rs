pub use super::{Snippet, SnippetMode, SnippetState};

#[path = "snippet_state_tests/basic_tests.rs"]
mod basic_tests;
#[path = "snippet_state_tests/create_tests.rs"]
mod create_tests;
#[path = "snippet_state_tests/delete_tests.rs"]
mod delete_tests;
#[path = "snippet_state_tests/description_tests.rs"]
mod description_tests;
#[path = "snippet_state_tests/edit_query_tests.rs"]
mod edit_query_tests;
#[path = "snippet_state_tests/hover_tests.rs"]
mod hover_tests;
#[path = "snippet_state_tests/navigation_tests.rs"]
mod navigation_tests;
#[path = "snippet_state_tests/rename_tests.rs"]
mod rename_tests;
#[path = "snippet_state_tests/scrollable_tests.rs"]
mod scrollable_tests;
#[path = "snippet_state_tests/search_tests.rs"]
mod search_tests;
#[path = "snippet_state_tests/update_tests.rs"]
mod update_tests;

fn two_snippets() -> Vec<Snippet> {
    vec![
        Snippet {
            name: "First".to_string(),
            query: ".f".to_string(),
            description: None,
        },
        Snippet {
            name: "Second".to_string(),
            query: ".s".to_string(),
            description: None,
        },
    ]
}

#[test]
fn test_next_field_noop_in_browse_mode() {
    let mut state = SnippetState::new_without_persistence();
    state.set_snippets(vec![Snippet {
        name: "A".to_string(),
        query: ".".to_string(),
        description: None,
    }]);
    assert_eq!(*state.mode(), SnippetMode::Browse);

    state.next_field();
    assert_eq!(*state.mode(), SnippetMode::Browse);

    state.enter_delete_mode();
    let mode_before = state.mode().clone();
    assert!(matches!(mode_before, SnippetMode::ConfirmDelete { .. }));
    state.next_field();
    assert_eq!(*state.mode(), mode_before);
}

#[test]
fn test_prev_field_noop_in_browse_mode() {
    let mut state = SnippetState::new_without_persistence();
    state.set_snippets(vec![Snippet {
        name: "A".to_string(),
        query: ".".to_string(),
        description: None,
    }]);
    assert_eq!(*state.mode(), SnippetMode::Browse);

    state.prev_field();
    assert_eq!(*state.mode(), SnippetMode::Browse);

    state.enter_delete_mode();
    let mode_before = state.mode().clone();
    assert!(matches!(mode_before, SnippetMode::ConfirmDelete { .. }));
    state.prev_field();
    assert_eq!(*state.mode(), mode_before);
}

#[test]
fn test_update_snippet_description_not_in_mode_fails() {
    let mut state = SnippetState::new_without_persistence();
    state.set_snippets(vec![Snippet {
        name: "A".to_string(),
        query: ".".to_string(),
        description: Some("d".to_string()),
    }]);
    assert_eq!(*state.mode(), SnippetMode::Browse);

    let result = state.update_snippet_description();

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Not in edit description mode"));
    assert_eq!(state.snippets()[0].description, Some("d".to_string()));
}

#[test]
fn test_confirm_delete_name_mismatch_fails() {
    let mut state = SnippetState::new_without_persistence();
    state.set_snippets(two_snippets());
    state.enter_delete_mode(); // captures "First" (selected index 0)
    state.set_selected_index(1); // selection now points at "Second"

    let result = state.confirm_delete();

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("does not match"));
    assert_eq!(state.snippets().len(), 2);
    assert_eq!(state.snippets()[0].name, "First");
    assert_eq!(state.snippets()[1].name, "Second");
}

#[test]
fn test_confirm_update_name_mismatch_fails() {
    let mut state = SnippetState::new_without_persistence();
    state.set_snippets(two_snippets());
    state
        .enter_update_confirmation(".f_new".to_string())
        .unwrap(); // captures "First"
    state.set_selected_index(1); // selection now points at "Second"

    let result = state.confirm_update();

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("does not match"));
    assert_eq!(state.snippets()[0].query, ".f");
    assert_eq!(state.snippets()[1].query, ".s");
}

#[test]
fn test_adjust_scroll_offset_moves_up_when_selection_above_viewport() {
    let mut state = SnippetState::new_without_persistence();
    state.set_snippets(
        (0..20)
            .map(|i| Snippet {
                name: format!("s{i}"),
                query: ".".to_string(),
                description: None,
            })
            .collect(),
    );
    state.set_visible_count(5);

    state.set_selected_index(10); // offset becomes 10 - 5 + 1 = 6
    assert_eq!(state.scroll_offset(), 6);

    state.set_selected_index(2); // 2 < 6 -> offset pulled up to 2
    assert_eq!(state.scroll_offset(), 2);
}
