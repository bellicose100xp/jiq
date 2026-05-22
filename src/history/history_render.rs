use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Rect},
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

/// Width of the trailing ` [✕] ` button column rendered when an entry is hovered or selected.
const DELETE_BUTTON_WIDTH: u16 = 5;

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

    // Reserve space for the right-edge delete button on top of the existing
    // 6-cell padding (border + indicator + trailing space).
    let max_text_len = (list_area.width as usize)
        .saturating_sub(6)
        .saturating_sub(DELETE_BUTTON_WIDTH as usize);

    let items: Vec<ListItem> = if app.history.filtered_count() == 0 {
        vec![
            ListItem::new(Line::from("")),
            ListItem::new(Line::from(Span::styled(
                "  No matches",
                Style::default().fg(theme::history::NO_MATCHES),
            ))),
            ListItem::new(Line::from("")),
        ]
    } else {
        let mut list_items: Vec<ListItem> = Vec::new();

        // Top padding
        list_items.push(ListItem::new(Line::from("")));

        let hovered_index = app.history.hovered_index();

        for (display_idx, entry) in app.history.visible_entries() {
            let display_text = if entry.chars().count() > max_text_len {
                let truncated: String = entry.chars().take(max_text_len).collect();
                format!("{}…", truncated)
            } else {
                entry.to_string()
            };

            let is_selected = display_idx == app.history.selected_index();
            let is_hovered = hovered_index == Some(display_idx);

            let (bg_color, prefix) = if is_selected {
                (
                    theme::history::ITEM_SELECTED_BG,
                    vec![Span::styled(
                        " ▌ ",
                        Style::default()
                            .fg(theme::history::ITEM_SELECTED_INDICATOR)
                            .bg(theme::history::ITEM_SELECTED_BG),
                    )],
                )
            } else {
                (
                    theme::history::ITEM_NORMAL_BG,
                    vec![Span::styled(
                        "   ",
                        Style::default().bg(theme::history::ITEM_NORMAL_BG),
                    )],
                )
            };

            let mut spans = prefix;

            let highlighted = JqHighlighter::highlight(&display_text);
            let mut text_width: usize = 0;
            for span in highlighted {
                let style = if is_selected {
                    span.style.bg(bg_color)
                } else {
                    Style::default()
                        .fg(theme::history::ITEM_NORMAL_FG)
                        .bg(bg_color)
                };
                text_width += span.content.chars().count();
                spans.push(Span::styled(span.content, style));
            }

            // Right-align the delete column independently of List's truncation.
            let inner_width = list_area.width.saturating_sub(2) as usize;
            let used = 3 + text_width;
            let trailing_btn_width = DELETE_BUTTON_WIDTH as usize;
            let pad = inner_width
                .saturating_sub(used)
                .saturating_sub(trailing_btn_width);
            if pad > 0 {
                spans.push(Span::styled(" ".repeat(pad), Style::default().bg(bg_color)));
            }

            // Reserve the column even when invisible so the row layout is
            // stable as hover toggles on and off.
            let show_button = is_hovered || is_selected;
            if show_button {
                let btn_color = if is_hovered {
                    theme::history::DELETE_BUTTON_HOVER
                } else {
                    theme::history::DELETE_BUTTON
                };
                spans.push(Span::styled(
                    " [✕] ",
                    Style::default().fg(btn_color).bg(bg_color),
                ));
            } else {
                spans.push(Span::styled("     ", Style::default().bg(bg_color)));
            }

            list_items.push(ListItem::new(Line::from(spans)));
        }

        // Bottom padding
        list_items.push(ListItem::new(Line::from("")));

        list_items
    };

    let bottom_hints = theme::border_hints::build_hints(
        &[("Enter", "Select"), ("Ctrl+D", "Delete"), ("Esc", "Close")],
        theme::history::BORDER,
    );

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(title)
        .title_bottom(bottom_hints.alignment(Alignment::Center))
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

/// Resolves the display index for an entry at screen position `(x, y)`.
///
/// Returns `None` when the cursor is on padding rows, the search area, or
/// outside the popup. Display index matches [`HistoryState::visible_entries`].
pub fn display_index_at(app: &App, x: u16, y: u16) -> Option<usize> {
    let popup = app.layout_regions.history_popup?;
    if x < popup.x || x >= popup.x + popup.width {
        return None;
    }

    let visible_count = app.history.filtered_count().min(MAX_VISIBLE_HISTORY) as u16;
    if visible_count == 0 {
        return None;
    }

    let entries_top = popup.y.saturating_add(2);
    let entries_bottom = entries_top.saturating_add(visible_count);
    if y < entries_top || y >= entries_bottom {
        return None;
    }

    let row_in_list = (y - entries_top) as usize;
    let scroll = app.history.scroll_offset();
    let visible = visible_count as usize;
    Some(scroll + visible - 1 - row_in_list)
}

/// Returns the display index whose ` [✕] ` delete button is at `(x, y)`.
pub fn delete_button_at(app: &App, x: u16, y: u16) -> Option<usize> {
    let popup = app.layout_regions.history_popup?;
    let display_idx = display_index_at(app, x, y)?;

    let inner_right = popup.x.saturating_add(popup.width).saturating_sub(1);
    let btn_start = inner_right.saturating_sub(DELETE_BUTTON_WIDTH);
    if x < btn_start || x >= inner_right {
        return None;
    }

    Some(display_idx)
}

#[cfg(test)]
#[path = "history_render_tests.rs"]
mod history_render_tests;
