//! Unit tests for widget background styling
//!
//! **Validates: Widget-level background application for selected suggestions**

use super::*;
use crate::ai::ai_state::lifecycle::TEST_MAX_CONTEXT_LENGTH;
use crate::ai::ai_state::{Suggestion, SuggestionType};
use crate::theme;
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::layout::Rect;

/// Create a test terminal and render, returning the backend for inspection
fn render_and_get_backend(ai_state: &mut AiState, width: u16, height: u16) -> TestBackend {
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
    terminal.backend().clone()
}

#[test]
fn test_selected_suggestion_has_background() {
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
            description: "First suggestion".to_string(),
            suggestion_type: SuggestionType::Fix,
        },
        Suggestion {
            query: ".second".to_string(),
            description: "Second suggestion".to_string(),
            suggestion_type: SuggestionType::Next,
        },
    ];

    // Select second suggestion
    state.selection.navigate_next(state.suggestions.len());
    state.selection.navigate_next(state.suggestions.len());

    let backend = render_and_get_backend(&mut state, 100, 30);
    let buffer = backend.buffer();

    // Find the row where the second suggestion starts (contains "2. [Next]")
    let mut found_second_suggestion_row = None;
    for y in 0..buffer.area.height {
        let mut row_text = String::new();
        for x in 0..buffer.area.width {
            let idx = (y * buffer.area.width + x) as usize;
            if idx < buffer.content.len() {
                row_text.push_str(buffer.content[idx].symbol());
            }
        }
        if row_text.contains("2.") && row_text.contains("[Next]") {
            found_second_suggestion_row = Some(y);
            break;
        }
    }

    assert!(
        found_second_suggestion_row.is_some(),
        "Should find the second suggestion in the rendered output"
    );

    // Check that cells in the selected suggestion's row have the theme background
    let row = found_second_suggestion_row.unwrap();
    let expected_bg = theme::ai::SUGGESTION_SELECTED_BG;
    let mut cells_with_selected_bg = 0;
    let mut total_non_empty_cells = 0;

    for x in 0..buffer.area.width {
        let idx = (row * buffer.area.width + x) as usize;
        if idx < buffer.content.len() {
            let cell = &buffer.content[idx];
            let symbol = cell.symbol();

            // Count non-empty cells (inside the popup box)
            if !symbol.trim().is_empty() || symbol == " " {
                // Check if we're inside the popup area (not in the outer padding)
                let is_inside_popup = symbol != " " || x > 50; // Rough heuristic
                if is_inside_popup {
                    total_non_empty_cells += 1;
                    if cell.bg == expected_bg {
                        cells_with_selected_bg += 1;
                    }
                }
            }
        }
    }

    // Verify that at least some cells have the selected background
    // (We expect the entire row width within the popup to have it)
    assert!(
        cells_with_selected_bg > 0,
        "Selected suggestion should have cells with selected background. Found {} cells with selected bg out of {} total cells",
        cells_with_selected_bg,
        total_non_empty_cells
    );
}

#[test]
fn test_unselected_suggestion_no_background() {
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
            description: "First suggestion".to_string(),
            suggestion_type: SuggestionType::Fix,
        },
        Suggestion {
            query: ".second".to_string(),
            description: "Second suggestion".to_string(),
            suggestion_type: SuggestionType::Next,
        },
    ];

    // Select second suggestion (so first is unselected)
    state.selection.navigate_next(state.suggestions.len());
    state.selection.navigate_next(state.suggestions.len());

    let backend = render_and_get_backend(&mut state, 100, 30);
    let buffer = backend.buffer();

    // Find the row where the first suggestion starts (contains "1. [Fix]")
    let mut found_first_suggestion_row = None;
    for y in 0..buffer.area.height {
        let mut row_text = String::new();
        for x in 0..buffer.area.width {
            let idx = (y * buffer.area.width + x) as usize;
            if idx < buffer.content.len() {
                row_text.push_str(buffer.content[idx].symbol());
            }
        }
        if row_text.contains("1.") && row_text.contains("[Fix]") {
            found_first_suggestion_row = Some(y);
            break;
        }
    }

    assert!(
        found_first_suggestion_row.is_some(),
        "Should find the first suggestion in the rendered output"
    );

    // Check that cells in the unselected suggestion's row do NOT have selected background
    let row = found_first_suggestion_row.unwrap();
    let selected_bg = theme::ai::SUGGESTION_SELECTED_BG;
    let mut cells_with_selected_bg = 0;

    for x in 0..buffer.area.width {
        let idx = (row * buffer.area.width + x) as usize;
        if idx < buffer.content.len() {
            let cell = &buffer.content[idx];
            if cell.bg == selected_bg {
                cells_with_selected_bg += 1;
            }
        }
    }

    // Unselected suggestion should have NO selected background cells
    assert_eq!(
        cells_with_selected_bg, 0,
        "Unselected suggestion should NOT have selected background. Found {} cells with selected bg",
        cells_with_selected_bg
    );
}
