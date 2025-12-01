//! History popup rendering
//!
//! This module handles rendering of the history popup.

use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

use crate::app::App;
use crate::history::MAX_VISIBLE_HISTORY;
use crate::widgets::popup;

// History popup display constants
pub const HISTORY_SEARCH_HEIGHT: u16 = 3;

/// Render the history popup above the input field
pub fn render_popup(app: &mut App, frame: &mut Frame, input_area: Rect) {
    // Calculate dimensions - ensure minimum 1 row for "No matches" message
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

    // Clear background
    popup::clear_area(frame, popup_area);

    // Split into list area and search area
    let layout = Layout::vertical([
        Constraint::Min(3),                    // History list
        Constraint::Length(HISTORY_SEARCH_HEIGHT), // Search box
    ])
    .split(popup_area);

    let list_area = layout[0];
    let search_area = layout[1];

    // Build title with match count
    let title = format!(
        " History ({}/{}) ",
        app.history.filtered_count(),
        app.history.total_count()
    );

    // Calculate max text length based on available width
    // Format: " ► text " with borders -> overhead = 6 chars (borders + padding + arrow)
    let max_text_len = (list_area.width as usize).saturating_sub(6);

    // Create list items
    let items: Vec<ListItem> = if app.history.filtered_count() == 0 {
        // Show "No matches" when search has no results
        vec![ListItem::new(Line::from(Span::styled(
            "   No matches",
            Style::default().fg(Color::DarkGray),
        )))]
    } else {
        app.history
            .visible_entries()
            .map(|(display_idx, entry)| {
                // Truncate long entries (char-safe for UTF-8)
                let display_text = if entry.chars().count() > max_text_len {
                    let truncated: String = entry.chars().take(max_text_len).collect();
                    format!("{}…", truncated)
                } else {
                    entry.to_string()
                };

                let line = if display_idx == app.history.selected_index() {
                    // Selected item
                    Line::from(vec![Span::styled(
                        format!(" ► {} ", display_text),
                        Style::default()
                            .fg(Color::Black)
                            .bg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    )])
                } else {
                    // Unselected item
                    Line::from(vec![Span::styled(
                        format!("   {} ", display_text),
                        Style::default().fg(Color::White).bg(Color::Black),
                    )])
                };

                ListItem::new(line)
            })
            .collect()
    };

    // Render list
    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(title)
            .border_style(Style::default().fg(Color::Cyan))
            .style(Style::default().bg(Color::Black)),
    );
    frame.render_widget(list, list_area);

    // Render search box using TextArea
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
}
