use ratatui::{
    Frame,
    layout::{Constraint, Layout},
};

use super::app_state::App;
use crate::notification::render_notification;

impl App {
    pub fn render(&mut self, frame: &mut Frame) {
        self.frame_count = self.frame_count.wrapping_add(1);

        let layout = Layout::vertical([
            Constraint::Min(3),
            Constraint::Length(3),
            Constraint::Length(1),
        ])
        .split(frame.area());

        let results_area = layout[0];
        let input_area = layout[1];
        let help_area = layout[2];

        crate::results::results_render::render_pane(self, frame, results_area);

        crate::input::input_render::render_field(self, frame, input_area);

        crate::help::help_line_render::render_line(self, frame, help_area);

        if self.ai.visible && self.query.is_some() {
            crate::ai::ai_render::render_popup(&mut self.ai, frame, input_area);
        } else if self.tooltip.should_show() {
            crate::tooltip::tooltip_render::render_popup(self, frame, input_area);
        }

        if self.autocomplete.is_visible() {
            crate::autocomplete::autocomplete_render::render_popup(self, frame, input_area);
        }

        if self.history.is_visible() {
            crate::history::history_render::render_popup(self, frame, input_area);
        }

        if self.error_overlay_visible
            && let Some(query) = &self.query
            && query.result.is_err()
        {
            crate::results::results_render::render_error_overlay(self, frame, results_area);
        }

        if self.help.visible {
            crate::help::help_popup_render::render_popup(self, frame);
        }

        render_notification(frame, &mut self.notification);
    }
}
