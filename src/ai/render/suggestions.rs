//! Suggestion rendering for AI assistant
//!
//! Handles rendering of structured AI suggestions with selection highlighting.

use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};

use crate::ai::ai_state::AiState;

/// Render suggestions with selection highlighting
///
/// Phase 3: Renders suggestions with:
/// - Selection numbers (1-5) for first 5 suggestions
/// - Selection highlighting with background color
/// - Type labels with colors
/// - Wrapped query text
/// - Descriptions
///
/// # Arguments
/// * `ai_state` - The AI state containing suggestions and selection
/// * `max_width` - Maximum width for text wrapping
/// * `wrap_text_fn` - Function to wrap text to fit width
///
/// # Returns
/// Vector of rendered lines
pub fn render_suggestions<F>(
    ai_state: &AiState,
    max_width: u16,
    wrap_text_fn: F,
) -> Vec<Line<'static>>
where
    F: Fn(&str, usize) -> Vec<String>,
{
    let mut lines: Vec<Line> = Vec::new();

    // Phase 3: Get selected suggestion index for highlighting
    // Requirements: 4.5, 8.5
    let selected_index = ai_state.selection.get_selected();

    // Render structured suggestions with colors
    for (i, suggestion) in ai_state.suggestions.iter().enumerate() {
        // Number and type label with color
        let type_color = suggestion.suggestion_type.color();
        let type_label = suggestion.suggestion_type.label();

        // Phase 3: Add selection number (1-5) for first 5 suggestions only
        // Requirements: 4.1, 4.2, 4.3, 4.4
        let has_selection_number = i < 5;

        // Phase 3: Check if this suggestion is selected
        // Requirements: 4.5, 8.5
        let is_selected = selected_index == Some(i);

        // Calculate prefix length for query wrapping alignment
        // Format: "N. [Type] " where N is the suggestion number (1-5 only)
        let prefix = if has_selection_number {
            format!("{}. {} ", i + 1, type_label)
        } else {
            format!("{} ", type_label)
        };
        let prefix_len = prefix.len();

        // Wrap query text with proper indentation for continuation lines
        let query_max_width = max_width.saturating_sub(prefix_len as u16) as usize;
        let query_lines = wrap_text_fn(&suggestion.query, query_max_width);

        // Render first line with prefix
        if let Some(first_query_line) = query_lines.first() {
            let mut spans = Vec::new();

            // Add selection number (1-5) in dim gray for first 5 suggestions
            if has_selection_number {
                let mut style = Style::default().fg(Color::DarkGray);
                if is_selected {
                    style = style.bg(Color::DarkGray).fg(Color::Black);
                }
                spans.push(Span::styled(format!("{}. ", i + 1), style));
            }

            // Add type label with color
            let mut type_style = Style::default().fg(type_color).add_modifier(Modifier::BOLD);
            if is_selected {
                type_style = type_style.bg(Color::DarkGray);
            }
            spans.push(Span::styled(type_label.to_string(), type_style));

            let mut space_style = Style::default();
            if is_selected {
                space_style = space_style.bg(Color::DarkGray);
            }
            spans.push(Span::styled(" ", space_style));

            // Add query text
            let mut query_style = Style::default().fg(Color::Cyan);
            if is_selected {
                query_style = query_style.bg(Color::DarkGray);
            }
            spans.push(Span::styled(first_query_line.clone(), query_style));

            lines.push(Line::from(spans));
        }

        // Render continuation lines with proper indentation
        for query_line in query_lines.iter().skip(1) {
            let indent = " ".repeat(prefix_len);
            let mut style = Style::default().fg(Color::Cyan);
            if is_selected {
                style = style.bg(Color::DarkGray);
            }
            lines.push(Line::from(Span::styled(
                format!("{}{}", indent, query_line),
                style,
            )));
        }

        // Description with 3-space indent, wrapped
        if !suggestion.description.is_empty() {
            let desc_max_width = max_width.saturating_sub(3) as usize;
            for desc_line in wrap_text_fn(&suggestion.description, desc_max_width) {
                let mut style = Style::default().fg(Color::DarkGray);
                if is_selected {
                    style = style.bg(Color::DarkGray).fg(Color::Gray);
                }
                lines.push(Line::from(Span::styled(format!("   {}", desc_line), style)));
            }
        }

        // Add blank line between suggestions (except after last)
        if i < ai_state.suggestions.len() - 1 {
            lines.push(Line::from(""));
        }
    }

    lines
}
