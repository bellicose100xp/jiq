//! Notification module for jiq
//!
//! Provides a reusable notification system that displays transient messages.
//! Any component in the application can use this module to show notifications.

mod render;
mod state;

pub use render::render_notification;
pub use state::NotificationState;
