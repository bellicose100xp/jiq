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

    if !ai_state.configured {
        lines.push(Line::from(vec![
            Span::styled("⚙ ", Style::default().fg(Color::Yellow)),
            Span::styled(
                "AI provider not configured",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
        ]));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "To enable AI assistance, configure a provider",
            Style::default().fg(Color::Gray),
        )));
        lines.push(Line::from(Span::styled(
            "in ~/.config/jiq/config.toml:",
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
        lines.push(Line::from(Span::styled(
            "provider = \"anthropic\"  # or \"openai\", \"gemini\", \"bedrock\"",
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
            "model = \"claude-3-5-sonnet-20241022\"",
            Style::default().fg(Color::Cyan),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "For more details, see:",
            Style::default().fg(Color::Gray),
        )));
        lines.push(Line::from(Span::styled(
            "https://github.com/bellicose100xp/jiq#configuration",
            Style::default()
                .fg(Color::Blue)
                .add_modifier(Modifier::UNDERLINED),
        )));

        return Text::from(lines);
    }

    if let Some(error) = &ai_state.error {
        lines.push(Line::from(vec![
            Span::styled("⚠ ", Style::default().fg(Color::Red)),
            Span::styled(
                "Error",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
        ]));
        lines.push(Line::from(""));

        for line in wrap_text(error, max_width as usize) {
            lines.push(Line::from(Span::styled(
                line,
                Style::default().fg(Color::Red),
            )));
        }

        return Text::from(lines);
    }

    if ai_state.loading {
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

    if !ai_state.response.is_empty() {
        if !ai_state.suggestions.is_empty() {
            let suggestion_lines =
                crate::ai::render::suggestions::render_suggestions(ai_state, max_width, wrap_text);
            lines.extend(suggestion_lines);
        } else {
            for line in wrap_text(&ai_state.response, max_width as usize) {
                lines.push(Line::from(Span::styled(
                    line,
                    Style::default().fg(Color::White),
                )));
            }
        }

        return Text::from(lines);
    }

    Text::from(lines)
}
