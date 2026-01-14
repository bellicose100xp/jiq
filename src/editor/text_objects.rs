use tui_textarea::TextArea;

use crate::editor::mode::TextObjectScope;

/// Text object target types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextObjectTarget {
    Word,
    DoubleQuote,
    SingleQuote,
    Backtick,
    Parentheses,
    Brackets,
    Braces,
    Pipe,
}

impl TextObjectTarget {
    /// Parse a character into a text object target
    pub fn from_char(c: char) -> Option<Self> {
        match c {
            'w' => Some(TextObjectTarget::Word),
            '"' => Some(TextObjectTarget::DoubleQuote),
            '\'' => Some(TextObjectTarget::SingleQuote),
            '`' => Some(TextObjectTarget::Backtick),
            '(' | ')' | 'b' => Some(TextObjectTarget::Parentheses),
            '[' | ']' => Some(TextObjectTarget::Brackets),
            '{' | '}' | 'B' => Some(TextObjectTarget::Braces),
            '|' => Some(TextObjectTarget::Pipe),
            _ => None,
        }
    }

    /// Get the delimiter pair for bracket-type targets
    fn delimiters(self) -> Option<(char, char)> {
        match self {
            TextObjectTarget::DoubleQuote => Some(('"', '"')),
            TextObjectTarget::SingleQuote => Some(('\'', '\'')),
            TextObjectTarget::Backtick => Some(('`', '`')),
            TextObjectTarget::Parentheses => Some(('(', ')')),
            TextObjectTarget::Brackets => Some(('[', ']')),
            TextObjectTarget::Braces => Some(('{', '}')),
            TextObjectTarget::Word | TextObjectTarget::Pipe => None,
        }
    }
}

fn is_word_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// Find word boundaries around cursor position.
/// Returns (start, end) where end is exclusive.
pub fn find_word_bounds(
    text: &str,
    cursor_col: usize,
    scope: TextObjectScope,
) -> Option<(usize, usize)> {
    let chars: Vec<char> = text.chars().collect();

    if chars.is_empty() || cursor_col >= chars.len() {
        return None;
    }

    let cursor_char = chars[cursor_col];

    if !is_word_char(cursor_char) {
        return None;
    }

    let mut start = cursor_col;
    while start > 0 && is_word_char(chars[start - 1]) {
        start -= 1;
    }

    let mut end = cursor_col;
    while end < chars.len() && is_word_char(chars[end]) {
        end += 1;
    }

    match scope {
        TextObjectScope::Inner => Some((start, end)),
        TextObjectScope::Around => {
            if end < chars.len() && chars[end] == ' ' {
                let mut extended_end = end;
                while extended_end < chars.len() && chars[extended_end] == ' ' {
                    extended_end += 1;
                }
                Some((start, extended_end))
            } else if start > 0 && chars[start - 1] == ' ' {
                let mut extended_start = start;
                while extended_start > 0 && chars[extended_start - 1] == ' ' {
                    extended_start -= 1;
                }
                Some((extended_start, end))
            } else {
                Some((start, end))
            }
        }
    }
}

/// Find paired delimiter bounds (for quotes).
/// For same-character delimiters, finds the pair surrounding cursor.
pub fn find_quote_bounds(
    text: &str,
    cursor_col: usize,
    delimiter: char,
    scope: TextObjectScope,
) -> Option<(usize, usize)> {
    let chars: Vec<char> = text.chars().collect();

    if chars.is_empty() {
        return None;
    }

    let cursor_col = cursor_col.min(chars.len().saturating_sub(1));

    let mut open_pos = None;
    for i in (0..=cursor_col).rev() {
        if chars[i] == delimiter {
            let count_before = chars[..i].iter().filter(|&&c| c == delimiter).count();
            if count_before % 2 == 0 {
                open_pos = Some(i);
                break;
            }
        }
    }

    let open = open_pos?;

    let search_start = open + 1;
    let mut close_pos = None;
    for (i, &ch) in chars.iter().enumerate().skip(search_start) {
        if ch == delimiter {
            close_pos = Some(i);
            break;
        }
    }

    let close = close_pos?;

    if cursor_col > close {
        return None;
    }

    match scope {
        TextObjectScope::Inner => Some((open + 1, close)),
        TextObjectScope::Around => Some((open, close + 1)),
    }
}

/// Find bracket bounds with nesting support.
/// Finds the innermost matching pair containing the cursor.
pub fn find_bracket_bounds(
    text: &str,
    cursor_col: usize,
    open_delim: char,
    close_delim: char,
    scope: TextObjectScope,
) -> Option<(usize, usize)> {
    let chars: Vec<char> = text.chars().collect();

    if chars.is_empty() {
        return None;
    }

    let cursor_col = cursor_col.min(chars.len().saturating_sub(1));

    let mut open_pos = None;
    let mut depth = 0i32;

    // When cursor is on closing bracket, don't count it in depth
    let search_end = if chars[cursor_col] == close_delim {
        cursor_col.saturating_sub(1)
    } else {
        cursor_col
    };

    for i in (0..=search_end).rev() {
        if chars[i] == close_delim {
            depth += 1;
        } else if chars[i] == open_delim {
            if depth == 0 {
                open_pos = Some(i);
                break;
            }
            depth -= 1;
        }
    }

    let open = open_pos?;

    let mut close_pos = None;
    depth = 0;

    for (i, &ch) in chars.iter().enumerate().skip(open + 1) {
        if ch == open_delim {
            depth += 1;
        } else if ch == close_delim {
            if depth == 0 {
                close_pos = Some(i);
                break;
            }
            depth -= 1;
        }
    }

    let close = close_pos?;

    if cursor_col > close {
        return None;
    }

    match scope {
        TextObjectScope::Inner => Some((open + 1, close)),
        TextObjectScope::Around => Some((open, close + 1)),
    }
}

/// Find pipe-separated segment bounds.
/// Pipes act as simple separators (no nesting).
pub fn find_pipe_bounds(
    text: &str,
    cursor_col: usize,
    scope: TextObjectScope,
) -> Option<(usize, usize)> {
    let chars: Vec<char> = text.chars().collect();

    if chars.is_empty() {
        return None;
    }

    let cursor_col = cursor_col.min(chars.len().saturating_sub(1));

    // Find left boundary: nearest pipe to the left, or start of string
    let left_pipe = (0..cursor_col).rev().find(|&i| chars[i] == '|');

    // Find right boundary: nearest pipe to the right, or end of string
    let right_pipe = ((cursor_col + 1)..chars.len()).find(|&i| chars[i] == '|');

    // If cursor is on a pipe, treat it as part of the right segment
    let (start, end) = if chars[cursor_col] == '|' {
        let start = left_pipe.map(|p| p + 1).unwrap_or(0);
        (start, cursor_col)
    } else {
        let start = left_pipe.map(|p| p + 1).unwrap_or(0);
        let end = right_pipe.unwrap_or(chars.len());
        (start, end)
    };

    // Trim whitespace for inner scope
    match scope {
        TextObjectScope::Inner => {
            let trimmed_start = (start..end)
                .find(|&i| !chars[i].is_whitespace())
                .unwrap_or(end);
            let trimmed_end = (start..end)
                .rev()
                .find(|&i| !chars[i].is_whitespace())
                .map(|i| i + 1)
                .unwrap_or(start);
            if trimmed_start >= trimmed_end {
                None
            } else {
                Some((trimmed_start, trimmed_end))
            }
        }
        TextObjectScope::Around => {
            // First find trimmed content bounds
            let trimmed_start = (start..end)
                .find(|&i| !chars[i].is_whitespace())
                .unwrap_or(end);
            let trimmed_end = (start..end)
                .rev()
                .find(|&i| !chars[i].is_whitespace())
                .map(|i| i + 1)
                .unwrap_or(start);

            if trimmed_start >= trimmed_end {
                return None;
            }

            // Around always deletes content + one pipe:
            // - If trailing pipe exists: delete content + trailing pipe + whitespace after
            // - Else if leading pipe exists: delete leading pipe + content
            // - Else (single segment): same as inner
            match (right_pipe, left_pipe) {
                (Some(rp), _) if chars[cursor_col] != '|' => {
                    // Delete content + trailing pipe + whitespace after
                    let after_pipe = ((rp + 1)..chars.len())
                        .find(|&i| !chars[i].is_whitespace())
                        .unwrap_or(chars.len());
                    Some((trimmed_start, after_pipe))
                }
                (_, Some(lp)) => {
                    // Delete leading pipe + content
                    Some((lp, trimmed_end))
                }
                _ => {
                    // Single segment with no pipes: same as inner
                    Some((trimmed_start, trimmed_end))
                }
            }
        }
    }
}

/// Find text object boundaries based on target type.
pub fn find_text_object_bounds(
    text: &str,
    cursor_col: usize,
    target: TextObjectTarget,
    scope: TextObjectScope,
) -> Option<(usize, usize)> {
    match target {
        TextObjectTarget::Word => find_word_bounds(text, cursor_col, scope),
        TextObjectTarget::DoubleQuote => find_quote_bounds(text, cursor_col, '"', scope),
        TextObjectTarget::SingleQuote => find_quote_bounds(text, cursor_col, '\'', scope),
        TextObjectTarget::Backtick => find_quote_bounds(text, cursor_col, '`', scope),
        TextObjectTarget::Pipe => find_pipe_bounds(text, cursor_col, scope),
        TextObjectTarget::Parentheses | TextObjectTarget::Brackets | TextObjectTarget::Braces => {
            let (open, close) = target.delimiters()?;
            find_bracket_bounds(text, cursor_col, open, close, scope)
        }
    }
}

/// Execute text object operation: select and delete the text object.
/// Returns true if operation was successful.
pub fn execute_text_object(
    textarea: &mut TextArea,
    target: TextObjectTarget,
    scope: TextObjectScope,
) -> bool {
    let text = textarea.lines().first().map(|s| s.as_str()).unwrap_or("");
    let cursor_col = textarea.cursor().1;

    let Some((start, end)) = find_text_object_bounds(text, cursor_col, target, scope) else {
        return false;
    };

    if start >= end {
        return false;
    }

    textarea.cancel_selection();

    move_cursor_to(textarea, start);

    textarea.start_selection();

    for _ in start..end {
        textarea.move_cursor(tui_textarea::CursorMove::Forward);
    }

    textarea.cut();

    true
}

fn move_cursor_to(textarea: &mut TextArea, col: usize) {
    textarea.move_cursor(tui_textarea::CursorMove::Head);
    for _ in 0..col {
        textarea.move_cursor(tui_textarea::CursorMove::Forward);
    }
}

#[cfg(test)]
#[path = "text_objects_tests.rs"]
mod text_objects_tests;
