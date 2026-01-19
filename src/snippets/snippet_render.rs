use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use super::snippet_state::SnippetState;
use crate::widgets::popup;

pub fn render_popup(state: &SnippetState, frame: &mut Frame, results_area: Rect) {
    popup::clear_area(frame, results_area);

    let snippets = state.snippets();
    let content = if snippets.is_empty() {
        vec![Line::from(vec![Span::styled(
            "   No snippets yet. Press 'n' to create one.",
            Style::default().fg(Color::DarkGray),
        )])]
    } else {
        snippets
            .iter()
            .map(|s| Line::from(vec![Span::raw(format!("   {}", s.name))]))
            .collect()
    };

    let title = if snippets.is_empty() {
        " Snippets ".to_string()
    } else {
        format!(" Snippets ({}) ", snippets.len())
    };

    let popup = Paragraph::new(content).block(
        Block::default()
            .borders(Borders::ALL)
            .title(title)
            .border_style(Style::default().fg(Color::Cyan))
            .style(Style::default().bg(Color::Black)),
    );

    frame.render_widget(popup, results_area);
}

#[cfg(test)]
#[path = "snippet_render_tests.rs"]
mod snippet_render_tests;
