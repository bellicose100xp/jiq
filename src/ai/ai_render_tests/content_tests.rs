//! Header content tests for the AI popup

use super::*;
use crate::ai::ai_state::lifecycle::TEST_MAX_CONTEXT_LENGTH;
use crate::ai::render::text::wrap_text;
use crate::theme;
use proptest::prelude::*;
use ratatui::text::Line;

/// Flatten header lines into one string for substring assertions
fn lines_text(lines: &[Line<'_>]) -> String {
    lines
        .iter()
        .flat_map(|l| l.spans.iter())
        .map(|s| s.content.as_ref())
        .collect()
}

/// Text of every row of a test terminal buffer
fn buffer_rows(buffer: &ratatui::buffer::Buffer) -> Vec<String> {
    (0..buffer.area.height)
        .map(|y| {
            (0..buffer.area.width)
                .map(|x| buffer[(x, y)].symbol())
                .collect::<String>()
        })
        .collect()
}

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
fn test_build_header_empty_state() {
    let state = AiState::new_with_config(
        true,
        true,
        "Anthropic".to_string(),
        "claude-3-5-sonnet-20241022".to_string(),
        TEST_MAX_CONTEXT_LENGTH,
    );
    let lines = build_header_lines(&state, 60);

    // Empty state shows nothing - "Thinking..." appears when loading
    assert!(lines.is_empty());
}

#[test]
fn test_build_header_not_configured() {
    let state = AiState::new_with_config(
        true,
        false,
        "Anthropic".to_string(),
        "claude-3-5-sonnet-20241022".to_string(),
        TEST_MAX_CONTEXT_LENGTH,
    );
    let text = lines_text(&build_header_lines(&state, 60));

    assert!(text.contains("AI provider not configured"));
    assert!(text.contains("[ai]"));
    assert!(text.contains("provider"));
    assert!(text.contains("api_key"));
    assert!(text.contains("https://github.com/bellicose100xp/jiq#configuration"));
}

#[test]
fn test_build_header_loading() {
    let mut state = AiState::new_with_config(
        true,
        true,
        "Anthropic".to_string(),
        "claude-3-5-sonnet-20241022".to_string(),
        TEST_MAX_CONTEXT_LENGTH,
    );
    state.loading = true;

    let text = lines_text(&build_header_lines(&state, 60));
    assert!(text.contains("Thinking"));
}

#[test]
fn test_build_header_error() {
    let mut state = AiState::new_with_config(
        true,
        true,
        "Anthropic".to_string(),
        "claude-3-5-sonnet-20241022".to_string(),
        TEST_MAX_CONTEXT_LENGTH,
    );
    state.error = Some("Network error".to_string());

    let text = lines_text(&build_header_lines(&state, 60));
    assert!(text.contains("Error"));
    assert!(text.contains("Network error"));
}

#[test]
fn test_build_header_response_that_fails_to_parse_shows_friendly_error() {
    // When a response arrives that cannot be parsed into structured
    // suggestions, we must not dump raw/garbled text to the user — we
    // show a clear "could not parse" message instead.
    let mut state = AiState::new_with_config(
        true,
        true,
        "Anthropic".to_string(),
        "claude-3-5-sonnet-20241022".to_string(),
        TEST_MAX_CONTEXT_LENGTH,
    );
    state.response = "Try using .foo instead".to_string();
    state.parse_failed = true;

    let text = lines_text(&build_header_lines(&state, 60));
    assert!(text.contains("Could not parse AI response"));
    assert!(!text.contains("Try using .foo instead"));
}

#[test]
fn test_build_header_no_suggestions_shows_calm_message_not_parse_error() {
    // A valid but empty suggestion list must render as a calm "No suggestions"
    // message — NOT the "Could not parse" error banner.
    let mut state = AiState::new_with_config(
        true,
        true,
        "Bedrock".to_string(),
        "claude-haiku-4-5".to_string(),
        TEST_MAX_CONTEXT_LENGTH,
    );
    state.response = "{\"suggestions\":[]}".to_string();
    state.no_suggestions = true;

    let text = lines_text(&build_header_lines(&state, 60));
    assert!(text.contains("No suggestions"));
    assert!(text.contains("had no suggestions"));
    assert!(!text.contains("Could not parse"));
}

#[test]
fn test_build_header_answer_without_suggestions_has_no_trailing_blank() {
    let mut state = AiState::new_with_config(
        true,
        true,
        "Anthropic".to_string(),
        "claude-3-5-sonnet-20241022".to_string(),
        TEST_MAX_CONTEXT_LENGTH,
    );
    state.answer = Some("Just an answer".to_string());

    let lines = build_header_lines(&state, 60);
    assert_eq!(lines.len(), 1);
    assert_eq!(lines_text(&lines), "Just an answer");
}

#[test]
fn test_build_header_answer_with_suggestions_ends_with_blank_line() {
    use crate::ai::ai_state::{Suggestion, SuggestionType};

    let mut state = AiState::new_with_config(
        true,
        true,
        "Anthropic".to_string(),
        "claude-3-5-sonnet-20241022".to_string(),
        TEST_MAX_CONTEXT_LENGTH,
    );
    state.answer = Some("Answer".to_string());
    state.suggestions = vec![Suggestion {
        query: ".a".to_string(),
        description: String::new(),
        suggestion_type: SuggestionType::Query,
    }];

    let lines = build_header_lines(&state, 60);
    assert_eq!(lines.len(), 2);
    assert_eq!(lines_text(&lines[1..]), "");
}

#[test]
fn test_build_header_suggestions_only_is_empty() {
    use crate::ai::ai_state::{Suggestion, SuggestionType};

    let mut state = AiState::new_with_config(
        true,
        true,
        "Anthropic".to_string(),
        "claude-3-5-sonnet-20241022".to_string(),
        TEST_MAX_CONTEXT_LENGTH,
    );
    state.suggestions = vec![Suggestion {
        query: ".a".to_string(),
        description: String::new(),
        suggestion_type: SuggestionType::Fix,
    }];

    // Suggestions render as widgets below the header, so the header is empty
    assert!(build_header_lines(&state, 60).is_empty());
}

#[test]
fn test_build_header_error_wins_over_loading_and_answer() {
    let mut state = AiState::new_with_config(
        true,
        true,
        "Anthropic".to_string(),
        "claude-3-5-sonnet-20241022".to_string(),
        TEST_MAX_CONTEXT_LENGTH,
    );
    state.loading = true;
    state.answer = Some("stale".to_string());
    state.error = Some("boom".to_string());

    let text = lines_text(&build_header_lines(&state, 60));
    assert!(text.contains("boom"));
    assert!(!text.contains("Thinking"));
    assert!(!text.contains("stale"));
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

        let text = lines_text(&build_header_lines(&state, max_width));

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

        let text = lines_text(&build_header_lines(&state, max_width));

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

/// Render `state` on a terminal tall enough that no suggestion scrolls out
/// of view, and return the buffer rows.
fn render_rows_unclipped(state: &mut AiState, suggestion_count: usize) -> Vec<String> {
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;
    use ratatui::layout::Rect;

    // Each suggestion is at most 3 rows (query, description, spacing); the
    // popup may only take half of the space above the input bar.
    let terminal_height = 30 + (suggestion_count as u16 * 6);
    let backend = TestBackend::new(100, terminal_height);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
        .draw(|f| {
            let input_area = Rect {
                x: 0,
                y: terminal_height - 4,
                width: 100,
                height: 3,
            };
            render_popup(state, f, input_area, false);
        })
        .unwrap();
    buffer_rows(terminal.backend().buffer())
}

/// Number of rows whose popup content starts with a selection number
fn count_numbered_rows(rows: &[String]) -> usize {
    rows.iter()
        .filter(|row| {
            let content = row.trim_start().trim_start_matches('│').trim_start();
            (1..=9).any(|i| content.starts_with(&format!("{}. [", i)))
        })
        .count()
}

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

        let rows = render_rows_unclipped(&mut state, suggestion_count);

        // Verify each suggestion has its number (1-N) followed by its type label
        for i in 1..=suggestion_count {
            let marker = format!("{}. [Fix] .query{}", i, i - 1);
            prop_assert!(
                rows.iter().any(|row| row.contains(&marker)),
                "Suggestion {} should render as '{}'",
                i, marker
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

        let rows = render_rows_unclipped(&mut state, suggestion_count);

        // Every suggestion is on screen, but only five carry a number
        prop_assert!(
            rows.iter().any(|row| row.contains(&format!("[Fix] .query{}", suggestion_count - 1))),
            "Last suggestion should be rendered"
        );
        let numbered = count_numbered_rows(&rows);
        prop_assert_eq!(
            numbered, 5,
            "Should have exactly 5 numbered suggestions, found {}",
            numbered
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
            render_popup(&mut state_mut, f, input_area, false);
        }).unwrap();

        let buffer = terminal.backend().buffer();

        // Check that at least one cell has the selected background (widget-level styling)
        let expected_bg = theme::ai::suggestion_selected_bg();
        let has_background = buffer.content.iter().any(|cell| {
            cell.bg == expected_bg
        });

        prop_assert!(
            has_background,
            "Selected suggestion should have cells with selected background color"
        );
    }
}
