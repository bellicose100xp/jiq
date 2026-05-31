//! Error-overlay rendering.
//!
//! Draws the <kbd>Ctrl</kbd>+<kbd>E</kbd> error overlay. jq's raw stderr is
//! enhanced into a plain-language summary, a fix hint, and a source location
//! via [`crate::query::error_enhance`]; unrecognized messages fall back to the
//! raw text so no detail is lost.

use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, BorderType, Borders, Padding, Paragraph},
};

use crate::app::App;
use crate::query::error_enhance::EnhancedError;
use crate::theme;
use crate::widgets::popup;

/// Cap on body lines before the overlay truncates with a dimmed marker.
const MAX_CONTENT_LINES: usize = 7;

/// Render the error overlay.
///
/// Returns the error overlay area for region tracking.
pub fn render_error_overlay(app: &App, frame: &mut Frame, results_area: Rect) -> Option<Rect> {
    let query_state = match &app.query {
        Some(q) => q,
        None => return None,
    };

    if let Err(error) = &query_state.result {
        // Inset matches the legacy overlay; content width subtracts borders (2)
        // and the 1-cell horizontal padding on each side (2).
        let overlay_with_margins = popup::inset_rect(results_area, 2, 0);
        let content_width = overlay_with_margins.width.saturating_sub(4) as usize;

        // Enhance jq's raw stderr into plain-language lines. Unrecognized
        // messages (e.g. jiq-internal errors) fall back to the raw text.
        let body_lines =
            match crate::query::error_enhance::enhance_jq_error(error, app.input.query()) {
                Some(enhanced) => build_enhanced_error_lines(&enhanced, content_width),
                None => build_raw_error_lines(error, content_width),
            };

        let truncated = body_lines.len() > MAX_CONTENT_LINES;
        let mut display_lines: Vec<Line<'static>> =
            body_lines.into_iter().take(MAX_CONTENT_LINES).collect();
        if truncated {
            display_lines.push(Line::from(Span::styled(
                "… (error truncated)",
                Style::default().fg(theme::results::error_location()),
            )));
        }

        let content_lines = display_lines.len();
        // +2 for borders, +2 for top/bottom padding.
        let overlay_height = (content_lines as u16 + 4).clamp(5, 11);

        let overlay_y = results_area.bottom().saturating_sub(overlay_height + 1);
        let overlay_area = Rect {
            x: overlay_with_margins.x,
            y: overlay_y,
            width: overlay_with_margins.width,
            height: overlay_height,
        };

        popup::clear_area(frame, overlay_area);
        let close_hint =
            theme::border_hints::build_hints(&[("Ctrl+E", "Close")], theme::results::border_error());
        let error_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(" Error ")
            .title_bottom(close_hint.alignment(Alignment::Center))
            .border_style(Style::default().fg(theme::results::border_error()))
            .style(Style::default().bg(theme::results::background()))
            .padding(Padding::new(1, 1, 1, 1));

        let error_widget = Paragraph::new(Text::from(display_lines)).block(error_block);

        frame.render_widget(error_widget, overlay_area);
        return Some(overlay_area);
    }
    None
}

/// Build the styled, word-wrapped body lines for an enhanced jq error:
/// the summary in the error color, the fix hint as a cyan "Try:" callout,
/// and jq's source location dimmed beneath.
fn build_enhanced_error_lines(enhanced: &EnhancedError, width: usize) -> Vec<Line<'static>> {
    let mut lines: Vec<Line<'static>> = Vec::new();

    for chunk in enhanced.summary.split('\n') {
        for wrapped in wrap_plain(chunk, width) {
            lines.push(Line::from(Span::styled(
                wrapped,
                Style::default().fg(theme::results::error_summary()),
            )));
        }
    }

    if let Some(hint) = &enhanced.hint {
        lines.push(Line::from(""));
        // "Try: " label plus the hint, wrapped together so the label width is
        // accounted for on the first line.
        let label = "Try: ";
        let hint_lines = wrap_plain(hint, width.saturating_sub(label.len()).max(1));
        for (i, wrapped) in hint_lines.into_iter().enumerate() {
            let mut spans: Vec<Span<'static>> = Vec::new();
            if i == 0 {
                spans.push(Span::styled(
                    label,
                    Style::default()
                        .fg(theme::results::error_hint_label())
                        .add_modifier(Modifier::BOLD),
                ));
            } else {
                spans.push(Span::raw("     "));
            }
            spans.push(Span::styled(
                wrapped,
                Style::default().fg(theme::results::error_hint_text()),
            ));
            lines.push(Line::from(spans));
        }
    }

    if let Some(location) = &enhanced.location {
        lines.push(Line::from(Span::styled(
            format!("jq: {location}"),
            Style::default().fg(theme::results::error_location()),
        )));
    }

    lines
}

/// Fall-back rendering for an unrecognized error: the raw text, wrapped, in
/// the error color. Keeps behavior close to the legacy verbatim overlay.
fn build_raw_error_lines(raw: &str, width: usize) -> Vec<Line<'static>> {
    raw.lines()
        .flat_map(|line| wrap_plain(line, width))
        .map(|wrapped| {
            Line::from(Span::styled(
                wrapped,
                Style::default().fg(theme::results::error_summary()),
            ))
        })
        .collect()
}

/// Greedy word-wrap by display width, preserving existing spacing within a
/// line. A token longer than `width` is emitted on its own (over-long) line
/// rather than split, which is fine for the short fix hints here.
fn wrap_plain(text: &str, width: usize) -> Vec<String> {
    use unicode_width::UnicodeWidthStr;

    if width == 0 {
        return vec![text.to_string()];
    }

    let mut lines: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut current_width = 0usize;

    for word in text.split(' ') {
        let word_width = UnicodeWidthStr::width(word);
        if current.is_empty() {
            current.push_str(word);
            current_width = word_width;
        } else if current_width + 1 + word_width <= width {
            current.push(' ');
            current.push_str(word);
            current_width += 1 + word_width;
        } else {
            lines.push(std::mem::take(&mut current));
            current.push_str(word);
            current_width = word_width;
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }
    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}

#[cfg(test)]
#[path = "error_overlay_render_tests.rs"]
mod error_overlay_render_tests;
