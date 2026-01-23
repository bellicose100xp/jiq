use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph},
};

use crate::app::App;
use crate::search::Match;
use crate::search::search_render::SEARCH_BAR_HEIGHT;
use crate::widgets::{popup, scrollbar};

use crate::scroll::ScrollState;

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

fn format_position_indicator(scroll: &ScrollState, line_count: u32) -> String {
    if line_count == 0 {
        return String::new();
    }
    let start = scroll.offset as u32 + 1;
    let end = (scroll.offset as u32 + scroll.viewport_height as u32).min(line_count);
    let percentage = if line_count > 0 {
        (scroll.offset as u32 * 100) / line_count
    } else {
        0
    };
    format!("L{}-{}/{} ({}%)", start, end, line_count, percentage)
}

fn render_scrollbar(frame: &mut Frame, area: Rect, scroll: &ScrollState, line_count: u32) {
    let scrollbar_area = Rect {
        x: area.x,
        y: area.y.saturating_add(1),
        width: area.width,
        height: area.height.saturating_sub(2),
    };
    scrollbar::render_vertical_scrollbar(
        frame,
        scrollbar_area,
        line_count as usize,
        scroll.viewport_height as usize,
        scroll.offset as usize,
    );
}

/// Render the results pane
///
/// Returns the (results_area, search_bar_area) tuple for region tracking.
pub fn render_pane(app: &mut App, frame: &mut Frame, area: Rect) -> (Rect, Option<Rect>) {
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
            return (results_area, search_area);
        }
    };

    let is_pending = query_state.is_pending();
    let stats_info = app.stats.display().unwrap_or_else(|| "Results".to_string());

    // Calculate viewport dimensions and position indicator early for title
    let viewport_height = results_area.height.saturating_sub(2);
    let viewport_width = results_area.width.saturating_sub(2);
    let line_count = app.results_line_count_u32();
    app.results_scroll
        .update_bounds(line_count, viewport_height);
    if let Some(q) = &app.query {
        app.results_scroll
            .update_h_bounds(q.max_line_width(), viewport_width);
    }
    let position_indicator = format_position_indicator(&app.results_scroll, line_count);

    let search_visible = app.search.is_visible();

    // When search is confirmed (navigating results), results pane is active (purple)
    // When search is not confirmed (editing search), results pane is inactive (gray)
    let search_text_color = if search_visible && app.search.is_confirmed() {
        Color::LightMagenta
    } else if search_visible {
        Color::DarkGray
    } else {
        Color::Reset
    };

    let (title, unfocused_border_color) = if query_state.result.is_err() {
        // ERROR: Yellow text, yellow border (unfocused) - or search color when search visible
        let text_color = if search_visible {
            search_text_color
        } else {
            Color::Yellow
        };
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
            Style::default().fg(text_color),
        ));
        if !stats_info.is_empty() {
            spans.push(Span::styled(
                format!("| {} | Showing last successful result ", stats_info),
                Style::default().fg(text_color),
            ));
        }
        (Line::from(spans), Color::Yellow)
    } else if query_state.is_empty_result {
        // EMPTY: Gray text, gray border (unfocused) - or search color when search visible
        let text_color = if search_visible {
            search_text_color
        } else {
            Color::Gray
        };
        let mut spans = Vec::new();
        if is_pending {
            let (spinner_char, spinner_color) = get_spinner(app.frame_count);
            spans.push(Span::styled(
                format!("{} ", spinner_char),
                Style::default().fg(spinner_color),
            ));
        }
        spans.push(Span::styled(
            format!(
                " ∅ No Results | {} | Showing last non-empty result ",
                stats_info
            ),
            Style::default().fg(text_color),
        ));
        (Line::from(spans), Color::DarkGray)
    } else {
        // SUCCESS: Green text, green border (unfocused) - or search color when search visible
        let text_color = if search_visible {
            search_text_color
        } else {
            Color::Green
        };
        if is_pending {
            let (spinner_char, spinner_color) = get_spinner(app.frame_count);
            (
                Line::from(vec![
                    Span::styled(
                        format!("{} ", spinner_char),
                        Style::default().fg(spinner_color),
                    ),
                    Span::styled(format!("{} ", stats_info), Style::default().fg(text_color)),
                ]),
                Color::Green,
            )
        } else {
            (
                Line::from(Span::styled(
                    format!(" {} ", stats_info),
                    Style::default().fg(text_color),
                )),
                Color::Green,
            )
        }
    };

    let right_title_color = if search_visible {
        search_text_color
    } else {
        unfocused_border_color
    };
    let right_title: Option<Line<'_>> = if !position_indicator.is_empty() {
        Some(Line::from(Span::styled(
            format!(" {} ", position_indicator),
            Style::default().fg(right_title_color),
        )))
    } else {
        None
    };

    // When search is confirmed (navigating), results pane is active (purple)
    // When search is not confirmed (editing), results pane is inactive (gray)
    let border_color = if search_visible {
        if app.search.is_confirmed() {
            Color::LightMagenta
        } else {
            Color::DarkGray
        }
    } else if app.focus == crate::app::Focus::ResultsPane {
        Color::Cyan
    } else {
        unfocused_border_color
    };

    let is_stale = query_state.result.is_err() || query_state.is_empty_result;

    // Always render from cached pre-rendered text
    if let Some(rendered) = &query_state.last_successful_result_rendered {
        let mut block = Block::default()
            .borders(Borders::ALL)
            .title(title)
            .border_style(Style::default().fg(border_color));
        if let Some(rt) = right_title.clone() {
            block = block.title_top(rt.alignment(Alignment::Right));
        }
        if search_visible && app.search.is_confirmed() {
            let hints = Line::from(vec![Span::styled(
                " [n/N] Next/Prev | [Enter] Next | [Ctrl+F or /] Edit | [Esc] Close",
                Style::default().fg(Color::LightMagenta),
            )]);
            block = block.title_bottom(hints.alignment(Alignment::Center));
        }

        // Add navigation hints when results pane is focused and search is not visible
        if !search_visible && app.focus == crate::app::Focus::ResultsPane {
            let hints = Line::from(vec![Span::styled(
                " [Tab/Shift+Tab] Edit query | [i] Edit query in INSERT mode | [?] Help ",
                Style::default().fg(Color::Cyan),
            )]);
            block = block.title_bottom(hints.alignment(Alignment::Center));
        }

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

        // Apply DIM effect for stale results
        let viewport_text = if is_stale {
            apply_dim_to_text(viewport_text)
        } else {
            viewport_text
        };

        // Apply search highlights only to visible viewport
        let final_text = if app.search.is_visible() && !app.search.matches().is_empty() {
            apply_search_highlights(
                viewport_text,
                &app.search,
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
        render_scrollbar(frame, results_area, &app.results_scroll, line_count);
    } else {
        // No successful result yet - show empty
        let mut block = Block::default()
            .borders(Borders::ALL)
            .title(title)
            .border_style(Style::default().fg(border_color));
        if let Some(rt) = right_title {
            block = block.title_top(rt.alignment(Alignment::Right));
        }
        if search_visible && app.search.is_confirmed() {
            let hints = Line::from(vec![Span::styled(
                " [n/N] Next/Prev | [Enter] Next | [Ctrl+F or /] Edit | [Esc] Close",
                Style::default().fg(Color::LightMagenta),
            )]);
            block = block.title_bottom(hints.alignment(Alignment::Center));
        } else if !search_visible && app.focus == crate::app::Focus::ResultsPane {
            let hints = Line::from(vec![Span::styled(
                " [Tab/Shift+Tab] Edit query | [i] Edit query in INSERT mode | [?] Help ",
                Style::default().fg(Color::Cyan),
            )]);
            block = block.title_bottom(hints.alignment(Alignment::Center));
        }

        let empty_text = Text::from("");
        let content = Paragraph::new(empty_text).block(block);

        frame.render_widget(content, results_area);
    }
    if let Some(search_rect) = search_area {
        crate::search::search_render::render_bar(app, frame, search_rect);
    }

    (results_area, search_area)
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

/// Render the error overlay
///
/// Returns the error overlay area for region tracking.
pub fn render_error_overlay(app: &App, frame: &mut Frame, results_area: Rect) -> Option<Rect> {
    // Only render if query state is available
    let query_state = match &app.query {
        Some(q) => q,
        None => return None,
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
        return Some(overlay_area);
    }
    None
}

fn apply_dim_to_text(text: Text<'_>) -> Text<'static> {
    Text::from(
        text.lines
            .into_iter()
            .map(|line| {
                Line::from(
                    line.spans
                        .into_iter()
                        .map(|span| {
                            Span::styled(
                                span.content.to_string(),
                                span.style.add_modifier(Modifier::DIM),
                            )
                        })
                        .collect::<Vec<_>>(),
                )
            })
            .collect::<Vec<_>>(),
    )
}

fn apply_search_highlights(
    text: Text<'_>,
    search_state: &crate::search::SearchState,
    scroll_offset: u16,
    viewport_height: u16,
) -> Text<'static> {
    let matches = search_state.matches();
    let current_match_index = search_state.current_index();

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
            let line_matches: Vec<(usize, &Match)> =
                search_state.matches_on_line(absolute_line as u32).collect();

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
