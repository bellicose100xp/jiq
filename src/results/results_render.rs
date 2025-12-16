use ansi_to_tui::IntoText;
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph},
};

use crate::app::App;
use crate::search::Match;
use crate::search::search_render::SEARCH_BAR_HEIGHT;
use crate::widgets::popup;

const MATCH_HIGHLIGHT_BG: Color = Color::Rgb(128, 128, 128);
const MATCH_HIGHLIGHT_FG: Color = Color::White;
const CURRENT_MATCH_HIGHLIGHT_BG: Color = Color::Rgb(255, 165, 0);
const CURRENT_MATCH_HIGHLIGHT_FG: Color = Color::Black;
pub fn render_pane(app: &mut App, frame: &mut Frame, area: Rect) {
    let (results_area, search_area) = if app.search.is_visible() {
        let layout = Layout::vertical([Constraint::Min(3), Constraint::Length(SEARCH_BAR_HEIGHT)])
            .split(area);
        (layout[0], Some(layout[1]))
    } else {
        (area, None)
    };

    let border_color = if app.focus == crate::app::Focus::ResultsPane {
        Color::Cyan
    } else {
        Color::DarkGray
    };
    let title = if app.query.result.is_err() {
        let stats_info = app.stats.display().unwrap_or_default();
        if stats_info.is_empty() {
            Line::from(vec![Span::styled(
                " ⚠ Syntax Error ",
                Style::default().fg(Color::Yellow),
            )])
        } else {
            Line::from(vec![
                Span::styled(" ⚠ Syntax Error ", Style::default().fg(Color::Yellow)),
                Span::styled(
                    format!("({} - last successful query result) ", stats_info),
                    Style::default().fg(Color::Gray),
                ),
            ])
        }
    } else {
        let stats_info = app.stats.display().unwrap_or_else(|| "Results".to_string());
        Line::from(Span::styled(
            format!(" {} ", stats_info),
            Style::default().fg(Color::Cyan),
        ))
    };

    match &app.query.result {
        Ok(result) => {
            let viewport_height = results_area.height.saturating_sub(2);
            let viewport_width = results_area.width.saturating_sub(2);
            let line_count = app.results_line_count_u32();
            app.results_scroll
                .update_bounds(line_count, viewport_height);
            app.results_scroll
                .update_h_bounds(app.query.max_line_width(), viewport_width);

            let block = Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(Style::default().fg(border_color));

            let colored_text = result
                .as_bytes()
                .to_vec()
                .into_text()
                .unwrap_or_else(|_| Text::raw(result));
            let final_text = if app.search.is_visible() && !app.search.matches().is_empty() {
                #[cfg(debug_assertions)]
                {
                    if let Some(current_match) = app.search.current_match() {
                        log::debug!(
                            "render: applying highlights, current_match at line={} col={} len={}, scroll=({}, {})",
                            current_match.line,
                            current_match.col,
                            current_match.len,
                            app.results_scroll.offset,
                            app.results_scroll.h_offset
                        );
                    }
                }
                apply_search_highlights(
                    colored_text,
                    app.search.matches(),
                    app.search.current_index(),
                    app.results_scroll.offset,
                    viewport_height,
                )
            } else {
                colored_text
            };

            let content = Paragraph::new(final_text)
                .block(block)
                .scroll((app.results_scroll.offset, app.results_scroll.h_offset));

            frame.render_widget(content, results_area);
        }
        Err(_error) => {
            let viewport_height = results_area.height.saturating_sub(2);
            let viewport_width = results_area.width.saturating_sub(2);
            let line_count = app.results_line_count_u32();
            app.results_scroll
                .update_bounds(line_count, viewport_height);
            app.results_scroll
                .update_h_bounds(app.query.max_line_width(), viewport_width);

            if let Some(last_result) = &app.query.last_successful_result {
                let results_block = Block::default()
                    .borders(Borders::ALL)
                    .title(title)
                    .border_style(Style::default().fg(border_color));

                let colored_text = last_result
                    .as_bytes()
                    .to_vec()
                    .into_text()
                    .unwrap_or_else(|_| Text::raw(last_result));
                let final_text = if app.search.is_visible() && !app.search.matches().is_empty() {
                    apply_search_highlights(
                        colored_text,
                        app.search.matches(),
                        app.search.current_index(),
                        app.results_scroll.offset,
                        viewport_height,
                    )
                } else {
                    colored_text
                };

                let results_widget = Paragraph::new(final_text)
                    .block(results_block)
                    .scroll((app.results_scroll.offset, app.results_scroll.h_offset));

                frame.render_widget(results_widget, results_area);
            } else {
                let block = Block::default()
                    .borders(Borders::ALL)
                    .title(title)
                    .border_style(Style::default().fg(border_color));

                let empty_text = Text::from("");
                let content = Paragraph::new(empty_text).block(block);

                frame.render_widget(content, results_area);
            }
        }
    }
    if let Some(search_rect) = search_area {
        crate::search::search_render::render_bar(app, frame, search_rect);
    }
}
pub fn render_error_overlay(app: &App, frame: &mut Frame, results_area: Rect) {
    if let Err(error) = &app.query.result {
        let error_lines: Vec<&str> = error.lines().collect();
        let max_content_lines = 5;
        let (display_error, truncated) = if error_lines.len() > max_content_lines {
            let truncated_lines = &error_lines[..max_content_lines];
            let mut display = truncated_lines.join("\n");
            display.push_str("\n... (error truncated)");
            (display, true)
        } else {
            (error.clone(), false)
        };

        let content_lines = if truncated {
            max_content_lines + 1
        } else {
            error_lines.len()
        };
        let overlay_height = (content_lines as u16 + 2).clamp(3, 7);

        let overlay_y = results_area.bottom().saturating_sub(overlay_height + 1);

        let overlay_with_margins = popup::inset_rect(results_area, 2, 0);
        let overlay_area = Rect {
            x: overlay_with_margins.x,
            y: overlay_y,
            width: overlay_with_margins.width,
            height: overlay_height,
        };

        popup::clear_area(frame, overlay_area);
        let error_block = Block::default()
            .borders(Borders::ALL)
            .title(" Syntax Error (Ctrl+E to close) ")
            .border_style(Style::default().fg(Color::Red))
            .style(Style::default().bg(Color::Black));

        let error_widget = Paragraph::new(display_error.as_str())
            .block(error_block)
            .style(Style::default().fg(Color::Red));

        frame.render_widget(error_widget, overlay_area);
    }
}
fn apply_search_highlights(
    text: Text<'_>,
    matches: &[Match],
    current_match_index: usize,
    scroll_offset: u16,
    viewport_height: u16,
) -> Text<'static> {
    if matches.is_empty() {
        return Text::from(
            text.lines
                .into_iter()
                .map(|line| {
                    Line::from(
                        line.spans
                            .into_iter()
                            .map(|span| Span::styled(span.content.to_string(), span.style))
                            .collect::<Vec<_>>(),
                    )
                })
                .collect::<Vec<_>>(),
        );
    }

    let _ = (scroll_offset, viewport_height);
    let highlighted_lines: Vec<Line<'static>> = text
        .lines
        .into_iter()
        .enumerate()
        .map(|(line_idx, line)| {
            let line_matches: Vec<(usize, &Match)> = matches
                .iter()
                .enumerate()
                .filter(|(_, m)| m.line as usize == line_idx)
                .collect();

            if line_matches.is_empty() {
                Line::from(
                    line.spans
                        .into_iter()
                        .map(|span| Span::styled(span.content.to_string(), span.style))
                        .collect::<Vec<_>>(),
                )
            } else {
                apply_highlights_to_line(line, &line_matches, current_match_index)
            }
        })
        .collect();

    Text::from(highlighted_lines)
}
fn apply_highlights_to_line(
    line: Line<'_>,
    matches: &[(usize, &Match)],
    current_match_index: usize,
) -> Line<'static> {
    let mut char_styles: Vec<(char, Style)> = Vec::new();

    for span in &line.spans {
        for ch in span.content.chars() {
            char_styles.push((ch, span.style));
        }
    }

    for (match_idx, m) in matches {
        let col_start = m.col as usize;
        let col_end = col_start + m.len as usize;

        let highlight_style = if *match_idx == current_match_index {
            Style::default()
                .fg(CURRENT_MATCH_HIGHLIGHT_FG)
                .bg(CURRENT_MATCH_HIGHLIGHT_BG)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
                .fg(MATCH_HIGHLIGHT_FG)
                .bg(MATCH_HIGHLIGHT_BG)
        };

        for i in col_start..col_end.min(char_styles.len()) {
            char_styles[i].1 = highlight_style;
        }
    }

    let visible_chars: Vec<(char, Style)> = char_styles;
    let mut result_spans: Vec<Span<'static>> = Vec::new();
    let mut current_text = String::new();
    let mut current_style: Option<Style> = None;

    for (ch, style) in visible_chars {
        match current_style {
            Some(s) if s == style => {
                current_text.push(ch);
            }
            _ => {
                if !current_text.is_empty()
                    && let Some(s) = current_style
                {
                    result_spans.push(Span::styled(current_text.clone(), s));
                }
                current_text = ch.to_string();
                current_style = Some(style);
            }
        }
    }
    if !current_text.is_empty()
        && let Some(s) = current_style
    {
        result_spans.push(Span::styled(current_text, s));
    }

    Line::from(result_spans)
}
