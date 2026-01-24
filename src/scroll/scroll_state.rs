#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScrollState {
    pub offset: u16,
    pub max_offset: u16,
    pub viewport_height: u16,
    pub h_offset: u16,
    pub max_h_offset: u16,
    pub viewport_width: u16,
}

impl ScrollState {
    pub fn new() -> Self {
        Self {
            offset: 0,
            max_offset: 0,
            viewport_height: 0,
            h_offset: 0,
            max_h_offset: 0,
            viewport_width: 0,
        }
    }

    pub fn update_bounds(&mut self, content_lines: u32, viewport_height: u16) {
        self.viewport_height = viewport_height;

        // Clamp to u16::MAX for ratatui compatibility
        self.max_offset = content_lines
            .saturating_sub(viewport_height as u32)
            .min(u16::MAX as u32) as u16;

        self.offset = self.offset.min(self.max_offset);
    }

    pub fn scroll_down(&mut self, lines: u16) {
        self.offset = self.offset.saturating_add(lines).min(self.max_offset);
    }

    pub fn scroll_up(&mut self, lines: u16) {
        self.offset = self.offset.saturating_sub(lines);
    }

    pub fn page_down(&mut self) {
        let half_page = self.viewport_height / 2;
        self.scroll_down(half_page);
    }

    pub fn page_up(&mut self) {
        let half_page = self.viewport_height / 2;
        self.scroll_up(half_page);
    }

    pub fn jump_to_top(&mut self) {
        self.offset = 0;
    }

    pub fn jump_to_bottom(&mut self) {
        self.offset = self.max_offset;
    }

    pub fn update_h_bounds(&mut self, max_line_width: u16, viewport_width: u16) {
        self.viewport_width = viewport_width;
        self.max_h_offset = max_line_width.saturating_sub(viewport_width);
        self.h_offset = self.h_offset.min(self.max_h_offset);
    }

    pub fn scroll_right(&mut self, cols: u16) {
        self.h_offset = self.h_offset.saturating_add(cols).min(self.max_h_offset);
    }

    pub fn scroll_left(&mut self, cols: u16) {
        self.h_offset = self.h_offset.saturating_sub(cols);
    }

    pub fn jump_to_left(&mut self) {
        self.h_offset = 0;
    }

    pub fn reset(&mut self) {
        self.offset = 0;
        self.h_offset = 0;
    }

    pub fn ensure_cursor_visible(&mut self, cursor_line: u32) {
        use crate::results::cursor_state::SCROLLOFF;

        if self.viewport_height == 0 {
            return;
        }

        let cursor = cursor_line.min(u16::MAX as u32) as u16;
        let effective_scrolloff = SCROLLOFF.min(self.viewport_height / 2);

        let visible_start = self.offset;
        let visible_end = self.offset.saturating_add(self.viewport_height);

        if cursor < visible_start.saturating_add(effective_scrolloff) {
            self.offset = cursor.saturating_sub(effective_scrolloff);
        } else if cursor >= visible_end.saturating_sub(effective_scrolloff) {
            let new_offset = cursor
                .saturating_add(effective_scrolloff)
                .saturating_add(1)
                .saturating_sub(self.viewport_height);
            self.offset = new_offset.min(self.max_offset);
        }
    }
}

impl Default for ScrollState {
    fn default() -> Self {
        Self::new()
    }
}
