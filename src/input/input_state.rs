use ratatui::{
    style::{Color, Style},
    widgets::{Block, Borders},
};
use tui_textarea::TextArea;

use crate::autocomplete::BraceTracker;
use crate::editor::EditorMode;

pub struct InputState {
    pub textarea: TextArea<'static>,
    pub editor_mode: EditorMode,
    pub scroll_offset: usize,
    pub brace_tracker: BraceTracker,
}

impl InputState {
    pub fn new() -> Self {
        let mut textarea = TextArea::default();

        textarea.set_block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Query ")
                .border_style(Style::default().fg(Color::DarkGray)),
        );

        textarea.set_cursor_line_style(Style::default());

        Self {
            textarea,
            editor_mode: EditorMode::default(),
            scroll_offset: 0,
            brace_tracker: BraceTracker::new(),
        }
    }

    pub fn query(&self) -> &str {
        self.textarea.lines()[0].as_ref()
    }

    pub fn calculate_scroll_offset(&mut self, viewport_width: usize) {
        let cursor_col = self.textarea.cursor().1;
        let text_length = self.query().chars().count();

        let mut new_scroll = self.scroll_offset;

        if cursor_col < new_scroll {
            new_scroll = cursor_col;
        } else if cursor_col >= new_scroll + viewport_width {
            new_scroll = cursor_col + 1 - viewport_width;
        }

        if text_length < new_scroll + viewport_width {
            let min_scroll = text_length.saturating_sub(viewport_width);
            let max_scroll_for_cursor = cursor_col.saturating_sub(viewport_width - 1);
            new_scroll = new_scroll.min(min_scroll.max(max_scroll_for_cursor));
        }

        self.scroll_offset = new_scroll;
    }
}

impl Default for InputState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[path = "input_state_tests.rs"]
mod input_state_tests;
