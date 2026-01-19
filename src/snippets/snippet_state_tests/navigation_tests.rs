use super::*;

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
