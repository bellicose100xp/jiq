use super::*;

#[test]
fn test_enter_edit_query_mode() {
    let mut state = SnippetState::new_without_persistence();
    state.set_snippets(vec![Snippet {
        name: "My Snippet".to_string(),
        query: ".test | keys".to_string(),
        description: None,
    }]);

    state.enter_edit_query_mode();

    assert!(matches!(
        state.mode(),
        SnippetMode::EditQuery { snippet_name } if snippet_name == "My Snippet"
    ));
    assert!(state.is_editing());
    assert_eq!(state.query_input(), ".test | keys");
}

#[test]
fn test_enter_edit_query_mode_with_no_snippets() {
    let mut state = SnippetState::new_without_persistence();

    state.enter_edit_query_mode();

    assert_eq!(*state.mode(), SnippetMode::Browse);
}

#[test]
fn test_cancel_edit_query() {
    let mut state = SnippetState::new_without_persistence();
    state.set_snippets(vec![Snippet {
        name: "My Snippet".to_string(),
        query: ".test".to_string(),
        description: None,
    }]);
    state.enter_edit_query_mode();

    state.cancel_edit_query();

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
    state.enter_edit_query_mode();

    state.query_textarea_mut().select_all();
    state.query_textarea_mut().cut();
    state.query_textarea_mut().insert_str(".new | keys");

    let result = state.update_snippet_query();
    assert!(result.is_ok());
    assert_eq!(state.snippets()[0].query, ".new | keys");
    assert_eq!(*state.mode(), SnippetMode::Browse);
}

#[test]
fn test_update_snippet_query_empty_fails() {
    let mut state = SnippetState::new_without_persistence();
    state.set_snippets(vec![Snippet {
        name: "My Snippet".to_string(),
        query: ".test".to_string(),
        description: None,
    }]);
    state.enter_edit_query_mode();

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
    state.enter_edit_query_mode();

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
    state.enter_edit_query_mode();

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
    state.enter_edit_query_mode();

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
    state.enter_edit_query_mode();

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
    state.enter_edit_query_mode();
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
    state.enter_edit_query_mode();

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
    state.enter_edit_query_mode();

    let result = state.update_snippet_query();
    assert!(result.is_ok());
    assert_eq!(state.snippets()[0].query, ".test");
}

#[test]
fn test_enter_edit_query_populates_textarea() {
    let mut state = SnippetState::new_without_persistence();
    state.set_snippets(vec![Snippet {
        name: "Complex Query".to_string(),
        query: ".data[] | select(.active) | {id, name}".to_string(),
        description: None,
    }]);

    state.enter_edit_query_mode();

    assert_eq!(
        state.query_input(),
        ".data[] | select(.active) | {id, name}"
    );
}
