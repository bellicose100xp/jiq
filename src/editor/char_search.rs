use tui_textarea::{CursorMove, TextArea};

/// Direction for character search
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchDirection {
    Forward,
    Backward,
}

impl SearchDirection {
    /// Returns the opposite direction
    pub fn opposite(self) -> Self {
        match self {
            SearchDirection::Forward => SearchDirection::Backward,
            SearchDirection::Backward => SearchDirection::Forward,
        }
    }
}

/// Type of character search
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchType {
    /// f/F - move TO the character
    Find,
    /// t/T - move BEFORE/AFTER the character
    Till,
}

/// Stores last character search for repeat with ; and ,
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CharSearchState {
    pub character: char,
    pub direction: SearchDirection,
    pub search_type: SearchType,
}

/// Find position of character in text from cursor position.
/// Returns the new cursor column position, or None if not found.
pub fn find_char_position(
    text: &str,
    cursor_col: usize,
    target: char,
    direction: SearchDirection,
    search_type: SearchType,
) -> Option<usize> {
    let chars: Vec<char> = text.chars().collect();

    match direction {
        SearchDirection::Forward => {
            let search_start = cursor_col + 1;
            if search_start >= chars.len() {
                return None;
            }

            for (i, &ch) in chars.iter().enumerate().skip(search_start) {
                if ch == target {
                    return Some(match search_type {
                        SearchType::Find => i,
                        SearchType::Till => i.saturating_sub(1).max(cursor_col + 1),
                    });
                }
            }
            None
        }
        SearchDirection::Backward => {
            if cursor_col == 0 {
                return None;
            }

            for i in (0..cursor_col).rev() {
                if chars.get(i) == Some(&target) {
                    return Some(match search_type {
                        SearchType::Find => i,
                        SearchType::Till => (i + 1).min(cursor_col.saturating_sub(1)),
                    });
                }
            }
            None
        }
    }
}

/// Execute character search and move cursor.
/// Returns true if a match was found and cursor was moved.
pub fn execute_char_search(
    textarea: &mut TextArea,
    target: char,
    direction: SearchDirection,
    search_type: SearchType,
) -> bool {
    let cursor_col = textarea.cursor().1;
    let text = textarea.lines().first().map(|s| s.as_str()).unwrap_or("");

    if let Some(new_col) = find_char_position(text, cursor_col, target, direction, search_type) {
        textarea.move_cursor(CursorMove::Head);
        for _ in 0..new_col {
            textarea.move_cursor(CursorMove::Forward);
        }
        true
    } else {
        false
    }
}

#[cfg(test)]
#[path = "char_search_tests.rs"]
mod char_search_tests;
