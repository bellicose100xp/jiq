//! Content building tests for AI render module

use super::*;
use crate::ai::render::text::wrap_text;
use proptest::prelude::*;
use ratatui::style::Color;

// =========================================================================
// Unit Tests
// =========================================================================

#[test]
fn test_wrap_text_short() {
    let result = wrap_text("hello world", 50);
    assert_eq!(result, vec!["hello world"]);
}

#[test]
fn test_wrap_text_long() {
    let result = wrap_text("hello world this is a long line", 15);
    assert_eq!(result, vec!["hello world", "this is a long", "line"]);
}

#[test]
fn test_wrap_text_empty() {
    let result = wrap_text("", 50);
    assert_eq!(result, vec![""]);
}

#[test]
fn test_wrap_text_multiline() {
    let result = wrap_text("line one\nline two", 50);
    assert_eq!(result, vec!["line one", "line two"]);
}

#[test]
fn test_build_content_empty_state() {
    let state = AiState::new_with_config(true, true);
    let content = build_content(&state, 60);

    // Empty state shows nothing - "Thinking..." appears when loading
    assert!(content.lines.is_empty());
}

#[test]
fn test_build_content_not_configured() {
    let state = AiState::new_with_config(true, false);
    let content = build_content(&state, 60);
    let text: String = content
        .lines
        .iter()
        .flat_map(|l| l.spans.iter())
        .map(|s| s.content.as_ref())
        .collect();

    assert!(text.contains("Setup Required"));
    assert!(text.contains("[ai]"));
    assert!(text.contains("api_key"));
}

#[test]
fn test_build_content_loading() {
    let mut state = AiState::new_with_config(true, true);
    state.loading = true;

    let content = build_content(&state, 60);
    let text: String = content
        .lines
        .iter()
        .flat_map(|l| l.spans.iter())
        .map(|s| s.content.as_ref())
        .collect();

    assert!(text.contains("Thinking"));
}

#[test]
fn test_build_content_error() {
    let mut state = AiState::new_with_config(true, true);
    state.error = Some("Network error".to_string());

    let content = build_content(&state, 60);
    let text: String = content
        .lines
        .iter()
        .flat_map(|l| l.spans.iter())
        .map(|s| s.content.as_ref())
        .collect();

    assert!(text.contains("Error"));
    assert!(text.contains("Network error"));
}

#[test]
fn test_build_content_response() {
    let mut state = AiState::new_with_config(true, true);
    state.response = "Try using .foo instead".to_string();

    let content = build_content(&state, 60);
    let text: String = content
        .lines
        .iter()
        .flat_map(|l| l.spans.iter())
        .map(|s| s.content.as_ref())
        .collect();

    assert!(text.contains("Try using .foo instead"));
}

#[test]
fn test_build_content_loading_with_previous() {
    let mut state = AiState::new_with_config(true, true);
    state.loading = true;
    state.previous_response = Some("Previous answer".to_string());

    let content = build_content(&state, 60);
    let text: String = content
        .lines
        .iter()
        .flat_map(|l| l.spans.iter())
        .map(|s| s.content.as_ref())
        .collect();

    assert!(text.contains("Previous answer"));
    assert!(text.contains("Thinking"));
}

// =========================================================================
// Phase 3: Selection Property-Based Tests
// =========================================================================

// **Feature: ai-assistant-phase3-actionable-suggestions, Property 11: Selection number rendering**
// *For any* AI popup with N suggestions where N ≤ 5, each suggestion should be rendered
// with its selection number (1 through N) at the start of the line in a distinct color
// (dim white or gray), followed by the suggestion type and query.
// **Validates: Requirements 4.1, 4.2, 4.3**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop_selection_number_rendering(suggestion_count in 1usize..=5) {
        use crate::ai::ai_state::{Suggestion, SuggestionType};

        let mut state = AiState::new_with_config(true, true);
        state.visible = true;
        state.response = "AI response".to_string();

        // Create N suggestions
        state.suggestions = (0..suggestion_count)
            .map(|i| Suggestion {
                query: format!(".query{}", i),
                description: format!("Description {}", i),
                suggestion_type: SuggestionType::Fix,
            })
            .collect();

        let content = build_content(&state, 80);
        let text: String = content
            .lines
            .iter()
            .flat_map(|l| l.spans.iter())
            .map(|s| s.content.as_ref())
            .collect();

        // Verify each suggestion has its number (1-N)
        for i in 1..=suggestion_count {
            prop_assert!(
                text.contains(&format!("{}.", i)),
                "Suggestion {} should have selection number '{}.'",
                i, i
            );
        }
    }
}

// **Feature: ai-assistant-phase3-actionable-suggestions, Property 12: Selection number limit**
// *For any* AI popup with N suggestions where N > 5, only the first 5 suggestions should be
// rendered with selection numbers (1-5), and suggestions 6 through N should be rendered
// without selection numbers.
// **Validates: Requirements 4.4**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop_selection_number_limit(suggestion_count in 6usize..15) {
        use crate::ai::ai_state::{Suggestion, SuggestionType};

        let mut state = AiState::new_with_config(true, true);
        state.visible = true;
        state.response = "AI response".to_string();

        // Create N suggestions (N > 5)
        state.suggestions = (0..suggestion_count)
            .map(|i| Suggestion {
                query: format!(".query{}", i),
                description: format!("Description {}", i),
                suggestion_type: SuggestionType::Fix,
            })
            .collect();

        let content = build_content(&state, 80);

        // Check each line to see if it starts with a number
        let mut numbered_suggestions = 0;
        for line in &content.lines {
            let line_text: String = line.spans.iter()
                .map(|s| s.content.as_ref())
                .collect();

            // Check if line starts with "N. " where N is 1-5
            for i in 1..=5 {
                if line_text.trim_start().starts_with(&format!("{}. ", i)) {
                    numbered_suggestions += 1;
                    break;
                }
            }
        }

        // Should have exactly 5 numbered suggestions
        prop_assert_eq!(
            numbered_suggestions, 5,
            "Should have exactly 5 numbered suggestions, found {}",
            numbered_suggestions
        );
    }
}

// **Feature: ai-assistant-phase3-actionable-suggestions, Property 6: Selection highlight visibility**
// *For any* suggestion selected via Alt+Up/Down navigation, the selected suggestion should be
// rendered with a distinct background color or visual indicator that differs from unselected suggestions.
// **Validates: Requirements 4.5, 8.5**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop_selection_highlight_visibility(
        suggestion_count in 1usize..10,
        selected_index in 0usize..10
    ) {
        use crate::ai::ai_state::{Suggestion, SuggestionType};

        prop_assume!(selected_index < suggestion_count);

        let mut state = AiState::new_with_config(true, true);
        state.visible = true;
        state.response = "AI response".to_string();

        // Create N suggestions
        state.suggestions = (0..suggestion_count)
            .map(|i| Suggestion {
                query: format!(".query{}", i),
                description: format!("Description {}", i),
                suggestion_type: SuggestionType::Fix,
            })
            .collect();

        // Select a suggestion via navigation
        for _ in 0..=selected_index {
            state.selection.navigate_next(suggestion_count);
        }

        let content = build_content(&state, 80);

        // Check that at least one span has a background color (indicating selection)
        let has_background = content.lines.iter().any(|line| {
            line.spans.iter().any(|span| {
                span.style.bg.is_some() && span.style.bg != Some(Color::Black)
            })
        });

        prop_assert!(
            has_background,
            "Selected suggestion should have a distinct background color"
        );
    }
}
