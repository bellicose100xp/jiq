//! Tests for notification_state

use super::*;
use crate::theme;
use std::thread;

#[test]
fn test_info_notification() {
    let notif = Notification::new("Test message");
    assert_eq!(notif.message, "Test message");
    assert_eq!(notif.notification_type, NotificationType::Info);
    assert_eq!(notif.duration, Some(Duration::from_millis(1500)));
    assert_eq!(notif.style.fg, theme::notification::INFO.fg);
    assert_eq!(notif.style.bg, theme::notification::INFO.bg);
    assert!(!notif.is_expired());
}

#[test]
fn test_warning_notification() {
    let notif = Notification::with_type("Warning!", NotificationType::Warning);
    assert_eq!(notif.message, "Warning!");
    assert_eq!(notif.notification_type, NotificationType::Warning);
    assert_eq!(notif.duration, Some(Duration::from_secs(10)));
    assert_eq!(notif.style.fg, theme::notification::WARNING.fg);
    assert_eq!(notif.style.bg, theme::notification::WARNING.bg);
}

#[test]
fn test_error_notification() {
    let notif = Notification::with_type("Error!", NotificationType::Error);
    assert_eq!(notif.message, "Error!");
    assert_eq!(notif.notification_type, NotificationType::Error);
    assert_eq!(notif.duration, None); // Permanent
    assert_eq!(notif.style.fg, theme::notification::ERROR.fg);
    assert_eq!(notif.style.bg, theme::notification::ERROR.bg);
}

#[test]
fn test_notification_expiration() {
    let mut notif = Notification::new("Expiring");
    notif.duration = Some(Duration::from_millis(10));
    assert!(!notif.is_expired());
    thread::sleep(Duration::from_millis(20));
    assert!(notif.is_expired());
}

#[test]
fn test_notification_state_show() {
    let mut state = NotificationState::new();
    assert!(state.current().is_none());

    state.show("Hello");
    assert!(state.current().is_some());
    assert_eq!(state.current_message(), Some("Hello"));
}

#[test]
fn test_notification_state_show_warning() {
    let mut state = NotificationState::new();
    state.show_warning("Config invalid");

    let notif = state.current().unwrap();
    assert_eq!(notif.message, "Config invalid");
    assert_eq!(notif.notification_type, NotificationType::Warning);
    assert_eq!(notif.duration, Some(Duration::from_secs(10)));
}

#[test]
fn test_notification_replacement() {
    let mut state = NotificationState::new();
    state.show("First");
    assert_eq!(state.current_message(), Some("First"));

    state.show("Second");
    assert_eq!(state.current_message(), Some("Second"));
}

#[test]
fn test_clear_if_expired() {
    let mut state = NotificationState::new();
    state.show("Test");

    // Manually set a very short duration
    if let Some(ref mut notif) = state.current {
        notif.duration = Some(Duration::from_millis(10));
    }

    assert!(!state.clear_if_expired()); // Not expired yet
    thread::sleep(Duration::from_millis(20));
    assert!(state.clear_if_expired()); // Now expired
    assert!(state.current().is_none());
}

#[test]
fn test_error_notification_never_expires() {
    let mut state = NotificationState::new();
    state.show_error("Critical error");

    let notif = state.current().unwrap();
    assert_eq!(notif.notification_type, NotificationType::Error);
    assert_eq!(notif.duration, None);
    assert!(!notif.is_expired()); // Should never expire
    assert!(!state.clear_if_expired()); // Should not clear
    assert!(state.current().is_some());
}
