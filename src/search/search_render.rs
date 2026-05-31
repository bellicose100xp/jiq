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
    let is_confirmed = app.search.is_confirmed();

    // When confirmed (inactive), search bar is gray; when editing (active), it's purple
    let border_color = if is_confirmed {
        theme::search::border_inactive()
    } else {
        theme::search::border_active()
    };

    // Text color: gray when inactive, white when active
    let text_color = if is_confirmed {
        theme::search::text_inactive()
    } else {
        theme::search::text_active()
    };

    let title = " Search ";

    let mut block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(title)
        .border_style(Style::default().fg(border_color))
        .style(Style::default().bg(theme::search::background()));

    // Only show badge on search input when not confirmed (editing mode)
    // When confirmed, the badge moves to the results pane
    if !is_confirmed {
        let match_count = app.search.match_count_display();
        let match_count_style = if app.search.matches().is_empty() && !app.search.query().is_empty()
        {
            theme::search::badge_no_matches()
        } else {
            theme::search::badge_match_count()
        };
        block = block.title_top(
            Line::from(vec![
                Span::raw(" "),
                Span::styled(format!("  {}  ", match_count), match_count_style),
                Span::raw(" "),
            ])
            .alignment(Alignment::Right),
        );
        // While editing the search query, drill chords are typed as
        // literal text so the user can grep for `>`, `<`, etc. The strip
        // therefore advertises only the keys that ARE wired in this mode:
        // confirm, navigate, close.
        let hints: Vec<(&'static str, &'static str)> =
            vec![("Enter", "Confirm"), ("Tab", "Confirm"), ("Esc", "Close")];
        block = block.title_bottom(
            theme::border_hints::build_hints(&hints, theme::search::hints())
                .alignment(Alignment::Center),
        );
    }

    let inner_area = block.inner(area);
    frame.render_widget(block, area);

    let search_textarea = app.search.search_textarea_mut();
    search_textarea.set_style(
        Style::default()
            .fg(text_color)
            .bg(theme::search::background()),
    );
    search_textarea.set_cursor_line_style(Style::default());

    if is_confirmed {
        search_textarea.set_cursor_style(Style::default());
    } else {
        search_textarea.set_cursor_style(theme::palette::cursor());
    }

    frame.render_widget(&*search_textarea, inner_area);
}
