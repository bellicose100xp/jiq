use ratatui::{
    Frame,
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem},
};

use crate::app::App;
use crate::autocomplete::SuggestionType;
use crate::autocomplete::autocomplete_state::MAX_VISIBLE_SUGGESTIONS;
use crate::scroll::Scrollable;
use crate::theme;
use crate::widgets::{popup, scrollbar};

const MAX_POPUP_WIDTH: usize = 60;
const POPUP_BORDER_HEIGHT: u16 = 2;
const POPUP_PADDING: u16 = 4;
const POPUP_OFFSET_X: u16 = 2;
const TYPE_LABEL_SPACING: usize = 1;
const FIELD_PREFIX_LEN: usize = 2;

fn get_type_label(suggestion: &crate::autocomplete::Suggestion) -> String {
    match &suggestion.suggestion_type {
        SuggestionType::Field => {
            if let Some(field_type) = &suggestion.field_type {
                format!("[field: {}]", field_type)
            } else {
                format!("[{}]", suggestion.suggestion_type)
            }
        }
        _ => format!("[{}]", suggestion.suggestion_type),
    }
}

fn get_display_text(suggestion: &crate::autocomplete::Suggestion) -> &str {
    match suggestion.suggestion_type {
        SuggestionType::Function => suggestion.signature.as_deref().unwrap_or(&suggestion.text),
        _ => &suggestion.text,
    }
}

/// Render the autocomplete popup
///
/// Returns the popup area for region tracking.
pub fn render_popup(app: &App, frame: &mut Frame, input_area: Rect) -> Option<Rect> {
    let suggestions = app.autocomplete.suggestions();
    if suggestions.is_empty() {
        return None;
    }

    let visible_count = suggestions.len().min(MAX_VISIBLE_SUGGESTIONS);
    let popup_height = (visible_count as u16) + POPUP_BORDER_HEIGHT;

    let max_type_label_len = suggestions
        .iter()
        .map(|s| get_type_label(s).len())
        .max()
        .unwrap_or(0);

    let max_display_text_len = suggestions
        .iter()
        .map(|s| get_display_text(s).len())
        .max()
        .unwrap_or(0);

    let ideal_width =
        FIELD_PREFIX_LEN + max_display_text_len + TYPE_LABEL_SPACING + max_type_label_len;
    let content_width = ideal_width.min(MAX_POPUP_WIDTH);
    let popup_width = (content_width as u16) + POPUP_PADDING;

    let popup_area =
        popup::popup_above_anchor(input_area, popup_width, popup_height, POPUP_OFFSET_X);

    let available_for_text =
        content_width.saturating_sub(FIELD_PREFIX_LEN + TYPE_LABEL_SPACING + max_type_label_len);

    let items: Vec<ListItem> = app
        .autocomplete
        .visible_suggestions()
        .map(|(abs_idx, suggestion)| {
            let type_color = match suggestion.suggestion_type {
                SuggestionType::Function => theme::autocomplete::TYPE_FUNCTION,
                SuggestionType::Field => theme::autocomplete::TYPE_FIELD,
                SuggestionType::Operator => theme::autocomplete::TYPE_OPERATOR,
                SuggestionType::Pattern => theme::autocomplete::TYPE_PATTERN,
                SuggestionType::Variable => theme::autocomplete::TYPE_VARIABLE,
            };

            let type_label = get_type_label(suggestion);
            let display_text = get_display_text(suggestion);

            let truncated_text = if display_text.len() > available_for_text {
                format!(
                    "{}...",
                    &display_text[..available_for_text.saturating_sub(3)]
                )
            } else {
                display_text.to_string()
            };

            let padding_needed = available_for_text.saturating_sub(truncated_text.len());
            let padding = " ".repeat(padding_needed);

            let line = if abs_idx == app.autocomplete.selected_index() {
                Line::from(vec![
                    Span::styled(
                        format!("► {}{}", truncated_text, padding),
                        Style::default()
                            .fg(theme::autocomplete::ITEM_SELECTED_FG)
                            .bg(theme::autocomplete::ITEM_SELECTED_BG)
                            .add_modifier(theme::autocomplete::ITEM_SELECTED_MODIFIER),
                    ),
                    Span::styled(
                        format!(" {}", type_label),
                        Style::default()
                            .fg(theme::autocomplete::ITEM_SELECTED_FG)
                            .bg(theme::autocomplete::ITEM_SELECTED_BG),
                    ),
                ])
            } else {
                Line::from(vec![
                    Span::styled(
                        format!("  {}{}", truncated_text, padding),
                        Style::default()
                            .fg(theme::autocomplete::ITEM_NORMAL_FG)
                            .bg(theme::autocomplete::ITEM_NORMAL_BG),
                    ),
                    Span::styled(
                        format!(" {}", type_label),
                        Style::default()
                            .fg(type_color)
                            .bg(theme::autocomplete::ITEM_NORMAL_BG),
                    ),
                ])
            };

            ListItem::new(line)
        })
        .collect();

    popup::clear_area(frame, popup_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(" Suggestions ")
        .border_style(Style::default().fg(theme::autocomplete::BORDER))
        .style(Style::default().bg(theme::autocomplete::BACKGROUND));

    let list = List::new(items).block(block);
    frame.render_widget(list, popup_area);

    // Render scrollbar on border (excluding corners), matching border color
    let scrollbar_area = Rect {
        x: popup_area.x,
        y: popup_area.y.saturating_add(1),
        width: popup_area.width,
        height: popup_area.height.saturating_sub(2),
    };
    let total = app.autocomplete.suggestions().len();
    let viewport = app.autocomplete.viewport_size();
    let max_scroll = app.autocomplete.max_scroll();
    let clamped_offset = app.autocomplete.scroll_offset().min(max_scroll);
    scrollbar::render_vertical_scrollbar_styled(
        frame,
        scrollbar_area,
        total,
        viewport,
        clamped_offset,
        theme::autocomplete::SCROLLBAR,
    );

    Some(popup_area)
}

#[cfg(test)]
#[path = "autocomplete_render_tests.rs"]
mod autocomplete_render_tests;
