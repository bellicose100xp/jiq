//! Selection state management for AI suggestions
//!
//! Tracks the currently selected suggestion index and navigation state.

use crate::scroll::Scrollable;

/// Selection state for AI suggestion navigation
///
/// Tracks which suggestion is currently selected (if any) and whether
/// the user is actively navigating through suggestions.
#[derive(Debug, Clone, Default)]
pub struct SelectionState {
    /// Currently selected suggestion index (None = no selection)
    selected_index: Option<usize>,
    /// Whether navigation mode is active (user has used Alt+Up/Down/j/k)
    navigation_active: bool,
    /// Current vertical scroll offset in lines
    scroll_offset: u16,
    /// Viewport height in lines
    viewport_height: u16,
    /// Y position (in lines) where each suggestion starts
    suggestion_y_positions: Vec<u16>,
    /// Height (in lines) of each suggestion
    suggestion_heights: Vec<u16>,
    /// Currently hovered suggestion index (from mouse hover)
    hovered_index: Option<usize>,
}

impl SelectionState {
    /// Create a new SelectionState with no selection
    pub fn new() -> Self {
        Self {
            selected_index: None,
            navigation_active: false,
            scroll_offset: 0,
            viewport_height: 0,
            suggestion_y_positions: Vec::new(),
            suggestion_heights: Vec::new(),
            hovered_index: None,
        }
    }

    /// Select a specific suggestion index (test helper)
    ///
    /// Note: Production code uses navigate_next/prev instead.
    #[cfg(test)]
    pub fn select_index(&mut self, index: usize) {
        self.selected_index = Some(index);
        self.navigation_active = false;
    }

    /// Clear the current selection
    pub fn clear_selection(&mut self) {
        self.selected_index = None;
        self.navigation_active = false;
    }

    /// Get the currently selected suggestion index
    pub fn get_selected(&self) -> Option<usize> {
        self.selected_index
    }

    /// Check if navigation mode is active
    ///
    /// Navigation mode is active when the user has used Alt+Up/Down/j/k
    /// to navigate through suggestions. In this mode, Enter applies
    /// the selected suggestion.
    pub fn is_navigation_active(&self) -> bool {
        self.navigation_active
    }

    /// Navigate to the next suggestion (Alt+Down or Alt+j)
    ///
    /// Stops at the last suggestion (no wrap-around).
    /// Activates navigation mode.
    ///
    /// # Arguments
    /// * `suggestion_count` - Total number of available suggestions
    ///
    /// # Requirements
    /// - 8.1: Alt+Down/j moves selection to next suggestion
    pub fn navigate_next(&mut self, suggestion_count: usize) {
        if suggestion_count == 0 {
            return;
        }

        self.navigation_active = true;

        match self.selected_index {
            Some(current) => {
                // Stop at last suggestion (no wrap-around)
                if current + 1 < suggestion_count {
                    self.selected_index = Some(current + 1);
                }
            }
            None => {
                // Start at first suggestion
                self.selected_index = Some(0);
            }
        }

        // Ensure the newly selected suggestion is visible
        self.ensure_selected_visible();
    }

    /// Navigate to the previous suggestion (Alt+Up or Alt+k)
    ///
    /// Stops at the first suggestion (no wrap-around).
    /// Activates navigation mode.
    ///
    /// # Arguments
    /// * `suggestion_count` - Total number of available suggestions
    ///
    /// # Requirements
    /// - 8.2: Alt+Up/k moves selection to previous suggestion
    pub fn navigate_previous(&mut self, suggestion_count: usize) {
        if suggestion_count == 0 {
            return;
        }

        self.navigation_active = true;

        match self.selected_index {
            Some(current) => {
                // Stop at first suggestion (no wrap-around)
                if current > 0 {
                    self.selected_index = Some(current - 1);
                }
            }
            None => {
                // Start at last suggestion
                self.selected_index = Some(suggestion_count - 1);
            }
        }

        // Ensure the newly selected suggestion is visible
        self.ensure_selected_visible();
    }

    /// Update layout information for suggestion scrolling
    ///
    /// Stores the height of each suggestion and calculates Y positions.
    /// This must be called before rendering to enable proper scrolling.
    ///
    /// # Arguments
    /// * `heights` - Height (in lines) of each suggestion (spacing already included)
    /// * `viewport` - Visible viewport height in lines
    pub fn update_layout(&mut self, heights: Vec<u16>, viewport: u16) {
        self.viewport_height = viewport;
        self.suggestion_heights = heights;

        // Calculate Y positions (heights already include spacing lines)
        self.suggestion_y_positions.clear();
        let mut current_y = 0u16;
        for &height in self.suggestion_heights.iter() {
            self.suggestion_y_positions.push(current_y);
            current_y = current_y.saturating_add(height);
        }
    }

    /// Adjust scroll offset to ensure the selected suggestion is visible
    ///
    /// Scrolls up if selection is above viewport, down if below viewport.
    pub fn ensure_selected_visible(&mut self) {
        let Some(selected_idx) = self.selected_index else {
            return;
        };

        if selected_idx >= self.suggestion_y_positions.len() {
            return;
        }

        let suggestion_start = self.suggestion_y_positions[selected_idx];
        let suggestion_height = self
            .suggestion_heights
            .get(selected_idx)
            .copied()
            .unwrap_or(1);
        let suggestion_end = suggestion_start.saturating_add(suggestion_height);

        // If suggestion starts above viewport, scroll up
        if suggestion_start < self.scroll_offset {
            self.scroll_offset = suggestion_start;
        }
        // If suggestion ends below viewport, scroll down
        else if suggestion_end > self.scroll_offset.saturating_add(self.viewport_height) {
            self.scroll_offset = suggestion_end.saturating_sub(self.viewport_height);
        }
    }

    /// Get the current scroll offset in lines (raw u16 value for rendering)
    pub fn scroll_offset_u16(&self) -> u16 {
        self.scroll_offset
    }

    /// Clear layout information (called when suggestions change)
    pub fn clear_layout(&mut self) {
        self.scroll_offset = 0;
        self.viewport_height = 0;
        self.suggestion_y_positions.clear();
        self.suggestion_heights.clear();
    }

    /// Get the total content height in lines (used by Scrollable impl)
    #[allow(dead_code)]
    fn total_content_height(&self) -> u16 {
        self.suggestion_heights.iter().copied().sum()
    }

    /// Find which suggestion is at a given Y coordinate within the inner area
    ///
    /// # Arguments
    /// * `inner_y` - Y coordinate relative to the inner area top (0-based)
    ///
    /// # Returns
    /// The suggestion index if one is found at that position, None otherwise
    pub fn suggestion_at_y(&self, inner_y: u16) -> Option<usize> {
        let content_y = inner_y.saturating_add(self.scroll_offset);
        for (i, &pos) in self.suggestion_y_positions.iter().enumerate() {
            let height = self.suggestion_heights.get(i).copied().unwrap_or(1);
            if content_y >= pos && content_y < pos.saturating_add(height) {
                return Some(i);
            }
        }
        None
    }

    /// Get the currently hovered suggestion index
    pub fn get_hovered(&self) -> Option<usize> {
        self.hovered_index
    }

    /// Set the hovered suggestion index
    pub fn set_hovered(&mut self, index: Option<usize>) {
        self.hovered_index = index;
    }

    /// Clear the hovered state
    pub fn clear_hover(&mut self) {
        self.hovered_index = None;
    }
}

impl Scrollable for SelectionState {
    fn scroll_view_up(&mut self, lines: usize) {
        self.scroll_offset = self.scroll_offset.saturating_sub(lines as u16);
    }

    fn scroll_view_down(&mut self, lines: usize) {
        let max = self.max_scroll();
        self.scroll_offset = (self.scroll_offset as usize + lines).min(max) as u16;
    }

    fn scroll_offset(&self) -> usize {
        self.scroll_offset as usize
    }

    fn max_scroll(&self) -> usize {
        let total = self.total_content_height() as usize;
        total.saturating_sub(self.viewport_height as usize)
    }

    fn viewport_size(&self) -> usize {
        self.viewport_height as usize
    }
}

#[cfg(test)]
#[path = "state_tests.rs"]
mod state_tests;
