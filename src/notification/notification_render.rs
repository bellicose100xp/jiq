use ratatui::{
    Frame,
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
};

use super::notification_state::NotificationState;
use crate::widgets::popup;

pub fn render_notification(frame: &mut Frame, notification: &mut NotificationState) {
    notification.clear_if_expired();

    let notif = match notification.current() {
        Some(n) => n,
        None => return,
    };

    let message = &notif.message;
    let style = &notif.style;

    let content_width = message.len() as u16;
    let notification_width = content_width + 4;
    let notification_height = 3;

    let frame_area = frame.area();
    let margin = 2;
    let notification_x = frame_area.width.saturating_sub(notification_width + margin);
    let notification_y = margin;

    let notification_area = Rect {
        x: notification_x,
        y: notification_y,
        width: notification_width.min(frame_area.width.saturating_sub(margin * 2)),
        height: notification_height.min(frame_area.height.saturating_sub(margin * 2)),
    };

    // Don't render if area is too small
    if notification_area.width < 5 || notification_area.height < 3 {
        return;
    }

    popup::clear_area(frame, notification_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(style.border).bg(style.bg))
        .style(Style::default().bg(style.bg));

    let text = Line::from(Span::styled(
        format!(" {} ", message),
        Style::default().fg(style.fg).bg(style.bg),
    ));

    let paragraph = Paragraph::new(text).block(block);

    frame.render_widget(paragraph, notification_area);
}

#[cfg(test)]
#[path = "notification_render_tests.rs"]
mod notification_render_tests;
