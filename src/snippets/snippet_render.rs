use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use super::snippet_state::SnippetState;
use crate::widgets::popup;

pub fn render_popup(state: &SnippetState, frame: &mut Frame, results_area: Rect) {
    popup::clear_area(frame, results_area);

    let snippets = state.snippets();
    let selected_index = state.selected_index();

    let content = if snippets.is_empty() {
        vec![Line::from(vec![Span::styled(
            "   No snippets yet. Press 'n' to create one.",
            Style::default().fg(Color::DarkGray),
        )])]
    } else {
        snippets
            .iter()
            .enumerate()
            .map(|(i, s)| {
                let is_selected = i == selected_index;
                let prefix = if is_selected { " ► " } else { "   " };
                let style = if is_selected {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };
                Line::from(vec![Span::styled(format!("{}{}", prefix, s.name), style)])
            })
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
