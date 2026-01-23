use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
};

use crate::app::App;
use crate::history::MAX_VISIBLE_HISTORY;
use crate::scroll::Scrollable;
use crate::widgets::{popup, scrollbar};

pub const HISTORY_SEARCH_HEIGHT: u16 = 3;

/// Render the history popup
///
/// Returns the popup area for region tracking.
pub fn render_popup(app: &mut App, frame: &mut Frame, input_area: Rect) -> Option<Rect> {
    let visible_count = app.history.filtered_count().min(MAX_VISIBLE_HISTORY);
    let list_height = (visible_count as u16).max(1) + 2; // +2 for borders, min 1 row
    let total_height = list_height + HISTORY_SEARCH_HEIGHT;

    // Position popup above input (full width)
    let popup_y = input_area.y.saturating_sub(total_height);

    let popup_area = Rect {
        x: input_area.x,
        y: popup_y,
        width: input_area.width,
        height: total_height.min(input_area.y),
    };

    popup::clear_area(frame, popup_area);

    let layout = Layout::vertical([
        Constraint::Min(3),                        // History list
        Constraint::Length(HISTORY_SEARCH_HEIGHT), // Search box
    ])
    .split(popup_area);

    let list_area = layout[0];
    let search_area = layout[1];

    let title = format!(
        " History ({}/{}) ",
        app.history.filtered_count(),
        app.history.total_count()
    );

    let max_text_len = (list_area.width as usize).saturating_sub(6);

    let items: Vec<ListItem> = if app.history.filtered_count() == 0 {
        vec![ListItem::new(Line::from(Span::styled(
            "   No matches",
            Style::default().fg(Color::DarkGray),
        )))]
    } else {
        app.history
            .visible_entries()
            .map(|(display_idx, entry)| {
                let display_text = if entry.chars().count() > max_text_len {
                    let truncated: String = entry.chars().take(max_text_len).collect();
                    format!("{}…", truncated)
                } else {
                    entry.to_string()
                };

                let line = if display_idx == app.history.selected_index() {
                    Line::from(vec![Span::styled(
                        format!(" ► {} ", display_text),
                        Style::default()
                            .fg(Color::Black)
                            .bg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    )])
                } else {
                    Line::from(vec![Span::styled(
                        format!("   {} ", display_text),
                        Style::default().fg(Color::White).bg(Color::Black),
                    )])
                };

                ListItem::new(line)
            })
            .collect()
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(Style::default().fg(Color::Cyan))
        .style(Style::default().bg(Color::Black));

    let list = List::new(items).block(block);
    frame.render_widget(list, list_area);

    // Render scrollbar on border (excluding corners), matching border color
    // History list is displayed reversed (newest at bottom), so invert scroll position
    let scrollbar_area = Rect {
        x: list_area.x,
        y: list_area.y.saturating_add(1),
        width: list_area.width,
        height: list_area.height.saturating_sub(2),
    };
    let viewport = app.history.viewport_size();
    let max_scroll = app.history.max_scroll();
    let clamped_offset = app.history.scroll_offset().min(max_scroll);
    let inverted_scroll = max_scroll.saturating_sub(clamped_offset);
    scrollbar::render_vertical_scrollbar_styled(
        frame,
        scrollbar_area,
        app.history.filtered_count(),
        viewport,
        inverted_scroll,
        Color::Cyan,
    );

    let search_textarea = app.history.search_textarea_mut();
    search_textarea.set_block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Search ")
            .border_style(Style::default().fg(Color::Cyan))
            .style(Style::default().bg(Color::Black)),
    );
    search_textarea.set_style(Style::default().fg(Color::White).bg(Color::Black));
    frame.render_widget(&*search_textarea, search_area);

    Some(popup_area)
}
