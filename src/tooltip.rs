mod detector;
mod operator_content;
mod tooltip_content;
pub mod tooltip_events;
pub mod tooltip_render;
mod tooltip_state;

pub use detector::detect_function_at_cursor;
pub use detector::detect_operator_at_cursor;
pub use operator_content::get_operator_content;
pub use tooltip_content::get_tooltip_content;
pub use tooltip_state::TooltipState;
pub use tooltip_state::update_tooltip_from_app;
