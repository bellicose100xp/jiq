use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Block, BorderType, Borders},
};

use crate::app::App;
use crate::theme;

pub const SEARCH_BAR_HEIGHT: u16 = 3;

pub fn render_bar(app: &mut App, frame: &mut Frame, area: Rect) {
    let match_count = app.search.match_count_display();
    let is_confirmed = app.search.is_confirmed();

    // When confirmed (inactive), search bar is gray; when editing (active), it's purple
    let border_color = if is_confirmed {
        theme::search::BORDER_INACTIVE
    } else {
        theme::search::BORDER_ACTIVE
    };

    // Text color: gray when inactive, white when active
    let text_color = if is_confirmed {
        theme::search::TEXT_INACTIVE
    } else {
        theme::search::TEXT_ACTIVE
    };

    let match_count_style = if app.search.matches().is_empty() && !app.search.query().is_empty() {
        Style::default().fg(theme::search::NO_MATCHES)
    } else if is_confirmed {
        Style::default().fg(theme::search::MATCH_COUNT_CONFIRMED)
    } else {
        Style::default().fg(theme::search::MATCH_COUNT)
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
        .style(Style::default().bg(theme::search::BACKGROUND));

    if !is_confirmed {
        block = block.title_bottom(
            theme::border_hints::build_hints(
                &[("Enter", "Confirm"), ("Esc", "Close")],
                theme::search::HINTS,
            )
            .alignment(Alignment::Center),
        );
    }

    let inner_area = block.inner(area);
    frame.render_widget(block, area);

    let search_textarea = app.search.search_textarea_mut();
    search_textarea.set_style(
        Style::default()
            .fg(text_color)
            .bg(theme::search::BACKGROUND),
    );
    search_textarea.set_cursor_line_style(Style::default());

    if is_confirmed {
        search_textarea.set_cursor_style(Style::default());
    } else {
        search_textarea.set_cursor_style(theme::palette::CURSOR);
    }

    frame.render_widget(&*search_textarea, inner_area);
}
