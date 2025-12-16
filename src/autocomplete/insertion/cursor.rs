//! Cursor positioning utilities for autocomplete insertion

use tui_textarea::{CursorMove, TextArea};

/// Move cursor to a specific column position
pub fn move_cursor_to_column(textarea: &mut TextArea<'_>, target_col: usize) {
    let current_col = textarea.cursor().1;

    match target_col.cmp(&current_col) {
        std::cmp::Ordering::Less => {
            // Move backward
            for _ in 0..(current_col - target_col) {
                textarea.move_cursor(CursorMove::Back);
            }
        }
        std::cmp::Ordering::Greater => {
            // Move forward
            for _ in 0..(target_col - current_col) {
                textarea.move_cursor(CursorMove::Forward);
            }
        }
        std::cmp::Ordering::Equal => {
            // Already at target position
        }
    }
}
