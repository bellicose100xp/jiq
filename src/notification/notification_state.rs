use ratatui::style::Color;
use std::time::{Duration, Instant};

use crate::theme;

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
    fn duration(self) -> Option<Duration> {
        match self {
            NotificationType::Info => Some(Duration::from_millis(1500)),
            NotificationType::Warning => Some(Duration::from_secs(10)),
            NotificationType::Error => None, // Permanent
        }
    }

    fn style(self) -> NotificationStyle {
        match self {
            NotificationType::Info => NotificationStyle {
                fg: theme::notification::INFO.fg,
                bg: theme::notification::INFO.bg,
                border: theme::notification::INFO.border,
            },
            NotificationType::Warning => NotificationStyle {
                fg: theme::notification::WARNING.fg,
                bg: theme::notification::WARNING.bg,
                border: theme::notification::WARNING.border,
            },
            NotificationType::Error => NotificationStyle {
                fg: theme::notification::ERROR.fg,
                bg: theme::notification::ERROR.bg,
                border: theme::notification::ERROR.border,
            },
        }
    }
}

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

#[derive(Debug, Clone)]
pub struct Notification {
    pub message: String,
    pub style: NotificationStyle,
    /// The type of notification (test assertion metadata)
    #[allow(dead_code)]
    pub notification_type: NotificationType,
    pub created_at: Instant,
    pub duration: Option<Duration>,
}

impl Notification {
    pub fn new(message: &str) -> Self {
        Self::with_type(message, NotificationType::Info)
    }

    pub fn with_type(message: &str, notification_type: NotificationType) -> Self {
        Self {
            message: message.to_string(),
            style: notification_type.style(),
            notification_type,
            created_at: Instant::now(),
            duration: notification_type.duration(),
        }
    }

    pub fn is_expired(&self) -> bool {
        match self.duration {
            Some(d) => self.created_at.elapsed() > d,
            None => false, // Permanent notifications never expire
        }
    }
}

#[derive(Debug, Default)]
pub struct NotificationState {
    pub current: Option<Notification>,
}

impl NotificationState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn show(&mut self, message: &str) {
        self.current = Some(Notification::new(message));
    }

    pub fn show_with_type(&mut self, message: &str, notification_type: NotificationType) {
        self.current = Some(Notification::with_type(message, notification_type));
    }

    pub fn show_warning(&mut self, message: &str) {
        self.show_with_type(message, NotificationType::Warning);
    }

    /// Show an error notification (red, permanent until dismissed).
    ///
    /// This method is kept for future use (e.g., critical errors that block operation).
    pub fn show_error(&mut self, message: &str) {
        self.show_with_type(message, NotificationType::Error);
    }

    /// Dismiss the current notification (test helper)
    ///
    /// Note: Production code uses auto-expiry via clear_if_expired().
    #[cfg(test)]
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

    pub fn current(&self) -> Option<&Notification> {
        self.current.as_ref()
    }

    #[cfg(test)]
    pub fn current_message(&self) -> Option<&str> {
        self.current.as_ref().map(|n| n.message.as_str())
    }
}

#[cfg(test)]
#[path = "notification_state_tests.rs"]
mod notification_state_tests;
