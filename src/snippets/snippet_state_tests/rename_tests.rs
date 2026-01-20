use super::*;

#[test]
fn test_enter_edit_mode() {
    let mut state = SnippetState::new_without_persistence();
    state.set_snippets(vec![Snippet {
        name: "My Snippet".to_string(),
        query: ".test".to_string(),
        description: None,
    }]);

    state.enter_edit_mode();

    assert!(matches!(
        state.mode(),
        SnippetMode::EditName { original_name } if original_name == "My Snippet"
    ));
    assert!(state.is_editing());
    assert_eq!(state.name_input(), "My Snippet");
}

#[test]
fn test_enter_edit_mode_with_no_snippets() {
    let mut state = SnippetState::new_without_persistence();

    state.enter_edit_mode();

    assert_eq!(*state.mode(), SnippetMode::Browse);
}

#[test]
fn test_cancel_edit() {
    let mut state = SnippetState::new_without_persistence();
    state.set_snippets(vec![Snippet {
        name: "My Snippet".to_string(),
        query: ".test".to_string(),
        description: None,
    }]);
    state.enter_edit_mode();

    state.cancel_edit();

    assert_eq!(*state.mode(), SnippetMode::Browse);
    assert_eq!(state.name_input(), "");
}

#[test]
fn test_update_snippet_name_success() {
    let mut state = SnippetState::new_without_persistence();
    state.set_snippets(vec![Snippet {
        name: "Old Name".to_string(),
        query: ".test".to_string(),
        description: None,
    }]);
    state.enter_edit_mode();

    state.name_textarea_mut().select_all();
    state.name_textarea_mut().cut();
    state.name_textarea_mut().insert_str("New Name");

    let result = state.update_snippet_name();
    assert!(result.is_ok());
    assert_eq!(state.snippets()[0].name, "New Name");
    // Update methods no longer change mode - caller handles navigation
    assert!(matches!(state.mode(), SnippetMode::EditName { .. }));
}

#[test]
fn test_update_snippet_name_empty_fails() {
    let mut state = SnippetState::new_without_persistence();
    state.set_snippets(vec![Snippet {
        name: "My Snippet".to_string(),
        query: ".test".to_string(),
        description: None,
    }]);
    state.enter_edit_mode();

    state.name_textarea_mut().select_all();
    state.name_textarea_mut().cut();

    let result = state.update_snippet_name();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("empty"));
    assert_eq!(state.snippets()[0].name, "My Snippet");
}

#[test]
fn test_update_snippet_name_whitespace_only_fails() {
    let mut state = SnippetState::new_without_persistence();
    state.set_snippets(vec![Snippet {
        name: "My Snippet".to_string(),
        query: ".test".to_string(),
        description: None,
    }]);
    state.enter_edit_mode();

    state.name_textarea_mut().select_all();
    state.name_textarea_mut().cut();
    state.name_textarea_mut().insert_str("   ");

    let result = state.update_snippet_name();
    assert!(result.is_err());
    assert_eq!(state.snippets()[0].name, "My Snippet");
}

#[test]
fn test_update_snippet_name_trims_name() {
    let mut state = SnippetState::new_without_persistence();
    state.set_snippets(vec![Snippet {
        name: "Old Name".to_string(),
        query: ".test".to_string(),
        description: None,
    }]);
    state.enter_edit_mode();

    state.name_textarea_mut().select_all();
    state.name_textarea_mut().cut();
    state.name_textarea_mut().insert_str("  New Name  ");

    let result = state.update_snippet_name();
    assert!(result.is_ok());
    assert_eq!(state.snippets()[0].name, "New Name");
}

#[test]
fn test_update_snippet_name_duplicate_fails() {
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
    state.enter_edit_mode();

    state.name_textarea_mut().select_all();
    state.name_textarea_mut().cut();
    state.name_textarea_mut().insert_str("Second");

    let result = state.update_snippet_name();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("already exists"));
    assert_eq!(state.snippets()[0].name, "First");
}

#[test]
fn test_update_snippet_name_case_insensitive_duplicate() {
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
    state.enter_edit_mode();

    state.name_textarea_mut().select_all();
    state.name_textarea_mut().cut();
    state.name_textarea_mut().insert_str("SECOND");

    let result = state.update_snippet_name();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("already exists"));
}

#[test]
fn test_update_snippet_name_same_name_allowed() {
    let mut state = SnippetState::new_without_persistence();
    state.set_snippets(vec![Snippet {
        name: "My Snippet".to_string(),
        query: ".test".to_string(),
        description: None,
    }]);
    state.enter_edit_mode();

    let result = state.update_snippet_name();
    assert!(result.is_ok());
    assert_eq!(state.snippets()[0].name, "My Snippet");
}

#[test]
fn test_update_snippet_name_same_name_different_case_allowed() {
    let mut state = SnippetState::new_without_persistence();
    state.set_snippets(vec![Snippet {
        name: "my snippet".to_string(),
        query: ".test".to_string(),
        description: None,
    }]);
    state.enter_edit_mode();

    state.name_textarea_mut().select_all();
    state.name_textarea_mut().cut();
    state.name_textarea_mut().insert_str("My Snippet");

    let result = state.update_snippet_name();
    assert!(result.is_ok());
    assert_eq!(state.snippets()[0].name, "My Snippet");
}

#[test]
fn test_edit_name_keeps_snippet_position() {
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
    state.enter_edit_mode();

    state.name_textarea_mut().select_all();
    state.name_textarea_mut().cut();
    state.name_textarea_mut().insert_str("Renamed");

    state.update_snippet_name().unwrap();

    assert_eq!(state.snippets()[0].name, "First");
    assert_eq!(state.snippets()[1].name, "Renamed");
    assert_eq!(state.snippets()[2].name, "Third");
}

#[test]
fn test_edit_name_preserves_query_and_description() {
    let mut state = SnippetState::new_without_persistence();
    state.set_snippets(vec![Snippet {
        name: "Old Name".to_string(),
        query: ".complex | query".to_string(),
        description: Some("My description".to_string()),
    }]);
    state.enter_edit_mode();

    state.name_textarea_mut().select_all();
    state.name_textarea_mut().cut();
    state.name_textarea_mut().insert_str("New Name");

    state.update_snippet_name().unwrap();

    assert_eq!(state.snippets()[0].name, "New Name");
    assert_eq!(state.snippets()[0].query, ".complex | query");
    assert_eq!(
        state.snippets()[0].description,
        Some("My description".to_string())
    );
}

#[test]
fn test_update_name_not_in_edit_mode_fails() {
    let mut state = SnippetState::new_without_persistence();
    state.set_snippets(vec![Snippet {
        name: "My Snippet".to_string(),
        query: ".test".to_string(),
        description: None,
    }]);

    let result = state.update_snippet_name();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Not in edit name mode"));
}

#[test]
fn test_is_editing_in_edit_mode() {
    let mut state = SnippetState::new_without_persistence();
    state.set_snippets(vec![Snippet {
        name: "My Snippet".to_string(),
        query: ".test".to_string(),
        description: None,
    }]);

    assert!(!state.is_editing());
    state.enter_edit_mode();
    assert!(state.is_editing());
}

#[test]
fn test_close_resets_edit_mode() {
    let mut state = SnippetState::new_without_persistence();
    state.set_snippets(vec![Snippet {
        name: "My Snippet".to_string(),
        query: ".test".to_string(),
        description: None,
    }]);
    state.open();
    state.enter_edit_mode();

    state.close();

    assert_eq!(*state.mode(), SnippetMode::Browse);
    assert_eq!(state.name_input(), "");
}
