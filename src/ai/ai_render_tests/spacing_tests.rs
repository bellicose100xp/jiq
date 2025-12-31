//! Spacing validation tests for AI suggestions
//!
//! **Validates: Consistent spacing between suggestions**

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
fn snapshot_consistent_spacing_two_suggestions() {
    let mut state = AiState::new_with_config(
        true,
        true,
        "Anthropic".to_string(),
        "claude-3-5-sonnet-20241022".to_string(),
        TEST_MAX_CONTEXT_LENGTH,
    );
    state.visible = true;
    state.response = "AI response".to_string();
    state.suggestions = vec![
        Suggestion {
            query: ".first".to_string(),
            description: "First suggestion".to_string(),
            suggestion_type: SuggestionType::Fix,
        },
        Suggestion {
            query: ".second".to_string(),
            description: "Second suggestion".to_string(),
            suggestion_type: SuggestionType::Next,
        },
    ];

    let output = render_ai_popup_to_string(&mut state, 100, 30);
    assert_snapshot!(output);
}

#[test]
fn snapshot_consistent_spacing_three_suggestions() {
    let mut state = AiState::new_with_config(
        true,
        true,
        "Anthropic".to_string(),
        "claude-3-5-sonnet-20241022".to_string(),
        TEST_MAX_CONTEXT_LENGTH,
    );
    state.visible = true;
    state.response = "AI response".to_string();
    state.suggestions = vec![
        Suggestion {
            query: ".first".to_string(),
            description: "First suggestion".to_string(),
            suggestion_type: SuggestionType::Fix,
        },
        Suggestion {
            query: ".second".to_string(),
            description: "Second suggestion".to_string(),
            suggestion_type: SuggestionType::Next,
        },
        Suggestion {
            query: ".third".to_string(),
            description: "Third suggestion".to_string(),
            suggestion_type: SuggestionType::Optimize,
        },
    ];

    let output = render_ai_popup_to_string(&mut state, 100, 30);
    assert_snapshot!(output);
}

#[test]
fn snapshot_consistent_spacing_five_suggestions() {
    let mut state = AiState::new_with_config(
        true,
        true,
        "Anthropic".to_string(),
        "claude-3-5-sonnet-20241022".to_string(),
        TEST_MAX_CONTEXT_LENGTH,
    );
    state.visible = true;
    state.response = "AI response".to_string();
    state.suggestions = vec![
        Suggestion {
            query: ".suggestion1".to_string(),
            description: "Description 1".to_string(),
            suggestion_type: SuggestionType::Fix,
        },
        Suggestion {
            query: ".suggestion2".to_string(),
            description: "Description 2".to_string(),
            suggestion_type: SuggestionType::Next,
        },
        Suggestion {
            query: ".suggestion3".to_string(),
            description: "Description 3".to_string(),
            suggestion_type: SuggestionType::Optimize,
        },
        Suggestion {
            query: ".suggestion4".to_string(),
            description: "Description 4".to_string(),
            suggestion_type: SuggestionType::Fix,
        },
        Suggestion {
            query: ".suggestion5".to_string(),
            description: "Description 5".to_string(),
            suggestion_type: SuggestionType::Next,
        },
    ];

    let output = render_ai_popup_to_string(&mut state, 100, 30);
    assert_snapshot!(output);
}

#[test]
fn snapshot_spacing_with_varying_lengths() {
    let mut state = AiState::new_with_config(
        true,
        true,
        "Anthropic".to_string(),
        "claude-3-5-sonnet-20241022".to_string(),
        TEST_MAX_CONTEXT_LENGTH,
    );
    state.visible = true;
    state.response = "AI response".to_string();
    state.suggestions = vec![
        Suggestion {
            query: ".short".to_string(),
            description: "Short".to_string(),
            suggestion_type: SuggestionType::Fix,
        },
        Suggestion {
            query: ".users[] | select(.active == true and .age > 18)".to_string(),
            description: "This is a much longer description that wraps across multiple lines to test spacing consistency".to_string(),
            suggestion_type: SuggestionType::Next,
        },
        Suggestion {
            query: ".medium".to_string(),
            description: "Medium length description here".to_string(),
            suggestion_type: SuggestionType::Optimize,
        },
    ];

    let output = render_ai_popup_to_string(&mut state, 100, 30);
    assert_snapshot!(output);
}

#[test]
fn snapshot_no_spacing_after_last_suggestion() {
    let mut state = AiState::new_with_config(
        true,
        true,
        "Anthropic".to_string(),
        "claude-3-5-sonnet-20241022".to_string(),
        TEST_MAX_CONTEXT_LENGTH,
    );
    state.visible = true;
    state.response = "AI response".to_string();
    state.suggestions = vec![
        Suggestion {
            query: ".first".to_string(),
            description: "First".to_string(),
            suggestion_type: SuggestionType::Fix,
        },
        Suggestion {
            query: ".last".to_string(),
            description: "Last suggestion should have no spacing after it".to_string(),
            suggestion_type: SuggestionType::Next,
        },
    ];

    let output = render_ai_popup_to_string(&mut state, 100, 30);
    // Verify last suggestion is directly above the bottom border
    assert!(output.contains("[Next] .last"));
    assert_snapshot!(output);
}

#[test]
fn snapshot_spacing_maintained_with_first_option_selected() {
    // Regression test for spacing bug: verify spacing is consistent when first option is selected
    let mut state = AiState::new_with_config(
        true,
        true,
        "Anthropic".to_string(),
        "claude-3-5-sonnet-20241022".to_string(),
        TEST_MAX_CONTEXT_LENGTH,
    );
    state.visible = true;
    state.response = "AI response".to_string();
    state.suggestions = vec![
        Suggestion {
            query: ".first".to_string(),
            description: "First suggestion".to_string(),
            suggestion_type: SuggestionType::Fix,
        },
        Suggestion {
            query: ".second".to_string(),
            description: "Second suggestion".to_string(),
            suggestion_type: SuggestionType::Next,
        },
        Suggestion {
            query: ".third".to_string(),
            description: "Third suggestion".to_string(),
            suggestion_type: SuggestionType::Optimize,
        },
    ];

    // Select first option
    state.selection.select_index(0);

    let output = render_ai_popup_to_string(&mut state, 100, 30);
    assert_snapshot!(output);
}

#[test]
fn snapshot_spacing_maintained_with_middle_option_selected() {
    // Regression test for spacing bug: verify spacing is consistent when middle option is selected
    let mut state = AiState::new_with_config(
        true,
        true,
        "Anthropic".to_string(),
        "claude-3-5-sonnet-20241022".to_string(),
        TEST_MAX_CONTEXT_LENGTH,
    );
    state.visible = true;
    state.response = "AI response".to_string();
    state.suggestions = vec![
        Suggestion {
            query: ".first".to_string(),
            description: "First suggestion".to_string(),
            suggestion_type: SuggestionType::Fix,
        },
        Suggestion {
            query: ".second".to_string(),
            description: "Second suggestion".to_string(),
            suggestion_type: SuggestionType::Next,
        },
        Suggestion {
            query: ".third".to_string(),
            description: "Third suggestion".to_string(),
            suggestion_type: SuggestionType::Optimize,
        },
    ];

    // Select middle option
    state.selection.select_index(1);

    let output = render_ai_popup_to_string(&mut state, 100, 30);
    assert_snapshot!(output);
}

#[test]
fn snapshot_spacing_maintained_with_last_option_selected() {
    // Regression test for spacing bug: verify spacing is consistent when last option is selected
    // This is the critical test case that would fail with the double-spacing bug
    let mut state = AiState::new_with_config(
        true,
        true,
        "Anthropic".to_string(),
        "claude-3-5-sonnet-20241022".to_string(),
        TEST_MAX_CONTEXT_LENGTH,
    );
    state.visible = true;
    state.response = "AI response".to_string();
    state.suggestions = vec![
        Suggestion {
            query: ".first".to_string(),
            description: "First suggestion".to_string(),
            suggestion_type: SuggestionType::Fix,
        },
        Suggestion {
            query: ".second".to_string(),
            description: "Second suggestion".to_string(),
            suggestion_type: SuggestionType::Next,
        },
        Suggestion {
            query: ".third".to_string(),
            description: "Third suggestion".to_string(),
            suggestion_type: SuggestionType::Optimize,
        },
    ];

    // Select last option - this would trigger the spacing bug before the fix
    state.selection.select_index(2);

    let output = render_ai_popup_to_string(&mut state, 100, 30);
    assert_snapshot!(output);
}
