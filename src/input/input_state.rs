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

        state.textarea.insert_str("1234567890123456789012345");
        state.calculate_scroll_offset(viewport_width);

        assert_eq!(state.textarea.cursor().1, 25);
        assert_eq!(state.scroll_offset, 6);

        for _ in 0..10 {
            state.textarea.delete_char();
        }

        assert_eq!(state.textarea.cursor().1, 15);
        assert_eq!(state.query().chars().count(), 15);

        state.calculate_scroll_offset(viewport_width);
        assert_eq!(state.scroll_offset, 0);
    }

    #[test]
    fn test_scroll_offset_deletion_middle() {
        let mut state = InputState::new();
        let viewport_width = 10;

        state.textarea.insert_str("abcdefghijklmnopqrstuvwxyz");

        for _ in 0..11 {
            state.textarea.move_cursor(tui_textarea::CursorMove::Back);
        }
        assert_eq!(state.textarea.cursor().1, 15);

        state.scroll_offset = 10;
        state.calculate_scroll_offset(viewport_width);
        assert_eq!(state.scroll_offset, 10);

        for _ in 0..4 {
            state.textarea.delete_next_char();
        }

        assert_eq!(state.query().chars().count(), 22);
        assert_eq!(state.textarea.cursor().1, 15);

        state.calculate_scroll_offset(viewport_width);
        assert_eq!(state.scroll_offset, 10);

        for _ in 0..4 {
            state.textarea.delete_next_char();
        }

        assert_eq!(state.query().chars().count(), 18);

        state.calculate_scroll_offset(viewport_width);
        assert_eq!(state.scroll_offset, 8);
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

        state.textarea.insert_str("abcdefghijklmnopqrstuvwxyz");

        for target_pos in [0, 5, 10, 15, 20, 25] {
            while state.textarea.cursor().1 > 0 {
                state.textarea.move_cursor(tui_textarea::CursorMove::Head);
            }

            for _ in 0..target_pos {
                state
                    .textarea
                    .move_cursor(tui_textarea::CursorMove::Forward);
            }

            let cursor_col = state.textarea.cursor().1;
            state.calculate_scroll_offset(viewport_width);

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

        state.textarea.insert_str("Hello👋World🌍Test");

        let text_len = state.query().chars().count();
        assert_eq!(text_len, 16);

        state.calculate_scroll_offset(viewport_width);

        assert_eq!(state.scroll_offset, 7);

        for _ in 0..5 {
            state.textarea.delete_char();
        }

        state.calculate_scroll_offset(viewport_width);
        let cursor = state.textarea.cursor().1;

        assert!(cursor >= state.scroll_offset);
        assert!(cursor < state.scroll_offset + viewport_width);
    }

    #[test]
    fn test_scroll_offset_empty_input() {
        let mut state = InputState::new();
        let viewport_width = 10;

        assert_eq!(state.query(), "");
        state.calculate_scroll_offset(viewport_width);

        assert_eq!(state.scroll_offset, 0);
        assert_eq!(state.textarea.cursor().1, 0);
    }

    #[test]
    fn test_scroll_offset_exactly_fits_viewport() {
        let mut state = InputState::new();
        let viewport_width = 10;

        state.textarea.insert_str("1234567890");
        state.calculate_scroll_offset(viewport_width);

        assert_eq!(state.textarea.cursor().1, 10);
        assert_eq!(state.scroll_offset, 1);

        state.textarea.insert_char('x');
        state.calculate_scroll_offset(viewport_width);

        assert_eq!(state.textarea.cursor().1, 11);
        assert_eq!(state.scroll_offset, 2);

        state.textarea.delete_char();
        state.calculate_scroll_offset(viewport_width);

        assert_eq!(state.textarea.cursor().1, 10);
        assert_eq!(state.scroll_offset, 1);
    }

    #[test]
    fn test_sync_delete_from_end_realistic_scenario() {
        let mut state = InputState::new();
        let viewport_width = 20;

        state.textarea.insert_str(
            ".services | select(.[].capacityProviderStrategy | length > 0) | .[0].capacity",
        );
        state.calculate_scroll_offset(viewport_width);

        assert_cursor_visible(&state, viewport_width);

        for i in 0..30 {
            state.textarea.delete_char();
            state.calculate_scroll_offset(viewport_width);

            assert_cursor_visible(&state, viewport_width);

            let visible = get_visible_text(&state, viewport_width);
            let cursor = state.textarea.cursor().1;
            let scroll = state.scroll_offset;

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

        state
            .textarea
            .insert_str("abcdefghijklmnopqrstuvwxyz0123456789");
        state.calculate_scroll_offset(viewport_width);
        assert_cursor_visible(&state, viewport_width);

        for _ in 0..20 {
            state.textarea.move_cursor(tui_textarea::CursorMove::Back);
            state.calculate_scroll_offset(viewport_width);
            assert_cursor_visible(&state, viewport_width);
        }

        for _ in 0..10 {
            state.textarea.delete_next_char();
            state.calculate_scroll_offset(viewport_width);
            assert_cursor_visible(&state, viewport_width);
        }

        for _ in 0..10 {
            state.textarea.delete_char();
            state.calculate_scroll_offset(viewport_width);
            assert_cursor_visible(&state, viewport_width);
        }

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

        state.textarea.insert_str("0123456789ABCDEFGHIJKLMNOP");
        state.calculate_scroll_offset(viewport_width);

        let test_positions = vec![0, 5, 10, 15, 20, 25];

        for &target_pos in &test_positions {
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

        state.textarea.insert_str("12345678901234567890ABCDE");
        state.calculate_scroll_offset(viewport_width);
        assert_cursor_visible(&state, viewport_width);

        for _ in 0..15 {
            state.textarea.delete_char();
            state.calculate_scroll_offset(viewport_width);
            assert_cursor_visible(&state, viewport_width);
        }

        let text_len = state.query().chars().count();
        if text_len < viewport_width {
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

        for _ in 0..11 {
            state.textarea.move_cursor(tui_textarea::CursorMove::Back);
        }
        state.scroll_offset = 10;
        state.calculate_scroll_offset(viewport_width);

        for i in 0..8 {
            let cursor_before = state.textarea.cursor().1;
            state.textarea.delete_next_char();
            state.calculate_scroll_offset(viewport_width);

            assert_cursor_visible(&state, viewport_width);

            let cursor_after = state.textarea.cursor().1;

            assert_eq!(
                cursor_after, cursor_before,
                "Iteration {}: delete_next_char moved cursor from {} to {}",
                i, cursor_before, cursor_after
            );
        }
    }

    #[test]
    fn test_sync_textarea_actually_scrolls() {
        let mut state = InputState::new();
        let viewport_width = 20;

        state
            .textarea
            .insert_str("ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789");
        state.calculate_scroll_offset(viewport_width);

        let initial_scroll = state.scroll_offset;

        for _ in 0..20 {
            state.textarea.delete_char();
        }
        state.calculate_scroll_offset(viewport_width);

        assert!(
            state.scroll_offset < initial_scroll,
            "Expected scroll to reduce from {} after deletions, but got {}",
            initial_scroll,
            state.scroll_offset
        );

        let scroll_before_move = state.scroll_offset;
        let cursor_before_move = state.textarea.cursor().1;

        state.textarea.move_cursor(tui_textarea::CursorMove::Back);
        state.calculate_scroll_offset(viewport_width);

        let scroll_after_move = state.scroll_offset;
        let cursor_after_move = state.textarea.cursor().1;

        assert_eq!(
            cursor_after_move,
            cursor_before_move - 1,
            "Cursor should move one position back"
        );

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
