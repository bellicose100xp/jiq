//! Results pane rendering
//!
//! This module handles rendering of the results pane and error overlay.

use ansi_to_tui::IntoText;
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::App;
use crate::search::Match;
use crate::search::search_render::SEARCH_BAR_HEIGHT;
use crate::widgets::popup;

// Search highlight styles
// Non-selected matches: lighter gray for subtle indication
const MATCH_HIGHLIGHT_BG: Color = Color::Rgb(128, 128, 128); // Gray
const MATCH_HIGHLIGHT_FG: Color = Color::White;
// Current match: bright orange for clear visibility
const CURRENT_MATCH_HIGHLIGHT_BG: Color = Color::Rgb(255, 165, 0); // Orange
const CURRENT_MATCH_HIGHLIGHT_FG: Color = Color::Black;


/// Render the results pane (top)
pub fn render_pane(app: &mut App, frame: &mut Frame, area: Rect) {
    // Split area for search bar if visible
    let (results_area, search_area) = if app.search.is_visible() {
        let layout = Layout::vertical([
            Constraint::Min(3),                    // Results content
            Constraint::Length(SEARCH_BAR_HEIGHT), // Search bar (3 lines: borders + input)
        ])
        .split(area);
        (layout[0], Some(layout[1]))
    } else {
        (area, None)
    };

    // Set border color based on focus
    let border_color = if app.focus == crate::app::Focus::ResultsPane {
        Color::Cyan // Focused
    } else {
        Color::DarkGray // Unfocused
    };

    // Build title based on state: stats info replaces "Results" text
    // On error: "⚠ Syntax Error (Array [5 objects]) - last successful"
    // On success: "Array [5 objects]" or "Object" etc.
    let title = if app.query.result.is_err() {
        // Error state: show warning icon + stats from last successful result
        let stats_info = app.stats.display().unwrap_or_default();
        if stats_info.is_empty() {
            Line::from(vec![
                Span::styled(" ⚠ Syntax Error ", Style::default().fg(Color::Yellow)),
            ])
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
        // Success state: show stats info as title
        // Always use Cyan (highlighted border color) for stats text on success
        let stats_info = app.stats.display().unwrap_or_else(|| "Results".to_string());
        Line::from(Span::styled(
            format!(" {} ", stats_info),
            Style::default().fg(Color::Cyan),
        ))
    };


    match &app.query.result {
        Ok(result) => {
            // Update scroll bounds based on content and viewport
            let viewport_height = results_area.height.saturating_sub(2);
            let viewport_width = results_area.width.saturating_sub(2);
            let line_count = app.results_line_count_u32();
            app.results_scroll.update_bounds(line_count, viewport_height);
            app.results_scroll
                .update_h_bounds(app.query.max_line_width(), viewport_width);

            let block = Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(Style::default().fg(border_color));

            // Parse jq's ANSI color codes into Ratatui Text
            let colored_text = result
                .as_bytes()
                .to_vec()
                .into_text()
                .unwrap_or_else(|_| Text::raw(result)); // Fallback to plain text on parse error

            // Apply search highlights if search is active
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
            // When there's an error, show last successful result in full area (no splitting)
            // The error overlay will be rendered separately if user requests it with Ctrl+E
            let viewport_height = results_area.height.saturating_sub(2);
            let viewport_width = results_area.width.saturating_sub(2);
            let line_count = app.results_line_count_u32();
            app.results_scroll.update_bounds(line_count, viewport_height);
            app.results_scroll
                .update_h_bounds(app.query.max_line_width(), viewport_width);

            if let Some(last_result) = &app.query.last_successful_result {
                // Render last successful result with error title
                let results_block = Block::default()
                    .borders(Borders::ALL)
                    .title(title)
                    .border_style(Style::default().fg(border_color));

                // Parse cached result with colors
                let colored_text = last_result
                    .as_bytes()
                    .to_vec()
                    .into_text()
                    .unwrap_or_else(|_| Text::raw(last_result));

                // Apply search highlights if search is active
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
                // No cached result, show empty results pane with error title
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

    // Render search bar if visible
    if let Some(search_rect) = search_area {
        crate::search::search_render::render_bar(app, frame, search_rect);
    }
}


/// Render the error overlay (floating at the bottom of results pane)
pub fn render_error_overlay(app: &App, frame: &mut Frame, results_area: Rect) {
    // Only render if there's an error
    if let Err(error) = &app.query.result {
        // Truncate error to max 5 lines of content
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

        // Calculate overlay height (content lines + borders)
        let content_lines = if truncated { max_content_lines + 1 } else { error_lines.len() };
        let overlay_height = (content_lines as u16 + 2).clamp(3, 7); // Min 3, max 7

        // Position overlay at bottom of results pane, with 1 line gap from bottom border
        let overlay_y = results_area.bottom().saturating_sub(overlay_height + 1);

        // Create overlay area with margins and position at bottom
        let overlay_with_margins = popup::inset_rect(results_area, 2, 0);
        let overlay_area = Rect {
            x: overlay_with_margins.x,
            y: overlay_y,
            width: overlay_with_margins.width,
            height: overlay_height,
        };

        // Clear the background to make it truly floating
        popup::clear_area(frame, overlay_area);

        // Render error overlay with distinct styling
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


/// Apply search match highlighting to a Text object
///
/// This function takes the parsed ANSI text and overlays search match highlights
/// on the visible portion of the content.
///
/// # Arguments
/// * `text` - The parsed Text with ANSI colors
/// * `matches` - All search matches found in the content
/// * `current_match_index` - Index of the currently selected match
/// * `scroll_offset` - Current vertical scroll offset
/// * `viewport_height` - Height of the visible viewport
fn apply_search_highlights(
    text: Text<'_>,
    matches: &[Match],
    current_match_index: usize,
    scroll_offset: u16,
    viewport_height: u16,
) -> Text<'static> {
    if matches.is_empty() {
        // No matches, return text as-is (converted to owned)
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

    // Note: We apply highlights to ALL lines, not just visible ones.
    // The Paragraph::scroll() call handles showing the right portion at render time.
    // The scroll_offset and viewport_height parameters are kept for potential
    // future optimization (only processing visible lines).
    let _ = (scroll_offset, viewport_height); // Suppress unused warnings

    // Convert text lines to owned and apply highlights
    let highlighted_lines: Vec<Line<'static>> = text
        .lines
        .into_iter()
        .enumerate()
        .map(|(line_idx, line)| {
            // Find matches on this line
            let line_matches: Vec<(usize, &Match)> = matches
                .iter()
                .enumerate()
                .filter(|(_, m)| m.line as usize == line_idx)
                .collect();

            if line_matches.is_empty() {
                // No matches on this line, convert to owned
                Line::from(
                    line.spans
                        .into_iter()
                        .map(|span| Span::styled(span.content.to_string(), span.style))
                        .collect::<Vec<_>>(),
                )
            } else {
                // Apply highlights to this line
                apply_highlights_to_line(line, &line_matches, current_match_index)
            }
        })
        .collect();

    Text::from(highlighted_lines)
}


/// Apply search highlights to a single line
fn apply_highlights_to_line(
    line: Line<'_>,
    matches: &[(usize, &Match)],
    current_match_index: usize,
) -> Line<'static> {
    // First, flatten all spans into a single string with style info
    let mut char_styles: Vec<(char, Style)> = Vec::new();
    
    for span in &line.spans {
        for ch in span.content.chars() {
            char_styles.push((ch, span.style));
        }
    }

    // Apply match highlights (overriding existing styles)
    for (match_idx, m) in matches {
        let col_start = m.col as usize;
        let col_end = col_start + m.len as usize;
        
        // Determine highlight style based on whether this is the current match
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

        // Apply highlight to character range
        for i in col_start..col_end.min(char_styles.len()) {
            char_styles[i].1 = highlight_style;
        }
    }

    // Don't skip characters here - Paragraph::scroll() handles horizontal scrolling
    let visible_chars: Vec<(char, Style)> = char_styles;

    // Rebuild spans from character styles (grouping consecutive same-style chars)
    let mut result_spans: Vec<Span<'static>> = Vec::new();
    let mut current_text = String::new();
    let mut current_style: Option<Style> = None;

    for (ch, style) in visible_chars {
        match current_style {
            Some(s) if s == style => {
                current_text.push(ch);
            }
            _ => {
                if !current_text.is_empty() && let Some(s) = current_style {
                    result_spans.push(Span::styled(current_text.clone(), s));
                }
                current_text = ch.to_string();
                current_style = Some(style);
            }
        }
    }

    // Don't forget the last span
    if !current_text.is_empty() && let Some(s) = current_style {
        result_spans.push(Span::styled(current_text, s));
    }

    Line::from(result_spans)
}
