use super::*;

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
