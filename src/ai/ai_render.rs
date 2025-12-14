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
/// Maximum height as percentage of available space (Phase 2: reduced from 50%)
const MAX_HEIGHT_PERCENT: u16 = 40;
/// Maximum width as percentage of available space (Phase 2)
const MAX_WIDTH_PERCENT: u16 = 70;

/// Calculate the AI popup area based on frame dimensions
///
/// The popup is positioned on the right side, above the input bar,
/// reserving space for the autocomplete area on the left.
/// The bottom of the AI popup aligns with the bottom of the autocomplete popup.
///
/// # Arguments
/// * `frame_area` - The full frame area
/// * `input_area` - The input bar area (popup renders above this)
///
/// # Returns
/// A `Rect` for the AI popup, or `None` if there's not enough space
pub fn calculate_popup_area(frame_area: Rect, input_area: Rect) -> Option<Rect> {
    // Calculate available width after reserving autocomplete space
    let available_width = frame_area.width.saturating_sub(AUTOCOMPLETE_RESERVED_WIDTH);

    // Check if we have minimum width
    if available_width < AI_POPUP_MIN_WIDTH {
        return None;
    }

    // Phase 2: Use up to 70% of available width (after autocomplete reservation)
    let max_width = (available_width * MAX_WIDTH_PERCENT) / 100;
    let popup_width = available_width.min(max_width).max(AI_POPUP_MIN_WIDTH);

    // Calculate available height above input bar
    let available_height = input_area.y;

    // Phase 2: Max 40% of available height (reduced from 50%)
    let max_height = (available_height * MAX_HEIGHT_PERCENT) / 100;
    let popup_height = max_height.max(MIN_HEIGHT).min(available_height);

    // Check if we have enough vertical space
    if popup_height < MIN_HEIGHT {
        return None;
    }

    // Position on right side
    let popup_x = frame_area.width.saturating_sub(popup_width + 1);

    // Position above input bar (bottom of popup aligns with top of input)
    let popup_y = input_area.y.saturating_sub(popup_height);

    Some(Rect {
        x: popup_x,
        y: popup_y,
        width: popup_width,
        height: popup_height,
    })
}

/// Calculate dynamic word limit based on popup dimensions
///
/// Formula: (width - 4) * (height - 2) / 5, clamped to 100-800
/// - width - 4: accounts for borders (2) and padding (2)
/// - height - 2: accounts for top and bottom borders
/// - / 5: approximate characters per word with spacing (Phase 2.1: more generous)
///
/// # Requirements
/// - 7.1: Formula-based calculation
/// - 7.2: Minimum 100 words
/// - 7.3: Maximum 800 words (Phase 2.1: increased from 500)
/// - 7.5: Pure and deterministic
pub fn calculate_word_limit(width: u16, height: u16) -> u16 {
    let content_width = width.saturating_sub(4); // borders + padding
    let content_height = height.saturating_sub(2); // borders
    let raw_limit = (content_width as u32 * content_height as u32) / 5;
    raw_limit.clamp(100, 800) as u16
}

/// Render the AI assistant popup
///
/// # Arguments
/// * `ai_state` - The current AI state (mutable to update word_limit)
/// * `frame` - The frame to render to
/// * `input_area` - The input bar area (popup renders above this)
///
/// # Phase 2 Updates
/// - Calculates and stores word_limit in ai_state for next AI request
pub fn render_popup(ai_state: &mut AiState, frame: &mut Frame, input_area: Rect) {
    if !ai_state.visible {
        return;
    }

    let frame_area = frame.area();

    // Calculate popup area (positioned above input bar)
    let popup_area = match calculate_popup_area(frame_area, input_area) {
        Some(area) => area,
        None => return, // Not enough space
    };

    // Phase 2: Calculate and store word limit for next AI request
    // Requirements: 2.1, 7.4
    ai_state.word_limit = calculate_word_limit(popup_area.width, popup_area.height);

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
        // Phase 2: Check if we have parsed suggestions
        if !ai_state.suggestions.is_empty() {
            // Render structured suggestions with colors
            for (i, suggestion) in ai_state.suggestions.iter().enumerate() {
                // Number and type label with color
                let type_color = suggestion.suggestion_type.color();
                let type_label = suggestion.suggestion_type.label();

                // Calculate prefix length for query wrapping alignment
                // Format: "N. [Type] " where N is the suggestion number
                let prefix = format!("{}. {} ", i + 1, type_label);
                let prefix_len = prefix.len();

                // Wrap query text with proper indentation for continuation lines
                let query_max_width = max_width.saturating_sub(prefix_len as u16) as usize;
                let query_lines = wrap_text(&suggestion.query, query_max_width);

                // Render first line with prefix
                if let Some(first_query_line) = query_lines.first() {
                    lines.push(Line::from(vec![
                        Span::styled(
                            format!("{}. ", i + 1),
                            Style::default()
                                .fg(Color::White)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(
                            type_label.to_string(),
                            Style::default().fg(type_color).add_modifier(Modifier::BOLD),
                        ),
                        Span::raw(" "),
                        Span::styled(first_query_line.clone(), Style::default().fg(Color::Cyan)),
                    ]));
                }

                // Render continuation lines with proper indentation
                for query_line in query_lines.iter().skip(1) {
                    let indent = " ".repeat(prefix_len);
                    lines.push(Line::from(Span::styled(
                        format!("{}{}", indent, query_line),
                        Style::default().fg(Color::Cyan),
                    )));
                }

                // Description with 3-space indent, wrapped
                if !suggestion.description.is_empty() {
                    let desc_max_width = max_width.saturating_sub(3) as usize;
                    for desc_line in wrap_text(&suggestion.description, desc_max_width) {
                        lines.push(Line::from(Span::styled(
                            format!("   {}", desc_line),
                            Style::default().fg(Color::DarkGray),
                        )));
                    }
                }

                // Add blank line between suggestions (except after last)
                if i < ai_state.suggestions.len() - 1 {
                    lines.push(Line::from(""));
                }
            }
        } else {
            // Fallback: render raw response if no suggestions parsed
            for line in wrap_text(&ai_state.response, max_width as usize) {
                lines.push(Line::from(Span::styled(
                    line,
                    Style::default().fg(Color::White),
                )));
            }
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
            input_y in 10u16..50u16
        ) {
            let input_y = input_y.min(frame_height.saturating_sub(4));
            let frame = Rect { x: 0, y: 0, width: frame_width, height: frame_height };
            // Input area at bottom of screen (3 lines high)
            let input = Rect { x: 0, y: input_y, width: frame_width, height: 3 };

            if let Some(area) = calculate_popup_area(frame, input) {
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
            input_y in 10u16..50u16
        ) {
            let input_y = input_y.min(frame_height.saturating_sub(4));
            let frame = Rect { x: 0, y: 0, width: frame_width, height: frame_height };
            // Input area at bottom of screen (3 lines high)
            let input = Rect { x: 0, y: input_y, width: frame_width, height: 3 };

            if let Some(area) = calculate_popup_area(frame, input) {
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
    // Phase 2 Property-Based Tests
    // =========================================================================

    // **Feature: ai-assistant-phase2, Property 1: Popup width respects maximum**
    // *For any* terminal width, the AI popup width SHALL be at most 70% of available width.
    // **Validates: Requirements 1.5, 6.2**
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn prop_popup_width_respects_maximum(
            frame_width in 80u16..300u16,
            frame_height in 20u16..100u16,
            input_y in 10u16..50u16
        ) {
            let input_y = input_y.min(frame_height.saturating_sub(4));
            let frame = Rect { x: 0, y: 0, width: frame_width, height: frame_height };
            let input = Rect { x: 0, y: input_y, width: frame_width, height: 3 };

            if let Some(area) = calculate_popup_area(frame, input) {
                let available_width = frame_width.saturating_sub(AUTOCOMPLETE_RESERVED_WIDTH);
                let max_allowed = (available_width * 70) / 100;
                prop_assert!(
                    area.width <= max_allowed || area.width == AI_POPUP_MIN_WIDTH,
                    "Popup width ({}) should be <= 70% of available ({}) or minimum ({})",
                    area.width, max_allowed, AI_POPUP_MIN_WIDTH
                );
            }
        }
    }

    // **Feature: ai-assistant-phase2, Property 2: Popup height respects maximum**
    // *For any* terminal height, the AI popup height SHALL be at most 40% of available vertical space.
    // **Validates: Requirements 1.2, 6.4**
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn prop_popup_height_respects_maximum(
            frame_width in 80u16..300u16,
            frame_height in 20u16..100u16,
            input_y in 10u16..50u16
        ) {
            let input_y = input_y.min(frame_height.saturating_sub(4));
            let frame = Rect { x: 0, y: 0, width: frame_width, height: frame_height };
            let input = Rect { x: 0, y: input_y, width: frame_width, height: 3 };

            if let Some(area) = calculate_popup_area(frame, input) {
                let available_height = input_y;
                let max_allowed = (available_height * 40) / 100;
                prop_assert!(
                    area.height <= available_height && (area.height <= max_allowed || area.height == MIN_HEIGHT),
                    "Popup height ({}) should be <= 40% of available ({}) or minimum ({})",
                    area.height, max_allowed, MIN_HEIGHT
                );
            }
        }
    }

    // **Feature: ai-assistant-phase2, Property 3: Minimum dimensions enforced**
    // *For any* terminal size, the AI popup width SHALL be >= 40 and height >= 6, or not displayed.
    // **Validates: Requirements 6.1, 6.3, 6.5**
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn prop_minimum_dimensions_enforced(
            frame_width in 40u16..300u16,
            frame_height in 10u16..100u16,
            input_y in 5u16..50u16
        ) {
            let input_y = input_y.min(frame_height.saturating_sub(4));
            let frame = Rect { x: 0, y: 0, width: frame_width, height: frame_height };
            let input = Rect { x: 0, y: input_y, width: frame_width, height: 3 };

            match calculate_popup_area(frame, input) {
                Some(area) => {
                    prop_assert!(
                        area.width >= AI_POPUP_MIN_WIDTH,
                        "Popup width ({}) must be >= minimum ({})",
                        area.width, AI_POPUP_MIN_WIDTH
                    );
                    prop_assert!(
                        area.height >= MIN_HEIGHT,
                        "Popup height ({}) must be >= minimum ({})",
                        area.height, MIN_HEIGHT
                    );
                }
                None => {
                    // If None, it means there wasn't enough space - that's valid
                }
            }
        }
    }

    // **Feature: ai-assistant-phase2, Property 4: Word limit formula correctness**
    // *For any* popup dimensions (w, h), the word limit SHALL equal clamp((w-4)*(h-2)/5, 100, 800).
    // **Validates: Requirements 7.1, 7.2, 7.3**
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn prop_word_limit_formula_correctness(
            width in 40u16..200u16,
            height in 6u16..50u16
        ) {
            let result = calculate_word_limit(width, height);
            let content_width = width.saturating_sub(4);
            let content_height = height.saturating_sub(2);
            let expected_raw = (content_width as u32 * content_height as u32) / 5;
            let expected = expected_raw.clamp(100, 800) as u16;

            prop_assert_eq!(
                result, expected,
                "Word limit for {}x{} should be {} (raw: {})",
                width, height, expected, expected_raw
            );
        }
    }

    // **Feature: ai-assistant-phase2, Property 5: Word limit determinism**
    // *For any* given popup dimensions, calling calculate_word_limit multiple times SHALL return the same value.
    // **Validates: Requirements 7.5**
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn prop_word_limit_determinism(
            width in 40u16..200u16,
            height in 6u16..50u16
        ) {
            let result1 = calculate_word_limit(width, height);
            let result2 = calculate_word_limit(width, height);
            let result3 = calculate_word_limit(width, height);

            prop_assert_eq!(result1, result2, "Word limit should be deterministic");
            prop_assert_eq!(result2, result3, "Word limit should be deterministic");
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

    // =========================================================================
    // Word Limit Unit Tests (Phase 2)
    // =========================================================================

    #[test]
    fn test_word_limit_minimum_clamp() {
        // Very small dimensions should clamp to 100
        let result = calculate_word_limit(10, 5);
        assert_eq!(result, 100);
    }

    #[test]
    fn test_word_limit_maximum_clamp() {
        // Very large dimensions should clamp to 800
        let result = calculate_word_limit(200, 100);
        assert_eq!(result, 800);
    }

    #[test]
    fn test_word_limit_typical_small() {
        // 44 width, 8 height: (44-4)*(8-2)/5 = 40*6/5 = 48 -> clamped to 100
        let result = calculate_word_limit(44, 8);
        assert_eq!(result, 100);
    }

    #[test]
    fn test_word_limit_typical_medium() {
        // 60 width, 15 height: (60-4)*(15-2)/5 = 56*13/5 = 145
        let result = calculate_word_limit(60, 15);
        assert_eq!(result, 145);
    }

    #[test]
    fn test_word_limit_typical_large() {
        // 80 width, 20 height: (80-4)*(20-2)/5 = 76*18/5 = 273
        let result = calculate_word_limit(80, 20);
        assert_eq!(result, 273);
    }

    #[test]
    fn test_word_limit_boundary_100() {
        // Find dimensions that give exactly 100 (or just above)
        // (w-4)*(h-2)/5 = 100 -> (w-4)*(h-2) = 500
        // e.g., 44 width, 14 height: 40*12/5 = 96 -> clamped to 100
        let result = calculate_word_limit(44, 14);
        assert_eq!(result, 100);
    }

    #[test]
    fn test_calculate_popup_area_basic() {
        let frame = Rect {
            x: 0,
            y: 0,
            width: 120,
            height: 40,
        };
        // Input area at bottom (y=37, height=3)
        let input = Rect {
            x: 0,
            y: 37,
            width: 120,
            height: 3,
        };

        let area = calculate_popup_area(frame, input);
        assert!(area.is_some());

        let area = area.unwrap();
        // Should be on right side, after autocomplete reserved space
        assert!(area.x >= AUTOCOMPLETE_RESERVED_WIDTH);
        assert!(area.width >= AI_POPUP_MIN_WIDTH);
        // Should be positioned above input bar
        assert!(area.y + area.height <= input.y);
    }

    #[test]
    fn test_calculate_popup_area_too_narrow() {
        let frame = Rect {
            x: 0,
            y: 0,
            width: 50,
            height: 40,
        };
        let input = Rect {
            x: 0,
            y: 37,
            width: 50,
            height: 3,
        };

        let area = calculate_popup_area(frame, input);
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
        let input = Rect {
            x: 0,
            y: 37,
            width: 80,
            height: 3,
        };

        let area = calculate_popup_area(frame, input);
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
        let mut state = AiState::new_with_config(true, true, 1000);
        state.visible = true;

        let output = render_ai_popup_to_string(&mut state, 100, 30);
        assert_snapshot!(output);
    }

    #[test]
    fn snapshot_ai_popup_loading_state() {
        let mut state = AiState::new_with_config(true, true, 1000);
        state.visible = true;
        state.loading = true;

        let output = render_ai_popup_to_string(&mut state, 100, 30);
        assert_snapshot!(output);
    }

    #[test]
    fn snapshot_ai_popup_error_state() {
        let mut state = AiState::new_with_config(true, true, 1000);
        state.visible = true;
        state.error = Some("API Error: Rate limit exceeded. Please try again later.".to_string());

        let output = render_ai_popup_to_string(&mut state, 100, 30);
        assert_snapshot!(output);
    }

    #[test]
    fn snapshot_ai_popup_response_state() {
        let mut state = AiState::new_with_config(true, true, 1000);
        state.visible = true;
        state.response = "The error in your query `.foo[` is a missing closing bracket.\n\nTry using `.foo[]` to iterate over the array, or `.foo[0]` to access the first element.".to_string();

        let output = render_ai_popup_to_string(&mut state, 100, 30);
        assert_snapshot!(output);
    }

    #[test]
    fn snapshot_ai_popup_loading_with_previous() {
        let mut state = AiState::new_with_config(true, true, 1000);
        state.visible = true;
        state.loading = true;
        state.previous_response = Some("Previous suggestion: Use .foo instead of .bar".to_string());

        let output = render_ai_popup_to_string(&mut state, 100, 30);
        assert_snapshot!(output);
    }

    #[test]
    fn snapshot_ai_popup_not_visible() {
        let mut state = AiState::new_with_config(true, true, 1000);
        // Phase 2: Explicitly set visible to false to test hidden state
        state.visible = false;

        let output = render_ai_popup_to_string(&mut state, 100, 30);
        assert_snapshot!(output);
    }

    #[test]
    fn snapshot_ai_popup_not_configured() {
        let mut state = AiState::new_with_config(true, false, 1000);
        state.visible = true;

        let output = render_ai_popup_to_string(&mut state, 100, 30);
        assert_snapshot!(output);
    }

    // =========================================================================
    // Phase 2: Suggestion Display Snapshot Tests
    // =========================================================================

    #[test]
    fn snapshot_ai_popup_with_suggestions() {
        use super::super::ai_state::{Suggestion, SuggestionType};

        let mut state = AiState::new_with_config(true, true, 1000);
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
        let mut state = AiState::new_with_config(true, true, 1000);
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
        use super::super::ai_state::{Suggestion, SuggestionType};

        let mut state = AiState::new_with_config(true, true, 1000);
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
}
