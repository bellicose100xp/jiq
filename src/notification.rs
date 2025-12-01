//! Notification module for jiq
//!
//! Provides a reusable notification system that displays transient messages.
//! Any component in the application can use this module to show notifications.

mod notification_render;
mod notification_state;

pub use notification_render::render_notification;
pub use notification_state::NotificationState;
