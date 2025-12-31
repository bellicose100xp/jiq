//! Widget-based selection rendering tests
//!
//! **Validates: Full-width background highlighting for selected suggestions**

use super::*;
use crate::ai::ai_state::lifecycle::TEST_MAX_CONTEXT_LENGTH;
use crate::ai::ai_state::{Suggestion, SuggestionType};
use insta::assert_snapshot;
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::layout::Rect;

/// Create a test terminal with specified dimensions
fn create_test_terminal(width: u16, height: u16) -> Terminal<TestBackend> {
    let backend = TestBackend::new(width, height);
    Terminal::new(backend).unwrap()
}

/// Render AI popup to a test terminal and return the buffer as a string
fn render_ai_popup_to_string(ai_state: &mut AiState, width: u16, height: u16) -> String {
    let mut terminal = create_test_terminal(width, height);
    terminal
        .draw(|f| {
            let input_area = Rect {
                x: 0,
                y: height - 4,
                width,
                height: 3,
            };
            render_popup(ai_state, f, input_area);
        })
        .unwrap();
    terminal.backend().to_string()
}

#[test]
fn snapshot_first_suggestion_selected() {
    let mut state = AiState::new_with_config(
        true,
        true,
        "Anthropic".to_string(),
        "claude-3-5-sonnet-20241022".to_string(),
        TEST_MAX_CONTEXT_LENGTH,
    );
    state.visible = true;
    state.response = "AI response with suggestions".to_string();
    state.suggestions = vec![
        Suggestion {
            query: ".users[] | select(.active)".to_string(),
            description: "Filters to only active users".to_string(),
            suggestion_type: SuggestionType::Fix,
        },
        Suggestion {
            query: ".items[] | .price".to_string(),
            description: "Extract prices".to_string(),
            suggestion_type: SuggestionType::Next,
        },
        Suggestion {
            query: ".data | length".to_string(),
            description: "Count items".to_string(),
            suggestion_type: SuggestionType::Optimize,
        },
    ];

    // Select first suggestion (index 0)
    state.selection.navigate_next(state.suggestions.len());

    let output = render_ai_popup_to_string(&mut state, 100, 30);
    assert_snapshot!(output);
}

#[test]
fn snapshot_middle_suggestion_selected() {
    let mut state = AiState::new_with_config(
        true,
        true,
        "Anthropic".to_string(),
        "claude-3-5-sonnet-20241022".to_string(),
        TEST_MAX_CONTEXT_LENGTH,
    );
    state.visible = true;
    state.response = "AI response with suggestions".to_string();
    state.suggestions = vec![
        Suggestion {
            query: ".users[] | select(.active)".to_string(),
            description: "Filters to only active users".to_string(),
            suggestion_type: SuggestionType::Fix,
        },
        Suggestion {
            query: ".items[] | .price".to_string(),
            description: "Extract prices".to_string(),
            suggestion_type: SuggestionType::Next,
        },
        Suggestion {
            query: ".data | length".to_string(),
            description: "Count items".to_string(),
            suggestion_type: SuggestionType::Optimize,
        },
    ];

    // Select middle suggestion (index 1)
    state.selection.navigate_next(state.suggestions.len());
    state.selection.navigate_next(state.suggestions.len());

    let output = render_ai_popup_to_string(&mut state, 100, 30);
    assert_snapshot!(output);
}

#[test]
fn snapshot_last_suggestion_selected() {
    let mut state = AiState::new_with_config(
        true,
        true,
        "Anthropic".to_string(),
        "claude-3-5-sonnet-20241022".to_string(),
        TEST_MAX_CONTEXT_LENGTH,
    );
    state.visible = true;
    state.response = "AI response with suggestions".to_string();
    state.suggestions = vec![
        Suggestion {
            query: ".users[] | select(.active)".to_string(),
            description: "Filters to only active users".to_string(),
            suggestion_type: SuggestionType::Fix,
        },
        Suggestion {
            query: ".items[] | .price".to_string(),
            description: "Extract prices".to_string(),
            suggestion_type: SuggestionType::Next,
        },
        Suggestion {
            query: ".data | length".to_string(),
            description: "Count items".to_string(),
            suggestion_type: SuggestionType::Optimize,
        },
    ];

    // Select last suggestion (index 2)
    state.selection.navigate_next(state.suggestions.len());
    state.selection.navigate_next(state.suggestions.len());
    state.selection.navigate_next(state.suggestions.len());

    let output = render_ai_popup_to_string(&mut state, 100, 30);
    assert_snapshot!(output);
}

#[test]
fn snapshot_selected_with_wrapped_query() {
    let mut state = AiState::new_with_config(
        true,
        true,
        "Anthropic".to_string(),
        "claude-3-5-sonnet-20241022".to_string(),
        TEST_MAX_CONTEXT_LENGTH,
    );
    state.visible = true;
    state.response = "AI response with suggestions".to_string();
    state.suggestions = vec![
        Suggestion {
            query: ".users[] | select(.active == true and .age > 18 and .verified == true) | {name: .name, email: .email}".to_string(),
            description: "Filters active verified adult users and extracts contact info".to_string(),
            suggestion_type: SuggestionType::Fix,
        },
        Suggestion {
            query: ".items[] | .price".to_string(),
            description: "Extract prices".to_string(),
            suggestion_type: SuggestionType::Next,
        },
    ];

    // Select first suggestion with long query
    state.selection.navigate_next(state.suggestions.len());

    // Use narrower width to force wrapping
    let output = render_ai_popup_to_string(&mut state, 70, 30);
    assert_snapshot!(output);
}

#[test]
fn snapshot_selected_with_long_description() {
    let mut state = AiState::new_with_config(
        true,
        true,
        "Anthropic".to_string(),
        "claude-3-5-sonnet-20241022".to_string(),
        TEST_MAX_CONTEXT_LENGTH,
    );
    state.visible = true;
    state.response = "AI response with suggestions".to_string();
    state.suggestions = vec![
        Suggestion {
            query: ".users[] | select(.active)".to_string(),
            description: "This is a very long description that explains in great detail what this query does and why it's useful for filtering active users from the dataset.".to_string(),
            suggestion_type: SuggestionType::Fix,
        },
        Suggestion {
            query: ".items[] | .price".to_string(),
            description: "Extract prices".to_string(),
            suggestion_type: SuggestionType::Next,
        },
    ];

    // Select first suggestion with long description
    state.selection.navigate_next(state.suggestions.len());

    let output = render_ai_popup_to_string(&mut state, 80, 30);
    assert_snapshot!(output);
}

#[test]
fn snapshot_single_suggestion_selected() {
    let mut state = AiState::new_with_config(
        true,
        true,
        "Anthropic".to_string(),
        "claude-3-5-sonnet-20241022".to_string(),
        TEST_MAX_CONTEXT_LENGTH,
    );
    state.visible = true;
    state.response = "AI response with suggestions".to_string();
    state.suggestions = vec![Suggestion {
        query: ".users[] | select(.active)".to_string(),
        description: "Filters to only active users".to_string(),
        suggestion_type: SuggestionType::Fix,
    }];

    // Select the only suggestion
    state.selection.navigate_next(state.suggestions.len());

    let output = render_ai_popup_to_string(&mut state, 100, 30);
    assert_snapshot!(output);
}

#[test]
fn snapshot_selection_cycling() {
    let mut state = AiState::new_with_config(
        true,
        true,
        "Anthropic".to_string(),
        "claude-3-5-sonnet-20241022".to_string(),
        TEST_MAX_CONTEXT_LENGTH,
    );
    state.visible = true;
    state.response = "AI response with suggestions".to_string();
    state.suggestions = vec![
        Suggestion {
            query: ".first".to_string(),
            description: "First".to_string(),
            suggestion_type: SuggestionType::Fix,
        },
        Suggestion {
            query: ".second".to_string(),
            description: "Second".to_string(),
            suggestion_type: SuggestionType::Next,
        },
    ];

    // Navigate past the end - should wrap to first
    state.selection.navigate_next(state.suggestions.len());
    state.selection.navigate_next(state.suggestions.len());
    state.selection.navigate_next(state.suggestions.len()); // Should wrap to index 0

    let output = render_ai_popup_to_string(&mut state, 100, 30);
    assert_snapshot!(output);
}
