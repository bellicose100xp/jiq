use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, BorderType, Borders, Paragraph},
};

use crate::app::App;
use crate::tooltip::{get_operator_content, get_tooltip_content};
use crate::widgets::popup;

const TOOLTIP_MIN_WIDTH: u16 = 40;
const TOOLTIP_MAX_WIDTH: u16 = 90;
const TOOLTIP_BORDER_HEIGHT: u16 = 2;
const TOOLTIP_BORDER_WIDTH: u16 = 4; // left border + padding + right border + padding
const TOOLTIP_MIN_HEIGHT: u16 = 8;
const TOOLTIP_MAX_HEIGHT: u16 = 18;

fn wrap_text(text: &str, max_width: usize) -> Vec<String> {
    if text.len() <= max_width {
        return vec![text.to_string()];
    }

    let mut lines = Vec::new();
    let mut current_line = String::new();

    for word in text.split_whitespace() {
        if current_line.is_empty() {
            current_line = word.to_string();
        } else if current_line.len() + 1 + word.len() <= max_width {
            current_line.push(' ');
            current_line.push_str(word);
        } else {
            lines.push(current_line);
            current_line = word.to_string();
        }
    }

    if !current_line.is_empty() {
        lines.push(current_line);
    }

    // Limit to 2 lines max for tips
    if lines.len() > 2 {
        lines.truncate(2);
    }

    lines
}

/// Render the tooltip popup
///
/// Returns the popup area for region tracking.
pub fn render_popup(app: &App, frame: &mut Frame, input_area: Rect) -> Option<Rect> {
    // Determine what to show: function takes priority over operator
    let (title_prefix, name, content) = if let Some(func) = &app.tooltip.current_function {
        if let Some(c) = get_tooltip_content(func) {
            ("fn", func.as_str(), c)
        } else {
            return None;
        }
    } else if let Some(op) = &app.tooltip.current_operator {
        if let Some(c) = get_operator_content(op) {
            ("operator", op.as_str(), c)
        } else {
            return None;
        }
    } else {
        return None;
    };

    // Parse examples into (code, description) pairs
    let parsed_examples: Vec<(&str, &str)> = content
        .examples
        .iter()
        .map(|e| {
            if let Some(idx) = e.find('#') {
                (e[..idx].trim_end(), e[idx + 1..].trim_start())
            } else {
                (*e, "")
            }
        })
        .collect();

    // Calculate the max code width for alignment
    let max_code_width = parsed_examples
        .iter()
        .map(|(code, _)| code.len())
        .max()
        .unwrap_or(0);

    // Calculate required width based on content (code + separator + description)
    let description_width = content.description.len();
    let max_example_width = parsed_examples
        .iter()
        .map(|(code, desc)| {
            if desc.is_empty() {
                code.len() + 2 // just code + indent
            } else {
                max_code_width + 3 + desc.len() + 2 // code + " │ " + desc + indent
            }
        })
        .max()
        .unwrap_or(0);
    // Don't let tip width drive popup width - tips will wrap
    let dismiss_hint_len = 19; // "Ctrl+T to dismiss"
    // Title format: "fn: name" or "operator: op"
    let title_width = title_prefix.len() + 2 + name.len() + dismiss_hint_len + 4; // prefix + ": " + name + dismiss + spacing

    let content_width = description_width.max(max_example_width).max(title_width);

    let popup_width =
        ((content_width as u16) + TOOLTIP_BORDER_WIDTH).clamp(TOOLTIP_MIN_WIDTH, TOOLTIP_MAX_WIDTH);

    // Calculate tip wrapping - available width for tip text
    let tip_available_width = (popup_width as usize).saturating_sub(6); // borders + padding + emoji
    let wrapped_tip_lines: Vec<String> = if let Some(tip) = content.tip {
        wrap_text(tip, tip_available_width)
    } else {
        Vec::new()
    };
    let tip_line_count = wrapped_tip_lines.len() as u16;

    // Calculate content height: description (1) + blank (1) + examples + tip (blank + lines)
    let example_count = parsed_examples.len() as u16;
    let tip_height = if content.tip.is_some() {
        1 + tip_line_count // blank line + wrapped tip lines
    } else {
        0
    };
    let content_height = 1 + 1 + example_count + tip_height; // description + blank + examples + tip
    let popup_height =
        (content_height + TOOLTIP_BORDER_HEIGHT).clamp(TOOLTIP_MIN_HEIGHT, TOOLTIP_MAX_HEIGHT);

    // Position popup on the right side, above input box
    let frame_area = frame.area();
    // Allow up to 75% of screen width for tooltip
    let max_allowed_width = (frame_area.width * 3) / 4;
    let final_width = popup_width.min(max_allowed_width);

    // Position on right side with some margin
    let popup_x = frame_area.width.saturating_sub(final_width + 2);
    let popup_y = input_area.y.saturating_sub(popup_height);

    let popup_area = Rect {
        x: popup_x,
        y: popup_y,
        width: final_width,
        height: popup_height.min(input_area.y),
    };

    // Clear the background for floating effect
    popup::clear_area(frame, popup_area);

    // Build content lines
    let mut lines: Vec<Line> = Vec::new();

    // Description line
    lines.push(Line::from(vec![Span::styled(
        content.description,
        Style::default().fg(Color::White),
    )]));

    // Blank line before examples
    lines.push(Line::from(""));

    // Examples with two-column layout: code │ description
    for (code, desc) in &parsed_examples {
        if desc.is_empty() {
            // No description, just show code
            lines.push(Line::from(vec![Span::styled(
                format!("  {}", code),
                Style::default().fg(Color::Cyan),
            )]));
        } else {
            // Two-column: code (padded) │ description
            let padded_code = format!("{:width$}", code, width = max_code_width);
            lines.push(Line::from(vec![
                Span::styled(
                    format!("  {}", padded_code),
                    Style::default().fg(Color::Cyan),
                ),
                Span::styled(" │ ", Style::default().fg(Color::DarkGray)),
                Span::styled(*desc, Style::default().fg(Color::Gray)),
            ]));
        }
    }

    // Optional tip (with wrapping)
    if content.tip.is_some() && !wrapped_tip_lines.is_empty() {
        lines.push(Line::from(""));
        // First line with emoji prefix
        lines.push(Line::from(vec![
            Span::styled("💡 ", Style::default()),
            Span::styled(
                wrapped_tip_lines[0].clone(),
                Style::default().fg(Color::Yellow),
            ),
        ]));
        // Subsequent lines with spacing to align with first line
        for line in wrapped_tip_lines.iter().skip(1) {
            lines.push(Line::from(vec![
                Span::raw("   "), // 3 spaces to align with text after emoji
                Span::styled(line.clone(), Style::default().fg(Color::Yellow)),
            ]));
        }
    }

    let text = Text::from(lines);

    // Build title with prefix and name in purple (left side)
    // Format: "fn: select" or "operator: //"
    let title = Line::from(vec![
        Span::raw(" "),
        Span::styled(
            format!("{}: {}", title_prefix, name),
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" "),
    ]);

    // Build dismiss hint for top-right of border
    let dismiss_hint = Line::from(vec![Span::styled(
        " Ctrl+T to dismiss ",
        Style::default().fg(Color::DarkGray),
    )]);

    // Create the popup widget with purple border
    // Title on top-left, dismiss hint on top-right
    let popup_widget = Paragraph::new(text).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(title)
            .title_top(dismiss_hint.alignment(ratatui::layout::Alignment::Right))
            .border_style(Style::default().fg(Color::Magenta))
            .style(Style::default().bg(Color::Black)),
    );

    frame.render_widget(popup_widget, popup_area);

    Some(popup_area)
}

#[cfg(test)]
pub fn format_tooltip_title(is_function: bool, name: &str) -> String {
    if is_function {
        format!("fn: {}", name)
    } else {
        format!("operator: {}", name)
    }
}

#[cfg(test)]
#[path = "tooltip_render_tests.rs"]
mod tooltip_render_tests;
