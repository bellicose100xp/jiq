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

    pub fn jump_to_right(&mut self) {
        self.h_offset = self.max_h_offset;
    }

    pub fn reset(&mut self) {
        self.offset = 0;
        self.h_offset = 0;
    }
}

impl Default for ScrollState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_scroll_state() {
        let scroll = ScrollState::new();
        assert_eq!(scroll.offset, 0);
        assert_eq!(scroll.max_offset, 0);
        assert_eq!(scroll.viewport_height, 0);
        assert_eq!(scroll.h_offset, 0);
        assert_eq!(scroll.max_h_offset, 0);
    }

    #[test]
    fn test_update_bounds_small_content() {
        let mut scroll = ScrollState::new();

        // Content fits in viewport
        scroll.update_bounds(10, 20);
        assert_eq!(scroll.max_offset, 0);
        assert_eq!(scroll.viewport_height, 20);
        assert_eq!(scroll.offset, 0);
    }

    #[test]
    fn test_update_bounds_large_content() {
        let mut scroll = ScrollState::new();

        // Content larger than viewport
        scroll.update_bounds(100, 20);
        assert_eq!(scroll.max_offset, 80);
        assert_eq!(scroll.viewport_height, 20);
    }

    #[test]
    fn test_update_bounds_clamps_offset() {
        let mut scroll = ScrollState::new();
        scroll.update_bounds(100, 20);
        scroll.offset = 80; // Set to max

        // Reduce content size
        scroll.update_bounds(50, 20);
        assert_eq!(scroll.max_offset, 30);
        assert_eq!(scroll.offset, 30); // Clamped from 80 to 30
    }

    #[test]
    fn test_update_bounds_very_large_content() {
        let mut scroll = ScrollState::new();

        // Content with >65K lines (exceeds u16::MAX)
        scroll.update_bounds(70000, 20);
        assert_eq!(scroll.max_offset, u16::MAX);
        assert_eq!(scroll.viewport_height, 20);
    }

    #[test]
    fn test_scroll_down() {
        let mut scroll = ScrollState::new();
        scroll.update_bounds(100, 20);

        scroll.scroll_down(10);
        assert_eq!(scroll.offset, 10);

        scroll.scroll_down(5);
        assert_eq!(scroll.offset, 15);
    }

    #[test]
    fn test_scroll_down_clamped() {
        let mut scroll = ScrollState::new();
        scroll.update_bounds(100, 20);

        scroll.scroll_down(100); // Try to scroll past end
        assert_eq!(scroll.offset, 80); // Clamped to max_offset
    }

    #[test]
    fn test_scroll_up() {
        let mut scroll = ScrollState::new();
        scroll.update_bounds(100, 20);
        scroll.offset = 50;

        scroll.scroll_up(10);
        assert_eq!(scroll.offset, 40);

        scroll.scroll_up(5);
        assert_eq!(scroll.offset, 35);
    }

    #[test]
    fn test_scroll_up_clamped() {
        let mut scroll = ScrollState::new();
        scroll.update_bounds(100, 20);
        scroll.offset = 10;

        scroll.scroll_up(20); // Try to scroll past top
        assert_eq!(scroll.offset, 0); // Clamped to 0
    }

    #[test]
    fn test_page_down() {
        let mut scroll = ScrollState::new();
        scroll.update_bounds(100, 20);

        scroll.page_down();
        assert_eq!(scroll.offset, 10); // Half of viewport_height (20/2)

        scroll.page_down();
        assert_eq!(scroll.offset, 20);
    }

    #[test]
    fn test_page_up() {
        let mut scroll = ScrollState::new();
        scroll.update_bounds(100, 20);
        scroll.offset = 50;

        scroll.page_up();
        assert_eq!(scroll.offset, 40); // Half of viewport_height (20/2)

        scroll.page_up();
        assert_eq!(scroll.offset, 30);
    }

    #[test]
    fn test_jump_to_top() {
        let mut scroll = ScrollState::new();
        scroll.update_bounds(100, 20);
        scroll.offset = 50;

        scroll.jump_to_top();
        assert_eq!(scroll.offset, 0);
    }

    #[test]
    fn test_jump_to_bottom() {
        let mut scroll = ScrollState::new();
        scroll.update_bounds(100, 20);

        scroll.jump_to_bottom();
        assert_eq!(scroll.offset, 80); // max_offset
    }

    #[test]
    fn test_reset() {
        let mut scroll = ScrollState::new();
        scroll.update_bounds(100, 20);
        scroll.offset = 50;

        scroll.reset();
        assert_eq!(scroll.offset, 0);
    }

    #[test]
    fn test_default() {
        let scroll = ScrollState::default();
        assert_eq!(scroll, ScrollState::new());
    }
}
