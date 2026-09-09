//! Header content for the AI popup: everything rendered above the numbered
//! suggestions.
//!
//! Top to bottom: dimmed transcript of earlier exchanges, the current
//! question, then the active response's prose answer or its status line
//! (thinking, error, no suggestions, parse failure, not configured).

use ratatui::{
    style::{Modifier, Style},
    text::{Line, Span},
};

use crate::ai::ai_state::AiState;
use crate::ai::chat::ChatExchange;
use crate::ai::render::text::wrap_text;
use crate::theme;

/// Marker in front of a user question.
const QUESTION_PREFIX: &str = "❯ ";

/// Build the header lines for the popup at the given content width.
pub fn build_header_lines(ai_state: &AiState, max_width: u16) -> Vec<Line<'static>> {
    if !ai_state.configured {
        return not_configured_lines();
    }

    let width = max_width as usize;
    let mut lines: Vec<Line> = Vec::new();

    for exchange in &ai_state.history {
        push_transcript_exchange(&mut lines, exchange, width);
    }

    if let Some(question) = &ai_state.current_question {
        push_question(&mut lines, question, width, theme::ai::chat_question());
    }

    if let Some(error) = &ai_state.error {
        lines.push(Line::from(vec![
            Span::styled("⚠ ", Style::default().fg(theme::ai::error_icon())),
            Span::styled("Error", theme::ai::error_title()),
        ]));
        lines.push(Line::from(""));
        for line in wrap_text(error, width) {
            lines.push(Line::from(Span::styled(
                line,
                Style::default().fg(theme::ai::error_message()),
            )));
        }
        return lines;
    }

    if ai_state.loading {
        lines.push(Line::from(vec![
            Span::styled("⏳ ", Style::default().fg(theme::ai::thinking_icon())),
            Span::styled("Thinking...", theme::ai::thinking_text()),
        ]));
        return lines;
    }

    if let Some(answer) = &ai_state.answer {
        for line in wrap_text(answer, width) {
            lines.push(Line::from(Span::styled(
                line,
                Style::default().fg(theme::ai::chat_answer()),
            )));
        }
        if !ai_state.suggestions.is_empty() {
            lines.push(Line::from(""));
        }
        return lines;
    }

    if !ai_state.suggestions.is_empty() {
        return lines;
    }

    if ai_state.no_suggestions {
        lines.push(Line::from(vec![
            Span::styled("✓ ", Style::default().fg(theme::ai::empty_icon())),
            Span::styled("No suggestions", theme::ai::empty_title()),
        ]));
        lines.push(Line::from(""));
        for line in wrap_text("The AI had no suggestions for this query.", width) {
            lines.push(Line::from(Span::styled(
                line,
                Style::default().fg(theme::ai::empty_message()),
            )));
        }
        return lines;
    }

    if ai_state.parse_failed {
        lines.push(Line::from(vec![
            Span::styled("⚠ ", Style::default().fg(theme::ai::error_icon())),
            Span::styled("Could not parse AI response", theme::ai::error_title()),
        ]));
        lines.push(Line::from(""));
        for line in wrap_text("The response did not match the expected format.", width) {
            lines.push(Line::from(Span::styled(
                line,
                Style::default().fg(theme::ai::error_message()),
            )));
        }
        return lines;
    }

    if lines.is_empty() {
        // Canary: nothing to show at all. Rare; in a --debug session it points
        // at a state combination that leaves the user with no feedback.
        log::debug!(
            "build_header_lines: empty popup -> loading={} suggestions={} no_suggestions={} parse_failed={} error={}",
            ai_state.loading,
            ai_state.suggestions.len(),
            ai_state.no_suggestions,
            ai_state.parse_failed,
            ai_state.error.is_some()
        );
    }

    lines
}

/// An earlier exchange, rendered dimmed: question, answer, and its queries
/// without numbers or descriptions.
fn push_transcript_exchange(lines: &mut Vec<Line<'static>>, exchange: &ChatExchange, width: usize) {
    let dim = Style::default().fg(theme::ai::chat_transcript());
    push_question(
        lines,
        &exchange.question,
        width,
        dim.add_modifier(Modifier::BOLD),
    );
    if let Some(answer) = &exchange.answer {
        for line in wrap_text(answer, width) {
            lines.push(Line::from(Span::styled(line, dim)));
        }
    }
    for suggestion in &exchange.suggestions {
        let text = format!(
            "{} {}",
            suggestion.suggestion_type.label(),
            suggestion.query
        );
        for line in wrap_text(&text, width.saturating_sub(2)) {
            lines.push(Line::from(Span::styled(format!("  {}", line), dim)));
        }
    }
    lines.push(Line::from(""));
}

/// A question line with the `❯` marker; continuation lines are indented.
fn push_question(lines: &mut Vec<Line<'static>>, question: &str, width: usize, style: Style) {
    let indent = " ".repeat(QUESTION_PREFIX.chars().count());
    let wrapped = wrap_text(question, width.saturating_sub(indent.len()));
    for (i, line) in wrapped.into_iter().enumerate() {
        let prefix = if i == 0 { QUESTION_PREFIX } else { &indent };
        lines.push(Line::from(Span::styled(
            format!("{}{}", prefix, line),
            style,
        )));
    }
}

fn not_configured_lines() -> Vec<Line<'static>> {
    let desc = Style::default().fg(theme::ai::config_desc());
    let code = Style::default().fg(theme::ai::config_code());
    vec![
        Line::from(vec![
            Span::styled("⚙ ", Style::default().fg(theme::ai::config_icon())),
            Span::styled("AI provider not configured", theme::ai::config_title()),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "To enable AI assistance, configure a provider",
            desc,
        )),
        Line::from(Span::styled("in ~/.config/jiq/config.toml:", desc)),
        Line::from(""),
        Line::from(Span::styled("[ai]", code)),
        Line::from(Span::styled("enabled = true", code)),
        Line::from(Span::styled(
            "provider = \"anthropic\"  # or \"openai\", \"gemini\", \"bedrock\"",
            code,
        )),
        Line::from(""),
        Line::from(Span::styled("[ai.anthropic]", code)),
        Line::from(Span::styled("api_key = \"sk-ant-...\"", code)),
        Line::from(Span::styled("model = \"claude-3-5-sonnet-20241022\"", code)),
        Line::from(""),
        Line::from(Span::styled("For more details, see:", desc)),
        Line::from(Span::styled(
            "https://github.com/bellicose100xp/jiq#configuration",
            theme::ai::config_link(),
        )),
    ]
}
