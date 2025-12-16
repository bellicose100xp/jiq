//! AI popup rendering
//!
//! Renders the AI assistant popup on the right side of the results pane.
//! The popup displays AI responses for error troubleshooting and query help.

use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};

use super::ai_state::AiState;
use crate::widgets::popup;

// Use modules from render submodule instead of loading them directly
use super::render::layout;

// Re-export public items from sub-modules
pub use self::content::build_content;
pub use layout::{calculate_popup_area, calculate_word_limit};

// Module declarations - only content is local
#[path = "render/content.rs"]
mod content;

/// Render the AI assistant popup
///
/// # Arguments
/// * `ai_state` - The current AI state (mutable to update word_limit)
/// * `frame` - The frame to render to
/// * `input_area` - The input bar area (popup renders above this)
pub fn render_popup(ai_state: &mut AiState, frame: &mut Frame, input_area: Rect) {
    if !ai_state.visible {
        return;
    }

    let frame_area = frame.area();

    let popup_area = match calculate_popup_area(frame_area, input_area) {
        Some(area) => area,
        None => return,
    };

    ai_state.word_limit = calculate_word_limit(popup_area.width, popup_area.height);

    popup::clear_area(frame, popup_area);

    let content = build_content(ai_state, popup_area.width.saturating_sub(4));

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

    let hints = if !ai_state.suggestions.is_empty() {
        Line::from(vec![Span::styled(
            " Alt+1-5 or Alt+↑↓+Enter to apply | Ctrl+A to close ",
            Style::default().fg(Color::DarkGray),
        )])
    } else {
        Line::from(vec![Span::styled(
            " Ctrl+A to close ",
            Style::default().fg(Color::DarkGray),
        )])
    };

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
