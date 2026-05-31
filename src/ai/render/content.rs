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
            Span::styled("⚙ ", Style::default().fg(theme::ai::config_icon())),
            Span::styled("AI provider not configured", theme::ai::config_title()),
        ]));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "To enable AI assistance, configure a provider",
            Style::default().fg(theme::ai::config_desc()),
        )));
        lines.push(Line::from(Span::styled(
            "in ~/.config/jiq/config.toml:",
            Style::default().fg(theme::ai::config_desc()),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "[ai]",
            Style::default().fg(theme::ai::config_code()),
        )));
        lines.push(Line::from(Span::styled(
            "enabled = true",
            Style::default().fg(theme::ai::config_code()),
        )));
        lines.push(Line::from(Span::styled(
            "provider = \"anthropic\"  # or \"openai\", \"gemini\", \"bedrock\"",
            Style::default().fg(theme::ai::config_code()),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "[ai.anthropic]",
            Style::default().fg(theme::ai::config_code()),
        )));
        lines.push(Line::from(Span::styled(
            "api_key = \"sk-ant-...\"",
            Style::default().fg(theme::ai::config_code()),
        )));
        lines.push(Line::from(Span::styled(
            "model = \"claude-3-5-sonnet-20241022\"",
            Style::default().fg(theme::ai::config_code()),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "For more details, see:",
            Style::default().fg(theme::ai::config_desc()),
        )));
        lines.push(Line::from(Span::styled(
            "https://github.com/bellicose100xp/jiq#configuration",
            theme::ai::config_link(),
        )));

        return Text::from(lines);
    }

    if let Some(error) = &ai_state.error {
        lines.push(Line::from(vec![
            Span::styled("⚠ ", Style::default().fg(theme::ai::error_icon())),
            Span::styled("Error", theme::ai::error_title()),
        ]));
        lines.push(Line::from(""));

        for line in wrap_text(error, max_width as usize) {
            lines.push(Line::from(Span::styled(
                line,
                Style::default().fg(theme::ai::error_message()),
            )));
        }

        return Text::from(lines);
    }

    if ai_state.loading {
        if let Some(prev) = &ai_state.previous_response {
            for line in wrap_text(prev, max_width as usize) {
                lines.push(Line::from(Span::styled(
                    line,
                    Style::default().fg(theme::ai::previous_response()),
                )));
            }
            lines.push(Line::from(""));
        }

        lines.push(Line::from(vec![
            Span::styled("⏳ ", Style::default().fg(theme::ai::thinking_icon())),
            Span::styled("Thinking...", theme::ai::thinking_text()),
        ]));

        return Text::from(lines);
    }

    if !ai_state.suggestions.is_empty() {
        let suggestion_lines =
            crate::ai::render::suggestions::render_suggestions(ai_state, max_width, wrap_text);
        lines.extend(suggestion_lines);
        return Text::from(lines);
    }

    if ai_state.no_suggestions {
        lines.push(Line::from(vec![
            Span::styled("✓ ", Style::default().fg(theme::ai::empty_icon())),
            Span::styled("No suggestions", theme::ai::empty_title()),
        ]));
        lines.push(Line::from(""));
        for line in wrap_text(
            "The AI had no suggestions for this query.",
            max_width as usize,
        ) {
            lines.push(Line::from(Span::styled(
                line,
                Style::default().fg(theme::ai::empty_message()),
            )));
        }
        return Text::from(lines);
    }

    if ai_state.parse_failed {
        lines.push(Line::from(vec![
            Span::styled("⚠ ", Style::default().fg(theme::ai::error_icon())),
            Span::styled("Could not parse AI response", theme::ai::error_title()),
        ]));
        lines.push(Line::from(""));
        for line in wrap_text(
            "The response did not match the expected format.",
            max_width as usize,
        ) {
            lines.push(Line::from(Span::styled(
                line,
                Style::default().fg(theme::ai::error_message()),
            )));
        }
        return Text::from(lines);
    }

    // Canary: every earlier branch returned, so the popup is about to render an
    // empty box. This should be rare; if it shows up in a --debug session it
    // signals a state combination that leaves the user with no feedback.
    log::debug!(
        "build_content: empty popup (no branch matched) -> loading={} suggestions={} no_suggestions={} parse_failed={} error={}",
        ai_state.loading,
        ai_state.suggestions.len(),
        ai_state.no_suggestions,
        ai_state.parse_failed,
        ai_state.error.is_some()
    );

    Text::from(lines)
}
