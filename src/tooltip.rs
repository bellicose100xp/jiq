//! Tooltip module
//!
//! Provides TLDR-style contextual help for jq functions and operators.
//! When enabled (default), a tooltip automatically appears when the cursor
//! is on a recognized jq function or operator.

mod tooltip_content;
mod operator_content;
mod detector;
pub mod tooltip_events;
pub mod tooltip_render;
mod tooltip_state;

pub use tooltip_content::get_tooltip_content;
pub use operator_content::get_operator_content;
pub use detector::detect_function_at_cursor;
pub use detector::detect_operator_at_cursor;
pub use tooltip_state::TooltipState;
pub use tooltip_state::update_tooltip_from_app;
