use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use crate::app::{App, Focus};
use crate::editor::EditorMode;
use crate::syntax_highlight::JqHighlighter;
use crate::syntax_highlight::overlay::{extract_visible_spans, insert_cursor_into_spans};

pub fn render_field(app: &mut App, frame: &mut Frame, area: Rect) {
    let viewport_width = area.width.saturating_sub(2) as usize;
    app.input.calculate_scroll_offset(viewport_width);

    let mode_color = match app.input.editor_mode {
        EditorMode::Insert => Color::Cyan,
        EditorMode::Normal => Color::Yellow,
        EditorMode::Operator(_) => Color::Green,
        EditorMode::CharSearch(_, _) => Color::Magenta,
        EditorMode::TextObject(_, _) => Color::Green,
    };

    let border_color = if app.focus == Focus::InputField {
        mode_color
    } else {
        Color::DarkGray
    };

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

    if let Some(query) = &app.query
        && query.result.is_err()
    {
        title_spans.push(Span::styled(
            "⚠ Syntax Error (Ctrl+E to view)",
            Style::default().fg(Color::Yellow),
        ));
    }

    let title = Line::from(title_spans);

    let tooltip_hint = if !app.tooltip.enabled && app.tooltip.current_function.is_some() {
        Some(Line::from(vec![Span::styled(
            " Ctrl+T for tooltip ",
            Style::default().fg(Color::Magenta),
        )]))
    } else {
        None
    };

    let ai_hint = if !app.ai.visible {
        Some(Line::from(vec![Span::styled(
            " Press Ctrl+A for AI Assistant ",
            Style::default().fg(Color::Cyan),
        )]))
    } else {
        None
    };

    let mut block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(Style::default().fg(border_color));

    if let Some(hint) = tooltip_hint {
        block = block.title_top(hint.alignment(Alignment::Right));
    } else if let Some(hint) = ai_hint {
        block = block.title_top(hint.alignment(Alignment::Right));
    }

    let query = app.query();
    let cursor_col = app.input.textarea.cursor().1;
    let scroll_offset = app.input.scroll_offset;

    if query.is_empty() {
        let cursor_spans = insert_cursor_into_spans(vec![], 0);
        let paragraph = Paragraph::new(Line::from(cursor_spans)).block(block);
        frame.render_widget(paragraph, area);
    } else {
        let highlighted_spans = JqHighlighter::highlight(query);
        let visible_spans =
            extract_visible_spans(&highlighted_spans, scroll_offset, viewport_width);
        let cursor_in_viewport = cursor_col.saturating_sub(scroll_offset);
        let spans_with_cursor = insert_cursor_into_spans(visible_spans, cursor_in_viewport);

        let paragraph = Paragraph::new(Line::from(spans_with_cursor)).block(block);
        frame.render_widget(paragraph, area);
    }
}
