use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use crate::widgets::popup;

pub fn render_popup(frame: &mut Frame, results_area: Rect) {
    popup::clear_area(frame, results_area);

    let content = vec![Line::from(vec![Span::styled(
        "   No snippets yet. Press 'n' to create one.",
        Style::default().fg(Color::DarkGray),
    )])];

    let popup = Paragraph::new(content).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Snippets ")
            .border_style(Style::default().fg(Color::Cyan))
            .style(Style::default().bg(Color::Black)),
    );

    frame.render_widget(popup, results_area);
}

#[cfg(test)]
#[path = "snippet_render_tests.rs"]
mod snippet_render_tests;
