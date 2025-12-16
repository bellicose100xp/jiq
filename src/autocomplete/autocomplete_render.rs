use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
};

use crate::app::App;
use crate::autocomplete::SuggestionType;
use crate::widgets::popup;

const MAX_VISIBLE_SUGGESTIONS: usize = 10;
const MAX_POPUP_WIDTH: usize = 60;
const POPUP_BORDER_HEIGHT: u16 = 2;
const POPUP_PADDING: u16 = 4;
const POPUP_OFFSET_X: u16 = 2;
const TYPE_LABEL_SPACING: usize = 3;

pub fn render_popup(app: &App, frame: &mut Frame, input_area: Rect) {
    let suggestions = app.autocomplete.suggestions();
    if suggestions.is_empty() {
        return;
    }

    let visible_count = suggestions.len().min(MAX_VISIBLE_SUGGESTIONS);
    let popup_height = (visible_count as u16) + POPUP_BORDER_HEIGHT;
    let max_text_width = suggestions
        .iter()
        .map(|s| {
            let display_text_len = match s.suggestion_type {
                SuggestionType::Function => s
                    .signature
                    .as_ref()
                    .map(|sig| sig.len())
                    .unwrap_or(s.text.len()),
                _ => s.text.len(),
            };

            let type_label_len = match &s.suggestion_type {
                SuggestionType::Field => {
                    if let Some(field_type) = &s.field_type {
                        9 + field_type.to_string().len()
                    } else {
                        7
                    }
                }
                _ => s.suggestion_type.to_string().len() + 2,
            };
            display_text_len + type_label_len + TYPE_LABEL_SPACING
        })
        .max()
        .unwrap_or(20)
        .min(MAX_POPUP_WIDTH);
    let popup_width = (max_text_width as u16) + POPUP_PADDING;

    let popup_area =
        popup::popup_above_anchor(input_area, popup_width, popup_height, POPUP_OFFSET_X);
    let max_display_width = suggestions
        .iter()
        .take(MAX_VISIBLE_SUGGESTIONS)
        .map(|s| match s.suggestion_type {
            SuggestionType::Function => s
                .signature
                .as_ref()
                .map(|sig| sig.len())
                .unwrap_or(s.text.len()),
            _ => s.text.len(),
        })
        .max()
        .unwrap_or(0);

    let items: Vec<ListItem> = suggestions
        .iter()
        .take(MAX_VISIBLE_SUGGESTIONS)
        .enumerate()
        .map(|(i, suggestion)| {
            let type_color = match suggestion.suggestion_type {
                SuggestionType::Function => Color::Yellow,
                SuggestionType::Field => Color::Cyan,
                SuggestionType::Operator => Color::Magenta,
                SuggestionType::Pattern => Color::Green,
            };

            let type_label = match &suggestion.suggestion_type {
                SuggestionType::Field => {
                    if let Some(field_type) = &suggestion.field_type {
                        format!("[field: {}]", field_type)
                    } else {
                        format!("[{}]", suggestion.suggestion_type)
                    }
                }
                _ => format!("[{}]", suggestion.suggestion_type),
            };

            let display_text = match suggestion.suggestion_type {
                SuggestionType::Function => {
                    suggestion.signature.as_deref().unwrap_or(&suggestion.text)
                }
                _ => &suggestion.text,
            };

            let padding_needed = max_display_width.saturating_sub(display_text.len());
            let padding = " ".repeat(padding_needed);

            let line = if i == app.autocomplete.selected_index() {
                Line::from(vec![
                    Span::styled(
                        format!("► {} {}", display_text, padding),
                        Style::default()
                            .fg(Color::Black)
                            .bg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        format!(" {}", type_label),
                        Style::default().fg(Color::Black).bg(Color::Cyan),
                    ),
                ])
            } else {
                Line::from(vec![
                    Span::styled(
                        format!("  {} {}", display_text, padding),
                        Style::default().fg(Color::White).bg(Color::Black),
                    ),
                    Span::styled(
                        format!(" {}", type_label),
                        Style::default().fg(type_color).bg(Color::Black),
                    ),
                ])
            };

            ListItem::new(line)
        })
        .collect();

    popup::clear_area(frame, popup_area);

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Suggestions ")
            .border_style(Style::default().fg(Color::Cyan))
            .style(Style::default().bg(Color::Black)),
    );

    frame.render_widget(list, popup_area);
}
