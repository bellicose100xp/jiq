use ratatui::{
    Frame,
    layout::{Constraint, Layout},
};

use super::app_state::App;
use crate::notification::render_notification;

impl App {
    pub fn render(&mut self, frame: &mut Frame) {
        self.frame_count = self.frame_count.wrapping_add(1);
        self.layout_regions.clear();

        let overlay_visible = self.search.is_visible() || self.snippets.is_visible();

        let (results_area, input_area, help_area) = if overlay_visible {
            let layout =
                Layout::vertical([Constraint::Min(3), Constraint::Length(1)]).split(frame.area());
            (layout[0], None, layout[1])
        } else {
            let layout = Layout::vertical([
                Constraint::Min(3),
                Constraint::Length(3),
                Constraint::Length(1),
            ])
            .split(frame.area());
            (layout[0], Some(layout[1]), layout[2])
        };

        let (results_rect, search_rect) =
            crate::results::results_render::render_pane(self, frame, results_area);
        self.layout_regions.results_pane = Some(results_rect);
        if let Some(search_rect) = search_rect {
            self.layout_regions.search_bar = Some(search_rect);
        }

        if let Some(input_area) = input_area {
            let input_rect = crate::input::input_render::render_field(self, frame, input_area);
            self.layout_regions.input_field = Some(input_rect);
        }

        crate::help::help_line_render::render_line(self, frame, help_area);

        if let Some(input_area) = input_area {
            if self.ai.visible
                && self.query.is_some()
                && let Some(ai_rect) =
                    crate::ai::ai_render::render_popup(&mut self.ai, frame, input_area)
            {
                self.layout_regions.ai_window = Some(ai_rect);
            } else if self.tooltip.should_show()
                && let Some(tooltip_rect) =
                    crate::tooltip::tooltip_render::render_popup(self, frame, input_area)
            {
                self.layout_regions.tooltip = Some(tooltip_rect);
            }

            if self.autocomplete.is_visible()
                && let Some(autocomplete_rect) =
                    crate::autocomplete::autocomplete_render::render_popup(self, frame, input_area)
            {
                self.layout_regions.autocomplete = Some(autocomplete_rect);
            }

            if self.history.is_visible()
                && let Some(history_rect) =
                    crate::history::history_render::render_popup(self, frame, input_area)
            {
                self.layout_regions.history_popup = Some(history_rect);
            }
        }

        if self.snippets.is_visible() {
            let (list_rect, preview_rect) = crate::snippets::snippet_render::render_popup(
                &mut self.snippets,
                frame,
                results_area,
            );
            if let Some(list_rect) = list_rect {
                self.layout_regions.snippet_list = Some(list_rect);
            }
            if let Some(preview_rect) = preview_rect {
                self.layout_regions.snippet_preview = Some(preview_rect);
            }
        }

        if self.error_overlay_visible
            && let Some(query) = &self.query
            && query.result.is_err()
            && let Some(error_rect) =
                crate::results::results_render::render_error_overlay(self, frame, results_area)
        {
            self.layout_regions.error_overlay = Some(error_rect);
        }

        if self.help.visible
            && let Some(help_rect) = crate::help::help_popup_render::render_popup(self, frame)
        {
            self.layout_regions.help_popup = Some(help_rect);
        }

        render_notification(frame, &mut self.notification);
    }
}
