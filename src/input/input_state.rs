use ratatui::{
    style::Style,
    widgets::{Block, Borders},
};
use tui_textarea::TextArea;

use crate::autocomplete::BraceTracker;
use crate::editor::{CharSearchState, EditorMode};
use crate::theme;

pub struct InputState {
    pub textarea: TextArea<'static>,
    pub editor_mode: EditorMode,
    pub scroll_offset: usize,
    pub brace_tracker: BraceTracker,
    pub last_char_search: Option<CharSearchState>,
    pub manual_scroll_active: bool,
}

impl InputState {
    pub fn new() -> Self {
        let mut textarea = TextArea::default();

        textarea.set_block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Query ")
                .border_style(Style::default().fg(theme::input::BORDER_UNFOCUSED)),
        );

        textarea.set_cursor_line_style(Style::default());

        Self {
            textarea,
            editor_mode: EditorMode::default(),
            scroll_offset: 0,
            brace_tracker: BraceTracker::new(),
            last_char_search: None,
            manual_scroll_active: false,
        }
    }

    pub fn query(&self) -> &str {
        self.textarea.lines()[0].as_ref()
    }

    pub fn calculate_scroll_offset(&mut self, viewport_width: usize) {
        let cursor_col = self.textarea.cursor().1;
        let text_length = self.query().chars().count();

        let mut new_scroll = self.scroll_offset;

        if !self.manual_scroll_active {
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
        } else if text_length < new_scroll + viewport_width {
            new_scroll = new_scroll.min(text_length.saturating_sub(viewport_width));
        }

        self.scroll_offset = new_scroll;
    }

    /// Scroll the input horizontally by the given amount
    ///
    /// Positive values scroll right (showing later characters),
    /// negative values scroll left (showing earlier characters).
    /// The scroll is clamped to valid bounds.
    pub fn scroll_horizontal(&mut self, delta: isize, text_length: usize) {
        let new_offset = if delta < 0 {
            self.scroll_offset.saturating_sub(delta.unsigned_abs())
        } else {
            self.scroll_offset.saturating_add(delta as usize)
        };
        self.scroll_offset = new_offset.min(text_length);
        self.manual_scroll_active = true;
    }

    pub fn reset_manual_scroll(&mut self) {
        self.manual_scroll_active = false;
    }

    /// Move cursor to a specific column position
    pub fn set_cursor_column(&mut self, target_col: usize) {
        use tui_textarea::CursorMove;

        let current_col = self.textarea.cursor().1;
        let text_length = self.query().chars().count();
        let target_col = target_col.min(text_length);

        match target_col.cmp(&current_col) {
            std::cmp::Ordering::Less => {
                for _ in 0..(current_col - target_col) {
                    self.textarea.move_cursor(CursorMove::Back);
                }
            }
            std::cmp::Ordering::Greater => {
                for _ in 0..(target_col - current_col) {
                    self.textarea.move_cursor(CursorMove::Forward);
                }
            }
            std::cmp::Ordering::Equal => {}
        }
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
