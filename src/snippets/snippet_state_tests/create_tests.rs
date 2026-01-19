use super::*;

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
