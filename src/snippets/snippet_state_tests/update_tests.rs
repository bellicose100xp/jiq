use super::*;

#[test]
fn test_enter_update_confirmation_success() {
    let mut state = SnippetState::new_without_persistence();
    state.set_snippets(vec![Snippet {
        name: "My Snippet".to_string(),
        query: ".old".to_string(),
        description: None,
    }]);

    let result = state.enter_update_confirmation(".new".to_string());

    assert!(result.is_ok());
    assert!(matches!(
        state.mode(),
        SnippetMode::ConfirmUpdate {
            snippet_name,
            old_query,
            new_query
        } if snippet_name == "My Snippet" && old_query == ".old" && new_query == ".new"
    ));
}

#[test]
fn test_enter_update_confirmation_with_no_snippets() {
    let mut state = SnippetState::new_without_persistence();

    let result = state.enter_update_confirmation(".new".to_string());

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("No snippet selected"));
    assert_eq!(*state.mode(), SnippetMode::Browse);
}

#[test]
fn test_enter_update_confirmation_with_identical_query() {
    let mut state = SnippetState::new_without_persistence();
    state.set_snippets(vec![Snippet {
        name: "My Snippet".to_string(),
        query: ".same".to_string(),
        description: None,
    }]);

    let result = state.enter_update_confirmation(".same".to_string());

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("No changes to update"));
    assert_eq!(*state.mode(), SnippetMode::Browse);
}

#[test]
fn test_enter_update_confirmation_with_identical_query_trimmed() {
    let mut state = SnippetState::new_without_persistence();
    state.set_snippets(vec![Snippet {
        name: "My Snippet".to_string(),
        query: ".same".to_string(),
        description: None,
    }]);

    let result = state.enter_update_confirmation("  .same  ".to_string());

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("No changes to update"));
}

#[test]
fn test_cancel_update() {
    let mut state = SnippetState::new_without_persistence();
    state.set_snippets(vec![Snippet {
        name: "My Snippet".to_string(),
        query: ".old".to_string(),
        description: None,
    }]);
    state.enter_update_confirmation(".new".to_string()).unwrap();

    state.cancel_update();

    assert_eq!(*state.mode(), SnippetMode::Browse);
    assert_eq!(state.snippets()[0].query, ".old");
}

#[test]
fn test_confirm_update_success() {
    let mut state = SnippetState::new_without_persistence();
    state.set_snippets(vec![Snippet {
        name: "My Snippet".to_string(),
        query: ".old".to_string(),
        description: None,
    }]);
    state.enter_update_confirmation(".new".to_string()).unwrap();

    let result = state.confirm_update();

    assert!(result.is_ok());
    assert_eq!(state.snippets()[0].query, ".new");
    assert_eq!(*state.mode(), SnippetMode::Browse);
}

#[test]
fn test_confirm_update_not_in_mode_fails() {
    let mut state = SnippetState::new_without_persistence();
    state.set_snippets(vec![Snippet {
        name: "My Snippet".to_string(),
        query: ".old".to_string(),
        description: None,
    }]);

    let result = state.confirm_update();

    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .contains("Not in update confirmation mode")
    );
    assert_eq!(state.snippets()[0].query, ".old");
}

#[test]
fn test_confirm_update_preserves_other_fields() {
    let mut state = SnippetState::new_without_persistence();
    state.set_snippets(vec![Snippet {
        name: "My Snippet".to_string(),
        query: ".old".to_string(),
        description: Some("A description".to_string()),
    }]);
    state.enter_update_confirmation(".new".to_string()).unwrap();

    state.confirm_update().unwrap();

    assert_eq!(state.snippets()[0].name, "My Snippet");
    assert_eq!(state.snippets()[0].query, ".new");
    assert_eq!(
        state.snippets()[0].description,
        Some("A description".to_string())
    );
}

#[test]
fn test_update_middle_snippet() {
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
    state
        .enter_update_confirmation(".second_updated".to_string())
        .unwrap();

    state.confirm_update().unwrap();

    assert_eq!(state.snippets()[0].query, ".first");
    assert_eq!(state.snippets()[1].query, ".second_updated");
    assert_eq!(state.snippets()[2].query, ".third");
}

#[test]
fn test_is_editing_not_in_update_mode() {
    let mut state = SnippetState::new_without_persistence();
    state.set_snippets(vec![Snippet {
        name: "My Snippet".to_string(),
        query: ".test".to_string(),
        description: None,
    }]);

    assert!(!state.is_editing());
    state.enter_update_confirmation(".new".to_string()).unwrap();
    assert!(!state.is_editing());
}

#[test]
fn test_close_resets_update_mode() {
    let mut state = SnippetState::new_without_persistence();
    state.set_snippets(vec![Snippet {
        name: "My Snippet".to_string(),
        query: ".old".to_string(),
        description: None,
    }]);
    state.enter_update_confirmation(".new".to_string()).unwrap();

    state.close();

    assert_eq!(*state.mode(), SnippetMode::Browse);
}

#[test]
fn test_update_with_search_filter_active() {
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
    state
        .enter_update_confirmation(".beta_updated".to_string())
        .unwrap();

    state.confirm_update().unwrap();

    assert_eq!(state.snippets()[1].query, ".beta_updated");
}

#[test]
fn test_update_long_query() {
    let mut state = SnippetState::new_without_persistence();
    let long_query = ".foo | .bar | .baz | select(.value > 10) | map(.name)";
    state.set_snippets(vec![Snippet {
        name: "Complex".to_string(),
        query: ".simple".to_string(),
        description: None,
    }]);
    state
        .enter_update_confirmation(long_query.to_string())
        .unwrap();

    state.confirm_update().unwrap();

    assert_eq!(state.snippets()[0].query, long_query);
}

#[test]
fn test_update_does_not_affect_filtered_indices() {
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
    state
        .enter_update_confirmation(".first_new".to_string())
        .unwrap();

    state.confirm_update().unwrap();

    assert_eq!(state.filtered_count(), 2);
}
