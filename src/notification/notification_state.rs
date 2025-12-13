//! Notification state management
//!
//! Provides structures for displaying transient notifications in the UI.

use ratatui::style::Color;
use std::time::{Duration, Instant};

/// Notification type - determines style and duration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NotificationType {
    /// Info (gray) - short duration (1.5s) - for confirmations like "Copied!"
    #[default]
    Info,
    /// Warning (yellow) - long duration (5s) - for warnings like invalid config
    Warning,
    /// Error (red) - permanent until dismissed - for critical errors
    Error,
}

impl NotificationType {
    /// Get the duration for this notification type
    fn duration(self) -> Option<Duration> {
        match self {
            NotificationType::Info => Some(Duration::from_millis(1500)),
            NotificationType::Warning => Some(Duration::from_secs(10)),
            NotificationType::Error => None, // Permanent
        }
    }

    /// Get the style for this notification type
    fn style(self) -> NotificationStyle {
        match self {
            NotificationType::Info => NotificationStyle {
                fg: Color::White,
                bg: Color::DarkGray,
                border: Color::Gray,
            },
            NotificationType::Warning => NotificationStyle {
                fg: Color::Black,
                bg: Color::Yellow,
                border: Color::Yellow,
            },
            NotificationType::Error => NotificationStyle {
                fg: Color::White,
                bg: Color::Red,
                border: Color::LightRed,
            },
        }
    }
}

/// Style configuration for a notification
#[derive(Debug, Clone)]
pub struct NotificationStyle {
    pub fg: Color,
    pub bg: Color,
    pub border: Color,
}

impl Default for NotificationStyle {
    fn default() -> Self {
        NotificationType::Info.style()
    }
}

/// A single notification with message, timing, and style
#[derive(Debug, Clone)]
pub struct Notification {
    pub message: String,
    pub style: NotificationStyle,
    /// The type of notification (Info, Warning, Error).
    ///
    /// TODO: Remove #[allow(dead_code)] when this field is used in production code.
    /// Currently only used in tests for assertions, but kept as useful metadata.
    #[allow(dead_code)]
    pub notification_type: NotificationType,
    pub created_at: Instant,
    pub duration: Option<Duration>, // None = permanent
}

impl Notification {
    /// Create a new info notification (short duration, gray style)
    pub fn new(message: &str) -> Self {
        Self::with_type(message, NotificationType::Info)
    }

    /// Create a notification with specified type
    pub fn with_type(message: &str, notification_type: NotificationType) -> Self {
        Self {
            message: message.to_string(),
            style: notification_type.style(),
            notification_type,
            created_at: Instant::now(),
            duration: notification_type.duration(),
        }
    }

    /// Check if notification has expired
    pub fn is_expired(&self) -> bool {
        match self.duration {
            Some(d) => self.created_at.elapsed() > d,
            None => false, // Permanent notifications never expire
        }
    }
}

/// Notification state manager for the application
#[derive(Debug, Default)]
pub struct NotificationState {
    pub current: Option<Notification>,
}

impl NotificationState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Show an info notification (gray, 1.5s)
    pub fn show(&mut self, message: &str) {
        self.current = Some(Notification::new(message));
    }

    /// Show a notification with specified type
    pub fn show_with_type(&mut self, message: &str, notification_type: NotificationType) {
        self.current = Some(Notification::with_type(message, notification_type));
    }

    /// Show a warning notification (yellow, 5s)
    pub fn show_warning(&mut self, message: &str) {
        self.show_with_type(message, NotificationType::Warning);
    }

    /// Show an error notification (red, permanent until dismissed).
    ///
    /// TODO: Remove #[allow(dead_code)] when this method is used in production code.
    /// This method is kept for future use (e.g., critical errors that block operation).
    #[allow(dead_code)]
    pub fn show_error(&mut self, message: &str) {
        self.show_with_type(message, NotificationType::Error);
    }

    /// Dismiss the current notification.
    ///
    /// TODO: Remove #[allow(dead_code)] when this method is used in production code.
    /// This method is kept for future use with permanent error notifications that
    /// need to be dismissed by user action (e.g., pressing a key).
    #[allow(dead_code)]
    pub fn dismiss(&mut self) {
        self.current = None;
    }

    /// Clear expired notification, returns true if cleared
    pub fn clear_if_expired(&mut self) -> bool {
        if let Some(ref notif) = self.current
            && notif.is_expired()
        {
            self.current = None;
            return true;
        }
        false
    }

    /// Get current notification if visible
    pub fn current(&self) -> Option<&Notification> {
        self.current.as_ref()
    }

    /// Get current notification message if visible (test-only)
    #[cfg(test)]
    pub fn current_message(&self) -> Option<&str> {
        self.current.as_ref().map(|n| n.message.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    // ==================== Unit Tests ====================

    #[test]
    fn test_info_notification() {
        let notif = Notification::new("Test message");
        assert_eq!(notif.message, "Test message");
        assert_eq!(notif.notification_type, NotificationType::Info);
        assert_eq!(notif.duration, Some(Duration::from_millis(1500)));
        assert_eq!(notif.style.fg, Color::White);
        assert_eq!(notif.style.bg, Color::DarkGray);
        assert!(!notif.is_expired());
    }

    #[test]
    fn test_warning_notification() {
        let notif = Notification::with_type("Warning!", NotificationType::Warning);
        assert_eq!(notif.message, "Warning!");
        assert_eq!(notif.notification_type, NotificationType::Warning);
        assert_eq!(notif.duration, Some(Duration::from_secs(10)));
        assert_eq!(notif.style.fg, Color::Black);
        assert_eq!(notif.style.bg, Color::Yellow);
    }

    #[test]
    fn test_error_notification() {
        let notif = Notification::with_type("Error!", NotificationType::Error);
        assert_eq!(notif.message, "Error!");
        assert_eq!(notif.notification_type, NotificationType::Error);
        assert_eq!(notif.duration, None); // Permanent
        assert_eq!(notif.style.fg, Color::White);
        assert_eq!(notif.style.bg, Color::Red);
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
        assert!(state.current().is_some()); // Still there
    }

    // ==================== Property-Based Tests ====================

    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// Feature: clipboard, Property 4: Notification replacement
        ///
        /// For any sequence of notification messages, only the most recent
        /// notification should be visible.
        #[test]
        fn prop_notification_replacement(messages in prop::collection::vec("[a-zA-Z0-9 ]{1,50}", 1..10)) {
            let mut state = NotificationState::new();

            for msg in &messages {
                state.show(msg);
            }

            // Only the last message should be current
            let last_message = messages.last().unwrap();
            prop_assert_eq!(state.current_message(), Some(last_message.as_str()));
        }
    }
}
