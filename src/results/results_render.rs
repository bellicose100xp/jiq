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

// Rainbow spinner animation
const SPINNER_CHARS: &[char] = &['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
const SPINNER_COLORS: &[Color] = &[
    Color::Rgb(255, 107, 107), // Red/Coral
    Color::Rgb(255, 159, 67),  // Orange
    Color::Rgb(254, 202, 87),  // Yellow
    Color::Rgb(72, 219, 147),  // Green
    Color::Rgb(69, 170, 242),  // Blue
    Color::Rgb(120, 111, 213), // Indigo
    Color::Rgb(214, 128, 255), // Violet
    Color::Rgb(255, 121, 198), // Pink
];

fn get_spinner(frame_count: u64) -> (char, Color) {
    // Change every ~8 frames for visible but not too fast animation (~133ms at 60fps)
    let index = (frame_count / 8) as usize;
    let char_idx = index % SPINNER_CHARS.len();
    let color_idx = index % SPINNER_COLORS.len();
    (SPINNER_CHARS[char_idx], SPINNER_COLORS[color_idx])
}

pub fn render_pane(app: &mut App, frame: &mut Frame, area: Rect) {
    let (results_area, search_area) = if app.search.is_visible() {
        let layout = Layout::vertical([Constraint::Min(3), Constraint::Length(SEARCH_BAR_HEIGHT)])
            .split(area);
        (layout[0], Some(layout[1]))
    } else {
        (area, None)
    };

    // Check if query is available
    let query_state = match &app.query {
        Some(q) => q,
        None => {
            // Show loading indicator or error if file loader is present
            if let Some(loader) = &app.file_loader {
                if loader.is_loading() {
                    render_loading_indicator(frame, results_area);
                } else if let crate::input::loader::LoadingState::Error(e) = loader.state() {
                    render_error_message(
                        frame,
                        results_area,
                        &format!("Failed to load file: {}", e),
                    );
                }
            }
            return;
        }
    };

    let is_pending = query_state.is_pending();
    let border_color = if app.focus == crate::app::Focus::ResultsPane {
        Color::Cyan
    } else {
        Color::DarkGray
    };
    let title = if query_state.result.is_err() {
        // Error title with optional rainbow spinner
        let stats_info = app.stats.display().unwrap_or_default();
        let mut spans = Vec::new();
        if is_pending {
            let (spinner_char, spinner_color) = get_spinner(app.frame_count);
            spans.push(Span::styled(
                format!("{} ", spinner_char),
                Style::default().fg(spinner_color),
            ));
        }
        spans.push(Span::styled(
            " ⚠ Syntax Error ",
            Style::default().fg(Color::Yellow),
        ));
        if !stats_info.is_empty() {
            spans.push(Span::styled(
                format!("({} - last successful query result) ", stats_info),
                Style::default().fg(Color::Gray),
            ));
        }
        Line::from(spans)
    } else {
        // Normal title with optional rainbow spinner
        let stats_info = app.stats.display().unwrap_or_else(|| "Results".to_string());
        if is_pending {
            let (spinner_char, spinner_color) = get_spinner(app.frame_count);
            Line::from(vec![
                Span::styled(
                    format!("{} ", spinner_char),
                    Style::default().fg(spinner_color),
                ),
                Span::styled(format!("{} ", stats_info), Style::default().fg(Color::Cyan)),
            ])
        } else {
            Line::from(Span::styled(
                format!(" {} ", stats_info),
                Style::default().fg(Color::Cyan),
            ))
        }
    };

    // Always render from cached pre-rendered text
    if let Some(rendered) = &query_state.last_successful_result_rendered {
        let viewport_height = results_area.height.saturating_sub(2);
        let viewport_width = results_area.width.saturating_sub(2);
        let line_count = app.results_line_count_u32();
        app.results_scroll
            .update_bounds(line_count, viewport_height);
        app.results_scroll
            .update_h_bounds(query_state.max_line_width(), viewport_width);

        let block = Block::default()
            .borders(Borders::ALL)
            .title(title)
            .border_style(Style::default().fg(border_color));

        // Use cached pre-rendered text
        // Optimization: Only clone visible viewport to avoid massive allocations
        let scroll_offset = app.results_scroll.offset as usize;
        let viewport_lines = viewport_height as usize;

        // Slice to viewport range (with bounds checking)
        let total_lines = rendered.lines.len();
        let end_line = (scroll_offset + viewport_lines).min(total_lines);
        let visible_lines = if scroll_offset < total_lines {
            &rendered.lines[scroll_offset..end_line]
        } else {
            &[]
        };

        // Clone only visible lines (50 lines instead of 100K+ for large files!)
        let viewport_text = Text::from(visible_lines.to_vec());

        // Apply search highlights only to visible viewport
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
                viewport_text,
                app.search.matches(),
                app.search.current_index(),
                app.results_scroll.offset,
                viewport_height,
            )
        } else {
            viewport_text
        };

        // Vertical scroll handled by viewport slicing, but horizontal scroll still needed
        let content = Paragraph::new(final_text)
            .block(block)
            .scroll((0, app.results_scroll.h_offset));
        frame.render_widget(content, results_area);
    } else {
        // No successful result yet - show empty
        let block = Block::default()
            .borders(Borders::ALL)
            .title(title)
            .border_style(Style::default().fg(border_color));

        let empty_text = Text::from("");
        let content = Paragraph::new(empty_text).block(block);

        frame.render_widget(content, results_area);
    }
    if let Some(search_rect) = search_area {
        crate::search::search_render::render_bar(app, frame, search_rect);
    }
}

fn render_loading_indicator(frame: &mut Frame, area: Rect) {
    let text = "Loading file...";
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Loading ")
        .border_style(Style::default().fg(Color::Yellow));

    let paragraph = Paragraph::new(text)
        .block(block)
        .style(Style::default().fg(Color::Yellow));

    frame.render_widget(paragraph, area);
}

fn render_error_message(frame: &mut Frame, area: Rect, message: &str) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Error ")
        .border_style(Style::default().fg(Color::Red));

    let paragraph = Paragraph::new(message)
        .block(block)
        .style(Style::default().fg(Color::Red));

    frame.render_widget(paragraph, area);
}

pub fn render_error_overlay(app: &App, frame: &mut Frame, results_area: Rect) {
    // Only render if query state is available
    let query_state = match &app.query {
        Some(q) => q,
        None => return,
    };

    if let Err(error) = &query_state.result {
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

    let _ = viewport_height;
    let highlighted_lines: Vec<Line<'static>> = text
        .lines
        .into_iter()
        .enumerate()
        .map(|(line_idx, line)| {
            // Adjust line_idx by scroll_offset to get absolute line number
            let absolute_line = line_idx + scroll_offset as usize;
            let line_matches: Vec<(usize, &Match)> = matches
                .iter()
                .enumerate()
                .filter(|(_, m)| m.line as usize == absolute_line)
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

#[cfg(test)]
#[path = "results_render_tests.rs"]
mod results_render_tests;
