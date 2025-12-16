use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders},
};

use crate::app::App;

pub const SEARCH_BAR_HEIGHT: u16 = 3;

pub fn render_bar(app: &mut App, frame: &mut Frame, area: Rect) {
    let match_count = app.search.match_count_display();
    let match_count_style = if app.search.matches().is_empty() && !app.search.query().is_empty() {
        Style::default().fg(Color::Red)
    } else {
        Style::default().fg(Color::Gray)
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Search: ")
        .title_top(
            Line::from(Span::styled(
                format!(" {} ", match_count),
                match_count_style,
            ))
            .alignment(Alignment::Right),
        )
        .border_style(Style::default().fg(Color::Cyan))
        .style(Style::default().bg(Color::Black));

    let inner_area = block.inner(area);
    frame.render_widget(block, area);

    let search_textarea = app.search.search_textarea_mut();
    search_textarea.set_style(Style::default().fg(Color::White).bg(Color::Black));
    search_textarea.set_cursor_line_style(Style::default());
    frame.render_widget(&*search_textarea, inner_area);
}
