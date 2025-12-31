//! Content building tests for AI render module

use super::*;
use crate::ai::ai_state::lifecycle::TEST_MAX_CONTEXT_LENGTH;
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
    let state = AiState::new_with_config(
        true,
        true,
        "Anthropic".to_string(),
        "claude-3-5-sonnet-20241022".to_string(),
        TEST_MAX_CONTEXT_LENGTH,
    );
    let content = build_content(&state, 60);

    // Empty state shows nothing - "Thinking..." appears when loading
    assert!(content.lines.is_empty());
}

#[test]
fn test_build_content_not_configured() {
    let state = AiState::new_with_config(
        true,
        false,
        "Anthropic".to_string(),
        "claude-3-5-sonnet-20241022".to_string(),
        TEST_MAX_CONTEXT_LENGTH,
    );
    let content = build_content(&state, 60);
    let text: String = content
        .lines
        .iter()
        .flat_map(|l| l.spans.iter())
        .map(|s| s.content.as_ref())
        .collect();

    assert!(text.contains("AI provider not configured"));
    assert!(text.contains("[ai]"));
    assert!(text.contains("provider"));
    assert!(text.contains("api_key"));
    assert!(text.contains("https://github.com/bellicose100xp/jiq#configuration"));
}

#[test]
fn test_build_content_loading() {
    let mut state = AiState::new_with_config(
        true,
        true,
        "Anthropic".to_string(),
        "claude-3-5-sonnet-20241022".to_string(),
        TEST_MAX_CONTEXT_LENGTH,
    );
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
    let mut state = AiState::new_with_config(
        true,
        true,
        "Anthropic".to_string(),
        "claude-3-5-sonnet-20241022".to_string(),
        TEST_MAX_CONTEXT_LENGTH,
    );
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
    let mut state = AiState::new_with_config(
        true,
        true,
        "Anthropic".to_string(),
        "claude-3-5-sonnet-20241022".to_string(),
        TEST_MAX_CONTEXT_LENGTH,
    );
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
    let mut state = AiState::new_with_config(
        true,
        true,
        "Anthropic".to_string(),
        "claude-3-5-sonnet-20241022".to_string(),
        TEST_MAX_CONTEXT_LENGTH,
    );
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
// No Default AI Provider Property-Based Tests
// =========================================================================

// **Feature: no-default-ai-provider, Property 2: Unconfigured state shows setup message**
// *For any* AiState where configured is false due to missing provider, the rendered content
// SHALL contain setup instructions including "provider" configuration guidance
// **Validates: Requirements 1.1, 3.1, 3.3**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop_unconfigured_state_shows_setup_message(
        enabled in proptest::bool::ANY,
        provider_name in "[a-zA-Z]{3,15}",
        model_name in "[a-zA-Z0-9-]{5,30}",
        max_width in 40u16..200u16
    ) {
        // Create an unconfigured state (configured = false)
        let state = AiState::new_with_config(
            enabled,
            false,  // configured = false
            provider_name,
            model_name,
            TEST_MAX_CONTEXT_LENGTH,
        );

        let content = build_content(&state, max_width);
        let text: String = content
            .lines
            .iter()
            .flat_map(|l| l.spans.iter())
            .map(|s| s.content.as_ref())
            .collect();

        // Verify setup instructions are present
        prop_assert!(
            text.contains("AI provider not configured"),
            "Content should contain 'AI provider not configured' message"
        );

        // Verify provider configuration guidance is present
        prop_assert!(
            text.contains("provider"),
            "Content should contain 'provider' configuration guidance"
        );

        // Verify example config shows provider selection
        prop_assert!(
            text.contains("anthropic") || text.contains("openai") || text.contains("gemini") || text.contains("bedrock"),
            "Content should show provider selection options"
        );
    }
}

// **Feature: no-default-ai-provider, Property 3: Unconfigured state includes README URL**
// *For any* AiState where configured is false due to missing provider, the rendered content
// SHALL contain the URL "https://github.com/bellicose100xp/jiq#configuration"
// **Validates: Requirements 1.2, 3.2**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop_unconfigured_state_includes_readme_url(
        enabled in proptest::bool::ANY,
        provider_name in "[a-zA-Z]{3,15}",
        model_name in "[a-zA-Z0-9-]{5,30}",
        max_width in 40u16..200u16
    ) {
        // Create an unconfigured state (configured = false)
        let state = AiState::new_with_config(
            enabled,
            false,  // configured = false
            provider_name,
            model_name,
            TEST_MAX_CONTEXT_LENGTH,
        );

        let content = build_content(&state, max_width);
        let text: String = content
            .lines
            .iter()
            .flat_map(|l| l.spans.iter())
            .map(|s| s.content.as_ref())
            .collect();

        // Verify README URL is present
        prop_assert!(
            text.contains("https://github.com/bellicose100xp/jiq#configuration"),
            "Content should contain the README configuration URL"
        );
    }
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

        let mut state = AiState::new_with_config(true, true, "Anthropic".to_string(), "claude-3-5-sonnet-20241022".to_string(), TEST_MAX_CONTEXT_LENGTH);
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

        let mut state = AiState::new_with_config(true, true, "Anthropic".to_string(), "claude-3-5-sonnet-20241022".to_string(), TEST_MAX_CONTEXT_LENGTH);
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
        use ratatui::Terminal;
        use ratatui::backend::TestBackend;
        use ratatui::layout::Rect;

        prop_assume!(selected_index < suggestion_count);

        let mut state = AiState::new_with_config(true, true, "Anthropic".to_string(), "claude-3-5-sonnet-20241022".to_string(), TEST_MAX_CONTEXT_LENGTH);
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

        // Render to TestBackend to check widget-level background styling
        // Use taller terminal to ensure space for all suggestions (each needs ~2 lines + 1 spacing)
        let terminal_height = 30 + (suggestion_count as u16 * 3);
        let backend = TestBackend::new(100, terminal_height);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut state_mut = state; // Move state to make it mutable
        terminal.draw(|f| {
            let input_area = Rect {
                x: 0,
                y: terminal_height - 4,
                width: 100,
                height: 3,
            };
            render_popup(&mut state_mut, f, input_area);
        }).unwrap();

        let buffer = terminal.backend().buffer();

        // Check that at least one cell has a DarkGray background (widget-level styling)
        let has_background = buffer.content.iter().any(|cell| {
            cell.bg == Color::DarkGray
        });

        prop_assert!(
            has_background,
            "Selected suggestion should have cells with DarkGray background color"
        );
    }
}
