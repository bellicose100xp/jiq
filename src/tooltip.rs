//! Tooltip module
//!
//! Provides TLDR-style contextual help for jq functions.
//! When enabled (default), a tooltip automatically appears when the cursor
//! is on a recognized jq function.

mod tooltip_content;
mod detector;
pub mod tooltip_events;
pub mod tooltip_render;
mod tooltip_state;

pub use tooltip_content::get_tooltip_content;
pub use detector::detect_function_at_cursor;
pub use tooltip_state::TooltipState;
