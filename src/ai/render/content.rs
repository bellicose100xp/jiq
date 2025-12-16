//! Content building for AI popup
//!
//! Handles building the content text based on AI state.

use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
};

use crate::ai::ai_state::AiState;
use crate::ai::render::text::wrap_text;

/// Build the content text based on AI state
pub fn build_content(ai_state: &AiState, max_width: u16) -> Text<'static> {
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
            // Phase 3: Render suggestions with selection highlighting
            // Extracted to separate module for maintainability
            let suggestion_lines =
                crate::ai::render::suggestions::render_suggestions(ai_state, max_width, wrap_text);
            lines.extend(suggestion_lines);
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
