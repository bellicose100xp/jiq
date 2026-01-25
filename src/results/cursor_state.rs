use std::sync::Arc;

pub const SCROLLOFF: u16 = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectionMode {
    Normal,
    Visual,
}

#[derive(Debug, Clone)]
pub struct CursorState {
    cursor_line: u32,
    mode: SelectionMode,
    selection_anchor: u32,
    hovered_line: Option<u32>,
    total_lines: u32,
    line_widths: Option<Arc<Vec<u16>>>,
}

impl Default for CursorState {
    fn default() -> Self {
        Self::new()
    }
}

impl CursorState {
    pub fn new() -> Self {
        Self {
            cursor_line: 0,
            mode: SelectionMode::Normal,
            selection_anchor: 0,
            hovered_line: None,
            total_lines: 0,
            line_widths: None,
        }
    }

    pub fn cursor_line(&self) -> u32 {
        self.cursor_line
    }

    #[allow(dead_code)]
    pub fn mode(&self) -> SelectionMode {
        self.mode
    }

    pub fn is_visual_mode(&self) -> bool {
        self.mode == SelectionMode::Visual
    }

    pub fn hovered_line(&self) -> Option<u32> {
        self.hovered_line
    }

    pub fn total_lines(&self) -> u32 {
        self.total_lines
    }

    pub fn update_total_lines(&mut self, total: u32) {
        self.total_lines = total;
        if self.cursor_line >= total && total > 0 {
            self.cursor_line = total - 1;
        }
        if self.selection_anchor >= total && total > 0 {
            self.selection_anchor = total - 1;
        }
    }

    pub fn update_line_widths(&mut self, widths: Arc<Vec<u16>>) {
        self.line_widths = Some(widths);
    }

    pub fn get_cursor_line_width(&self) -> u16 {
        self.get_line_width(self.cursor_line)
    }

    pub fn get_line_width(&self, line: u32) -> u16 {
        self.line_widths
            .as_ref()
            .and_then(|widths| widths.get(line as usize).copied())
            .unwrap_or(0)
    }

    pub fn get_max_selected_line_width(&self) -> u16 {
        if self.mode != SelectionMode::Visual {
            return self.get_cursor_line_width();
        }

        let (start, end) = self.selection_range();
        let widths = match &self.line_widths {
            Some(w) => w,
            None => return 0,
        };

        (start..=end)
            .filter_map(|line| widths.get(line as usize).copied())
            .max()
            .unwrap_or(0)
    }

    pub fn selection_range(&self) -> (u32, u32) {
        if self.mode == SelectionMode::Normal {
            (self.cursor_line, self.cursor_line)
        } else {
            let start = self.cursor_line.min(self.selection_anchor);
            let end = self.cursor_line.max(self.selection_anchor);
            (start, end)
        }
    }

    #[allow(dead_code)]
    pub fn is_line_selected(&self, line: u32) -> bool {
        if self.mode == SelectionMode::Normal {
            return false;
        }
        let (start, end) = self.selection_range();
        line >= start && line <= end
    }

    #[allow(dead_code)]
    pub fn is_cursor_line(&self, line: u32) -> bool {
        self.cursor_line == line
    }

    pub fn move_up(&mut self, lines: u32) {
        self.cursor_line = self.cursor_line.saturating_sub(lines);
    }

    pub fn move_down(&mut self, lines: u32) {
        if self.total_lines == 0 {
            return;
        }
        let max_line = self.total_lines.saturating_sub(1);
        self.cursor_line = self.cursor_line.saturating_add(lines).min(max_line);
    }

    pub fn move_to_first(&mut self) {
        self.cursor_line = 0;
    }

    pub fn move_to_last(&mut self) {
        if self.total_lines == 0 {
            return;
        }
        self.cursor_line = self.total_lines - 1;
    }

    pub fn move_to_line(&mut self, line: u32) {
        if self.total_lines == 0 {
            return;
        }
        self.cursor_line = line.min(self.total_lines - 1);
    }

    pub fn enter_visual_mode(&mut self) {
        self.mode = SelectionMode::Visual;
        self.selection_anchor = self.cursor_line;
    }

    pub fn exit_visual_mode(&mut self) {
        self.mode = SelectionMode::Normal;
    }

    #[allow(dead_code)]
    pub fn toggle_visual_mode(&mut self) {
        match self.mode {
            SelectionMode::Normal => self.enter_visual_mode(),
            SelectionMode::Visual => self.exit_visual_mode(),
        }
    }

    pub fn set_hovered(&mut self, line: Option<u32>) {
        self.hovered_line = line;
    }

    pub fn clear_hover(&mut self) {
        self.hovered_line = None;
    }

    pub fn reset(&mut self) {
        self.cursor_line = 0;
        self.mode = SelectionMode::Normal;
        self.selection_anchor = 0;
        self.hovered_line = None;
        self.line_widths = None;
    }

    pub fn click_select(&mut self, line: u32) {
        self.cursor_line = line.min(self.total_lines.saturating_sub(1));
        self.mode = SelectionMode::Visual;
        self.selection_anchor = self.cursor_line;
    }

    pub fn drag_extend(&mut self, line: u32) {
        if self.mode != SelectionMode::Visual {
            return;
        }
        self.cursor_line = line.min(self.total_lines.saturating_sub(1));
    }

    #[allow(dead_code)]
    pub fn selected_line_count(&self) -> u32 {
        let (start, end) = self.selection_range();
        end - start + 1
    }

    #[allow(dead_code)]
    pub fn compute_scroll_for_cursor(
        &self,
        current_offset: u16,
        viewport_height: u16,
        max_offset: u16,
    ) -> u16 {
        if viewport_height == 0 {
            return current_offset;
        }

        let cursor_line = self.cursor_line.min(u16::MAX as u32) as u16;
        let effective_scrolloff = SCROLLOFF.min(viewport_height / 2);

        let visible_start = current_offset;
        let visible_end = current_offset.saturating_add(viewport_height);

        if cursor_line < visible_start.saturating_add(effective_scrolloff) {
            cursor_line.saturating_sub(effective_scrolloff)
        } else if cursor_line >= visible_end.saturating_sub(effective_scrolloff) {
            cursor_line
                .saturating_add(effective_scrolloff)
                .saturating_add(1)
                .saturating_sub(viewport_height)
                .min(max_offset)
        } else {
            current_offset
        }
    }
}

#[cfg(test)]
#[path = "cursor_state_tests.rs"]
mod cursor_state_tests;
