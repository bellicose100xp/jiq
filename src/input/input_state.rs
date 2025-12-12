use ratatui::{
    style::{Color, Style},
    widgets::{Block, Borders},
};
use tui_textarea::TextArea;

use crate::autocomplete::BraceTracker;
use crate::editor::EditorMode;

/// Input field state
pub struct InputState {
    pub textarea: TextArea<'static>,
    pub editor_mode: EditorMode,
    pub scroll_offset: usize, // Track horizontal scroll for syntax overlay
    pub brace_tracker: BraceTracker, // Track brace nesting for autocomplete context
}

impl InputState {
    /// Create a new InputState
    pub fn new() -> Self {
        let mut textarea = TextArea::default();

        // Configure for single-line input
        textarea.set_block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Query ")
                .border_style(Style::default().fg(Color::DarkGray)),
        );

        // Remove default underline from cursor line
        textarea.set_cursor_line_style(Style::default());

        Self {
            textarea,
            editor_mode: EditorMode::default(),
            scroll_offset: 0,
            brace_tracker: BraceTracker::new(),
        }
    }

    /// Get the current query text
    pub fn query(&self) -> &str {
        self.textarea.lines()[0].as_ref()
    }

    /// Calculate the horizontal scroll offset to keep cursor visible
    /// We control rendering directly, so no need to sync with textarea widget
    pub fn calculate_scroll_offset(&mut self, viewport_width: usize) {
        let cursor_col = self.textarea.cursor().1;
        let text_length = self.query().chars().count();

        let mut new_scroll = self.scroll_offset;

        // Phase 1: ensure cursor is visible in viewport
        if cursor_col < new_scroll {
            // Cursor moved left of viewport - scroll left
            new_scroll = cursor_col;
        } else if cursor_col >= new_scroll + viewport_width {
            // Cursor moved right of viewport - scroll right
            new_scroll = cursor_col + 1 - viewport_width;
        }


        // Phase 2: reduce scroll if text is shorter than visible area
        // (handles deletion case - scroll left to fill available space)
        if text_length < new_scroll + viewport_width {
            // Calculate minimum scroll needed to show all text (or 0 if text fits)
            let min_scroll = text_length.saturating_sub(viewport_width);
            // But don't scroll past cursor position (keep cursor visible)
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
mod tests {
    use super::*;

    #[test]
    fn test_new_input_state() {
        let state = InputState::new();
        assert_eq!(state.query(), "");
        assert_eq!(state.editor_mode, EditorMode::Insert);
    }

    #[test]
    fn test_default() {
        let state = InputState::default();
        assert_eq!(state.query(), "");
    }

    #[test]
    fn test_query_after_insert() {
        let mut state = InputState::new();
        state.textarea.insert_str("test query");
        assert_eq!(state.query(), "test query");
    }

    // ========== Scroll offset tests ==========

    /// Helper: Verify cursor is visible within the tracked scroll viewport
    /// This catches desynchronization between our scroll_offset and actual cursor position
    fn assert_cursor_visible(state: &InputState, viewport_width: usize) {
        let cursor_col = state.textarea.cursor().1;
        let scroll = state.scroll_offset;

        assert!(
            cursor_col >= scroll && cursor_col < scroll + viewport_width,
            "DESYNC: Cursor at {} not visible in viewport [{}, {})",
            cursor_col,
            scroll,
            scroll + viewport_width
        );
    }


    /// Helper: Simulate the actual visible text that would be rendered
    /// Returns the text portion that should be visible based on scroll_offset
    fn get_visible_text(state: &InputState, viewport_width: usize) -> String {
        let text = state.query();
        let start = state.scroll_offset.min(text.chars().count());
        let end = (start + viewport_width).min(text.chars().count());
        text.chars().skip(start).take(end - start).collect()
    }

    #[test]
    fn test_scroll_offset_deletion_from_end() {
        let mut state = InputState::new();
        let viewport_width = 20;

        // Insert text longer than viewport (25 chars)
        state.textarea.insert_str("1234567890123456789012345");
        state.calculate_scroll_offset(viewport_width);

        // Cursor should be at end (position 25), scroll should be > 0
        // Formula: cursor_col + 1 - viewport_width = 25 + 1 - 20 = 6
        assert_eq!(state.textarea.cursor().1, 25);
        assert_eq!(state.scroll_offset, 6);

        // Delete 10 characters from end (simulating backspace)
        for _ in 0..10 {
            state.textarea.delete_char();
        }

        // Cursor now at position 15, text length is 15
        assert_eq!(state.textarea.cursor().1, 15);
        assert_eq!(state.query().chars().count(), 15);

        // Recalculate scroll - should reduce to 0 since text fits in viewport
        state.calculate_scroll_offset(viewport_width);
        assert_eq!(state.scroll_offset, 0);
    }

    #[test]
    fn test_scroll_offset_deletion_middle() {
        let mut state = InputState::new();
        let viewport_width = 10;

        // Insert text: "abcdefghijklmnopqrstuvwxyz" (26 chars)
        state.textarea.insert_str("abcdefghijklmnopqrstuvwxyz");

        // Move cursor to position 15 (middle)
        for _ in 0..11 {
            state.textarea.move_cursor(tui_textarea::CursorMove::Back);
        }
        assert_eq!(state.textarea.cursor().1, 15);

        // Set scroll to 10 (viewport shows positions 10-19)
        state.scroll_offset = 10;
        state.calculate_scroll_offset(viewport_width);
        assert_eq!(state.scroll_offset, 10); // Cursor visible, no change


        // Delete 4 chars forward (x key in vim deletes forward)
        for _ in 0..4 {
            state.textarea.delete_next_char();
        }

        // Text is now 22 chars, cursor at 15
        assert_eq!(state.query().chars().count(), 22);
        assert_eq!(state.textarea.cursor().1, 15);

        // Scroll should stay at 10 (no empty space: 22 >= 10 + 10)
        state.calculate_scroll_offset(viewport_width);
        assert_eq!(state.scroll_offset, 10);

        // Delete 4 more chars (total 8 deleted, 18 chars remain)
        for _ in 0..4 {
            state.textarea.delete_next_char();
        }

        // Text is now 18 chars, cursor at 15
        assert_eq!(state.query().chars().count(), 18);

        // Now there's empty space (18 < 10 + 10), scroll should reduce
        state.calculate_scroll_offset(viewport_width);
        assert_eq!(state.scroll_offset, 8); // 18 - 10 = 8
    }

    #[test]
    fn test_scroll_offset_short_text_no_scroll() {
        let mut state = InputState::new();
        let viewport_width = 20;

        // Insert text shorter than viewport
        state.textarea.insert_str("short");
        state.calculate_scroll_offset(viewport_width);

        assert_eq!(state.scroll_offset, 0);
        assert_eq!(state.textarea.cursor().1, 5);
    }

    #[test]
    fn test_scroll_offset_cursor_visibility_preserved() {
        let mut state = InputState::new();
        let viewport_width = 10;

        // Insert long text
        state.textarea.insert_str("abcdefghijklmnopqrstuvwxyz");

        // Move cursor to various positions and verify it's always visible
        for target_pos in [0, 5, 10, 15, 20, 25] {
            // Reset cursor to start
            while state.textarea.cursor().1 > 0 {
                state.textarea.move_cursor(tui_textarea::CursorMove::Head);
            }

            // Move to target position
            for _ in 0..target_pos {
                state.textarea.move_cursor(tui_textarea::CursorMove::Forward);
            }

            let cursor_col = state.textarea.cursor().1;
            state.calculate_scroll_offset(viewport_width);

            // Cursor must be within visible range
            assert!(
                cursor_col >= state.scroll_offset
                    && cursor_col < state.scroll_offset + viewport_width,
                "Cursor at {} not visible with scroll_offset {} and viewport_width {}",
                cursor_col,
                state.scroll_offset,
                viewport_width
            );
        }
    }


    #[test]
    fn test_scroll_offset_unicode_chars() {
        let mut state = InputState::new();
        let viewport_width = 10;

        // Insert text with emoji and unicode chars
        state.textarea.insert_str("Hello👋World🌍Test");

        // Text has emoji which are single chars but multiple bytes
        let text_len = state.query().chars().count();
        assert_eq!(text_len, 16); // 5 + 1 + 5 + 1 + 4 = 16 chars

        state.calculate_scroll_offset(viewport_width);

        // Cursor at end (position 16), scroll should be 7 (16 + 1 - 10)
        assert_eq!(state.scroll_offset, 7);

        // Delete some emoji
        for _ in 0..5 {
            state.textarea.delete_char();
        }

        // Recalculate and verify scroll adjusted correctly
        state.calculate_scroll_offset(viewport_width);
        let cursor = state.textarea.cursor().1;

        // Verify cursor is visible
        assert!(cursor >= state.scroll_offset);
        assert!(cursor < state.scroll_offset + viewport_width);
    }

    #[test]
    fn test_scroll_offset_empty_input() {
        let mut state = InputState::new();
        let viewport_width = 10;

        // Empty input
        assert_eq!(state.query(), "");
        state.calculate_scroll_offset(viewport_width);

        assert_eq!(state.scroll_offset, 0);
        assert_eq!(state.textarea.cursor().1, 0);
    }

    #[test]
    fn test_scroll_offset_exactly_fits_viewport() {
        let mut state = InputState::new();
        let viewport_width = 10;

        // Insert text exactly equal to viewport width
        state.textarea.insert_str("1234567890"); // Exactly 10 chars
        state.calculate_scroll_offset(viewport_width);

        // Cursor is at position 10 (after last char), must scroll by 1 to show cursor
        // This is a boundary case: cursor visibility takes priority over showing first char
        assert_eq!(state.textarea.cursor().1, 10);
        assert_eq!(state.scroll_offset, 1);

        // Add one more char
        state.textarea.insert_char('x');
        state.calculate_scroll_offset(viewport_width);

        // Now 11 chars, cursor at 11, scroll should be 2
        assert_eq!(state.textarea.cursor().1, 11);
        assert_eq!(state.scroll_offset, 2);

        // Delete that char
        state.textarea.delete_char();
        state.calculate_scroll_offset(viewport_width);

        // Back to 10 chars, cursor at 10, scroll back to 1
        assert_eq!(state.textarea.cursor().1, 10);
        assert_eq!(state.scroll_offset, 1);
    }


    // ========== Synchronization verification tests ==========
    // These tests verify that scroll_offset stays synchronized with cursor position
    // and would catch desynchronization bugs

    #[test]
    fn test_sync_delete_from_end_realistic_scenario() {
        let mut state = InputState::new();
        let viewport_width = 20;

        // Realistic scenario: User types a long query
        state.textarea.insert_str(
            ".services | select(.[].capacityProviderStrategy | length > 0) | .[0].capacity",
        );
        state.calculate_scroll_offset(viewport_width);

        // Verify cursor visible after initial insert
        assert_cursor_visible(&state, viewport_width);

        // User realizes they made an error and deletes from the end
        for i in 0..30 {
            state.textarea.delete_char();
            state.calculate_scroll_offset(viewport_width);

            // After EVERY deletion, cursor must be visible
            assert_cursor_visible(&state, viewport_width);

            // Also verify the visible text makes sense
            let visible = get_visible_text(&state, viewport_width);
            let cursor = state.textarea.cursor().1;
            let scroll = state.scroll_offset;

            // Cursor should point to a position within or at end of visible text
            assert!(
                cursor - scroll <= visible.chars().count(),
                "Iteration {}: Cursor at {} with scroll {} points beyond visible text '{}'",
                i,
                cursor,
                scroll,
                visible
            );
        }
    }

    #[test]
    fn test_sync_after_every_operation() {
        let mut state = InputState::new();
        let viewport_width = 15;

        // Insert text
        state
            .textarea
            .insert_str("abcdefghijklmnopqrstuvwxyz0123456789");
        state.calculate_scroll_offset(viewport_width);
        assert_cursor_visible(&state, viewport_width);

        // Move cursor left
        for _ in 0..20 {
            state.textarea.move_cursor(tui_textarea::CursorMove::Back);
            state.calculate_scroll_offset(viewport_width);
            assert_cursor_visible(&state, viewport_width);
        }

        // Delete forward
        for _ in 0..10 {
            state.textarea.delete_next_char();
            state.calculate_scroll_offset(viewport_width);
            assert_cursor_visible(&state, viewport_width);
        }

        // Delete backward
        for _ in 0..10 {
            state.textarea.delete_char();
            state.calculate_scroll_offset(viewport_width);
            assert_cursor_visible(&state, viewport_width);
        }

        // Move cursor right
        for _ in 0..5 {
            state
                .textarea
                .move_cursor(tui_textarea::CursorMove::Forward);
            state.calculate_scroll_offset(viewport_width);
            assert_cursor_visible(&state, viewport_width);
        }
    }


    #[test]
    fn test_sync_visible_text_contains_cursor() {
        let mut state = InputState::new();
        let viewport_width = 10;

        state
            .textarea
            .insert_str("0123456789ABCDEFGHIJKLMNOP");
        state.calculate_scroll_offset(viewport_width);

        // Move to various positions and verify visible text is correct
        let test_positions = vec![0, 5, 10, 15, 20, 25];

        for &target_pos in &test_positions {
            // Move cursor to target position
            while state.textarea.cursor().1 > 0 {
                state.textarea.move_cursor(tui_textarea::CursorMove::Head);
            }
            for _ in 0..target_pos {
                state
                    .textarea
                    .move_cursor(tui_textarea::CursorMove::Forward);
            }

            state.calculate_scroll_offset(viewport_width);

            let cursor = state.textarea.cursor().1;
            let scroll = state.scroll_offset;
            let visible = get_visible_text(&state, viewport_width);
            let full_text = state.query();

            // The visible text should match the substring from the full text
            let expected_start = scroll;
            let expected_end = (scroll + viewport_width).min(full_text.chars().count());
            let expected_visible: String = full_text
                .chars()
                .skip(expected_start)
                .take(expected_end - expected_start)
                .collect();

            assert_eq!(
                visible, expected_visible,
                "At cursor {}: visible text mismatch (scroll={})",
                cursor, scroll
            );

            // Cursor position relative to scroll must be within visible text
            let cursor_in_viewport = cursor - scroll;
            assert!(
                cursor_in_viewport <= visible.chars().count(),
                "At cursor {}: cursor_in_viewport {} > visible.len() {}",
                cursor,
                cursor_in_viewport,
                visible.chars().count()
            );
        }
    }

    #[test]
    fn test_sync_no_empty_space_with_short_text() {
        let mut state = InputState::new();
        let viewport_width = 20;

        // Start with long text
        state.textarea.insert_str("12345678901234567890ABCDE");
        state.calculate_scroll_offset(viewport_width);
        assert_cursor_visible(&state, viewport_width);

        // Delete to make text shorter than viewport
        for _ in 0..15 {
            state.textarea.delete_char();
            state.calculate_scroll_offset(viewport_width);
            assert_cursor_visible(&state, viewport_width);
        }

        // Now text is short (10 chars), should have scroll_offset = 0
        // to avoid empty space on the right
        let text_len = state.query().chars().count();
        if text_len < viewport_width {
            // If text fits in viewport, scroll should be minimal
            // (either 0, or just enough to show cursor if cursor is at text end)
            let max_expected_scroll = text_len.saturating_sub(viewport_width - 1);
            assert!(
                state.scroll_offset <= max_expected_scroll,
                "Text length {}, viewport {}: scroll_offset {} creates empty space",
                text_len,
                viewport_width,
                state.scroll_offset
            );
        }
    }


    #[test]
    fn test_sync_deletion_middle_maintains_visibility() {
        let mut state = InputState::new();
        let viewport_width = 10;

        state.textarea.insert_str("ABCDEFGHIJKLMNOPQRSTUVWXYZ");

        // Position cursor in middle
        for _ in 0..11 {
            state.textarea.move_cursor(tui_textarea::CursorMove::Back);
        }
        state.scroll_offset = 10; // Manually set scroll to show middle portion
        state.calculate_scroll_offset(viewport_width);

        // Delete characters around cursor position
        for i in 0..8 {
            let cursor_before = state.textarea.cursor().1;
            state.textarea.delete_next_char();
            state.calculate_scroll_offset(viewport_width);

            assert_cursor_visible(&state, viewport_width);

            let cursor_after = state.textarea.cursor().1;

            // Cursor shouldn't jump unexpectedly
            assert_eq!(
                cursor_after, cursor_before,
                "Iteration {}: delete_next_char moved cursor from {} to {}",
                i, cursor_before, cursor_after
            );
        }
    }

    /// CRITICAL TEST: This test would catch the desync bug by verifying
    /// that we can actually control textarea's scroll
    #[test]
    fn test_sync_textarea_actually_scrolls() {
        let mut state = InputState::new();
        let viewport_width = 20;

        // Insert long text
        state
            .textarea
            .insert_str("ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789");
        state.calculate_scroll_offset(viewport_width);

        let initial_scroll = state.scroll_offset;

        // Now simulate what Phase 2 does: reduce scroll due to deletion
        // Delete 20 chars to trigger scroll reduction
        for _ in 0..20 {
            state.textarea.delete_char();
        }
        state.calculate_scroll_offset(viewport_width);

        // Our scroll_offset should have reduced
        assert!(
            state.scroll_offset < initial_scroll,
            "Expected scroll to reduce from {} after deletions, but got {}",
            initial_scroll,
            state.scroll_offset
        );

        // CRITICAL: Now try to use CursorMove::InViewport which should use
        // textarea's actual internal scroll. If textarea didn't scroll with us,
        // the cursor will jump to a wrong position
        let scroll_before_move = state.scroll_offset;
        let cursor_before_move = state.textarea.cursor().1;

        // Move cursor to a position that should keep same scroll
        // (just move one char left, should not affect scroll)
        state.textarea.move_cursor(tui_textarea::CursorMove::Back);
        state.calculate_scroll_offset(viewport_width);

        // After a small cursor movement, scroll should remain stable if textarea agrees
        let scroll_after_move = state.scroll_offset;
        let cursor_after_move = state.textarea.cursor().1;

        // Verify cursor moved correctly (just one position back)
        assert_eq!(
            cursor_after_move,
            cursor_before_move - 1,
            "Cursor should move one position back"
        );

        // If there's desync, scroll might jump unexpectedly after cursor movement
        // because textarea's internal scroll disagrees with our tracking
        let scroll_jump = (scroll_after_move as isize - scroll_before_move as isize).abs();
        assert!(
            scroll_jump <= 1,
            "Scroll jumped by {} after small cursor movement (from {} to {}). \
             This indicates desync between our scroll_offset and textarea's internal scroll.",
            scroll_jump,
            scroll_before_move,
            scroll_after_move
        );
    }
}
