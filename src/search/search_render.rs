use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders},
};

use crate::app::App;

pub const SEARCH_BAR_HEIGHT: u16 = 3;

pub fn render_bar(app: &mut App, frame: &mut Frame, area: Rect) {
    let match_count = app.search.match_count_display();
    let is_confirmed = app.search.is_confirmed();

    // When confirmed (inactive), search bar is gray; when editing (active), it's purple
    let border_color = if is_confirmed {
        Color::DarkGray
    } else {
        Color::LightMagenta
    };

    // Text color: gray when inactive, white when active
    let text_color = if is_confirmed {
        Color::DarkGray
    } else {
        Color::White
    };

    let match_count_style = if app.search.matches().is_empty() && !app.search.query().is_empty() {
        Style::default().fg(Color::Red)
    } else if is_confirmed {
        Style::default().fg(Color::DarkGray)
    } else {
        Style::default().fg(Color::Gray)
    };

    let title = if is_confirmed {
        " Search (press / to edit): "
    } else {
        " Search: "
    };

    let mut block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(title)
        .title_top(
            Line::from(Span::styled(
                format!(" {} ", match_count),
                match_count_style,
            ))
            .alignment(Alignment::Right),
        )
        .border_style(Style::default().fg(border_color))
        .style(Style::default().bg(Color::Black));

    if !is_confirmed {
        let hints = Line::from(vec![Span::styled(
            " [Enter] Confirm | [Esc] Close ",
            Style::default().fg(Color::LightMagenta),
        )]);
        block = block.title_bottom(hints.alignment(Alignment::Center));
    }

    let inner_area = block.inner(area);
    frame.render_widget(block, area);

    let search_textarea = app.search.search_textarea_mut();
    search_textarea.set_style(Style::default().fg(text_color).bg(Color::Black));
    search_textarea.set_cursor_line_style(Style::default());

    if is_confirmed {
        search_textarea.set_cursor_style(Style::default());
    } else {
        search_textarea.set_cursor_style(Style::default().add_modifier(Modifier::REVERSED));
    }

    frame.render_widget(&*search_textarea, inner_area);
}
