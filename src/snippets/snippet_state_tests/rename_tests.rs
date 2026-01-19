use super::*;

#[test]
fn test_enter_rename_mode() {
    let mut state = SnippetState::new_without_persistence();
    state.set_snippets(vec![Snippet {
        name: "My Snippet".to_string(),
        query: ".test".to_string(),
        description: None,
    }]);

    state.enter_rename_mode();

    assert!(matches!(
        state.mode(),
        SnippetMode::EditName { original_name } if original_name == "My Snippet"
    ));
    assert!(state.is_editing());
    assert_eq!(state.name_input(), "My Snippet");
}

#[test]
fn test_enter_rename_mode_with_no_snippets() {
    let mut state = SnippetState::new_without_persistence();

    state.enter_rename_mode();

    assert_eq!(*state.mode(), SnippetMode::Browse);
}

#[test]
fn test_cancel_rename() {
    let mut state = SnippetState::new_without_persistence();
    state.set_snippets(vec![Snippet {
        name: "My Snippet".to_string(),
        query: ".test".to_string(),
        description: None,
    }]);
    state.enter_rename_mode();

    state.cancel_rename();

    assert_eq!(*state.mode(), SnippetMode::Browse);
    assert_eq!(state.name_input(), "");
}

#[test]
fn test_rename_snippet_success() {
    let mut state = SnippetState::new_without_persistence();
    state.set_snippets(vec![Snippet {
        name: "Old Name".to_string(),
        query: ".test".to_string(),
        description: None,
    }]);
    state.enter_rename_mode();

    state.name_textarea_mut().select_all();
    state.name_textarea_mut().cut();
    state.name_textarea_mut().insert_str("New Name");

    let result = state.rename_snippet();
    assert!(result.is_ok());
    assert_eq!(state.snippets()[0].name, "New Name");
    assert_eq!(*state.mode(), SnippetMode::Browse);
}

#[test]
fn test_rename_snippet_empty_name_fails() {
    let mut state = SnippetState::new_without_persistence();
    state.set_snippets(vec![Snippet {
        name: "My Snippet".to_string(),
        query: ".test".to_string(),
        description: None,
    }]);
    state.enter_rename_mode();

    state.name_textarea_mut().select_all();
    state.name_textarea_mut().cut();

    let result = state.rename_snippet();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("empty"));
    assert_eq!(state.snippets()[0].name, "My Snippet");
}

#[test]
fn test_rename_snippet_whitespace_only_fails() {
    let mut state = SnippetState::new_without_persistence();
    state.set_snippets(vec![Snippet {
        name: "My Snippet".to_string(),
        query: ".test".to_string(),
        description: None,
    }]);
    state.enter_rename_mode();

    state.name_textarea_mut().select_all();
    state.name_textarea_mut().cut();
    state.name_textarea_mut().insert_str("   ");

    let result = state.rename_snippet();
    assert!(result.is_err());
    assert_eq!(state.snippets()[0].name, "My Snippet");
}

#[test]
fn test_rename_snippet_trims_name() {
    let mut state = SnippetState::new_without_persistence();
    state.set_snippets(vec![Snippet {
        name: "Old Name".to_string(),
        query: ".test".to_string(),
        description: None,
    }]);
    state.enter_rename_mode();

    state.name_textarea_mut().select_all();
    state.name_textarea_mut().cut();
    state.name_textarea_mut().insert_str("  New Name  ");

    let result = state.rename_snippet();
    assert!(result.is_ok());
    assert_eq!(state.snippets()[0].name, "New Name");
}

#[test]
fn test_rename_snippet_duplicate_name_fails() {
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
    state.enter_rename_mode();

    state.name_textarea_mut().select_all();
    state.name_textarea_mut().cut();
    state.name_textarea_mut().insert_str("Second");

    let result = state.rename_snippet();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("already exists"));
    assert_eq!(state.snippets()[0].name, "First");
}

#[test]
fn test_rename_snippet_case_insensitive_duplicate() {
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
    state.enter_rename_mode();

    state.name_textarea_mut().select_all();
    state.name_textarea_mut().cut();
    state.name_textarea_mut().insert_str("SECOND");

    let result = state.rename_snippet();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("already exists"));
}

#[test]
fn test_rename_snippet_same_name_allowed() {
    let mut state = SnippetState::new_without_persistence();
    state.set_snippets(vec![Snippet {
        name: "My Snippet".to_string(),
        query: ".test".to_string(),
        description: None,
    }]);
    state.enter_rename_mode();

    let result = state.rename_snippet();
    assert!(result.is_ok());
    assert_eq!(state.snippets()[0].name, "My Snippet");
}

#[test]
fn test_rename_snippet_same_name_different_case_allowed() {
    let mut state = SnippetState::new_without_persistence();
    state.set_snippets(vec![Snippet {
        name: "my snippet".to_string(),
        query: ".test".to_string(),
        description: None,
    }]);
    state.enter_rename_mode();

    state.name_textarea_mut().select_all();
    state.name_textarea_mut().cut();
    state.name_textarea_mut().insert_str("My Snippet");

    let result = state.rename_snippet();
    assert!(result.is_ok());
    assert_eq!(state.snippets()[0].name, "My Snippet");
}

#[test]
fn test_rename_keeps_snippet_position() {
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
    state.enter_rename_mode();

    state.name_textarea_mut().select_all();
    state.name_textarea_mut().cut();
    state.name_textarea_mut().insert_str("Renamed");

    state.rename_snippet().unwrap();

    assert_eq!(state.snippets()[0].name, "First");
    assert_eq!(state.snippets()[1].name, "Renamed");
    assert_eq!(state.snippets()[2].name, "Third");
}

#[test]
fn test_rename_preserves_query_and_description() {
    let mut state = SnippetState::new_without_persistence();
    state.set_snippets(vec![Snippet {
        name: "Old Name".to_string(),
        query: ".complex | query".to_string(),
        description: Some("My description".to_string()),
    }]);
    state.enter_rename_mode();

    state.name_textarea_mut().select_all();
    state.name_textarea_mut().cut();
    state.name_textarea_mut().insert_str("New Name");

    state.rename_snippet().unwrap();

    assert_eq!(state.snippets()[0].name, "New Name");
    assert_eq!(state.snippets()[0].query, ".complex | query");
    assert_eq!(
        state.snippets()[0].description,
        Some("My description".to_string())
    );
}

#[test]
fn test_rename_not_in_rename_mode_fails() {
    let mut state = SnippetState::new_without_persistence();
    state.set_snippets(vec![Snippet {
        name: "My Snippet".to_string(),
        query: ".test".to_string(),
        description: None,
    }]);

    let result = state.rename_snippet();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Not in rename mode"));
}

#[test]
fn test_is_editing_in_rename_mode() {
    let mut state = SnippetState::new_without_persistence();
    state.set_snippets(vec![Snippet {
        name: "My Snippet".to_string(),
        query: ".test".to_string(),
        description: None,
    }]);

    assert!(!state.is_editing());
    state.enter_rename_mode();
    assert!(state.is_editing());
}

#[test]
fn test_close_resets_rename_mode() {
    let mut state = SnippetState::new_without_persistence();
    state.set_snippets(vec![Snippet {
        name: "My Snippet".to_string(),
        query: ".test".to_string(),
        description: None,
    }]);
    state.open();
    state.enter_rename_mode();

    state.close();

    assert_eq!(*state.mode(), SnippetMode::Browse);
    assert_eq!(state.name_input(), "");
}
