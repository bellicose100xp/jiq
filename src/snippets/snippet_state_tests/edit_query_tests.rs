use super::*;

fn enter_edit_query_mode(state: &mut SnippetState) {
    state.enter_edit_mode();
    state.next_field(); // EditName -> EditQuery
}

#[test]
fn test_enter_edit_query_via_next_field() {
    let mut state = SnippetState::new_without_persistence();
    state.set_snippets(vec![Snippet {
        name: "My Snippet".to_string(),
        query: ".test | keys".to_string(),
        description: None,
    }]);

    state.enter_edit_mode();
    assert!(matches!(state.mode(), SnippetMode::EditName { .. }));

    state.next_field();
    assert!(matches!(
        state.mode(),
        SnippetMode::EditQuery { original_query } if original_query == ".test | keys"
    ));
    assert!(state.is_editing());
    assert_eq!(state.query_input(), ".test | keys");
}

#[test]
fn test_cancel_edit_in_query_mode() {
    let mut state = SnippetState::new_without_persistence();
    state.set_snippets(vec![Snippet {
        name: "My Snippet".to_string(),
        query: ".test".to_string(),
        description: None,
    }]);
    enter_edit_query_mode(&mut state);

    state.cancel_edit();

    assert_eq!(*state.mode(), SnippetMode::Browse);
    assert_eq!(state.query_input(), "");
}

#[test]
fn test_update_snippet_query_success() {
    let mut state = SnippetState::new_without_persistence();
    state.set_snippets(vec![Snippet {
        name: "My Snippet".to_string(),
        query: ".old".to_string(),
        description: None,
    }]);
    enter_edit_query_mode(&mut state);

    state.query_textarea_mut().select_all();
    state.query_textarea_mut().cut();
    state.query_textarea_mut().insert_str(".new | keys");

    let result = state.update_snippet_query();
    assert!(result.is_ok());
    assert_eq!(state.snippets()[0].query, ".new | keys");
    // Update methods no longer change mode - caller handles navigation
    assert!(matches!(state.mode(), SnippetMode::EditQuery { .. }));
}

#[test]
fn test_update_snippet_query_empty_fails() {
    let mut state = SnippetState::new_without_persistence();
    state.set_snippets(vec![Snippet {
        name: "My Snippet".to_string(),
        query: ".test".to_string(),
        description: None,
    }]);
    enter_edit_query_mode(&mut state);

    state.query_textarea_mut().select_all();
    state.query_textarea_mut().cut();

    let result = state.update_snippet_query();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("empty"));
    assert_eq!(state.snippets()[0].query, ".test");
}

#[test]
fn test_update_snippet_query_whitespace_only_fails() {
    let mut state = SnippetState::new_without_persistence();
    state.set_snippets(vec![Snippet {
        name: "My Snippet".to_string(),
        query: ".test".to_string(),
        description: None,
    }]);
    enter_edit_query_mode(&mut state);

    state.query_textarea_mut().select_all();
    state.query_textarea_mut().cut();
    state.query_textarea_mut().insert_str("   ");

    let result = state.update_snippet_query();
    assert!(result.is_err());
    assert_eq!(state.snippets()[0].query, ".test");
}

#[test]
fn test_update_snippet_query_trims_whitespace() {
    let mut state = SnippetState::new_without_persistence();
    state.set_snippets(vec![Snippet {
        name: "My Snippet".to_string(),
        query: ".old".to_string(),
        description: None,
    }]);
    enter_edit_query_mode(&mut state);

    state.query_textarea_mut().select_all();
    state.query_textarea_mut().cut();
    state.query_textarea_mut().insert_str("  .new  ");

    let result = state.update_snippet_query();
    assert!(result.is_ok());
    assert_eq!(state.snippets()[0].query, ".new");
}

#[test]
fn test_edit_query_keeps_snippet_position() {
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
    enter_edit_query_mode(&mut state);

    state.query_textarea_mut().select_all();
    state.query_textarea_mut().cut();
    state.query_textarea_mut().insert_str(".updated");

    state.update_snippet_query().unwrap();

    assert_eq!(state.snippets()[0].query, ".first");
    assert_eq!(state.snippets()[1].query, ".updated");
    assert_eq!(state.snippets()[2].query, ".third");
}

#[test]
fn test_edit_query_preserves_name_and_description() {
    let mut state = SnippetState::new_without_persistence();
    state.set_snippets(vec![Snippet {
        name: "My Snippet".to_string(),
        query: ".old".to_string(),
        description: Some("My description".to_string()),
    }]);
    enter_edit_query_mode(&mut state);

    state.query_textarea_mut().select_all();
    state.query_textarea_mut().cut();
    state.query_textarea_mut().insert_str(".new");

    state.update_snippet_query().unwrap();

    assert_eq!(state.snippets()[0].name, "My Snippet");
    assert_eq!(state.snippets()[0].query, ".new");
    assert_eq!(
        state.snippets()[0].description,
        Some("My description".to_string())
    );
}

#[test]
fn test_update_snippet_query_not_in_edit_mode_fails() {
    let mut state = SnippetState::new_without_persistence();
    state.set_snippets(vec![Snippet {
        name: "My Snippet".to_string(),
        query: ".test".to_string(),
        description: None,
    }]);

    let result = state.update_snippet_query();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Not in edit query mode"));
}

#[test]
fn test_is_editing_in_edit_query_mode() {
    let mut state = SnippetState::new_without_persistence();
    state.set_snippets(vec![Snippet {
        name: "My Snippet".to_string(),
        query: ".test".to_string(),
        description: None,
    }]);

    assert!(!state.is_editing());
    enter_edit_query_mode(&mut state);
    assert!(state.is_editing());
}

#[test]
fn test_close_resets_edit_query_mode() {
    let mut state = SnippetState::new_without_persistence();
    state.set_snippets(vec![Snippet {
        name: "My Snippet".to_string(),
        query: ".test".to_string(),
        description: None,
    }]);
    state.open();
    enter_edit_query_mode(&mut state);

    state.close();

    assert_eq!(*state.mode(), SnippetMode::Browse);
    assert_eq!(state.query_input(), "");
}

#[test]
fn test_edit_query_same_query_succeeds() {
    let mut state = SnippetState::new_without_persistence();
    state.set_snippets(vec![Snippet {
        name: "My Snippet".to_string(),
        query: ".test".to_string(),
        description: None,
    }]);
    enter_edit_query_mode(&mut state);

    let result = state.update_snippet_query();
    assert!(result.is_ok());
    assert_eq!(state.snippets()[0].query, ".test");
}

#[test]
fn test_edit_query_populates_textarea() {
    let mut state = SnippetState::new_without_persistence();
    state.set_snippets(vec![Snippet {
        name: "Complex Query".to_string(),
        query: ".data[] | select(.active) | {id, name}".to_string(),
        description: None,
    }]);

    enter_edit_query_mode(&mut state);

    assert_eq!(
        state.query_input(),
        ".data[] | select(.active) | {id, name}"
    );
}

#[test]
fn test_edit_mode_field_cycling() {
    let mut state = SnippetState::new_without_persistence();
    state.set_snippets(vec![Snippet {
        name: "Test".to_string(),
        query: ".test".to_string(),
        description: Some("Desc".to_string()),
    }]);

    state.enter_edit_mode();
    assert!(matches!(state.mode(), SnippetMode::EditName { .. }));

    state.next_field();
    assert!(matches!(state.mode(), SnippetMode::EditQuery { .. }));

    state.next_field();
    assert!(matches!(state.mode(), SnippetMode::EditDescription { .. }));

    state.next_field();
    assert!(matches!(state.mode(), SnippetMode::EditName { .. }));
}

#[test]
fn test_edit_mode_prev_field_cycling() {
    let mut state = SnippetState::new_without_persistence();
    state.set_snippets(vec![Snippet {
        name: "Test".to_string(),
        query: ".test".to_string(),
        description: Some("Desc".to_string()),
    }]);

    state.enter_edit_mode();
    assert!(matches!(state.mode(), SnippetMode::EditName { .. }));

    state.prev_field();
    assert!(matches!(state.mode(), SnippetMode::EditDescription { .. }));

    state.prev_field();
    assert!(matches!(state.mode(), SnippetMode::EditQuery { .. }));

    state.prev_field();
    assert!(matches!(state.mode(), SnippetMode::EditName { .. }));
}
