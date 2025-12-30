//! Tests for notification_render

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

#[test]
fn snapshot_notification_error_brief() {
    let mut notification = NotificationState::new();
    notification.show_error("Failed to load file");

    let output = render_notification_to_string(&mut notification, 80, 24);
    assert_snapshot!(output);
}
