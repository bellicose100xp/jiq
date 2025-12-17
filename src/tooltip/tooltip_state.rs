use crate::app::App;
use crate::tooltip::{detect_function_at_cursor, detect_operator_at_cursor};

/// Update tooltip state based on cursor position. Functions take priority over operators.
pub fn update_tooltip_from_app(app: &mut App) {
    let query = app.input.query();
    let cursor_pos = app.input.textarea.cursor().1; // Column position

    // Detect function (takes priority)
    let detected_function = detect_function_at_cursor(query, cursor_pos);
    app.tooltip
        .set_current_function(detected_function.map(|s| s.to_string()));

    // Detect operator only if no function detected (function takes priority)
    let detected_operator = if detected_function.is_none() {
        detect_operator_at_cursor(query, cursor_pos)
    } else {
        None
    };
    app.tooltip
        .set_current_operator(detected_operator.map(|s| s.to_string()));
}

pub struct TooltipState {
    /// Whether tooltip feature is enabled (shows automatically)
    pub enabled: bool,
    /// Currently detected function name (if any)
    pub current_function: Option<String>,
    /// Currently detected operator (if any)
    pub current_operator: Option<String>,
}

impl TooltipState {
    pub fn new(auto_show: bool) -> Self {
        Self {
            enabled: auto_show,
            current_function: None,
            current_operator: None,
        }
    }

    pub fn toggle(&mut self) {
        self.enabled = !self.enabled;
    }

    pub fn set_current_function(&mut self, func: Option<String>) {
        self.current_function = func;
    }

    pub fn set_current_operator(&mut self, op: Option<String>) {
        self.current_operator = op;
    }

    pub fn should_show(&self) -> bool {
        self.enabled && (self.current_function.is_some() || self.current_operator.is_some())
    }
}

impl Default for TooltipState {
    fn default() -> Self {
        Self::new(true)
    }
}

#[cfg(test)]
#[path = "tooltip_state_tests.rs"]
mod tooltip_state_tests;
