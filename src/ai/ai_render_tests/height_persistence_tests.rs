//! Height persistence tests for AI popup
//!
//! **Validates: Popup maintains consistent height during loading transitions**

use super::*;
use crate::ai::ai_state::lifecycle::TEST_MAX_CONTEXT_LENGTH;
use crate::ai::ai_state::{Suggestion, SuggestionType};
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::layout::Rect;

/// Create a test terminal and render, returning the popup area height
fn render_and_get_popup_height(ai_state: &mut AiState, width: u16, height: u16) -> Option<u16> {
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).unwrap();
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

    let buffer = terminal.backend().buffer();

    // Find the popup borders to calculate height
    // Only check left border corners since scrollbar may replace right border
    let mut top_border = None;
    let mut bottom_border = None;

    for y in 0..buffer.area.height {
        let mut row_text = String::new();
        for x in 0..buffer.area.width {
            let idx = (y * buffer.area.width + x) as usize;
            if idx < buffer.content.len() {
                row_text.push_str(buffer.content[idx].symbol());
            }
        }

        if row_text.contains("╭") && top_border.is_none() {
            top_border = Some(y);
        }
        if row_text.contains("╰") {
            bottom_border = Some(y);
        }
    }

    if let (Some(top), Some(bottom)) = (top_border, bottom_border) {
        Some(bottom - top + 1)
    } else {
        None
    }
}

#[test]
fn test_height_stored_after_rendering_suggestions() {
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
            query: ".second".to_string(),
            description: "Second".to_string(),
            suggestion_type: SuggestionType::Next,
        },
    ];

    // Before rendering, no previous height
    assert_eq!(state.previous_popup_height, None);

    // Render suggestions
    let _height = render_and_get_popup_height(&mut state, 100, 30);

    // After rendering, previous height should be stored
    assert!(
        state.previous_popup_height.is_some(),
        "previous_popup_height should be stored after rendering suggestions"
    );
}

#[test]
fn test_height_maintained_during_loading() {
    let mut state = AiState::new_with_config(
        true,
        true,
        "Anthropic".to_string(),
        "claude-3-5-sonnet-20241022".to_string(),
        TEST_MAX_CONTEXT_LENGTH,
    );
    state.visible = true;
    state.response = "AI response".to_string();
    state.suggestions = vec![Suggestion {
        query: ".test".to_string(),
        description: "Test".to_string(),
        suggestion_type: SuggestionType::Fix,
    }];

    // Render with suggestions to establish baseline height
    let height_with_suggestions = render_and_get_popup_height(&mut state, 100, 30);
    assert!(height_with_suggestions.is_some());
    let baseline_height = height_with_suggestions.unwrap();

    // Simulate loading state (suggestions cleared, loading=true)
    state.loading = true;
    state.suggestions.clear();
    state.previous_response = Some(state.response.clone());
    state.response.clear();

    // Render during loading - should maintain same height
    let height_during_loading = render_and_get_popup_height(&mut state, 100, 30);
    assert!(height_during_loading.is_some());

    assert_eq!(
        height_during_loading.unwrap(),
        baseline_height,
        "Popup height should remain consistent during loading transition"
    );
}

#[test]
fn test_height_adjusts_with_new_suggestions() {
    let mut state = AiState::new_with_config(
        true,
        true,
        "Anthropic".to_string(),
        "claude-3-5-sonnet-20241022".to_string(),
        TEST_MAX_CONTEXT_LENGTH,
    );
    state.visible = true;
    state.response = "AI response".to_string();

    // Start with 2 short suggestions
    state.suggestions = vec![
        Suggestion {
            query: ".a".to_string(),
            description: "Short".to_string(),
            suggestion_type: SuggestionType::Fix,
        },
        Suggestion {
            query: ".b".to_string(),
            description: "Short".to_string(),
            suggestion_type: SuggestionType::Next,
        },
    ];

    // Use larger terminal to avoid both hitting the max height ceiling
    let height_with_short = render_and_get_popup_height(&mut state, 100, 50);
    let short_height = height_with_short.unwrap();

    // Update with 5 longer suggestions
    state.suggestions = vec![
        Suggestion {
            query: ".query1 with longer text".to_string(),
            description: "Longer description that explains more".to_string(),
            suggestion_type: SuggestionType::Fix,
        },
        Suggestion {
            query: ".query2 with longer text".to_string(),
            description: "Longer description that explains more".to_string(),
            suggestion_type: SuggestionType::Next,
        },
        Suggestion {
            query: ".query3 with longer text".to_string(),
            description: "Longer description that explains more".to_string(),
            suggestion_type: SuggestionType::Optimize,
        },
        Suggestion {
            query: ".query4 with longer text".to_string(),
            description: "Longer description that explains more".to_string(),
            suggestion_type: SuggestionType::Fix,
        },
        Suggestion {
            query: ".query5 with longer text".to_string(),
            description: "Longer description that explains more".to_string(),
            suggestion_type: SuggestionType::Next,
        },
    ];

    let height_with_long = render_and_get_popup_height(&mut state, 100, 50);
    let long_height = height_with_long.unwrap();

    assert!(
        long_height > short_height,
        "Popup should expand when more/longer suggestions are shown"
    );
}

#[test]
fn test_no_height_stored_for_error_state() {
    let mut state = AiState::new_with_config(
        true,
        true,
        "Anthropic".to_string(),
        "claude-3-5-sonnet-20241022".to_string(),
        TEST_MAX_CONTEXT_LENGTH,
    );
    state.visible = true;
    state.error = Some("API Error".to_string());

    // Render error state
    let _height = render_and_get_popup_height(&mut state, 100, 30);

    // previous_popup_height should not be set for error state
    // (it's only set when suggestions are rendered)
    assert_eq!(
        state.previous_popup_height, None,
        "Height should not be stored for error state"
    );
}

#[test]
fn test_height_persistence_across_multiple_loads() {
    let mut state = AiState::new_with_config(
        true,
        true,
        "Anthropic".to_string(),
        "claude-3-5-sonnet-20241022".to_string(),
        TEST_MAX_CONTEXT_LENGTH,
    );
    state.visible = true;
    state.response = "AI response".to_string();
    state.suggestions = vec![Suggestion {
        query: ".test".to_string(),
        description: "Test suggestion".to_string(),
        suggestion_type: SuggestionType::Fix,
    }];

    // First render with suggestions
    let height1 = render_and_get_popup_height(&mut state, 100, 30).unwrap();

    // Simulate loading
    state.loading = true;
    state.suggestions.clear();
    let height2 = render_and_get_popup_height(&mut state, 100, 30).unwrap();

    // Return to suggestions (same content)
    state.loading = false;
    state.suggestions = vec![Suggestion {
        query: ".test".to_string(),
        description: "Test suggestion".to_string(),
        suggestion_type: SuggestionType::Fix,
    }];
    let height3 = render_and_get_popup_height(&mut state, 100, 30).unwrap();

    // All heights should be consistent
    assert_eq!(height1, height2, "Height should persist during loading");
    assert_eq!(
        height2, height3,
        "Height should remain consistent after loading"
    );
}
