use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
};

use crate::app::{App, Focus};
use crate::editor::EditorMode;
use crate::syntax_highlight::JqHighlighter;
use crate::syntax_highlight::bracket_matcher::find_matching_bracket;
use crate::syntax_highlight::overlay::{
    extract_visible_spans, highlight_bracket_pairs, insert_cursor_into_spans,
};
use crate::theme;

/// Render the input field
///
/// Returns the input field area for region tracking.
pub fn render_field(app: &mut App, frame: &mut Frame, area: Rect) -> Rect {
    let viewport_width = area.width.saturating_sub(2) as usize;
    app.input.calculate_scroll_offset(viewport_width);

    let mode_color = match app.input.editor_mode {
        EditorMode::Insert => theme::input::MODE_INSERT,
        EditorMode::Normal => theme::input::MODE_NORMAL,
        EditorMode::Operator(_) => theme::input::MODE_OPERATOR,
        EditorMode::CharSearch(_, _) => theme::input::MODE_CHAR_SEARCH,
        EditorMode::OperatorCharSearch(_, _, _, _) => theme::input::MODE_OPERATOR,
        EditorMode::TextObject(_, _) => theme::input::MODE_OPERATOR,
    };

    let border_color = if app.focus == Focus::InputField {
        mode_color
    } else {
        theme::input::BORDER_UNFOCUSED
    };

    let is_focused = app.focus == Focus::InputField;
    let mode_display_color = if is_focused {
        mode_color
    } else {
        theme::input::UNFOCUSED_HINT
    };

    let mode_text = app.input.editor_mode.display();
    let mut title_spans = match app.input.editor_mode {
        EditorMode::Normal => {
            vec![
                Span::raw(" Query ["),
                Span::styled(mode_text, Style::default().fg(mode_display_color)),
                Span::raw("] (press 'i' to edit) "),
            ]
        }
        _ => {
            vec![
                Span::raw(" Query ["),
                Span::styled(mode_text, Style::default().fg(mode_display_color)),
                Span::raw("] "),
            ]
        }
    };

    if let Some(query) = &app.query
        && query.result.is_err()
    {
        title_spans.push(Span::styled(
            "⚠ Syntax Error (Ctrl+E to view)",
            Style::default().fg(theme::input::SYNTAX_ERROR_WARNING),
        ));
    }

    let title = Line::from(title_spans);

    let tooltip_hint = if !app.tooltip.enabled && app.tooltip.current_function.is_some() {
        Some(Line::from(vec![Span::styled(
            " Ctrl+T for tooltip ",
            Style::default().fg(theme::input::TOOLTIP_HINT),
        )]))
    } else {
        None
    };

    let ai_hint = if !app.ai.visible {
        Some(Line::from(vec![Span::styled(
            " Press Ctrl+A for AI Assistant ",
            Style::default().fg(theme::input::AI_HINT),
        )]))
    } else {
        None
    };

    let mut block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(title)
        .border_style(Style::default().fg(border_color));

    if let Some(hint) = tooltip_hint {
        block = block.title_top(hint.alignment(Alignment::Right));
    } else if let Some(hint) = ai_hint {
        block = block.title_top(hint.alignment(Alignment::Right));
    }

    if !is_focused {
        block = block.title_bottom(
            Line::from(vec![Span::styled(
                " Tab to edit query ",
                Style::default().fg(theme::input::UNFOCUSED_HINT),
            )])
            .alignment(Alignment::Center),
        );
    } else if !app.query().is_empty() {
        let key_style = Style::default().fg(mode_color);
        let desc_style = Style::default().fg(theme::help_line::DESCRIPTION);
        let sep_style = Style::default().fg(theme::help_line::SEPARATOR);
        block = block.title_bottom(
            Line::from(vec![
                Span::styled(" [Enter]", key_style),
                Span::styled(" Output Result ", desc_style),
                Span::styled("•", sep_style),
                Span::styled(" [Ctrl+Q]", key_style),
                Span::styled(" Output Query ", desc_style),
            ])
            .alignment(Alignment::Center),
        );
    }

    let query = app.query();
    let cursor_col = app.input.textarea.cursor().1;
    let scroll_offset = app.input.scroll_offset;

    if query.is_empty() {
        let final_spans = if is_focused {
            insert_cursor_into_spans(vec![], 0)
        } else {
            vec![]
        };
        let paragraph = Paragraph::new(Line::from(final_spans)).block(block);
        frame.render_widget(paragraph, area);
    } else {
        let highlighted_spans = JqHighlighter::highlight(query);

        let spans_with_brackets =
            if let Some(bracket_positions) = find_matching_bracket(query, cursor_col) {
                highlight_bracket_pairs(highlighted_spans, bracket_positions)
            } else {
                highlighted_spans
            };

        let visible_spans =
            extract_visible_spans(&spans_with_brackets, scroll_offset, viewport_width);

        let final_spans = if is_focused {
            let cursor_in_viewport = cursor_col.saturating_sub(scroll_offset);
            insert_cursor_into_spans(visible_spans, cursor_in_viewport)
        } else {
            visible_spans
                .into_iter()
                .map(|span| {
                    Span::styled(
                        span.content,
                        Style::default().fg(theme::input::QUERY_UNFOCUSED),
                    )
                })
                .collect()
        };

        let paragraph = Paragraph::new(Line::from(final_spans)).block(block);
        frame.render_widget(paragraph, area);
    }
    area
}
