//! AI popup rendering
//!
//! Renders the AI assistant popup on the right side of the results pane.
//! The popup displays AI responses for error troubleshooting and query help.

use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
};

use super::ai_state::AiState;
use crate::widgets::popup;

// AI popup display constants
/// Minimum width for the AI popup to ensure readability
pub const AI_POPUP_MIN_WIDTH: u16 = 40;
/// Reserved space for autocomplete area on the left (35 chars + 2 margin)
pub const AUTOCOMPLETE_RESERVED_WIDTH: u16 = 37;
// TODO: Remove #[allow(dead_code)] when BORDER_HEIGHT is used
#[allow(dead_code)] // Phase 1: Reserved for future layout calculations
/// Border height (top + bottom)
const BORDER_HEIGHT: u16 = 2;
/// Minimum height for the popup
const MIN_HEIGHT: u16 = 6;
/// Maximum height as percentage of available space
const MAX_HEIGHT_PERCENT: u16 = 80;

/// Calculate the AI popup area based on frame dimensions
///
/// The popup is positioned on the right side of the results pane,
/// reserving space for the autocomplete area on the left.
///
/// # Arguments
/// * `frame_area` - The full frame area
/// * `results_area` - The results pane area (popup renders within this)
///
/// # Returns
/// A `Rect` for the AI popup, or `None` if there's not enough space
pub fn calculate_popup_area(frame_area: Rect, results_area: Rect) -> Option<Rect> {
    // Calculate available width after reserving autocomplete space
    let available_width = frame_area.width.saturating_sub(AUTOCOMPLETE_RESERVED_WIDTH);

    // Check if we have minimum width
    if available_width < AI_POPUP_MIN_WIDTH {
        return None;
    }

    // Popup width: use available space, capped at reasonable max
    let popup_width = available_width.min(frame_area.width / 2);

    // Popup height: use most of results area
    let max_height = (results_area.height * MAX_HEIGHT_PERCENT) / 100;
    let popup_height = max_height.max(MIN_HEIGHT);

    // Position on right side, anchored to bottom
    let popup_x = frame_area.width.saturating_sub(popup_width + 1);

    // Ensure popup fits within results area
    let final_height = popup_height.min(results_area.height.saturating_sub(2));

    // Anchor to bottom-right of results area
    let popup_y = results_area.y + results_area.height.saturating_sub(final_height + 1);

    Some(Rect {
        x: popup_x,
        y: popup_y,
        width: popup_width,
        height: final_height,
    })
}

/// Render the AI assistant popup
///
/// # Arguments
/// * `ai_state` - The current AI state
/// * `frame` - The frame to render to
/// * `results_area` - The results pane area
pub fn render_popup(ai_state: &AiState, frame: &mut Frame, results_area: Rect) {
    if !ai_state.visible {
        return;
    }

    let frame_area = frame.area();

    // Calculate popup area
    let popup_area = match calculate_popup_area(frame_area, results_area) {
        Some(area) => area,
        None => return, // Not enough space
    };

    // Clear background for floating effect
    popup::clear_area(frame, popup_area);

    // Build content based on state
    let content = build_content(ai_state, popup_area.width.saturating_sub(4));

    // Build title with keybinding hints
    let title = Line::from(vec![
        Span::raw(" "),
        Span::styled(
            "AI Assistant",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" "),
    ]);

    // Build hints for top-right of border (only Ctrl+A toggles)
    let hints = Line::from(vec![Span::styled(
        " Ctrl+A to close ",
        Style::default().fg(Color::DarkGray),
    )]);

    // Create the popup widget with green border
    let popup_widget = Paragraph::new(content).wrap(Wrap { trim: false }).block(
        Block::default()
            .borders(Borders::ALL)
            .title(title)
            .title_top(hints.alignment(ratatui::layout::Alignment::Right))
            .border_style(Style::default().fg(Color::Green))
            .style(Style::default().bg(Color::Black)),
    );

    frame.render_widget(popup_widget, popup_area);
}

/// Build the content text based on AI state
fn build_content(ai_state: &AiState, max_width: u16) -> Text<'static> {
    let mut lines: Vec<Line> = Vec::new();

    // Show setup instructions if AI is not configured
    if !ai_state.configured {
        lines.push(Line::from(vec![
            Span::styled("⚙ ", Style::default().fg(Color::Yellow)),
            Span::styled(
                "Setup Required",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
        ]));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "To enable AI assistance, add this",
            Style::default().fg(Color::Gray),
        )));
        lines.push(Line::from(Span::styled(
            "to ~/.config/jiq/config.toml:",
            Style::default().fg(Color::Gray),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "[ai]",
            Style::default().fg(Color::Cyan),
        )));
        lines.push(Line::from(Span::styled(
            "enabled = true",
            Style::default().fg(Color::Cyan),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "[ai.anthropic]",
            Style::default().fg(Color::Cyan),
        )));
        lines.push(Line::from(Span::styled(
            "api_key = \"sk-ant-...\"",
            Style::default().fg(Color::Cyan),
        )));
        lines.push(Line::from(Span::styled(
            "model = \"your-model-name\"",
            Style::default().fg(Color::Cyan),
        )));

        return Text::from(lines);
    }

    // Show error if present
    if let Some(error) = &ai_state.error {
        lines.push(Line::from(vec![
            Span::styled("⚠ ", Style::default().fg(Color::Red)),
            Span::styled(
                "Error",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
        ]));
        lines.push(Line::from(""));

        // Wrap error message
        for line in wrap_text(error, max_width as usize) {
            lines.push(Line::from(Span::styled(
                line,
                Style::default().fg(Color::Red),
            )));
        }

        return Text::from(lines);
    }

    // Show loading indicator if loading
    if ai_state.loading {
        // Show previous response dimmed if available
        if let Some(prev) = &ai_state.previous_response {
            for line in wrap_text(prev, max_width as usize) {
                lines.push(Line::from(Span::styled(
                    line,
                    Style::default().fg(Color::DarkGray),
                )));
            }
            lines.push(Line::from(""));
        }

        lines.push(Line::from(vec![
            Span::styled("⏳ ", Style::default().fg(Color::Yellow)),
            Span::styled(
                "Thinking...",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::ITALIC),
            ),
        ]));

        return Text::from(lines);
    }

    // Show response if available
    if !ai_state.response.is_empty() {
        for line in wrap_text(&ai_state.response, max_width as usize) {
            lines.push(Line::from(Span::styled(
                line,
                Style::default().fg(Color::White),
            )));
        }

        return Text::from(lines);
    }

    // Empty state - show help text (no duplicate title - it's already in the border)
    lines.push(Line::from(Span::styled(
        "Ready to help with your jq queries.",
        Style::default().fg(Color::Gray),
    )));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "When you encounter an error, I'll",
        Style::default().fg(Color::Gray),
    )));
    lines.push(Line::from(Span::styled(
        "provide suggestions to fix it.",
        Style::default().fg(Color::Gray),
    )));

    Text::from(lines)
}

/// Wrap text to fit within a given width, breaking at word boundaries
fn wrap_text(text: &str, max_width: usize) -> Vec<String> {
    if max_width == 0 {
        return vec![text.to_string()];
    }

    let mut lines = Vec::new();

    for paragraph in text.lines() {
        if paragraph.is_empty() {
            lines.push(String::new());
            continue;
        }

        let mut current_line = String::new();

        for word in paragraph.split_whitespace() {
            if current_line.is_empty() {
                current_line = word.to_string();
            } else if current_line.len() + 1 + word.len() <= max_width {
                current_line.push(' ');
                current_line.push_str(word);
            } else {
                lines.push(current_line);
                current_line = word.to_string();
            }
        }

        if !current_line.is_empty() {
            lines.push(current_line);
        }
    }

    if lines.is_empty() {
        lines.push(String::new());
    }

    lines
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    // =========================================================================
    // Property-Based Tests
    // =========================================================================

    // **Feature: ai-assistant, Property 17: Autocomplete area reservation**
    // *For any* frame width and AI popup visibility, the popup x-position should be ≥ 37
    // (35 chars autocomplete + 2 char margin).
    // **Validates: Requirements 8.2**
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn prop_autocomplete_area_reservation(
            frame_width in 80u16..300u16,
            frame_height in 20u16..100u16,
            results_height in 10u16..50u16
        ) {
            let results_height = results_height.min(frame_height.saturating_sub(5));
            let frame = Rect { x: 0, y: 0, width: frame_width, height: frame_height };
            let results = Rect { x: 0, y: 0, width: frame_width, height: results_height };

            if let Some(area) = calculate_popup_area(frame, results) {
                // The popup x-position should leave room for autocomplete (37 chars)
                prop_assert!(
                    area.x >= AUTOCOMPLETE_RESERVED_WIDTH,
                    "Popup x ({}) should be >= {} to reserve autocomplete area",
                    area.x,
                    AUTOCOMPLETE_RESERVED_WIDTH
                );
            }
            // If None is returned, that's acceptable - not enough space
        }
    }

    // **Feature: ai-assistant, Property 18: Minimum popup width**
    // *For any* frame width ≥ 80, the AI popup width should be ≥ 40 characters.
    // **Validates: Requirements 8.3**
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn prop_minimum_popup_width(
            frame_width in 80u16..300u16,
            frame_height in 20u16..100u16,
            results_height in 10u16..50u16
        ) {
            let results_height = results_height.min(frame_height.saturating_sub(5));
            let frame = Rect { x: 0, y: 0, width: frame_width, height: frame_height };
            let results = Rect { x: 0, y: 0, width: frame_width, height: results_height };

            if let Some(area) = calculate_popup_area(frame, results) {
                // For frame width >= 80, popup should have minimum width
                prop_assert!(
                    area.width >= AI_POPUP_MIN_WIDTH,
                    "Popup width ({}) should be >= {} for frame width {}",
                    area.width,
                    AI_POPUP_MIN_WIDTH,
                    frame_width
                );
            }
            // If None is returned, that's acceptable - not enough space
        }
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
    fn test_calculate_popup_area_basic() {
        let frame = Rect {
            x: 0,
            y: 0,
            width: 120,
            height: 40,
        };
        let results = Rect {
            x: 0,
            y: 0,
            width: 120,
            height: 30,
        };

        let area = calculate_popup_area(frame, results);
        assert!(area.is_some());

        let area = area.unwrap();
        // Should be on right side, after autocomplete reserved space
        assert!(area.x >= AUTOCOMPLETE_RESERVED_WIDTH);
        assert!(area.width >= AI_POPUP_MIN_WIDTH);
    }

    #[test]
    fn test_calculate_popup_area_too_narrow() {
        let frame = Rect {
            x: 0,
            y: 0,
            width: 50,
            height: 40,
        };
        let results = Rect {
            x: 0,
            y: 0,
            width: 50,
            height: 30,
        };

        let area = calculate_popup_area(frame, results);
        // Should return None if not enough space after autocomplete reservation
        // 50 - 37 = 13, which is less than MIN_WIDTH (40)
        assert!(area.is_none());
    }

    #[test]
    fn test_calculate_popup_area_minimum_viable() {
        // Minimum viable: 37 (autocomplete) + 40 (min popup) = 77
        let frame = Rect {
            x: 0,
            y: 0,
            width: 80,
            height: 40,
        };
        let results = Rect {
            x: 0,
            y: 0,
            width: 80,
            height: 30,
        };

        let area = calculate_popup_area(frame, results);
        assert!(area.is_some());

        let area = area.unwrap();
        assert!(area.width >= AI_POPUP_MIN_WIDTH);
    }

    #[test]
    fn test_build_content_empty_state() {
        let state = AiState::new_with_config(true, true, 1000);
        let content = build_content(&state, 60);

        // Should have help text
        assert!(!content.lines.is_empty());
    }

    #[test]
    fn test_build_content_not_configured() {
        let state = AiState::new_with_config(true, false, 1000);
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
        let mut state = AiState::new_with_config(true, true, 1000);
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
        let mut state = AiState::new_with_config(true, true, 1000);
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
        let mut state = AiState::new_with_config(true, true, 1000);
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
        let mut state = AiState::new_with_config(true, true, 1000);
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
    // Snapshot Tests
    // =========================================================================
    // **Validates: Requirements 2.3, 4.2, 6.1**

    use insta::assert_snapshot;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    /// Create a test terminal with specified dimensions
    fn create_test_terminal(width: u16, height: u16) -> Terminal<TestBackend> {
        let backend = TestBackend::new(width, height);
        Terminal::new(backend).unwrap()
    }

    /// Render AI popup to a test terminal and return the buffer as a string
    fn render_ai_popup_to_string(ai_state: &AiState, width: u16, height: u16) -> String {
        let mut terminal = create_test_terminal(width, height);
        terminal
            .draw(|f| {
                let results_area = Rect {
                    x: 0,
                    y: 0,
                    width,
                    height: height - 4,
                };
                render_popup(ai_state, f, results_area);
            })
            .unwrap();
        terminal.backend().to_string()
    }

    #[test]
    fn snapshot_ai_popup_empty_state() {
        let mut state = AiState::new_with_config(true, true, 1000);
        state.visible = true;

        let output = render_ai_popup_to_string(&state, 100, 30);
        assert_snapshot!(output);
    }

    #[test]
    fn snapshot_ai_popup_loading_state() {
        let mut state = AiState::new_with_config(true, true, 1000);
        state.visible = true;
        state.loading = true;

        let output = render_ai_popup_to_string(&state, 100, 30);
        assert_snapshot!(output);
    }

    #[test]
    fn snapshot_ai_popup_error_state() {
        let mut state = AiState::new_with_config(true, true, 1000);
        state.visible = true;
        state.error = Some("API Error: Rate limit exceeded. Please try again later.".to_string());

        let output = render_ai_popup_to_string(&state, 100, 30);
        assert_snapshot!(output);
    }

    #[test]
    fn snapshot_ai_popup_response_state() {
        let mut state = AiState::new_with_config(true, true, 1000);
        state.visible = true;
        state.response = "The error in your query `.foo[` is a missing closing bracket.\n\nTry using `.foo[]` to iterate over the array, or `.foo[0]` to access the first element.".to_string();

        let output = render_ai_popup_to_string(&state, 100, 30);
        assert_snapshot!(output);
    }

    #[test]
    fn snapshot_ai_popup_loading_with_previous() {
        let mut state = AiState::new_with_config(true, true, 1000);
        state.visible = true;
        state.loading = true;
        state.previous_response = Some("Previous suggestion: Use .foo instead of .bar".to_string());

        let output = render_ai_popup_to_string(&state, 100, 30);
        assert_snapshot!(output);
    }

    #[test]
    fn snapshot_ai_popup_not_visible() {
        let state = AiState::new_with_config(true, true, 1000);
        // visible is false by default

        let output = render_ai_popup_to_string(&state, 100, 30);
        assert_snapshot!(output);
    }

    #[test]
    fn snapshot_ai_popup_not_configured() {
        let mut state = AiState::new_with_config(true, false, 1000);
        state.visible = true;

        let output = render_ai_popup_to_string(&state, 100, 30);
        assert_snapshot!(output);
    }
}
