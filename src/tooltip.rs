//! Tooltip module
//!
//! Provides TLDR-style contextual help for jq functions.
//! When enabled (default), a tooltip automatically appears when the cursor
//! is on a recognized jq function.

mod content;
mod detector;
pub mod events;
mod state;

pub use content::get_tooltip_content;
pub use detector::detect_function_at_cursor;
pub use state::TooltipState;
