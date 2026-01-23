//! Snapshot tests for AI render module
//!
//! **Validates: Requirements 2.3, 4.2, 6.1**

use super::*;
use crate::ai::ai_state::lifecycle::TEST_MAX_CONTEXT_LENGTH;
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
            // Input area is at the bottom (3 lines high, like in the real app)
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
fn snapshot_ai_popup_empty_state() {
    let mut state = AiState::new_with_config(
        true,
        true,
        "Anthropic".to_string(),
        "claude-3-5-sonnet-20241022".to_string(),
        TEST_MAX_CONTEXT_LENGTH,
    );
    state.visible = true;

    let output = render_ai_popup_to_string(&mut state, 100, 30);
    assert_snapshot!(output);
}

#[test]
fn snapshot_ai_popup_loading_state() {
    let mut state = AiState::new_with_config(
        true,
        true,
        "Anthropic".to_string(),
        "claude-3-5-sonnet-20241022".to_string(),
        TEST_MAX_CONTEXT_LENGTH,
    );
    state.visible = true;
    state.loading = true;

    let output = render_ai_popup_to_string(&mut state, 100, 30);
    assert_snapshot!(output);
}

#[test]
fn snapshot_ai_popup_error_state() {
    let mut state = AiState::new_with_config(
        true,
        true,
        "Anthropic".to_string(),
        "claude-3-5-sonnet-20241022".to_string(),
        TEST_MAX_CONTEXT_LENGTH,
    );
    state.visible = true;
    state.error = Some("API Error: Rate limit exceeded. Please try again later.".to_string());

    let output = render_ai_popup_to_string(&mut state, 100, 30);
    assert_snapshot!(output);
}

#[test]
fn snapshot_ai_popup_response_state() {
    let mut state = AiState::new_with_config(
        true,
        true,
        "Anthropic".to_string(),
        "claude-3-5-sonnet-20241022".to_string(),
        TEST_MAX_CONTEXT_LENGTH,
    );
    state.visible = true;
    state.response = "The error in your query `.foo[` is a missing closing bracket.\n\nTry using `.foo[]` to iterate over the array, or `.foo[0]` to access the first element.".to_string();

    let output = render_ai_popup_to_string(&mut state, 100, 30);
    assert_snapshot!(output);
}

#[test]
fn snapshot_ai_popup_loading_with_previous() {
    let mut state = AiState::new_with_config(
        true,
        true,
        "Anthropic".to_string(),
        "claude-3-5-sonnet-20241022".to_string(),
        TEST_MAX_CONTEXT_LENGTH,
    );
    state.visible = true;
    state.loading = true;
    state.previous_response = Some("Previous suggestion: Use .foo instead of .bar".to_string());

    let output = render_ai_popup_to_string(&mut state, 100, 30);
    assert_snapshot!(output);
}

#[test]
fn snapshot_ai_popup_not_visible() {
    let mut state = AiState::new_with_config(
        true,
        true,
        "Anthropic".to_string(),
        "claude-3-5-sonnet-20241022".to_string(),
        TEST_MAX_CONTEXT_LENGTH,
    );
    // Phase 2: Explicitly set visible to false to test hidden state
    state.visible = false;

    let output = render_ai_popup_to_string(&mut state, 100, 30);
    assert_snapshot!(output);
}

#[test]
fn snapshot_ai_popup_not_configured() {
    let mut state = AiState::new_with_config(
        true,
        false,
        "Anthropic".to_string(),
        "claude-3-5-sonnet-20241022".to_string(),
        TEST_MAX_CONTEXT_LENGTH,
    );
    state.visible = true;

    let output = render_ai_popup_to_string(&mut state, 100, 30);
    assert_snapshot!(output);
}

// =========================================================================
// Phase 2: Suggestion Display Snapshot Tests
// =========================================================================

#[test]
fn snapshot_ai_popup_with_suggestions() {
    use crate::ai::ai_state::{Suggestion, SuggestionType};

    let mut state = AiState::new_with_config(
        true,
        true,
        "Anthropic".to_string(),
        "claude-3-5-sonnet-20241022".to_string(),
        TEST_MAX_CONTEXT_LENGTH,
    );
    state.visible = true;
    state.response =
        "1. [Fix] .users[] | select(.active)\n   Filters to only active users".to_string();
    state.suggestions = vec![
        Suggestion {
            query: ".users[] | select(.active)".to_string(),
            description: "Filters to only active users".to_string(),
            suggestion_type: SuggestionType::Fix,
        },
        Suggestion {
            query: ".users[] | .email".to_string(),
            description: "Extracts email addresses from users".to_string(),
            suggestion_type: SuggestionType::Next,
        },
        Suggestion {
            query: ".users | map(.name)".to_string(),
            description: "More efficient mapping".to_string(),
            suggestion_type: SuggestionType::Optimize,
        },
    ];

    let output = render_ai_popup_to_string(&mut state, 100, 30);
    assert_snapshot!(output);
}

#[test]
fn snapshot_ai_popup_raw_response_fallback() {
    let mut state = AiState::new_with_config(
        true,
        true,
        "Anthropic".to_string(),
        "claude-3-5-sonnet-20241022".to_string(),
        TEST_MAX_CONTEXT_LENGTH,
    );
    state.visible = true;
    // Response without parseable suggestions - should fall back to raw display
    state.response = "This is a plain text response without structured suggestions.\n\nIt should be displayed as-is.".to_string();
    state.suggestions = vec![]; // No parsed suggestions

    let output = render_ai_popup_to_string(&mut state, 100, 30);
    assert_snapshot!(output);
}

// =========================================================================
// Phase 2.2: Query Wrapping Snapshot Test
// =========================================================================

#[test]
fn snapshot_ai_popup_long_query_wrapping() {
    use crate::ai::ai_state::{Suggestion, SuggestionType};

    let mut state = AiState::new_with_config(
        true,
        true,
        "Anthropic".to_string(),
        "claude-3-5-sonnet-20241022".to_string(),
        TEST_MAX_CONTEXT_LENGTH,
    );
    state.visible = true;
    // Set response to non-empty so suggestions are displayed
    state.response = "AI response with suggestions".to_string();
    // Create a suggestion with a very long query that will wrap
    state.suggestions = vec![
        Suggestion {
            query: ".users[] | select(.active == true and .age > 18) | {name: .name, email: .email, address: .address}".to_string(),
            description: "Filters active adult users and extracts their contact information".to_string(),
            suggestion_type: SuggestionType::Fix,
        },
        Suggestion {
            query: ".items | map(select(.price < 100)) | sort_by(.name) | .[0:10]".to_string(),
            description: "Gets first 10 items under $100 sorted by name".to_string(),
            suggestion_type: SuggestionType::Next,
        },
    ];

    // Use a narrower width to force wrapping
    let output = render_ai_popup_to_string(&mut state, 80, 30);
    assert_snapshot!(output);
}

// =========================================================================
// Phase 3: Selection Rendering Snapshot Tests
// =========================================================================

#[test]
fn snapshot_ai_popup_with_selection_numbers() {
    use crate::ai::ai_state::{Suggestion, SuggestionType};

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
            query: ".users[] | .email".to_string(),
            description: "Extracts email addresses".to_string(),
            suggestion_type: SuggestionType::Next,
        },
        Suggestion {
            query: ".users | map(.name)".to_string(),
            description: "More efficient mapping".to_string(),
            suggestion_type: SuggestionType::Optimize,
        },
    ];

    let output = render_ai_popup_to_string(&mut state, 100, 30);
    assert_snapshot!(output);
}

#[test]
fn snapshot_ai_popup_with_selected_suggestion() {
    use crate::ai::ai_state::{Suggestion, SuggestionType};

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
            query: ".users[] | .email".to_string(),
            description: "Extracts email addresses".to_string(),
            suggestion_type: SuggestionType::Next,
        },
        Suggestion {
            query: ".users | map(.name)".to_string(),
            description: "More efficient mapping".to_string(),
            suggestion_type: SuggestionType::Optimize,
        },
    ];

    // Select the second suggestion (index 1)
    state.selection.navigate_next(state.suggestions.len());
    state.selection.navigate_next(state.suggestions.len());

    let output = render_ai_popup_to_string(&mut state, 100, 30);
    assert_snapshot!(output);
}

#[test]
fn snapshot_ai_popup_with_selection_hints() {
    use crate::ai::ai_state::{Suggestion, SuggestionType};

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

    let output = render_ai_popup_to_string(&mut state, 100, 30);
    assert_snapshot!(output);
}

#[test]
fn snapshot_ai_popup_more_than_five_suggestions() {
    use crate::ai::ai_state::{Suggestion, SuggestionType};

    let mut state = AiState::new_with_config(
        true,
        true,
        "Anthropic".to_string(),
        "claude-3-5-sonnet-20241022".to_string(),
        TEST_MAX_CONTEXT_LENGTH,
    );
    state.visible = true;
    state.response = "AI response with many suggestions".to_string();
    state.suggestions = vec![
        Suggestion {
            query: ".users[0]".to_string(),
            description: "First user".to_string(),
            suggestion_type: SuggestionType::Fix,
        },
        Suggestion {
            query: ".users[1]".to_string(),
            description: "Second user".to_string(),
            suggestion_type: SuggestionType::Next,
        },
        Suggestion {
            query: ".users[2]".to_string(),
            description: "Third user".to_string(),
            suggestion_type: SuggestionType::Optimize,
        },
        Suggestion {
            query: ".users[3]".to_string(),
            description: "Fourth user".to_string(),
            suggestion_type: SuggestionType::Fix,
        },
        Suggestion {
            query: ".users[4]".to_string(),
            description: "Fifth user".to_string(),
            suggestion_type: SuggestionType::Next,
        },
        Suggestion {
            query: ".users[5]".to_string(),
            description: "Sixth user (no number)".to_string(),
            suggestion_type: SuggestionType::Optimize,
        },
        Suggestion {
            query: ".users[6]".to_string(),
            description: "Seventh user (no number)".to_string(),
            suggestion_type: SuggestionType::Fix,
        },
    ];

    let output = render_ai_popup_to_string(&mut state, 100, 30);
    assert_snapshot!(output);
}

// =========================================================================
// Scrollbar Position Tests - verify scrollbar reaches correct positions
// =========================================================================

fn create_ai_state_with_many_suggestions() -> AiState {
    use crate::ai::ai_state::{Suggestion, SuggestionType};

    let mut state = AiState::new_with_config(
        true,
        true,
        "Anthropic".to_string(),
        "claude-3-5-sonnet-20241022".to_string(),
        TEST_MAX_CONTEXT_LENGTH,
    );
    state.visible = true;
    state.response = "AI response with many suggestions".to_string();
    state.suggestions = vec![
        Suggestion {
            query: ".users[0]".to_string(),
            description: "First user".to_string(),
            suggestion_type: SuggestionType::Fix,
        },
        Suggestion {
            query: ".users[1]".to_string(),
            description: "Second user".to_string(),
            suggestion_type: SuggestionType::Next,
        },
        Suggestion {
            query: ".users[2]".to_string(),
            description: "Third user".to_string(),
            suggestion_type: SuggestionType::Optimize,
        },
        Suggestion {
            query: ".users[3]".to_string(),
            description: "Fourth user".to_string(),
            suggestion_type: SuggestionType::Fix,
        },
        Suggestion {
            query: ".users[4]".to_string(),
            description: "Fifth user".to_string(),
            suggestion_type: SuggestionType::Next,
        },
        Suggestion {
            query: ".users[5]".to_string(),
            description: "Sixth user".to_string(),
            suggestion_type: SuggestionType::Optimize,
        },
        Suggestion {
            query: ".users[6]".to_string(),
            description: "Seventh user".to_string(),
            suggestion_type: SuggestionType::Fix,
        },
    ];
    state
}

#[test]
fn snapshot_ai_scrollbar_at_top() {
    let mut state = create_ai_state_with_many_suggestions();
    // Default scroll position is at the top
    let output = render_ai_popup_to_string(&mut state, 100, 30);
    assert_snapshot!(output);
}

#[test]
fn snapshot_ai_scrollbar_at_middle() {
    let mut state = create_ai_state_with_many_suggestions();
    // Navigate to the middle suggestion
    for _ in 0..3 {
        state.selection.navigate_next(state.suggestions.len());
    }
    let output = render_ai_popup_to_string(&mut state, 100, 30);
    assert_snapshot!(output);
}

#[test]
fn snapshot_ai_scrollbar_at_bottom() {
    let mut state = create_ai_state_with_many_suggestions();
    // Navigate to the last suggestion to scroll to the bottom
    // Selection starts at None, first navigate_next goes to index 0,
    // so we need total_items navigations to reach the last item
    let total = state.suggestions.len();
    for _ in 0..total {
        state.selection.navigate_next(total);
    }
    let output = render_ai_popup_to_string(&mut state, 100, 30);
    assert_snapshot!(output);
}

// Test for provider name display
#[test]
fn snapshot_ai_popup_bedrock_provider() {
    let mut state = AiState::new_with_config(
        true,
        true,
        "Bedrock".to_string(),
        "anthropic.claude-3-5-sonnet-20241022-v2:0".to_string(),
        TEST_MAX_CONTEXT_LENGTH,
    );
    state.visible = true;
    state.response = "This is a response from Bedrock provider".to_string();

    let output = render_ai_popup_to_string(&mut state, 100, 30);
    assert_snapshot!(output);
}

#[test]
fn snapshot_ai_popup_openai_provider() {
    let mut state = AiState::new_with_config(
        true,
        true,
        "OpenAI".to_string(),
        "gpt-4o-mini".to_string(),
        TEST_MAX_CONTEXT_LENGTH,
    );
    state.visible = true;
    state.response = "This is a response from OpenAI provider".to_string();

    let output = render_ai_popup_to_string(&mut state, 100, 30);
    assert_snapshot!(output);
}

#[test]
fn snapshot_ai_popup_openai_compatible_provider() {
    let mut state = AiState::new_with_config(
        true,
        true,
        "OpenAI-compatible".to_string(),
        "grok-beta".to_string(),
        TEST_MAX_CONTEXT_LENGTH,
    );
    state.visible = true;
    state.response = "This is a response from OpenAI-compatible provider".to_string();

    let output = render_ai_popup_to_string(&mut state, 100, 30);
    assert_snapshot!(output);
}
