use super::*;

#[test]
fn test_enter_delete_mode() {
    let mut state = SnippetState::new_without_persistence();
    state.set_snippets(vec![Snippet {
        name: "My Snippet".to_string(),
        query: ".test".to_string(),
        description: None,
    }]);

    state.enter_delete_mode();

    assert!(matches!(
        state.mode(),
        SnippetMode::ConfirmDelete { snippet_name } if snippet_name == "My Snippet"
    ));
}

#[test]
fn test_enter_delete_mode_with_no_snippets() {
    let mut state = SnippetState::new_without_persistence();

    state.enter_delete_mode();

    assert_eq!(*state.mode(), SnippetMode::Browse);
}

#[test]
fn test_cancel_delete() {
    let mut state = SnippetState::new_without_persistence();
    state.set_snippets(vec![Snippet {
        name: "My Snippet".to_string(),
        query: ".test".to_string(),
        description: None,
    }]);
    state.enter_delete_mode();

    state.cancel_delete();

    assert_eq!(*state.mode(), SnippetMode::Browse);
    assert_eq!(state.snippets().len(), 1);
}

#[test]
fn test_confirm_delete_success() {
    let mut state = SnippetState::new_without_persistence();
    state.set_snippets(vec![Snippet {
        name: "My Snippet".to_string(),
        query: ".test".to_string(),
        description: None,
    }]);
    state.enter_delete_mode();

    let result = state.confirm_delete();

    assert!(result.is_ok());
    assert!(state.snippets().is_empty());
    assert_eq!(*state.mode(), SnippetMode::Browse);
}

#[test]
fn test_confirm_delete_not_in_mode_fails() {
    let mut state = SnippetState::new_without_persistence();
    state.set_snippets(vec![Snippet {
        name: "My Snippet".to_string(),
        query: ".test".to_string(),
        description: None,
    }]);

    let result = state.confirm_delete();

    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .contains("Not in delete confirmation mode")
    );
    assert_eq!(state.snippets().len(), 1);
}

#[test]
fn test_delete_first_snippet() {
    let mut state = SnippetState::new_without_persistence();
    state.set_snippets(vec![
        Snippet {
            name: "First".to_string(),
            query: ".first".to_string(),
            description: None,
        },
        Snippet {
            name: "Second".to_string(),
            query: ".second".to_string(),
            description: None,
        },
        Snippet {
            name: "Third".to_string(),
            query: ".third".to_string(),
            description: None,
        },
    ]);
    state.set_selected_index(0);
    state.enter_delete_mode();

    state.confirm_delete().unwrap();

    assert_eq!(state.snippets().len(), 2);
    assert_eq!(state.snippets()[0].name, "Second");
    assert_eq!(state.snippets()[1].name, "Third");
    assert_eq!(state.selected_index(), 0);
}

#[test]
fn test_delete_middle_snippet() {
    let mut state = SnippetState::new_without_persistence();
    state.set_snippets(vec![
        Snippet {
            name: "First".to_string(),
            query: ".first".to_string(),
            description: None,
        },
        Snippet {
            name: "Second".to_string(),
            query: ".second".to_string(),
            description: None,
        },
        Snippet {
            name: "Third".to_string(),
            query: ".third".to_string(),
            description: None,
        },
    ]);
    state.set_selected_index(1);
    state.enter_delete_mode();

    state.confirm_delete().unwrap();

    assert_eq!(state.snippets().len(), 2);
    assert_eq!(state.snippets()[0].name, "First");
    assert_eq!(state.snippets()[1].name, "Third");
    assert_eq!(state.selected_index(), 1);
}

#[test]
fn test_delete_last_snippet_adjusts_selection() {
    let mut state = SnippetState::new_without_persistence();
    state.set_snippets(vec![
        Snippet {
            name: "First".to_string(),
            query: ".first".to_string(),
            description: None,
        },
        Snippet {
            name: "Second".to_string(),
            query: ".second".to_string(),
            description: None,
        },
    ]);
    state.set_selected_index(1);
    state.enter_delete_mode();

    state.confirm_delete().unwrap();

    assert_eq!(state.snippets().len(), 1);
    assert_eq!(state.snippets()[0].name, "First");
    assert_eq!(state.selected_index(), 0);
}

#[test]
fn test_delete_only_snippet() {
    let mut state = SnippetState::new_without_persistence();
    state.set_snippets(vec![Snippet {
        name: "Only One".to_string(),
        query: ".only".to_string(),
        description: None,
    }]);
    state.enter_delete_mode();

    state.confirm_delete().unwrap();

    assert!(state.snippets().is_empty());
    assert_eq!(state.selected_index(), 0);
}

#[test]
fn test_delete_updates_filtered_indices() {
    let mut state = SnippetState::new_without_persistence();
    state.set_snippets(vec![
        Snippet {
            name: "First".to_string(),
            query: ".first".to_string(),
            description: None,
        },
        Snippet {
            name: "Second".to_string(),
            query: ".second".to_string(),
            description: None,
        },
    ]);
    state.enter_delete_mode();

    state.confirm_delete().unwrap();

    assert_eq!(state.filtered_count(), 1);
}

#[test]
fn test_is_editing_not_in_delete_mode() {
    let mut state = SnippetState::new_without_persistence();
    state.set_snippets(vec![Snippet {
        name: "My Snippet".to_string(),
        query: ".test".to_string(),
        description: None,
    }]);

    assert!(!state.is_editing());
    state.enter_delete_mode();
    assert!(!state.is_editing());
}

#[test]
fn test_close_resets_delete_mode() {
    let mut state = SnippetState::new_without_persistence();
    state.set_snippets(vec![Snippet {
        name: "My Snippet".to_string(),
        query: ".test".to_string(),
        description: None,
    }]);
    state.enter_delete_mode();

    state.close();

    assert_eq!(*state.mode(), SnippetMode::Browse);
}

#[test]
fn test_delete_with_search_filter_active() {
    let mut state = SnippetState::new_without_persistence();
    state.set_snippets(vec![
        Snippet {
            name: "Alpha".to_string(),
            query: ".alpha".to_string(),
            description: None,
        },
        Snippet {
            name: "Beta".to_string(),
            query: ".beta".to_string(),
            description: None,
        },
        Snippet {
            name: "Gamma".to_string(),
            query: ".gamma".to_string(),
            description: None,
        },
    ]);
    state.set_search_query("Beta");
    state.enter_delete_mode();

    state.confirm_delete().unwrap();

    assert_eq!(state.snippets().len(), 2);
    assert_eq!(state.snippets()[0].name, "Alpha");
    assert_eq!(state.snippets()[1].name, "Gamma");
}
