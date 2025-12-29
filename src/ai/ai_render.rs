//! AI popup rendering
//!
//! Renders the AI assistant popup on the right side of the results pane.
//! The popup displays AI responses for error troubleshooting and query help.

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
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

/// Calculate total height needed for suggestions
fn calculate_suggestions_height(ai_state: &AiState, max_width: u16) -> u16 {
    use crate::ai::render::text::wrap_text;

    let mut total_height = 0u16;

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
        total_height = total_height.saturating_add(query_lines.len() as u16);

        // Calculate description lines
        if !suggestion.description.is_empty() {
            let desc_max_width = max_width.saturating_sub(3) as usize;
            let desc_lines = wrap_text(&suggestion.description, desc_max_width).len();
            total_height = total_height.saturating_add(desc_lines as u16);
        }

        // Add spacing between suggestions (except after last one)
        if i < ai_state.suggestions.len() - 1 {
            total_height = total_height.saturating_add(1);
        }
    }

    total_height
}

/// Render suggestions as individual widgets with background highlighting
fn render_suggestions_as_widgets(
    ai_state: &AiState,
    frame: &mut Frame,
    inner_area: Rect,
    max_width: u16,
) {
    use crate::ai::render::text::wrap_text;

    // Pre-calculate lines and heights for each suggestion
    let mut suggestion_blocks: Vec<(Vec<Line<'static>>, bool)> = Vec::new();
    let selected_index = ai_state.selection.get_selected();

    for (i, suggestion) in ai_state.suggestions.iter().enumerate() {
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

        suggestion_blocks.push((lines, is_selected));
    }

    // Calculate layout constraints: each suggestion + 1 line spacing after each (except last)
    // Use Length for exact sizing to avoid extra space allocation
    let mut constraints: Vec<Constraint> = Vec::new();
    for (i, (lines, _)) in suggestion_blocks.iter().enumerate() {
        // Add constraint for the suggestion content - use Length for exact height
        constraints.push(Constraint::Length(lines.len() as u16));
        // Add 1-line spacing after each suggestion except the last
        if i < suggestion_blocks.len() - 1 {
            constraints.push(Constraint::Length(1));
        }
    }

    // Create layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(inner_area);

    // Render each suggestion in its chunk (skipping spacing chunks)
    let mut chunk_idx = 0;
    for (lines, is_selected) in suggestion_blocks {
        // Skip if chunk has zero height (layout ran out of space)
        if chunk_idx >= chunks.len() || chunks[chunk_idx].height == 0 {
            chunk_idx += 2; // Skip this suggestion and its spacing
            continue;
        }

        let style = if is_selected {
            Style::default().bg(Color::DarkGray)
        } else {
            Style::default()
        };

        let paragraph = Paragraph::new(lines).style(style);

        frame.render_widget(paragraph, chunks[chunk_idx]);
        chunk_idx += 2; // Move to next suggestion (skip spacing chunk)
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
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" "),
    ]);

    // Calculate max width for model name (50% of border width)
    let max_model_width = (popup_area.width / 2).saturating_sub(2);
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
        .title_top(model_name_title.alignment(ratatui::layout::Alignment::Right))
        .title_bottom(hints.alignment(ratatui::layout::Alignment::Center))
        .border_style(Style::default().fg(Color::Green))
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
