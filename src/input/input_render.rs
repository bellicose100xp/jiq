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

    let has_error = app.query.as_ref().is_some_and(|q| q.result.is_err());

    let border_color = if has_error {
        theme::input::BORDER_ERROR
    } else if app.focus == Focus::InputField {
        mode_color
    } else {
        theme::input::BORDER_UNFOCUSED
    };

    let is_focused = app.focus == Focus::InputField;
    let mode_display_color = if has_error && is_focused {
        theme::input::BORDER_ERROR
    } else if is_focused {
        mode_color
    } else {
        theme::input::UNFOCUSED_HINT
    };

    let mode_text = app.input.editor_mode.display();
    let title_spans = match app.input.editor_mode {
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

    let title = Line::from(title_spans);

    let tooltip_hint = if !app.tooltip.enabled && app.tooltip.current_function.is_some() {
        Some(Line::from(vec![Span::styled(
            " Ctrl+T for tooltip ",
            Style::default().fg(theme::input::TOOLTIP_HINT),
        )]))
    } else {
        None
    };

    let ai_hint = if is_focused && !app.ai.visible {
        Some(theme::border_hints::build_hints(
            &[("Ctrl+A", "AI Assistant")],
            mode_color,
        ))
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

    if is_focused {
        if has_error {
            block = block.title_bottom(
                theme::border_hints::build_hints(
                    &[("Ctrl+E", "Show Error")],
                    theme::input::BORDER_ERROR,
                )
                .alignment(Alignment::Center),
            );
        } else if !app.query().is_empty() {
            block = block.title_bottom(
                theme::border_hints::build_hints(
                    &[("Enter", "Output Result"), ("Ctrl+Q", "Output Query")],
                    mode_color,
                )
                .alignment(Alignment::Center),
            );
        } else {
            block = block.title_bottom(
                theme::border_hints::build_hints(
                    &[
                        ("Ctrl+P", "Previous Query"),
                        ("Ctrl+N", "Next Query"),
                        ("Ctrl+R", "History"),
                    ],
                    mode_color,
                )
                .alignment(Alignment::Center),
            );
        }
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
