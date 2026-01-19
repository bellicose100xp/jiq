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
    state: &SnippetState,
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
    let state = SnippetState::new();
    let results_area = Rect {
        x: 0,
        y: 0,
        width: 80,
        height: 20,
    };
    let output = render_snippet_popup_to_string(&state, results_area, 80, 24);
    assert_snapshot!(output);
}

#[test]
fn snapshot_snippet_popup_narrow_terminal() {
    let state = SnippetState::new();
    let results_area = Rect {
        x: 0,
        y: 0,
        width: 40,
        height: 15,
    };
    let output = render_snippet_popup_to_string(&state, results_area, 40, 20);
    assert_snapshot!(output);
}

#[test]
fn snapshot_snippet_popup_small_height() {
    let state = SnippetState::new();
    let results_area = Rect {
        x: 0,
        y: 0,
        width: 80,
        height: 6,
    };
    let output = render_snippet_popup_to_string(&state, results_area, 80, 10);
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
    let state = create_state_with_snippets(snippets);
    let results_area = Rect {
        x: 0,
        y: 0,
        width: 80,
        height: 20,
    };
    let output = render_snippet_popup_to_string(&state, results_area, 80, 24);
    assert_snapshot!(output);
}

#[test]
fn snapshot_snippet_popup_with_single_snippet() {
    let snippets = vec![Snippet {
        name: "Identity".to_string(),
        query: ".".to_string(),
        description: None,
    }];
    let state = create_state_with_snippets(snippets);
    let results_area = Rect {
        x: 0,
        y: 0,
        width: 80,
        height: 20,
    };
    let output = render_snippet_popup_to_string(&state, results_area, 80, 24);
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
    let state = create_state_with_snippets(snippets);
    let results_area = Rect {
        x: 0,
        y: 0,
        width: 40,
        height: 15,
    };
    let output = render_snippet_popup_to_string(&state, results_area, 40, 20);
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
    let output = render_snippet_popup_to_string(&state, results_area, 80, 24);
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
    let output = render_snippet_popup_to_string(&state, results_area, 80, 24);
    assert_snapshot!(output);
}
