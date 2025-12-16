use ratatui::{
    Frame,
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
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
mod tests {
    use super::*;
    use insta::assert_snapshot;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    fn create_test_terminal(width: u16, height: u16) -> Terminal<TestBackend> {
        let backend = TestBackend::new(width, height);
        Terminal::new(backend).unwrap()
    }

    fn render_notification_to_string(
        notification: &mut NotificationState,
        width: u16,
        height: u16,
    ) -> String {
        let mut terminal = create_test_terminal(width, height);
        terminal
            .draw(|f| render_notification(f, notification))
            .unwrap();
        terminal.backend().to_string()
    }

    #[test]
    fn snapshot_notification_overlay() {
        let mut notification = NotificationState::new();
        notification.show("Copied query!");

        let output = render_notification_to_string(&mut notification, 80, 24);
        assert_snapshot!(output);
    }

    #[test]
    fn snapshot_notification_top_right_position() {
        let mut notification = NotificationState::new();
        notification.show("Copied result!");

        // Use a wider terminal to verify top-right positioning
        let output = render_notification_to_string(&mut notification, 100, 30);
        assert_snapshot!(output);
    }

    #[test]
    fn snapshot_notification_no_active() {
        let mut notification = NotificationState::new();
        // No notification shown

        let output = render_notification_to_string(&mut notification, 80, 24);
        assert_snapshot!(output);
    }

    #[test]
    fn snapshot_notification_styled() {
        let mut notification = NotificationState::new();
        // Use warning duration to test different notification types
        notification.show_warning("Custom warning!");

        let output = render_notification_to_string(&mut notification, 80, 24);
        assert_snapshot!(output);
    }
}
