//! Input field rendering
//!
//! This module handles rendering of the query input field.

use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::{App, Focus};
use crate::editor::EditorMode;
use crate::syntax_highlight::JqHighlighter;
use crate::syntax_highlight::overlay::{extract_visible_spans, insert_cursor_into_spans};

/// Render the input field (bottom)
pub fn render_field(app: &mut App, frame: &mut Frame, area: Rect) {
    // Calculate viewport width (inside borders) and update scroll offset
    let viewport_width = area.width.saturating_sub(2) as usize;
    app.input.calculate_scroll_offset(viewport_width);

    // Choose color based on mode
    let mode_color = match app.input.editor_mode {
        EditorMode::Insert => Color::Cyan,        // Cyan for Insert
        EditorMode::Normal => Color::Yellow,      // Yellow for Normal
        EditorMode::Operator(_) => Color::Green,  // Green for Operator
    };

    // Set border color - mode color when focused, gray when unfocused
    let border_color = if app.focus == Focus::InputField {
        mode_color
    } else {
        Color::DarkGray
    };

    // Build title with colored mode indicator and hint
    let mode_text = app.input.editor_mode.display();
    let mut title_spans = match app.input.editor_mode {
        EditorMode::Normal => {
            vec![
                Span::raw(" Query ["),
                Span::styled(mode_text, Style::default().fg(mode_color)),
                Span::raw("] (press 'i' to edit) "),
            ]
        }
        _ => {
            vec![
                Span::raw(" Query ["),
                Span::styled(mode_text, Style::default().fg(mode_color)),
                Span::raw("] "),
            ]
        }
    };

    // Add error indicator if there's an error
    if app.query.result.is_err() {
        title_spans.push(Span::styled(
            "⚠ Syntax Error (Ctrl+E to view)",
            Style::default().fg(Color::Yellow),
        ));
    }

    let title = Line::from(title_spans);

    // Build tooltip hint for top-right of input border
    // Show hint when tooltip is disabled AND cursor is on a function
    let tooltip_hint = if !app.tooltip.enabled && app.tooltip.current_function.is_some() {
        Some(Line::from(vec![Span::styled(
            " Ctrl+T for tooltip ",
            Style::default().fg(Color::Magenta),
        )]))
    } else {
        None
    };

    // Create block with mode-aware styling
    let mut block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(Style::default().fg(border_color));

    // Add tooltip hint to top-right if applicable
    if let Some(hint) = tooltip_hint {
        block = block.title_top(hint.alignment(Alignment::Right));
    }

    // Get query text and render with syntax highlighting + cursor
    let query = app.query();
    let cursor_col = app.input.textarea.cursor().1;
    let scroll_offset = app.input.scroll_offset;

    if query.is_empty() {
        // Empty query - just show cursor
        let cursor_spans = insert_cursor_into_spans(vec![], 0);
        let paragraph = Paragraph::new(Line::from(cursor_spans)).block(block);
        frame.render_widget(paragraph, area);
    } else {
        // Render styled text with cursor
        let highlighted_spans = JqHighlighter::highlight(query);

        // Extract visible portion based on scroll offset
        let visible_spans = extract_visible_spans(
            &highlighted_spans,
            scroll_offset,
            viewport_width,
        );

        // Insert cursor at the correct position (relative to visible area)
        let cursor_in_viewport = cursor_col.saturating_sub(scroll_offset);
        let spans_with_cursor = insert_cursor_into_spans(visible_spans, cursor_in_viewport);

        let paragraph = Paragraph::new(Line::from(spans_with_cursor)).block(block);
        frame.render_widget(paragraph, area);
    }
}
