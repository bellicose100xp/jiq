//! AI popup rendering
//!
//! Renders the AI assistant popup on the right side of the results pane.
//! The popup displays AI responses for error troubleshooting and query help.

use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};

use super::ai_state::AiState;
use crate::widgets::popup;

// Use modules from render submodule instead of loading them directly
use super::render::layout;

// Re-export public items from sub-modules
pub use self::content::build_content;
pub use layout::{
    AUTOCOMPLETE_RESERVED_WIDTH, calculate_popup_area, calculate_popup_area_with_height,
};

// Module declarations - only content is local
#[path = "render/content.rs"]
mod content;

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

        // Calculate query lines
        let query_max_width = max_width.saturating_sub(prefix_len as u16) as usize;
        let query_lines = wrap_text(&suggestion.query, query_max_width);
        let mut suggestion_height = query_lines.len() as u16;

        // Calculate description lines
        if !suggestion.description.is_empty() {
            let desc_max_width = max_width.saturating_sub(3) as usize;
            let desc_lines = wrap_text(&suggestion.description, desc_max_width).len();
            suggestion_height = suggestion_height.saturating_add(desc_lines as u16);
        }

        // Add spacing line after each suggestion except the last
        if i < ai_state.suggestions.len() - 1 {
            suggestion_height = suggestion_height.saturating_add(1);
        }

        heights.push(suggestion_height);
    }

    heights
}

/// Calculate total height needed for suggestions (including spacing)
fn calculate_suggestions_height(ai_state: &AiState, max_width: u16) -> u16 {
    let heights = calculate_suggestion_heights(ai_state, max_width);
    heights.iter().sum::<u16>()
}

/// Render suggestions as individual widgets with background highlighting
fn render_suggestions_as_widgets(
    ai_state: &mut AiState,
    frame: &mut Frame,
    inner_area: Rect,
    max_width: u16,
) {
    use crate::ai::render::text::wrap_text;

    // Calculate heights and update selection state layout
    let heights = calculate_suggestion_heights(ai_state, max_width);
    ai_state
        .selection
        .update_layout(heights.clone(), inner_area.height);

    // Ensure selected suggestion is visible after layout update
    // This is necessary because navigation happens before layout is computed
    if ai_state.selection.get_selected().is_some() {
        ai_state.selection.ensure_selected_visible();
    }

    let scroll_offset = ai_state.selection.scroll_offset();
    let viewport_end = scroll_offset.saturating_add(inner_area.height);
    let selected_index = ai_state.selection.get_selected();

    // Track current Y position (in content space, not screen space)
    let mut current_y = 0u16;

    for (i, suggestion) in ai_state.suggestions.iter().enumerate() {
        let suggestion_height = heights[i];
        let suggestion_end = current_y.saturating_add(suggestion_height);

        // Skip if suggestion is fully above viewport
        if suggestion_end <= scroll_offset {
            current_y = suggestion_end;
            continue;
        }

        // Stop if suggestion starts below viewport
        if current_y >= viewport_end {
            break;
        }

        // Calculate render area in screen space
        let render_y = inner_area
            .y
            .saturating_add(current_y.saturating_sub(scroll_offset));
        let visible_height = suggestion_height.min(viewport_end.saturating_sub(current_y));

        let render_area = Rect {
            x: inner_area.x,
            y: render_y,
            width: inner_area.width,
            height: visible_height,
        };

        // Build suggestion lines
        let mut lines: Vec<Line> = Vec::new();
        let is_selected = selected_index == Some(i);

        let type_color = suggestion.suggestion_type.color();
        let type_label = suggestion.suggestion_type.label();
        let has_selection_number = i < 5;

        let prefix = if has_selection_number {
            format!("{}. {} ", i + 1, type_label)
        } else {
            format!("{} ", type_label)
        };
        let prefix_len = prefix.len();

        // Main line with query
        let query_max_width = max_width.saturating_sub(prefix_len as u16) as usize;
        let query_lines = wrap_text(&suggestion.query, query_max_width);

        if let Some(first_query_line) = query_lines.first() {
            let mut spans = Vec::new();

            if has_selection_number {
                let style = if is_selected {
                    Style::default().fg(Color::Black)
                } else {
                    Style::default().fg(Color::DarkGray)
                };
                spans.push(Span::styled(format!("{}. ", i + 1), style));
            }

            let type_style = Style::default().fg(type_color).add_modifier(Modifier::BOLD);
            spans.push(Span::styled(type_label.to_string(), type_style));
            spans.push(Span::styled(" ", Style::default()));

            let query_style = Style::default().fg(Color::Cyan);
            spans.push(Span::styled(first_query_line.clone(), query_style));

            lines.push(Line::from(spans));
        }

        // Wrapped query lines
        for query_line in query_lines.iter().skip(1) {
            let indent = " ".repeat(prefix_len);
            let style = Style::default().fg(Color::Cyan);
            lines.push(Line::from(Span::styled(
                format!("{}{}", indent, query_line),
                style,
            )));
        }

        // Description lines
        if !suggestion.description.is_empty() {
            let desc_max_width = max_width.saturating_sub(3) as usize;
            for desc_line in wrap_text(&suggestion.description, desc_max_width) {
                let style = if is_selected {
                    Style::default().fg(Color::Gray)
                } else {
                    Style::default().fg(Color::DarkGray)
                };
                lines.push(Line::from(Span::styled(format!("   {}", desc_line), style)));
            }
        }

        // Add spacing line after each suggestion except the last
        if i < ai_state.suggestions.len() - 1 {
            lines.push(Line::from(""));
        }

        // Render the suggestion
        let style = if is_selected {
            Style::default().bg(Color::DarkGray)
        } else {
            Style::default()
        };

        let paragraph = Paragraph::new(lines).style(style);
        frame.render_widget(paragraph, render_area);

        // Move to next suggestion
        current_y = suggestion_end;
    }
}

/// Render the AI assistant popup
///
/// # Arguments
/// * `ai_state` - The current AI state
/// * `frame` - The frame to render to
/// * `input_area` - The input bar area (popup renders above this)
pub fn render_popup(ai_state: &mut AiState, frame: &mut Frame, input_area: Rect) {
    if !ai_state.visible {
        return;
    }

    let frame_area = frame.area();

    // For suggestions, calculate height dynamically and position at bottom
    let has_suggestions = !ai_state.suggestions.is_empty()
        && ai_state.configured
        && !ai_state.loading
        && ai_state.error.is_none();

    let popup_area = if has_suggestions {
        // Pre-calculate content height for suggestions
        let max_content_width = frame_area
            .width
            .saturating_sub(AUTOCOMPLETE_RESERVED_WIDTH)
            .saturating_sub(4); // Account for borders
        let content_height = calculate_suggestions_height(ai_state, max_content_width);
        let area = match calculate_popup_area_with_height(frame_area, input_area, content_height) {
            Some(area) => area,
            None => return,
        };
        // Store the height for use during loading transitions
        ai_state.previous_popup_height = Some(area.height);
        area
    } else if let Some(prev_height) = ai_state.previous_popup_height {
        // Use previous height to maintain size during loading/transitions
        match calculate_popup_area_with_height(
            frame_area,
            input_area,
            prev_height.saturating_sub(4),
        ) {
            Some(area) => area,
            None => {
                // Fallback to default sizing if previous height doesn't fit
                match calculate_popup_area(frame_area, input_area) {
                    Some(area) => area,
                    None => return,
                }
            }
        }
    } else {
        // No previous height - use default sizing
        match calculate_popup_area(frame_area, input_area) {
            Some(area) => area,
            None => return,
        }
    };

    popup::clear_area(frame, popup_area);

    let title = Line::from(vec![
        Span::raw(" "),
        Span::styled(
            &ai_state.provider_name,
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" "),
    ]);

    let counter = if ai_state.suggestions.len() > 1 {
        let current = ai_state
            .selection
            .get_selected()
            .map(|i| i + 1)
            .unwrap_or(1);
        let total = ai_state.suggestions.len();
        Line::from(Span::styled(
            format!(" ({}/{}) ", current, total),
            Style::default().fg(Color::Yellow),
        ))
    } else {
        Line::default()
    };

    let counter_width = if ai_state.suggestions.len() > 1 {
        let current = ai_state
            .selection
            .get_selected()
            .map(|i| i + 1)
            .unwrap_or(1);
        let total = ai_state.suggestions.len();
        format!(" ({}/{}) ", current, total).len() as u16
    } else {
        0
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
        Span::styled(model_display, Style::default().fg(Color::Blue)),
        Span::raw(" "),
    ]);

    let hints = if !ai_state.suggestions.is_empty() {
        Line::from(vec![Span::styled(
            " Alt+1-5 or Alt+↑↓+Enter to apply | Ctrl+A to close ",
            Style::default().fg(Color::DarkGray),
        )])
    } else {
        Line::from(vec![Span::styled(
            " Ctrl+A to close ",
            Style::default().fg(Color::DarkGray),
        )])
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .title_top(counter.alignment(ratatui::layout::Alignment::Center))
        .title_top(model_name_title.alignment(ratatui::layout::Alignment::Right))
        .title_bottom(hints.alignment(ratatui::layout::Alignment::Center))
        .border_style(Style::default().fg(Color::Magenta))
        .style(Style::default().bg(Color::Black));

    // Check if we have suggestions - use widget-based rendering for better backgrounds
    if has_suggestions {
        // Render the border block first
        frame.render_widget(block.clone(), popup_area);

        // Get inner area and render suggestions as individual widgets
        let inner_area = block.inner(popup_area);
        let max_width = inner_area.width;
        render_suggestions_as_widgets(ai_state, frame, inner_area, max_width);
    } else {
        // Use traditional content-based rendering for non-suggestion content
        let content = build_content(ai_state, popup_area.width.saturating_sub(4));
        let popup_widget = Paragraph::new(content)
            .wrap(Wrap { trim: false })
            .block(block);
        frame.render_widget(popup_widget, popup_area);
    }
}
