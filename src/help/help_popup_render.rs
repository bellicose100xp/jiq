use ratatui::{
    Frame,
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph},
};

use crate::app::App;
use crate::help::{HELP_ENTRIES, HELP_FOOTER};
use crate::widgets::popup;

pub const HELP_POPUP_WIDTH: u16 = 70;
pub const HELP_POPUP_PADDING: u16 = 4;

pub fn render_popup(app: &mut App, frame: &mut Frame) {
    let content_height = HELP_ENTRIES.len() as u16;
    let ideal_popup_height = content_height + HELP_POPUP_PADDING;
    let ideal_popup_width = HELP_POPUP_WIDTH;

    let frame_area = frame.area();
    let popup_width = ideal_popup_width.min(frame_area.width);
    let popup_height = ideal_popup_height.min(frame_area.height);

    if frame_area.width < 20 || frame_area.height < 10 {
        return;
    }

    let popup_area = popup::centered_popup(frame_area, popup_width, popup_height);
    popup::clear_area(frame, popup_area);

    let mut lines: Vec<Line> = Vec::new();

    for (key, desc) in HELP_ENTRIES {
        if key.is_empty() && desc.is_empty() {
            lines.push(Line::from(""));
        } else if key.is_empty() {
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(
                    *desc,
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
            ]));
        } else {
            let key_span = Span::styled(
                format!("  {:<15}", key),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            );
            let desc_span = Span::styled(*desc, Style::default().fg(Color::White));
            lines.push(Line::from(vec![key_span, desc_span]));
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::from(vec![Span::styled(
        format!("           {}          ", HELP_FOOTER),
        Style::default().fg(Color::DarkGray),
    )]));

    let help_text = Text::from(lines.clone());

    let content_height = lines.len() as u32;
    let visible_height = popup_height.saturating_sub(2);
    app.help
        .scroll
        .update_bounds(content_height, visible_height);

    let popup = Paragraph::new(help_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Keyboard Shortcuts ")
                .border_style(Style::default().fg(Color::Cyan))
                .style(Style::default().bg(Color::Black)),
        )
        .scroll((app.help.scroll.offset, 0));

    frame.render_widget(popup, popup_area);
}
