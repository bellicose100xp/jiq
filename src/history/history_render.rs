use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem},
};

use crate::app::App;
use crate::history::MAX_VISIBLE_HISTORY;
use crate::scroll::Scrollable;
use crate::syntax_highlight::JqHighlighter;
use crate::theme;
use crate::widgets::{popup, scrollbar};

pub const HISTORY_SEARCH_HEIGHT: u16 = 3;

/// Render the history popup
///
/// Returns the popup area for region tracking.
pub fn render_popup(app: &mut App, frame: &mut Frame, input_area: Rect) -> Option<Rect> {
    let visible_count = app.history.filtered_count().min(MAX_VISIBLE_HISTORY);
    let list_height = (visible_count as u16).max(1) + 4; // +2 for borders, +2 for top/bottom padding
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
        vec![
            ListItem::new(Line::from("")),
            ListItem::new(Line::from(Span::styled(
                "     No matches",
                Style::default().fg(theme::history::NO_MATCHES),
            ))),
            ListItem::new(Line::from("")),
        ]
    } else {
        let mut list_items: Vec<ListItem> = Vec::new();

        // Top padding
        list_items.push(ListItem::new(Line::from("")));

        for (display_idx, entry) in app.history.visible_entries() {
            let display_text = if entry.chars().count() > max_text_len {
                let truncated: String = entry.chars().take(max_text_len).collect();
                format!("{}…", truncated)
            } else {
                entry.to_string()
            };

            let is_selected = display_idx == app.history.selected_index();
            let is_even = display_idx % 2 == 0;

            let (bg_color, bg_style, prefix) = if is_selected {
                (
                    theme::history::ITEM_SELECTED_BG,
                    Style::default().bg(theme::history::ITEM_SELECTED_BG),
                    vec![
                        Span::styled(
                            " ┃",
                            Style::default()
                                .fg(theme::history::ITEM_SELECTED_BAR)
                                .bg(theme::history::ITEM_SELECTED_BG),
                        ),
                        Span::styled("  ", Style::default().bg(theme::history::ITEM_SELECTED_BG)),
                    ],
                )
            } else {
                let bg = if is_even {
                    theme::history::ITEM_NORMAL_BG_EVEN
                } else {
                    theme::history::ITEM_NORMAL_BG_ODD
                };
                (
                    bg,
                    Style::default().bg(bg),
                    vec![Span::styled("     ", Style::default().bg(bg))],
                )
            };

            let mut spans = prefix;

            // Syntax highlighting for all items
            let highlighted = JqHighlighter::highlight(&display_text);
            for span in highlighted {
                let style = if is_selected {
                    // Selected: bright syntax colors with bold
                    span.style
                        .bg(bg_color)
                        .add_modifier(theme::history::ITEM_SELECTED_MODIFIER)
                } else {
                    // Normal: dimmed syntax colors
                    span.style
                        .bg(bg_color)
                        .add_modifier(theme::history::SYNTAX_DIM_MODIFIER)
                };
                spans.push(Span::styled(span.content, style));
            }
            spans.push(Span::styled(" ", bg_style));

            list_items.push(ListItem::new(Line::from(spans)));
        }

        // Bottom padding
        list_items.push(ListItem::new(Line::from("")));

        list_items
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(title)
        .border_style(Style::default().fg(theme::history::BORDER))
        .style(Style::default().bg(theme::history::BACKGROUND));

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
        theme::history::SCROLLBAR,
    );

    let search_textarea = app.history.search_textarea_mut();
    search_textarea.set_block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(" Search ")
            .border_style(Style::default().fg(theme::history::BORDER))
            .style(Style::default().bg(theme::history::BACKGROUND)),
    );
    search_textarea.set_style(
        Style::default()
            .fg(theme::history::SEARCH_TEXT)
            .bg(theme::history::SEARCH_BG),
    );
    frame.render_widget(&*search_textarea, search_area);

    Some(popup_area)
}
