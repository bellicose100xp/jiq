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
        .draw(|f| {
            let _ = render_popup(state, f, results_area);
        })
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

// Scrollbar position tests
fn create_many_snippets(count: usize) -> Vec<Snippet> {
    (0..count)
        .map(|i| Snippet {
            name: format!("Snippet {:02}", i),
            query: format!(".query{:02}", i),
            description: None,
        })
        .collect()
}

#[test]
fn snapshot_scrollbar_at_top() {
    let snippets = create_many_snippets(30);
    let mut state = create_state_with_snippets(snippets);
    state.set_visible_count(10);
    // scroll_offset defaults to 0, so scrollbar should be at top

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
fn snapshot_scrollbar_at_middle() {
    let snippets = create_many_snippets(30);
    let mut state = create_state_with_snippets(snippets);
    state.set_visible_count(10);
    // Select item in the middle to scroll
    for _ in 0..15 {
        state.select_next();
    }

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
fn snapshot_scrollbar_at_bottom() {
    let snippets = create_many_snippets(30);
    let mut state = create_state_with_snippets(snippets);
    state.set_visible_count(10);
    // Select last item to scroll to bottom
    for _ in 0..29 {
        state.select_next();
    }

    let results_area = Rect {
        x: 0,
        y: 0,
        width: 80,
        height: 20,
    };
    let output = render_snippet_popup_to_string(&mut state, results_area, 80, 24);
    assert_snapshot!(output);
}

// Helper: drive into edit mode for a single seeded snippet, advancing
// `next_field` `advances` times so we land on the desired Edit* field.
fn edit_state_advanced(snippet: Snippet, advances: usize) -> SnippetState {
    let mut state = SnippetState::new_without_persistence();
    state.set_snippets(vec![snippet]);
    state.enter_edit_mode();
    for _ in 0..advances {
        state.next_field();
    }
    state
}

#[test]
fn snapshot_confirm_update_mode() {
    let mut state = create_state_with_snippets(vec![Snippet {
        name: "My Snippet".to_string(),
        query: ".old | keys".to_string(),
        description: None,
    }]);
    state.disable_persistence();
    state
        .enter_update_confirmation(".new | keys | sort".to_string())
        .unwrap();

    assert!(
        matches!(state.mode(), SnippetMode::ConfirmUpdate { .. }),
        "expected ConfirmUpdate mode after enter_update_confirmation"
    );

    let results_area = Rect {
        x: 0,
        y: 0,
        width: 80,
        height: 24,
    };
    let output = render_snippet_popup_to_string(&mut state, results_area, 80, 24);
    // The diff-confirmation dialog must show its title, both query labels,
    // and the snippet name.
    assert!(output.contains("Replace Snippet Query"));
    assert!(output.contains("Old query:"));
    assert!(output.contains("New query:"));
    assert!(output.contains("My Snippet"));
    assert_snapshot!(output);
}

#[test]
fn snapshot_confirm_update_mode_long_name() {
    let long_name = "A".repeat(60);
    let mut state = create_state_with_snippets(vec![Snippet {
        name: long_name,
        query: ".old".to_string(),
        description: None,
    }]);
    state.disable_persistence();
    state
        .enter_update_confirmation(".brand_new_query".to_string())
        .unwrap();

    let results_area = Rect {
        x: 0,
        y: 0,
        width: 80,
        height: 24,
    };
    let output = render_snippet_popup_to_string(&mut state, results_area, 80, 24);
    // Names longer than 40 chars are truncated with an ellipsis (lines 971-972).
    assert!(
        output.contains('…'),
        "long name should be truncated with ellipsis"
    );
    assert_snapshot!(output);
}

#[test]
fn snapshot_edit_query_mode_active_full_height() {
    let mut state = edit_state_advanced(
        Snippet {
            name: "My Snippet".to_string(),
            query: ".test | keys".to_string(),
            description: Some("Existing description".to_string()),
        },
        1,
    );
    assert!(
        matches!(state.mode(), SnippetMode::EditQuery { .. }),
        "expected EditQuery mode after one next_field"
    );

    let results_area = Rect {
        x: 0,
        y: 0,
        width: 80,
        height: 20,
    };
    let output = render_snippet_popup_to_string(&mut state, results_area, 80, 24);
    assert!(output.contains("Name"));
    assert!(output.contains("Query"));
    assert_snapshot!(output);
}

#[test]
fn snapshot_edit_description_mode_active_full_height() {
    let mut state = edit_state_advanced(
        Snippet {
            name: "My Snippet".to_string(),
            query: ".test | keys".to_string(),
            description: Some("Existing description".to_string()),
        },
        2,
    );
    assert!(
        matches!(state.mode(), SnippetMode::EditDescription { .. }),
        "expected EditDescription mode after two next_field calls"
    );

    let results_area = Rect {
        x: 0,
        y: 0,
        width: 80,
        height: 20,
    };
    let output = render_snippet_popup_to_string(&mut state, results_area, 80, 24);
    assert!(output.contains("Description"));
    assert_snapshot!(output);
}

#[test]
fn snapshot_edit_query_mode_small_height_query_field() {
    let mut state = edit_state_advanced(
        Snippet {
            name: "My Snippet".to_string(),
            query: ".test | keys".to_string(),
            description: None,
        },
        1,
    );
    assert!(matches!(state.mode(), SnippetMode::EditQuery { .. }));

    let results_area = Rect {
        x: 0,
        y: 0,
        width: 80,
        height: 4,
    };
    let output = render_snippet_popup_to_string(&mut state, results_area, 80, 10);
    // The minimal-height fallback renders the dedicated Query title.
    assert!(output.contains("Edit Snippet - Query"));
    assert_snapshot!(output);
}

#[test]
fn snapshot_edit_description_mode_small_height() {
    let mut state = edit_state_advanced(
        Snippet {
            name: "My Snippet".to_string(),
            query: ".test | keys".to_string(),
            description: Some("Existing description".to_string()),
        },
        2,
    );
    assert!(matches!(state.mode(), SnippetMode::EditDescription { .. }));

    let results_area = Rect {
        x: 0,
        y: 0,
        width: 80,
        height: 4,
    };
    let output = render_snippet_popup_to_string(&mut state, results_area, 80, 10);
    assert!(output.contains("Edit Snippet - Description"));
    assert_snapshot!(output);
}

#[test]
fn snapshot_create_description_mode_active_full_height() {
    let mut state = SnippetState::new();
    state.enter_create_mode(".test | keys");
    state.next_field();
    state.next_field();
    assert_eq!(
        *state.mode(),
        SnippetMode::CreateDescription,
        "expected CreateDescription after two next_field calls"
    );

    let results_area = Rect {
        x: 0,
        y: 0,
        width: 80,
        height: 20,
    };
    let output = render_snippet_popup_to_string(&mut state, results_area, 80, 24);
    assert!(output.contains("Description (optional)"));
    assert_snapshot!(output);
}

#[test]
fn snapshot_create_description_mode_small_height_active() {
    let mut state = SnippetState::new();
    state.enter_create_mode(".test");
    state.next_field();
    state.next_field();
    assert_eq!(*state.mode(), SnippetMode::CreateDescription);

    let results_area = Rect {
        x: 0,
        y: 0,
        width: 80,
        height: 6,
    };
    let output = render_snippet_popup_to_string(&mut state, results_area, 80, 10);
    // The minimal-height create fallback for the Description field.
    assert!(output.contains("New Snippet - Description"));
    assert_snapshot!(output);
}

#[test]
fn snapshot_list_item_hovered() {
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
    ];
    let mut state = create_state_with_snippets(snippets);
    // selected_index stays at 0; hovered differs so the `&& !is_selected`
    // guard takes the hovered styling branch for item 1.
    state.set_hovered(Some(1));
    assert_eq!(state.get_hovered(), Some(1));

    let results_area = Rect {
        x: 0,
        y: 0,
        width: 80,
        height: 20,
    };
    let output = render_snippet_popup_to_string(&mut state, results_area, 80, 24);
    assert!(output.contains("Flatten arrays"));
    assert_snapshot!(output);
}

#[test]
fn snapshot_list_item_description_truncated() {
    let long_desc = "z".repeat(90);
    let snippets = vec![Snippet {
        name: "X".to_string(),
        query: ".".to_string(),
        description: Some(long_desc),
    }];
    let mut state = create_state_with_snippets(snippets);

    let results_area = Rect {
        x: 0,
        y: 0,
        width: 80,
        height: 20,
    };
    let output = render_snippet_popup_to_string(&mut state, results_area, 80, 24);
    // A 90-char description exceeds the available width on an 80-col list,
    // so it is truncated with a trailing ellipsis.
    assert!(
        output.contains('…'),
        "long description should be truncated with ellipsis"
    );
    assert_snapshot!(output);
}
