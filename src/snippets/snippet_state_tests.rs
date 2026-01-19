use super::*;

#[test]
fn test_new_snippet_state() {
    let state = SnippetState::new();
    assert!(!state.is_visible());
    assert_eq!(*state.mode(), SnippetMode::Browse);
}

#[test]
fn test_default_snippet_state() {
    let state = SnippetState::default();
    assert!(!state.is_visible());
}

#[test]
fn test_open_snippet_popup() {
    let mut state = SnippetState::new();
    assert!(!state.is_visible());

    state.open();
    assert!(state.is_visible());
}

#[test]
fn test_close_snippet_popup() {
    let mut state = SnippetState::new();
    state.open();
    assert!(state.is_visible());

    state.close();
    assert!(!state.is_visible());
}

#[test]
fn test_open_close_open() {
    let mut state = SnippetState::new();

    state.open();
    assert!(state.is_visible());

    state.close();
    assert!(!state.is_visible());

    state.open();
    assert!(state.is_visible());
}

#[test]
fn test_is_editing_returns_false_in_browse_mode() {
    let state = SnippetState::new();
    assert!(!state.is_editing());
}

#[test]
fn test_initial_selected_index_is_zero() {
    let state = SnippetState::new();
    assert_eq!(state.selected_index(), 0);
}

#[test]
fn test_selected_index_resets_on_open() {
    let mut state = SnippetState::new();
    state.set_snippets(vec![
        Snippet {
            name: "test1".to_string(),
            query: ".".to_string(),
            description: None,
        },
        Snippet {
            name: "test2".to_string(),
            query: ".".to_string(),
            description: None,
        },
    ]);
    state.select_next();
    assert_eq!(state.selected_index(), 1);

    state.open();
    assert_eq!(state.selected_index(), 0);
}

#[test]
fn test_select_next_increments_index() {
    let mut state = SnippetState::new();
    state.set_snippets(vec![
        Snippet {
            name: "test1".to_string(),
            query: ".".to_string(),
            description: None,
        },
        Snippet {
            name: "test2".to_string(),
            query: ".".to_string(),
            description: None,
        },
        Snippet {
            name: "test3".to_string(),
            query: ".".to_string(),
            description: None,
        },
    ]);

    assert_eq!(state.selected_index(), 0);
    state.select_next();
    assert_eq!(state.selected_index(), 1);
    state.select_next();
    assert_eq!(state.selected_index(), 2);
}

#[test]
fn test_select_next_stops_at_last_item() {
    let mut state = SnippetState::new();
    state.set_snippets(vec![
        Snippet {
            name: "test1".to_string(),
            query: ".".to_string(),
            description: None,
        },
        Snippet {
            name: "test2".to_string(),
            query: ".".to_string(),
            description: None,
        },
    ]);

    state.select_next();
    assert_eq!(state.selected_index(), 1);

    state.select_next();
    assert_eq!(state.selected_index(), 1);

    state.select_next();
    assert_eq!(state.selected_index(), 1);
}

#[test]
fn test_select_prev_decrements_index() {
    let mut state = SnippetState::new();
    state.set_snippets(vec![
        Snippet {
            name: "test1".to_string(),
            query: ".".to_string(),
            description: None,
        },
        Snippet {
            name: "test2".to_string(),
            query: ".".to_string(),
            description: None,
        },
        Snippet {
            name: "test3".to_string(),
            query: ".".to_string(),
            description: None,
        },
    ]);
    state.select_next();
    state.select_next();
    assert_eq!(state.selected_index(), 2);

    state.select_prev();
    assert_eq!(state.selected_index(), 1);
    state.select_prev();
    assert_eq!(state.selected_index(), 0);
}

#[test]
fn test_select_prev_stops_at_first_item() {
    let mut state = SnippetState::new();
    state.set_snippets(vec![
        Snippet {
            name: "test1".to_string(),
            query: ".".to_string(),
            description: None,
        },
        Snippet {
            name: "test2".to_string(),
            query: ".".to_string(),
            description: None,
        },
    ]);

    assert_eq!(state.selected_index(), 0);

    state.select_prev();
    assert_eq!(state.selected_index(), 0);

    state.select_prev();
    assert_eq!(state.selected_index(), 0);
}

#[test]
fn test_select_next_with_empty_list() {
    let mut state = SnippetState::new();
    assert_eq!(state.selected_index(), 0);

    state.select_next();
    assert_eq!(state.selected_index(), 0);
}

#[test]
fn test_select_prev_with_empty_list() {
    let mut state = SnippetState::new();
    assert_eq!(state.selected_index(), 0);

    state.select_prev();
    assert_eq!(state.selected_index(), 0);
}

#[test]
fn test_select_next_with_single_item() {
    let mut state = SnippetState::new();
    state.set_snippets(vec![Snippet {
        name: "test".to_string(),
        query: ".".to_string(),
        description: None,
    }]);

    assert_eq!(state.selected_index(), 0);
    state.select_next();
    assert_eq!(state.selected_index(), 0);
}

#[test]
fn test_selected_snippet_returns_correct_snippet() {
    let mut state = SnippetState::new();
    let snippets = vec![
        Snippet {
            name: "first".to_string(),
            query: ".first".to_string(),
            description: None,
        },
        Snippet {
            name: "second".to_string(),
            query: ".second".to_string(),
            description: Some("desc".to_string()),
        },
    ];
    state.set_snippets(snippets);

    let selected = state.selected_snippet().unwrap();
    assert_eq!(selected.name, "first");
    assert_eq!(selected.query, ".first");

    state.select_next();
    let selected = state.selected_snippet().unwrap();
    assert_eq!(selected.name, "second");
    assert_eq!(selected.query, ".second");
}

#[test]
fn test_selected_snippet_returns_none_for_empty_list() {
    let state = SnippetState::new();
    assert!(state.selected_snippet().is_none());
}

#[test]
fn test_set_snippets_resets_selected_index() {
    let mut state = SnippetState::new();
    state.set_snippets(vec![
        Snippet {
            name: "test1".to_string(),
            query: ".".to_string(),
            description: None,
        },
        Snippet {
            name: "test2".to_string(),
            query: ".".to_string(),
            description: None,
        },
    ]);
    state.select_next();
    assert_eq!(state.selected_index(), 1);

    state.set_snippets(vec![Snippet {
        name: "new".to_string(),
        query: ".".to_string(),
        description: None,
    }]);
    assert_eq!(state.selected_index(), 0);
}

#[test]
fn test_filtered_count_returns_all_when_no_search() {
    let mut state = SnippetState::new();
    state.set_snippets(vec![
        Snippet {
            name: "test1".to_string(),
            query: ".".to_string(),
            description: None,
        },
        Snippet {
            name: "test2".to_string(),
            query: ".".to_string(),
            description: None,
        },
        Snippet {
            name: "test3".to_string(),
            query: ".".to_string(),
            description: None,
        },
    ]);
    assert_eq!(state.filtered_count(), 3);
}

#[test]
fn test_search_filters_snippets() {
    let mut state = SnippetState::new();
    state.set_snippets(vec![
        Snippet {
            name: "Select keys".to_string(),
            query: "keys".to_string(),
            description: None,
        },
        Snippet {
            name: "Flatten arrays".to_string(),
            query: "flatten".to_string(),
            description: None,
        },
        Snippet {
            name: "Select items".to_string(),
            query: ".[]".to_string(),
            description: None,
        },
    ]);

    state.set_search_query("select");
    assert_eq!(state.filtered_count(), 2);
}

#[test]
fn test_search_no_matches() {
    let mut state = SnippetState::new();
    state.set_snippets(vec![
        Snippet {
            name: "Select keys".to_string(),
            query: "keys".to_string(),
            description: None,
        },
        Snippet {
            name: "Flatten arrays".to_string(),
            query: "flatten".to_string(),
            description: None,
        },
    ]);

    state.set_search_query("xyz123");
    assert_eq!(state.filtered_count(), 0);
    assert!(state.selected_snippet().is_none());
}

#[test]
fn test_search_clears_on_close() {
    let mut state = SnippetState::new();
    state.set_snippets(vec![
        Snippet {
            name: "test1".to_string(),
            query: ".".to_string(),
            description: None,
        },
        Snippet {
            name: "test2".to_string(),
            query: ".".to_string(),
            description: None,
        },
    ]);

    state.set_search_query("test1");
    assert_eq!(state.filtered_count(), 1);

    state.close();
    assert_eq!(state.search_query(), "");
    assert_eq!(state.filtered_count(), 2);
}

#[test]
fn test_search_resets_selection() {
    let mut state = SnippetState::new();
    state.set_snippets(vec![
        Snippet {
            name: "Select keys".to_string(),
            query: "keys".to_string(),
            description: None,
        },
        Snippet {
            name: "Flatten arrays".to_string(),
            query: "flatten".to_string(),
            description: None,
        },
        Snippet {
            name: "Select items".to_string(),
            query: ".[]".to_string(),
            description: None,
        },
    ]);

    state.select_next();
    state.select_next();
    assert_eq!(state.selected_index(), 2);

    state.set_search_query("select");
    assert_eq!(state.selected_index(), 0);
}

#[test]
fn test_on_search_input_changed_resets_selection() {
    let mut state = SnippetState::new();
    state.set_snippets(vec![
        Snippet {
            name: "test1".to_string(),
            query: ".".to_string(),
            description: None,
        },
        Snippet {
            name: "test2".to_string(),
            query: ".".to_string(),
            description: None,
        },
    ]);

    state.select_next();
    assert_eq!(state.selected_index(), 1);

    state.on_search_input_changed();
    assert_eq!(state.selected_index(), 0);
}

#[test]
fn test_selected_snippet_uses_filtered_indices() {
    let mut state = SnippetState::new();
    state.set_snippets(vec![
        Snippet {
            name: "Flatten arrays".to_string(),
            query: "flatten".to_string(),
            description: None,
        },
        Snippet {
            name: "Select keys".to_string(),
            query: "keys".to_string(),
            description: None,
        },
        Snippet {
            name: "Select items".to_string(),
            query: ".[]".to_string(),
            description: None,
        },
    ]);

    state.set_search_query("select");
    let selected = state.selected_snippet().unwrap();
    assert!(selected.name.contains("Select"));
}

#[test]
fn test_navigation_respects_filtered_list() {
    let mut state = SnippetState::new();
    state.set_snippets(vec![
        Snippet {
            name: "Flatten arrays".to_string(),
            query: "flatten".to_string(),
            description: None,
        },
        Snippet {
            name: "Select keys".to_string(),
            query: "keys".to_string(),
            description: None,
        },
        Snippet {
            name: "Select items".to_string(),
            query: ".[]".to_string(),
            description: None,
        },
    ]);

    state.set_search_query("select");
    assert_eq!(state.filtered_count(), 2);
    assert_eq!(state.selected_index(), 0);

    state.select_next();
    assert_eq!(state.selected_index(), 1);

    state.select_next();
    assert_eq!(state.selected_index(), 1);
}

#[test]
fn test_multi_term_search() {
    let mut state = SnippetState::new();
    state.set_snippets(vec![
        Snippet {
            name: "Select all keys".to_string(),
            query: "keys".to_string(),
            description: None,
        },
        Snippet {
            name: "Select items".to_string(),
            query: ".[]".to_string(),
            description: None,
        },
        Snippet {
            name: "Get all values".to_string(),
            query: "values".to_string(),
            description: None,
        },
    ]);

    state.set_search_query("select all");
    assert_eq!(state.filtered_count(), 1);
    let selected = state.selected_snippet().unwrap();
    assert_eq!(selected.name, "Select all keys");
}

#[test]
fn test_enter_create_mode() {
    let mut state = SnippetState::new();
    state.enter_create_mode(".test | keys");

    assert_eq!(*state.mode(), SnippetMode::CreateName);
    assert_eq!(state.pending_query(), ".test | keys");
    assert!(state.is_editing());
}

#[test]
fn test_cancel_create_mode() {
    let mut state = SnippetState::new();
    state.enter_create_mode(".test");
    assert_eq!(*state.mode(), SnippetMode::CreateName);

    state.cancel_create();
    assert_eq!(*state.mode(), SnippetMode::Browse);
    assert_eq!(state.pending_query(), "");
    assert!(!state.is_editing());
}

#[test]
fn test_is_editing_in_browse_mode() {
    let state = SnippetState::new();
    assert!(!state.is_editing());
}

#[test]
fn test_is_editing_in_create_mode() {
    let mut state = SnippetState::new();
    state.enter_create_mode(".test");
    assert!(state.is_editing());
}

#[test]
fn test_save_new_snippet_success() {
    let mut state = SnippetState::new_without_persistence();
    state.enter_create_mode(".test | keys");
    state.name_textarea_mut().insert_str("Test Snippet");

    let result = state.save_new_snippet();
    assert!(result.is_ok());
    assert_eq!(state.snippets().len(), 1);
    assert_eq!(state.snippets()[0].name, "Test Snippet");
    assert_eq!(state.snippets()[0].query, ".test | keys");
    assert_eq!(state.snippets()[0].description, None);
    assert_eq!(*state.mode(), SnippetMode::Browse);
}

#[test]
fn test_save_new_snippet_empty_name_fails() {
    let mut state = SnippetState::new_without_persistence();
    state.enter_create_mode(".test");

    let result = state.save_new_snippet();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("empty"));
    assert_eq!(state.snippets().len(), 0);
    assert_eq!(*state.mode(), SnippetMode::CreateName);
}

#[test]
fn test_save_new_snippet_whitespace_only_name_fails() {
    let mut state = SnippetState::new_without_persistence();
    state.enter_create_mode(".test");
    state.name_textarea_mut().insert_str("   ");

    let result = state.save_new_snippet();
    assert!(result.is_err());
    assert_eq!(state.snippets().len(), 0);
}

#[test]
fn test_save_new_snippet_duplicate_name_fails() {
    let mut state = SnippetState::new_without_persistence();
    state.set_snippets(vec![Snippet {
        name: "Existing".to_string(),
        query: ".foo".to_string(),
        description: None,
    }]);

    state.enter_create_mode(".bar");
    state.name_textarea_mut().insert_str("Existing");

    let result = state.save_new_snippet();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("already exists"));
    assert_eq!(state.snippets().len(), 1);
}

#[test]
fn test_save_new_snippet_trims_name() {
    let mut state = SnippetState::new_without_persistence();
    state.enter_create_mode(".test");
    state.name_textarea_mut().insert_str("  My Snippet  ");

    let result = state.save_new_snippet();
    assert!(result.is_ok());
    assert_eq!(state.snippets()[0].name, "My Snippet");
}

#[test]
fn test_close_resets_create_mode() {
    let mut state = SnippetState::new();
    state.enter_create_mode(".test");
    state.name_textarea_mut().insert_str("Test");
    assert_eq!(*state.mode(), SnippetMode::CreateName);

    state.close();
    assert_eq!(*state.mode(), SnippetMode::Browse);
    assert_eq!(state.pending_query(), "");
    assert_eq!(state.name_input(), "");
}

#[test]
fn test_name_textarea_input() {
    let mut state = SnippetState::new();
    state.enter_create_mode(".test");
    state.name_textarea_mut().insert_str("My Snippet");
    assert_eq!(state.name_input(), "My Snippet");
}

#[test]
fn test_filtered_indices_updated_after_save() {
    let mut state = SnippetState::new_without_persistence();
    state.set_snippets(vec![Snippet {
        name: "First".to_string(),
        query: ".first".to_string(),
        description: None,
    }]);
    assert_eq!(state.filtered_count(), 1);

    state.enter_create_mode(".second");
    state.name_textarea_mut().insert_str("Second");
    state.save_new_snippet().unwrap();

    assert_eq!(state.filtered_count(), 2);
}

#[test]
fn test_mode_default_is_browse() {
    let mode = SnippetMode::default();
    assert_eq!(mode, SnippetMode::Browse);
}

#[test]
fn test_snippet_mode_eq() {
    assert_eq!(SnippetMode::Browse, SnippetMode::Browse);
    assert_eq!(SnippetMode::CreateName, SnippetMode::CreateName);
    assert_ne!(SnippetMode::Browse, SnippetMode::CreateName);
}

#[test]
fn test_snippet_mode_clone() {
    let mode = SnippetMode::CreateName;
    let cloned = mode.clone();
    assert_eq!(mode, cloned);
}

#[test]
fn test_save_new_snippet_empty_query_fails() {
    let mut state = SnippetState::new_without_persistence();
    state.enter_create_mode("");
    state.name_textarea_mut().insert_str("My Snippet");

    let result = state.save_new_snippet();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Query cannot be empty"));
    assert_eq!(state.snippets().len(), 0);
    assert_eq!(*state.mode(), SnippetMode::CreateName);
}

#[test]
fn test_save_new_snippet_whitespace_only_query_fails() {
    let mut state = SnippetState::new_without_persistence();
    state.enter_create_mode("   ");
    state.name_textarea_mut().insert_str("My Snippet");

    let result = state.save_new_snippet();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Query cannot be empty"));
    assert_eq!(state.snippets().len(), 0);
}

#[test]
fn test_save_new_snippet_trims_query() {
    let mut state = SnippetState::new_without_persistence();
    state.enter_create_mode("  .test | keys  ");
    state.name_textarea_mut().insert_str("My Snippet");

    let result = state.save_new_snippet();
    assert!(result.is_ok());
    assert_eq!(state.snippets()[0].query, ".test | keys");
}

#[test]
fn test_save_new_snippet_case_insensitive_duplicate_uppercase() {
    let mut state = SnippetState::new_without_persistence();
    state.set_snippets(vec![Snippet {
        name: "existing".to_string(),
        query: ".foo".to_string(),
        description: None,
    }]);

    state.enter_create_mode(".bar");
    state.name_textarea_mut().insert_str("EXISTING");

    let result = state.save_new_snippet();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("already exists"));
    assert_eq!(state.snippets().len(), 1);
}

#[test]
fn test_save_new_snippet_case_insensitive_duplicate_mixedcase() {
    let mut state = SnippetState::new_without_persistence();
    state.set_snippets(vec![Snippet {
        name: "MySnippet".to_string(),
        query: ".foo".to_string(),
        description: None,
    }]);

    state.enter_create_mode(".bar");
    state.name_textarea_mut().insert_str("mysnippet");

    let result = state.save_new_snippet();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("already exists"));
}

#[test]
fn test_save_new_snippet_case_insensitive_duplicate_titlecase() {
    let mut state = SnippetState::new_without_persistence();
    state.set_snippets(vec![Snippet {
        name: "select keys".to_string(),
        query: ".foo".to_string(),
        description: None,
    }]);

    state.enter_create_mode(".bar");
    state.name_textarea_mut().insert_str("Select Keys");

    let result = state.save_new_snippet();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("already exists"));
}

#[test]
fn test_new_snippet_inserted_at_beginning() {
    let mut state = SnippetState::new_without_persistence();
    state.set_snippets(vec![
        Snippet {
            name: "Old First".to_string(),
            query: ".first".to_string(),
            description: None,
        },
        Snippet {
            name: "Old Second".to_string(),
            query: ".second".to_string(),
            description: None,
        },
    ]);

    state.enter_create_mode(".new");
    state.name_textarea_mut().insert_str("New Snippet");
    state.save_new_snippet().unwrap();

    assert_eq!(state.snippets().len(), 3);
    assert_eq!(state.snippets()[0].name, "New Snippet");
    assert_eq!(state.snippets()[0].query, ".new");
    assert_eq!(state.snippets()[1].name, "Old First");
    assert_eq!(state.snippets()[2].name, "Old Second");
}

#[test]
fn test_multiple_new_snippets_maintain_newest_first_order() {
    let mut state = SnippetState::new_without_persistence();

    state.enter_create_mode(".first");
    state.name_textarea_mut().insert_str("First");
    state.save_new_snippet().unwrap();

    state.enter_create_mode(".second");
    state.name_textarea_mut().insert_str("Second");
    state.save_new_snippet().unwrap();

    state.enter_create_mode(".third");
    state.name_textarea_mut().insert_str("Third");
    state.save_new_snippet().unwrap();

    assert_eq!(state.snippets().len(), 3);
    assert_eq!(state.snippets()[0].name, "Third");
    assert_eq!(state.snippets()[1].name, "Second");
    assert_eq!(state.snippets()[2].name, "First");
}
