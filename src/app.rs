mod app_events;
mod app_render;
mod app_state;
mod mouse_click;
mod mouse_events;
mod mouse_hover;
mod mouse_scroll;
mod paste_recovery_render;

#[cfg(test)]
mod app_render_tests;

// Re-export public types
pub use app_state::{App, Focus, OutputMode};
