use super::*;
use crate::snippets::Snippet;
use insta::assert_snapshot;
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::layout::Rect;

fn create_test_terminal(width: u16, height: u16) -> Terminal<TestBackend> {
    let backend = TestBackend::new(width, height);
    Terminal::new(backend).unwrap()
}

fn render_snippet_popup_to_string(
    state: &mut SnippetState,
    results_area: Rect,
    width: u16,
    height: u16,
) -> String {
    let mut terminal = create_test_terminal(width, height);
    terminal
        .draw(|f| render_popup(state, f, results_area))
        .unwrap();
    terminal.backend().to_string()
}

fn create_state_with_snippets(snippets: Vec<Snippet>) -> SnippetState {
    let mut state = SnippetState::new();
    state.set_snippets(snippets);
    state
}

#[test]
fn snapshot_empty_snippet_popup() {
    let mut state = SnippetState::new();
    let results_area = Rect {
        x: 0,
        y: 0,
        width: 80,
        height: 20,
    };
    let output = render_snippet_popup_to_string(&mut state, results_area, 80, 24);
    assert_snapshot!(output);
}

#[test]
fn snapshot_snippet_popup_narrow_terminal() {
    let mut state = SnippetState::new();
    let results_area = Rect {
        x: 0,
        y: 0,
        width: 40,
        height: 15,
    };
    let output = render_snippet_popup_to_string(&mut state, results_area, 40, 20);
    assert_snapshot!(output);
}

#[test]
fn snapshot_snippet_popup_small_height() {
    let mut state = SnippetState::new();
    let results_area = Rect {
        x: 0,
        y: 0,
        width: 80,
        height: 6,
    };
    let output = render_snippet_popup_to_string(&mut state, results_area, 80, 10);
    assert_snapshot!(output);
}

#[test]
fn snapshot_snippet_popup_with_snippets() {
    let snippets = vec![
        Snippet {
            name: "Select all keys".to_string(),
            query: "keys".to_string(),
            description: Some("Returns array of all keys".to_string()),
        },
        Snippet {
            name: "Flatten arrays".to_string(),
            query: "flatten".to_string(),
            description: None,
        },
        Snippet {
            name: "Filter by type".to_string(),
            query: ".[] | select(.type == \"error\")".to_string(),
            description: Some("Filter items by type".to_string()),
        },
    ];
    let mut state = create_state_with_snippets(snippets);
    let results_area = Rect {
        x: 0,
        y: 0,
        width: 80,
        height: 20,
    };
    let output = render_snippet_popup_to_string(&mut state, results_area, 80, 24);
    assert_snapshot!(output);
}

#[test]
fn snapshot_snippet_popup_with_single_snippet() {
    let snippets = vec![Snippet {
        name: "Identity".to_string(),
        query: ".".to_string(),
        description: None,
    }];
    let mut state = create_state_with_snippets(snippets);
    let results_area = Rect {
        x: 0,
        y: 0,
        width: 80,
        height: 20,
    };
    let output = render_snippet_popup_to_string(&mut state, results_area, 80, 24);
    assert_snapshot!(output);
}

#[test]
fn snapshot_snippet_popup_with_snippets_narrow() {
    let snippets = vec![
        Snippet {
            name: "Select all keys".to_string(),
            query: "keys".to_string(),
            description: None,
        },
        Snippet {
            name: "Flatten".to_string(),
            query: "flatten".to_string(),
            description: None,
        },
    ];
    let mut state = create_state_with_snippets(snippets);
    let results_area = Rect {
        x: 0,
        y: 0,
        width: 40,
        height: 15,
    };
    let output = render_snippet_popup_to_string(&mut state, results_area, 40, 20);
    assert_snapshot!(output);
}

#[test]
fn snapshot_snippet_popup_with_second_item_selected() {
    let snippets = vec![
        Snippet {
            name: "Select all keys".to_string(),
            query: "keys".to_string(),
            description: Some("Returns array of all keys".to_string()),
        },
        Snippet {
            name: "Flatten arrays".to_string(),
            query: "flatten".to_string(),
            description: None,
        },
        Snippet {
            name: "Filter by type".to_string(),
            query: ".[] | select(.type == \"error\")".to_string(),
            description: Some("Filter items by type".to_string()),
        },
    ];
    let mut state = create_state_with_snippets(snippets);
    state.set_selected_index(1);

    let results_area = Rect {
        x: 0,
        y: 0,
        width: 80,
        height: 20,
    };
    let output = render_snippet_popup_to_string(&mut state, results_area, 80, 24);
    assert_snapshot!(output);
}

#[test]
fn snapshot_snippet_popup_with_last_item_selected() {
    let snippets = vec![
        Snippet {
            name: "Select all keys".to_string(),
            query: "keys".to_string(),
            description: None,
        },
        Snippet {
            name: "Flatten arrays".to_string(),
            query: "flatten".to_string(),
            description: None,
        },
        Snippet {
            name: "Filter by type".to_string(),
            query: ".[] | select(.type == \"error\")".to_string(),
            description: None,
        },
    ];
    let mut state = create_state_with_snippets(snippets);
    state.set_selected_index(2);

    let results_area = Rect {
        x: 0,
        y: 0,
        width: 80,
        height: 20,
    };
    let output = render_snippet_popup_to_string(&mut state, results_area, 80, 24);
    assert_snapshot!(output);
}

#[test]
fn snapshot_preview_with_description() {
    let snippets = vec![
        Snippet {
            name: "Select all keys".to_string(),
            query: "keys".to_string(),
            description: Some("Returns an array of all keys in the object".to_string()),
        },
        Snippet {
            name: "Flatten arrays".to_string(),
            query: "flatten".to_string(),
            description: Some("Flattens nested arrays into a single array".to_string()),
        },
    ];
    let mut state = create_state_with_snippets(snippets);
    let results_area = Rect {
        x: 0,
        y: 0,
        width: 80,
        height: 20,
    };
    let output = render_snippet_popup_to_string(&mut state, results_area, 80, 24);
    assert_snapshot!(output);
}

#[test]
fn snapshot_preview_with_long_query_wrapping() {
    let snippets = vec![Snippet {
        name: "Complex filter".to_string(),
        query: ".data[] | select(.status == \"active\" and .type == \"premium\") | {id, name, email, created_at, metadata}".to_string(),
        description: Some("Filters active premium users and extracts key fields".to_string()),
    }];
    let mut state = create_state_with_snippets(snippets);
    let results_area = Rect {
        x: 0,
        y: 0,
        width: 80,
        height: 20,
    };
    let output = render_snippet_popup_to_string(&mut state, results_area, 80, 24);
    assert_snapshot!(output);
}

#[test]
fn snapshot_very_short_height_falls_back_to_list_only() {
    let snippets = vec![
        Snippet {
            name: "Keys".to_string(),
            query: "keys".to_string(),
            description: Some("Get keys".to_string()),
        },
        Snippet {
            name: "Flatten".to_string(),
            query: "flatten".to_string(),
            description: None,
        },
    ];
    let mut state = create_state_with_snippets(snippets);
    let results_area = Rect {
        x: 0,
        y: 0,
        width: 60,
        height: 5,
    };
    let output = render_snippet_popup_to_string(&mut state, results_area, 60, 8);
    assert_snapshot!(output);
}

#[test]
fn snapshot_preview_without_description() {
    let snippets = vec![Snippet {
        name: "Identity".to_string(),
        query: ".".to_string(),
        description: None,
    }];
    let mut state = create_state_with_snippets(snippets);
    let results_area = Rect {
        x: 0,
        y: 0,
        width: 80,
        height: 20,
    };
    let output = render_snippet_popup_to_string(&mut state, results_area, 80, 24);
    assert_snapshot!(output);
}

#[test]
fn snapshot_filtered_results_with_search() {
    let snippets = vec![
        Snippet {
            name: "Select all keys".to_string(),
            query: "keys".to_string(),
            description: Some("Returns array of all keys".to_string()),
        },
        Snippet {
            name: "Flatten arrays".to_string(),
            query: "flatten".to_string(),
            description: None,
        },
        Snippet {
            name: "Select items".to_string(),
            query: ".[]".to_string(),
            description: Some("Select all items".to_string()),
        },
    ];
    let mut state = create_state_with_snippets(snippets);
    state.set_search_query("select");

    let results_area = Rect {
        x: 0,
        y: 0,
        width: 80,
        height: 20,
    };
    let output = render_snippet_popup_to_string(&mut state, results_area, 80, 24);
    assert_snapshot!(output);
}

#[test]
fn snapshot_no_matches_message() {
    let snippets = vec![
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
    ];
    let mut state = create_state_with_snippets(snippets);
    state.set_search_query("xyz123");

    let results_area = Rect {
        x: 0,
        y: 0,
        width: 80,
        height: 20,
    };
    let output = render_snippet_popup_to_string(&mut state, results_area, 80, 24);
    assert_snapshot!(output);
}

#[test]
fn snapshot_create_mode_empty_name() {
    let mut state = SnippetState::new();
    state.enter_create_mode(".test | keys");

    let results_area = Rect {
        x: 0,
        y: 0,
        width: 80,
        height: 20,
    };
    let output = render_snippet_popup_to_string(&mut state, results_area, 80, 24);
    assert_snapshot!(output);
}

#[test]
fn snapshot_create_mode_with_name_typed() {
    let mut state = SnippetState::new();
    state.enter_create_mode(".test | keys");
    state.name_textarea_mut().insert_str("My Snippet");

    let results_area = Rect {
        x: 0,
        y: 0,
        width: 80,
        height: 20,
    };
    let output = render_snippet_popup_to_string(&mut state, results_area, 80, 24);
    assert_snapshot!(output);
}

#[test]
fn snapshot_create_mode_with_long_query() {
    let mut state = SnippetState::new();
    state.enter_create_mode(
        ".data[] | select(.status == \"active\" and .type == \"premium\") | {id, name, email}",
    );

    let results_area = Rect {
        x: 0,
        y: 0,
        width: 80,
        height: 20,
    };
    let output = render_snippet_popup_to_string(&mut state, results_area, 80, 24);
    assert_snapshot!(output);
}

#[test]
fn snapshot_create_mode_narrow_terminal() {
    let mut state = SnippetState::new();
    state.enter_create_mode(".test");
    state.name_textarea_mut().insert_str("Test");

    let results_area = Rect {
        x: 0,
        y: 0,
        width: 40,
        height: 15,
    };
    let output = render_snippet_popup_to_string(&mut state, results_area, 40, 20);
    assert_snapshot!(output);
}

#[test]
fn snapshot_create_mode_small_height() {
    let mut state = SnippetState::new();
    state.enter_create_mode(".test");

    let results_area = Rect {
        x: 0,
        y: 0,
        width: 80,
        height: 6,
    };
    let output = render_snippet_popup_to_string(&mut state, results_area, 80, 10);
    assert_snapshot!(output);
}

#[test]
fn snapshot_create_description_mode_empty() {
    let mut state = SnippetState::new();
    state.enter_create_mode(".test | keys");
    state.name_textarea_mut().insert_str("My Snippet");
    state.next_field();

    let results_area = Rect {
        x: 0,
        y: 0,
        width: 80,
        height: 20,
    };
    let output = render_snippet_popup_to_string(&mut state, results_area, 80, 24);
    assert_snapshot!(output);
}

#[test]
fn snapshot_create_description_mode_with_text() {
    let mut state = SnippetState::new();
    state.enter_create_mode(".test | keys");
    state.name_textarea_mut().insert_str("My Snippet");
    state.next_field();
    state
        .description_textarea_mut()
        .insert_str("A helpful description");

    let results_area = Rect {
        x: 0,
        y: 0,
        width: 80,
        height: 20,
    };
    let output = render_snippet_popup_to_string(&mut state, results_area, 80, 24);
    assert_snapshot!(output);
}

#[test]
fn snapshot_create_description_mode_narrow() {
    let mut state = SnippetState::new();
    state.enter_create_mode(".test");
    state.name_textarea_mut().insert_str("Test");
    state.next_field();
    state.description_textarea_mut().insert_str("Desc");

    let results_area = Rect {
        x: 0,
        y: 0,
        width: 40,
        height: 15,
    };
    let output = render_snippet_popup_to_string(&mut state, results_area, 40, 20);
    assert_snapshot!(output);
}

#[test]
fn snapshot_create_description_mode_small_height() {
    let mut state = SnippetState::new();
    state.enter_create_mode(".test");
    state.name_textarea_mut().insert_str("Test");
    state.next_field();

    let results_area = Rect {
        x: 0,
        y: 0,
        width: 80,
        height: 6,
    };
    let output = render_snippet_popup_to_string(&mut state, results_area, 80, 10);
    assert_snapshot!(output);
}

#[test]
fn snapshot_create_name_mode_with_description_field_visible() {
    let mut state = SnippetState::new();
    state.enter_create_mode(".test | keys");
    state.name_textarea_mut().insert_str("My Snippet");

    let results_area = Rect {
        x: 0,
        y: 0,
        width: 80,
        height: 20,
    };
    let output = render_snippet_popup_to_string(&mut state, results_area, 80, 24);
    assert_snapshot!(output);
}

#[test]
fn snapshot_rename_mode_with_original_name() {
    let mut state = SnippetState::new_without_persistence();
    state.set_snippets(vec![Snippet {
        name: "My Snippet".to_string(),
        query: ".test | keys".to_string(),
        description: None,
    }]);
    state.enter_edit_mode();

    let results_area = Rect {
        x: 0,
        y: 0,
        width: 80,
        height: 20,
    };
    let output = render_snippet_popup_to_string(&mut state, results_area, 80, 24);
    assert_snapshot!(output);
}

#[test]
fn snapshot_rename_mode_with_edited_name() {
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

    let results_area = Rect {
        x: 0,
        y: 0,
        width: 80,
        height: 20,
    };
    let output = render_snippet_popup_to_string(&mut state, results_area, 80, 24);
    assert_snapshot!(output);
}

#[test]
fn snapshot_rename_mode_narrow_terminal() {
    let mut state = SnippetState::new_without_persistence();
    state.set_snippets(vec![Snippet {
        name: "My Snippet".to_string(),
        query: ".test".to_string(),
        description: None,
    }]);
    state.enter_edit_mode();

    let results_area = Rect {
        x: 0,
        y: 0,
        width: 40,
        height: 15,
    };
    let output = render_snippet_popup_to_string(&mut state, results_area, 40, 20);
    assert_snapshot!(output);
}

#[test]
fn snapshot_rename_mode_small_height() {
    let mut state = SnippetState::new_without_persistence();
    state.set_snippets(vec![Snippet {
        name: "My Snippet".to_string(),
        query: ".test".to_string(),
        description: None,
    }]);
    state.enter_edit_mode();

    let results_area = Rect {
        x: 0,
        y: 0,
        width: 80,
        height: 4,
    };
    let output = render_snippet_popup_to_string(&mut state, results_area, 80, 10);
    assert_snapshot!(output);
}

#[test]
fn snapshot_edit_query_mode_with_original_query() {
    let mut state = SnippetState::new_without_persistence();
    state.set_snippets(vec![Snippet {
        name: "My Snippet".to_string(),
        query: ".test | keys".to_string(),
        description: None,
    }]);
    state.enter_edit_mode();

    let results_area = Rect {
        x: 0,
        y: 0,
        width: 80,
        height: 20,
    };
    let output = render_snippet_popup_to_string(&mut state, results_area, 80, 24);
    assert_snapshot!(output);
}

#[test]
fn snapshot_edit_query_mode_with_edited_query() {
    let mut state = SnippetState::new_without_persistence();
    state.set_snippets(vec![Snippet {
        name: "My Snippet".to_string(),
        query: ".old".to_string(),
        description: None,
    }]);
    state.enter_edit_mode();
    state.query_textarea_mut().select_all();
    state.query_textarea_mut().cut();
    state.query_textarea_mut().insert_str(".new | keys | sort");

    let results_area = Rect {
        x: 0,
        y: 0,
        width: 80,
        height: 20,
    };
    let output = render_snippet_popup_to_string(&mut state, results_area, 80, 24);
    assert_snapshot!(output);
}

#[test]
fn snapshot_edit_query_mode_narrow_terminal() {
    let mut state = SnippetState::new_without_persistence();
    state.set_snippets(vec![Snippet {
        name: "My Snippet".to_string(),
        query: ".test".to_string(),
        description: None,
    }]);
    state.enter_edit_mode();

    let results_area = Rect {
        x: 0,
        y: 0,
        width: 40,
        height: 15,
    };
    let output = render_snippet_popup_to_string(&mut state, results_area, 40, 20);
    assert_snapshot!(output);
}

#[test]
fn snapshot_edit_query_mode_small_height() {
    let mut state = SnippetState::new_without_persistence();
    state.set_snippets(vec![Snippet {
        name: "My Snippet".to_string(),
        query: ".test".to_string(),
        description: None,
    }]);
    state.enter_edit_mode();

    let results_area = Rect {
        x: 0,
        y: 0,
        width: 80,
        height: 4,
    };
    let output = render_snippet_popup_to_string(&mut state, results_area, 80, 10);
    assert_snapshot!(output);
}

#[test]
fn snapshot_confirm_delete_mode() {
    let mut state = SnippetState::new_without_persistence();
    state.set_snippets(vec![Snippet {
        name: "My Snippet".to_string(),
        query: ".test | keys".to_string(),
        description: None,
    }]);
    state.enter_delete_mode();

    let results_area = Rect {
        x: 0,
        y: 0,
        width: 80,
        height: 20,
    };
    let output = render_snippet_popup_to_string(&mut state, results_area, 80, 24);
    assert_snapshot!(output);
}

#[test]
fn snapshot_confirm_delete_mode_narrow_terminal() {
    let mut state = SnippetState::new_without_persistence();
    state.set_snippets(vec![Snippet {
        name: "My Snippet".to_string(),
        query: ".test".to_string(),
        description: None,
    }]);
    state.enter_delete_mode();

    let results_area = Rect {
        x: 0,
        y: 0,
        width: 40,
        height: 15,
    };
    let output = render_snippet_popup_to_string(&mut state, results_area, 40, 20);
    assert_snapshot!(output);
}

#[test]
fn snapshot_confirm_delete_mode_long_name() {
    let mut state = SnippetState::new_without_persistence();
    state.set_snippets(vec![Snippet {
        name: "This is a very long snippet name that should be truncated".to_string(),
        query: ".test".to_string(),
        description: None,
    }]);
    state.enter_delete_mode();

    let results_area = Rect {
        x: 0,
        y: 0,
        width: 80,
        height: 20,
    };
    let output = render_snippet_popup_to_string(&mut state, results_area, 80, 24);
    assert_snapshot!(output);
}

#[test]
fn snapshot_confirm_delete_mode_small_area() {
    let mut state = SnippetState::new_without_persistence();
    state.set_snippets(vec![Snippet {
        name: "Test".to_string(),
        query: ".test".to_string(),
        description: None,
    }]);
    state.enter_delete_mode();

    let results_area = Rect {
        x: 0,
        y: 0,
        width: 50,
        height: 10,
    };
    let output = render_snippet_popup_to_string(&mut state, results_area, 50, 15);
    assert_snapshot!(output);
}
