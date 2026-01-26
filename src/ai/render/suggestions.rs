//! Suggestion rendering for AI assistant
//!
//! Handles rendering of structured AI suggestions with selection highlighting.

use ratatui::{
    style::{Modifier, Style},
    text::{Line, Span},
};

use crate::ai::ai_state::AiState;
use crate::theme;

/// Render suggestions with selection highlighting
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

    let selected_index = ai_state.selection.get_selected();

    for (i, suggestion) in ai_state.suggestions.iter().enumerate() {
        let type_color = suggestion.suggestion_type.color();
        let type_label = suggestion.suggestion_type.label();

        let has_selection_number = i < 5;

        let is_selected = selected_index == Some(i);

        let prefix = if has_selection_number {
            format!("{}. {} ", i + 1, type_label)
        } else {
            format!("{} ", type_label)
        };
        let prefix_len = prefix.len();

        let query_max_width = max_width.saturating_sub(prefix_len as u16) as usize;
        let query_lines = wrap_text_fn(&suggestion.query, query_max_width);

        if let Some(first_query_line) = query_lines.first() {
            let mut spans = Vec::new();

            if has_selection_number {
                let mut style = Style::default().fg(theme::ai::SUGGESTION_TEXT_NORMAL);
                if is_selected {
                    style = style.fg(theme::ai::SUGGESTION_TEXT_SELECTED);
                }
                spans.push(Span::styled(format!("{}. ", i + 1), style));
            }

            let type_style = Style::default().fg(type_color).add_modifier(Modifier::BOLD);
            spans.push(Span::styled(type_label.to_string(), type_style));

            spans.push(Span::styled(" ", Style::default()));

            let query_style = Style::default().fg(theme::ai::QUERY_TEXT);
            spans.push(Span::styled(first_query_line.clone(), query_style));

            let mut line = Line::from(spans);
            if is_selected {
                line = line.style(Style::default().bg(theme::ai::SUGGESTION_SELECTED_BG));
            }
            lines.push(line);
        }

        for query_line in query_lines.iter().skip(1) {
            let indent = " ".repeat(prefix_len);
            let style = Style::default().fg(theme::ai::QUERY_TEXT);
            let mut line = Line::from(Span::styled(format!("{}{}", indent, query_line), style));
            if is_selected {
                line = line.style(Style::default().bg(theme::ai::SUGGESTION_SELECTED_BG));
            }
            lines.push(line);
        }

        if !suggestion.description.is_empty() {
            let desc_max_width = max_width.saturating_sub(3) as usize;
            for desc_line in wrap_text_fn(&suggestion.description, desc_max_width) {
                let mut style = Style::default().fg(theme::ai::SUGGESTION_DESC_NORMAL);
                if is_selected {
                    style = style.fg(theme::ai::SUGGESTION_DESC_MUTED);
                }
                let mut line = Line::from(Span::styled(format!("   {}", desc_line), style));
                if is_selected {
                    line = line.style(Style::default().bg(theme::ai::SUGGESTION_SELECTED_BG));
                }
                lines.push(line);
            }
        }

        if i < ai_state.suggestions.len() - 1 {
            lines.push(Line::from(""));
        }
    }

    lines
}
