//! Content building for AI popup
//!
//! Handles building the content text based on AI state.

use ratatui::{
    style::Style,
    text::{Line, Span, Text},
};

use crate::ai::ai_state::AiState;
use crate::ai::render::text::wrap_text;
use crate::theme;

/// Build the content text based on AI state
pub fn build_content(ai_state: &AiState, max_width: u16) -> Text<'static> {
    let mut lines: Vec<Line> = Vec::new();

    if !ai_state.configured {
        lines.push(Line::from(vec![
            Span::styled("⚙ ", Style::default().fg(theme::ai::CONFIG_ICON)),
            Span::styled("AI provider not configured", theme::ai::CONFIG_TITLE),
        ]));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "To enable AI assistance, configure a provider",
            Style::default().fg(theme::ai::CONFIG_DESC),
        )));
        lines.push(Line::from(Span::styled(
            "in ~/.config/jiq/config.toml:",
            Style::default().fg(theme::ai::CONFIG_DESC),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "[ai]",
            Style::default().fg(theme::ai::CONFIG_CODE),
        )));
        lines.push(Line::from(Span::styled(
            "enabled = true",
            Style::default().fg(theme::ai::CONFIG_CODE),
        )));
        lines.push(Line::from(Span::styled(
            "provider = \"anthropic\"  # or \"openai\", \"gemini\", \"bedrock\"",
            Style::default().fg(theme::ai::CONFIG_CODE),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "[ai.anthropic]",
            Style::default().fg(theme::ai::CONFIG_CODE),
        )));
        lines.push(Line::from(Span::styled(
            "api_key = \"sk-ant-...\"",
            Style::default().fg(theme::ai::CONFIG_CODE),
        )));
        lines.push(Line::from(Span::styled(
            "model = \"claude-3-5-sonnet-20241022\"",
            Style::default().fg(theme::ai::CONFIG_CODE),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "For more details, see:",
            Style::default().fg(theme::ai::CONFIG_DESC),
        )));
        lines.push(Line::from(Span::styled(
            "https://github.com/bellicose100xp/jiq#configuration",
            theme::ai::CONFIG_LINK,
        )));

        return Text::from(lines);
    }

    if let Some(error) = &ai_state.error {
        lines.push(Line::from(vec![
            Span::styled("⚠ ", Style::default().fg(theme::ai::ERROR_ICON)),
            Span::styled("Error", theme::ai::ERROR_TITLE),
        ]));
        lines.push(Line::from(""));

        for line in wrap_text(error, max_width as usize) {
            lines.push(Line::from(Span::styled(
                line,
                Style::default().fg(theme::ai::ERROR_MESSAGE),
            )));
        }

        return Text::from(lines);
    }

    if ai_state.loading {
        if let Some(prev) = &ai_state.previous_response {
            for line in wrap_text(prev, max_width as usize) {
                lines.push(Line::from(Span::styled(
                    line,
                    Style::default().fg(theme::ai::PREVIOUS_RESPONSE),
                )));
            }
            lines.push(Line::from(""));
        }

        lines.push(Line::from(vec![
            Span::styled("⏳ ", Style::default().fg(theme::ai::THINKING_ICON)),
            Span::styled("Thinking...", theme::ai::THINKING_TEXT),
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
                    Style::default().fg(theme::ai::RESULT_TEXT),
                )));
            }
        }

        return Text::from(lines);
    }

    Text::from(lines)
}
