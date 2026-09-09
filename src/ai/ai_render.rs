//! AI popup rendering
//!
//! Renders the AI assistant popup above the input bar. From top to bottom the
//! popup shows the dimmed transcript of earlier questions, the active
//! response (prose answer and numbered suggestions), and a chat input row
//! where the user types the next question.

use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
};

use super::ai_state::AiState;
use crate::scroll::Scrollable;
use crate::theme;
use crate::widgets::{popup, scrollbar};

const HORIZONTAL_PADDING: u16 = 1;
const VERTICAL_PADDING: u16 = 1;
/// Rows at the bottom of the popup taken by the chat input: a separator and
/// the input line itself.
pub const CHAT_INPUT_ROWS: u16 = 2;
/// Marker in front of the chat input.
const CHAT_PROMPT: &str = "› ";

use super::render::layout;

pub use self::content::build_header_lines;
pub use layout::{calculate_popup_area_with_height, popup_width};

#[path = "render/content.rs"]
mod content;

/// Geometry of the popup's inner regions, derived from the popup rectangle.
///
/// Mouse handlers use this to tell a click on a suggestion from a click on
/// the chat input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PopupRegions {
    /// Scrollable transcript + suggestions area
    pub content: Rect,
    /// Chat input row (below its separator)
    pub chat_input: Rect,
}

/// Split a popup rectangle into its content area and chat input row.
pub fn popup_regions(popup_area: Rect) -> PopupRegions {
    let inner = Rect {
        x: popup_area.x.saturating_add(1),
        y: popup_area.y.saturating_add(1),
        width: popup_area.width.saturating_sub(2),
        height: popup_area.height.saturating_sub(2),
    };
    let padded = popup::inset_rect(inner, HORIZONTAL_PADDING, VERTICAL_PADDING);
    let content_height = padded.height.saturating_sub(CHAT_INPUT_ROWS);
    let content = Rect {
        height: content_height,
        ..padded
    };
    let chat_input = Rect {
        y: padded.y.saturating_add(content_height).saturating_add(1),
        height: padded.height.saturating_sub(content_height).min(1),
        ..padded
    };
    PopupRegions {
        content,
        chat_input,
    }
}

/// Row of the popup content (0-based, before scrolling) under a screen row,
/// or `None` when the row is outside the content area.
pub fn content_row_at(popup_area: Rect, row: u16) -> Option<u16> {
    let content = popup_regions(popup_area).content;
    if row >= content.y && row < content.y.saturating_add(content.height) {
        Some(row - content.y)
    } else {
        None
    }
}

/// Whether a screen row falls on the chat input (or its separator).
pub fn is_chat_input_row(popup_area: Rect, row: u16) -> bool {
    let regions = popup_regions(popup_area);
    let separator_y = regions.chat_input.y.saturating_sub(1);
    row >= separator_y
        && row
            < regions
                .chat_input
                .y
                .saturating_add(regions.chat_input.height)
}

/// Calculate height of each suggestion
///
/// Returns a vector where each element is the height (in lines) of the
/// corresponding suggestion, including spacing line after each (except last).
fn calculate_suggestion_heights(ai_state: &AiState, max_width: u16) -> Vec<u16> {
    use crate::ai::render::text::wrap_text;

    let mut heights = Vec::with_capacity(ai_state.suggestions.len());

    for (i, suggestion) in ai_state.suggestions.iter().enumerate() {
        let type_label = suggestion.suggestion_type.label();
        let has_selection_number = i < 5;

        let prefix = if has_selection_number {
            format!("{}. {} ", i + 1, type_label)
        } else {
            format!("{} ", type_label)
        };
        let prefix_len = prefix.len();

        let query_max_width = max_width.saturating_sub(prefix_len as u16) as usize;
        let query_lines = wrap_text(&suggestion.query, query_max_width);
        let mut suggestion_height = query_lines.len() as u16;

        if !suggestion.description.is_empty() {
            let desc_max_width = max_width.saturating_sub(3) as usize;
            let desc_lines = wrap_text(&suggestion.description, desc_max_width).len();
            suggestion_height = suggestion_height.saturating_add(desc_lines as u16);
        }

        if i < ai_state.suggestions.len() - 1 {
            suggestion_height = suggestion_height.saturating_add(1);
        }

        heights.push(suggestion_height);
    }

    heights
}

/// Whether the active response's suggestions should be rendered as the
/// selectable list.
fn shows_suggestions(ai_state: &AiState) -> bool {
    !ai_state.suggestions.is_empty()
        && ai_state.configured
        && !ai_state.loading
        && ai_state.error.is_none()
}

/// Render suggestions as individual widgets with background highlighting,
/// starting `header_height` lines into the scrollable content.
fn render_suggestions_as_widgets(
    ai_state: &AiState,
    frame: &mut Frame,
    inner_area: Rect,
    max_width: u16,
    heights: &[u16],
) {
    use crate::ai::render::text::wrap_text;

    let scroll_offset = ai_state.selection.scroll_offset_u16();
    let viewport_end = scroll_offset.saturating_add(inner_area.height);
    let selected_index = ai_state.selection.get_selected();
    let hovered_index = ai_state.selection.get_hovered();

    // Y position in content space (not screen space)
    let mut current_y = ai_state.selection.header_height();

    for (i, suggestion) in ai_state.suggestions.iter().enumerate() {
        let suggestion_height = heights[i];
        let suggestion_end = current_y.saturating_add(suggestion_height);

        if suggestion_end <= scroll_offset {
            current_y = suggestion_end;
            continue;
        }

        if current_y >= viewport_end {
            break;
        }

        let render_y = inner_area
            .y
            .saturating_add(current_y.saturating_sub(scroll_offset));

        let visible_start = current_y.max(scroll_offset);
        let visible_end = suggestion_end.min(viewport_end);
        let visible_height = visible_end.saturating_sub(visible_start);

        let render_area = Rect {
            x: inner_area.x,
            y: render_y,
            width: inner_area.width,
            height: visible_height,
        };

        let mut lines: Vec<Line> = Vec::new();
        let is_selected = selected_index == Some(i);
        let is_hovered = hovered_index == Some(i) && !is_selected;

        let type_color = suggestion.suggestion_type.color();
        let type_label = suggestion.suggestion_type.label();
        let has_selection_number = i < 5;

        let prefix = if has_selection_number {
            format!("{}. {} ", i + 1, type_label)
        } else {
            format!("{} ", type_label)
        };
        let prefix_len = prefix.len();

        let query_max_width = max_width.saturating_sub(prefix_len as u16) as usize;
        let query_lines = wrap_text(&suggestion.query, query_max_width);

        if let Some(first_query_line) = query_lines.first() {
            let mut spans = Vec::new();

            if has_selection_number {
                let style = if is_selected {
                    Style::default().fg(theme::ai::suggestion_text_selected())
                } else {
                    Style::default().fg(theme::ai::suggestion_text_normal())
                };
                spans.push(Span::styled(format!("{}. ", i + 1), style));
            }

            let type_style = Style::default().fg(type_color).add_modifier(Modifier::BOLD);
            spans.push(Span::styled(type_label.to_string(), type_style));
            spans.push(Span::styled(" ", Style::default()));

            let query_style = Style::default().fg(theme::ai::query_text());
            spans.push(Span::styled(first_query_line.clone(), query_style));

            lines.push(Line::from(spans));
        }

        for query_line in query_lines.iter().skip(1) {
            let indent = " ".repeat(prefix_len);
            let style = Style::default().fg(theme::ai::query_text());
            lines.push(Line::from(Span::styled(
                format!("{}{}", indent, query_line),
                style,
            )));
        }

        if !suggestion.description.is_empty() {
            let desc_max_width = max_width.saturating_sub(3) as usize;
            for desc_line in wrap_text(&suggestion.description, desc_max_width) {
                let style = if is_selected {
                    Style::default().fg(theme::ai::suggestion_desc_muted())
                } else {
                    Style::default().fg(theme::ai::suggestion_desc_normal())
                };
                lines.push(Line::from(Span::styled(format!("   {}", desc_line), style)));
            }
        }

        if i < ai_state.suggestions.len() - 1 {
            lines.push(Line::from(""));
        }

        let style = if is_selected {
            Style::default().bg(theme::ai::suggestion_selected_bg())
        } else if is_hovered {
            Style::default().bg(theme::ai::suggestion_hovered_bg())
        } else {
            Style::default()
        };

        let line_scroll_offset = if current_y < scroll_offset {
            scroll_offset.saturating_sub(current_y)
        } else {
            0
        };

        let paragraph = Paragraph::new(lines)
            .style(style)
            .scroll((line_scroll_offset, 0));
        frame.render_widget(paragraph, render_area);

        current_y = suggestion_end;
    }
}

/// Render the chat separator and input row at the bottom of the popup.
fn render_chat_input(ai_state: &AiState, frame: &mut Frame, padded_area: Rect) {
    let regions_input = Rect {
        x: padded_area.x,
        y: padded_area
            .y
            .saturating_add(padded_area.height.saturating_sub(1)),
        width: padded_area.width,
        height: 1,
    };
    let separator = Rect {
        y: regions_input.y.saturating_sub(1),
        ..regions_input
    };

    if padded_area.height < CHAT_INPUT_ROWS {
        return;
    }

    let rule = "─".repeat(separator.width as usize);
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            rule,
            Style::default().fg(theme::ai::chat_separator()),
        ))),
        separator,
    );

    let prompt_width = CHAT_PROMPT.chars().count() as u16;
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            CHAT_PROMPT,
            Style::default()
                .fg(theme::ai::chat_prompt())
                .add_modifier(Modifier::BOLD),
        ))),
        Rect {
            width: prompt_width.min(regions_input.width),
            ..regions_input
        },
    );

    let input_rect = Rect {
        x: regions_input.x.saturating_add(prompt_width),
        width: regions_input.width.saturating_sub(prompt_width),
        ..regions_input
    };
    if input_rect.width > 0 {
        frame.render_widget(&ai_state.chat_input, input_rect);
    }
}

/// Bottom-border hints for the current popup state.
fn build_hints(ai_state: &AiState, chat_focused: bool) -> Line<'static> {
    let has_suggestions = !ai_state.suggestions.is_empty();
    let mut hints: Vec<(&'static str, &'static str)> = Vec::new();
    if has_suggestions {
        hints.push(("Alt+1-5", "Apply"));
        hints.push(("Alt+↑↓", "Select"));
    }
    if chat_focused {
        hints.push(("Enter", "Ask"));
        hints.push(("Esc", "Back"));
        hints.push(("Ctrl+L", "Clear"));
    } else if has_suggestions {
        hints.push(("Enter", "Apply Selection"));
    }
    hints.push(("Ctrl+A", "Close"));
    theme::border_hints::build_hints(&hints, theme::ai::border())
}

/// Render the AI assistant popup
///
/// Returns the popup area for region tracking.
///
/// # Arguments
/// * `ai_state` - The current AI state
/// * `frame` - The frame to render to
/// * `input_area` - The input bar area (popup renders above this)
/// * `chat_focused` - Whether keyboard focus is on the popup's chat input
pub fn render_popup(
    ai_state: &mut AiState,
    frame: &mut Frame,
    input_area: Rect,
    chat_focused: bool,
) -> Option<Rect> {
    if !ai_state.visible {
        return None;
    }

    let frame_area = frame.area();

    // Wrap to the width the content will really have: popup minus borders
    // and padding
    let max_content_width = popup_width(frame_area)?.saturating_sub(2 + HORIZONTAL_PADDING * 2);

    let header_lines = build_header_lines(ai_state, max_content_width);
    let header_height = header_lines.len() as u16;
    let show_suggestions = shows_suggestions(ai_state);
    let heights = if show_suggestions {
        calculate_suggestion_heights(ai_state, max_content_width)
    } else {
        Vec::new()
    };
    let suggestions_height: u16 = heights.iter().sum();

    let mut content_height = header_height
        .saturating_add(suggestions_height)
        .saturating_add(CHAT_INPUT_ROWS);
    if ai_state.loading
        && let Some(prev) = ai_state.previous_popup_height
    {
        // Keep the popup from collapsing while the next response streams in
        content_height = content_height.max(prev.saturating_sub(4));
    }
    let popup_area = calculate_popup_area_with_height(frame_area, input_area, content_height)?;
    if !ai_state.loading {
        ai_state.previous_popup_height = Some(popup_area.height);
    }

    popup::clear_area(frame, popup_area);

    let title = Line::from(vec![
        Span::raw(" "),
        Span::styled(&ai_state.provider_name, theme::ai::title()),
        Span::raw(" "),
    ]);

    let counter_text = if show_suggestions && ai_state.suggestions.len() > 1 {
        let current = ai_state
            .selection
            .get_selected()
            .map(|i| i + 1)
            .unwrap_or(1);
        format!(" ({}/{}) ", current, ai_state.suggestions.len())
    } else {
        String::new()
    };
    let counter_width = counter_text.len() as u16;
    let counter = if counter_text.is_empty() {
        Line::default()
    } else {
        Line::from(Span::styled(
            counter_text,
            Style::default().fg(theme::ai::counter()),
        ))
    };

    let max_model_width = (popup_area.width / 2)
        .saturating_sub(2)
        .saturating_sub(counter_width / 2);
    let model_display = if ai_state.model_name.len() > max_model_width as usize {
        format!(
            "{}...",
            &ai_state.model_name[..max_model_width.saturating_sub(3) as usize]
        )
    } else {
        ai_state.model_name.clone()
    };

    let model_name_title = Line::from(vec![
        Span::raw(" "),
        Span::styled(
            model_display,
            Style::default().fg(theme::ai::model_display()),
        ),
        Span::raw(" "),
    ]);

    let hints = build_hints(ai_state, chat_focused);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(title)
        .title_top(counter.alignment(ratatui::layout::Alignment::Center))
        .title_top(model_name_title.alignment(ratatui::layout::Alignment::Right))
        .title_bottom(hints.alignment(ratatui::layout::Alignment::Center))
        .border_style(Style::default().fg(theme::ai::border()))
        .style(Style::default().bg(theme::ai::background()));

    frame.render_widget(block.clone(), popup_area);

    let inner_area = block.inner(popup_area);
    let padded_area = popup::inset_rect(inner_area, HORIZONTAL_PADDING, VERTICAL_PADDING);
    let content_area = popup_regions(popup_area).content;

    ai_state.selection.update_layout_with_header(
        header_height,
        heights.clone(),
        content_area.height,
    );
    if ai_state.selection.get_selected().is_some() {
        ai_state.selection.ensure_selected_visible();
    }

    let scroll_offset = ai_state.selection.scroll_offset_u16();
    if header_height > scroll_offset && content_area.height > 0 {
        let visible = header_height
            .saturating_sub(scroll_offset)
            .min(content_area.height);
        let header_area = Rect {
            height: visible,
            ..content_area
        };
        frame.render_widget(
            Paragraph::new(header_lines).scroll((scroll_offset, 0)),
            header_area,
        );
    }

    if show_suggestions {
        render_suggestions_as_widgets(ai_state, frame, content_area, padded_area.width, &heights);
    }

    let total_content_height = ai_state.selection.total_content_height() as usize;
    let viewport = ai_state.selection.viewport_size();
    if total_content_height > viewport {
        let scrollbar_area = Rect {
            x: popup_area.x,
            y: popup_area.y.saturating_add(1),
            width: popup_area.width,
            height: content_area.height.saturating_add(VERTICAL_PADDING * 2),
        };
        let max_scroll = ai_state.selection.max_scroll();
        let clamped_offset = ai_state.selection.scroll_offset().min(max_scroll);
        scrollbar::render_vertical_scrollbar_styled(
            frame,
            scrollbar_area,
            total_content_height,
            viewport,
            clamped_offset,
            theme::ai::scrollbar(),
        );
    }

    render_chat_input(ai_state, frame, padded_area);

    Some(popup_area)
}
