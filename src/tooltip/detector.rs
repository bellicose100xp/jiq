use crate::autocomplete::jq_functions::JQ_FUNCTION_METADATA;

/// Detect jq function at cursor position. Returns innermost enclosing function
/// if cursor is on a function name or inside its parentheses.
pub fn detect_function_at_cursor(query: &str, cursor_pos: usize) -> Option<&'static str> {
    // Handle edge cases
    if query.is_empty() {
        return None;
    }

    // Convert to chars for proper Unicode handling
    let chars: Vec<char> = query.chars().collect();
    let len = chars.len();

    // Cursor can be at position 0 to len (inclusive)
    if cursor_pos > len {
        return None;
    }

    // Phase 1: Check if cursor is directly on a function name
    if let Some(func) = detect_function_at_word(&chars, cursor_pos) {
        return Some(func);
    }

    // Phase 2: Find enclosing function by scanning for unmatched opening paren
    find_enclosing_function(&chars, cursor_pos)
}

/// Phase 1: Detect function when cursor is directly on a word
fn detect_function_at_word(chars: &[char], cursor_pos: usize) -> Option<&'static str> {
    let (start, end) = find_word_boundaries(chars, cursor_pos);

    // If no word found
    if start == end {
        return None;
    }

    // Extract the token
    let token: String = chars[start..end].iter().collect();

    // Look up in JQ_FUNCTION_METADATA
    lookup_function(&token)
}

fn find_enclosing_function(chars: &[char], cursor_pos: usize) -> Option<&'static str> {
    let mut depth: i32 = 0;
    let scan_start = cursor_pos.min(chars.len());

    // Scan backwards from cursor position
    for i in (0..scan_start).rev() {
        match chars[i] {
            ')' => depth += 1,
            '(' => {
                depth -= 1;
                // Found an unmatched opening paren
                if depth < 0 {
                    // Look for function name immediately before this '('
                    if let Some(func) = find_function_before_paren(chars, i) {
                        return Some(func);
                    }
                    // Reset depth and continue looking for outer function
                    depth = 0;
                }
            }
            _ => {}
        }
    }

    None
}

fn find_function_before_paren(chars: &[char], paren_pos: usize) -> Option<&'static str> {
    if paren_pos == 0 {
        return None;
    }

    // Find the end of the word (should be right before the paren)
    let mut end = paren_pos;

    // Skip any whitespace between function name and paren (though jq typically doesn't have this)
    while end > 0 && chars[end - 1].is_whitespace() {
        end -= 1;
    }

    if end == 0 || !is_word_char(chars[end - 1]) {
        return None;
    }

    // Find the start of the word
    let mut start = end - 1;
    while start > 0 && is_word_char(chars[start - 1]) {
        start -= 1;
    }

    // Extract the token
    let token: String = chars[start..end].iter().collect();

    // Look up in JQ_FUNCTION_METADATA
    lookup_function(&token)
}

fn find_word_boundaries(chars: &[char], cursor_pos: usize) -> (usize, usize) {
    let len = chars.len();

    if len == 0 {
        return (0, 0);
    }

    // Determine the character position to start from
    let check_pos = if cursor_pos >= len {
        if len > 0 {
            len - 1
        } else {
            return (0, 0);
        }
    } else if !is_word_char(chars[cursor_pos]) {
        // Cursor is on a non-word char, check if previous char is a word char
        if cursor_pos > 0 && is_word_char(chars[cursor_pos - 1]) {
            cursor_pos - 1
        } else {
            return (0, 0);
        }
    } else {
        cursor_pos
    };

    // Now check_pos should be on a word character
    if !is_word_char(chars[check_pos]) {
        return (0, 0);
    }

    // Find start of word (scan backwards)
    let mut start = check_pos;
    while start > 0 && is_word_char(chars[start - 1]) {
        start -= 1;
    }

    // Find end of word (scan forwards)
    let mut end = check_pos + 1;
    while end < len && is_word_char(chars[end]) {
        end += 1;
    }

    (start, end)
}

fn is_word_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

fn lookup_function(token: &str) -> Option<&'static str> {
    JQ_FUNCTION_METADATA
        .iter()
        .find(|f| f.name == token)
        .map(|f| f.name)
}

/// Detect jq operator at cursor position. Checks multi-char operators first.
pub fn detect_operator_at_cursor(query: &str, cursor_pos: usize) -> Option<&'static str> {
    if query.is_empty() {
        return None;
    }

    let chars: Vec<char> = query.chars().collect();
    let len = chars.len();

    // First, try to detect operator at current cursor position
    if cursor_pos < len
        && let Some(op) = detect_operator_at_position(&chars, cursor_pos)
    {
        return Some(op);
    }

    // If cursor is after the last character, check if it's immediately after an operator
    // This handles the case when user just typed an operator (cursor is after it)
    if cursor_pos > 0
        && let Some(op) = detect_operator_at_position(&chars, cursor_pos - 1)
    {
        return Some(op);
    }

    None
}

fn detect_operator_at_position(chars: &[char], pos: usize) -> Option<&'static str> {
    let len = chars.len();

    if pos >= len {
        return None;
    }

    let current = chars[pos];

    // Only check if cursor is on a potential operator character
    if !matches!(current, '/' | '|' | '=' | '.') {
        return None;
    }

    // Check for //= (3-char operator) - must check first
    if let Some(op) = check_triple_slash_equals(chars, pos) {
        return Some(op);
    }

    // Check for // (2-char operator)
    if let Some(op) = check_double_slash(chars, pos) {
        return Some(op);
    }

    // Check for |= (2-char operator)
    if let Some(op) = check_pipe_equals(chars, pos) {
        return Some(op);
    }

    // Check for .. (2-char operator)
    if let Some(op) = check_double_dot(chars, pos) {
        return Some(op);
    }

    None
}

fn check_triple_slash_equals(chars: &[char], cursor_pos: usize) -> Option<&'static str> {
    let len = chars.len();
    let current = chars[cursor_pos];

    // Cursor could be on first /, second /, or =
    match current {
        '/' => {
            // Check if this is the first / of //=
            if cursor_pos + 2 < len && chars[cursor_pos + 1] == '/' && chars[cursor_pos + 2] == '='
            {
                return Some("//=");
            }
            // Check if this is the second / of //=
            if cursor_pos > 0
                && cursor_pos + 1 < len
                && chars[cursor_pos - 1] == '/'
                && chars[cursor_pos + 1] == '='
            {
                return Some("//=");
            }
        }
        '=' => {
            // Check if this is the = of //=
            if cursor_pos >= 2 && chars[cursor_pos - 1] == '/' && chars[cursor_pos - 2] == '/' {
                return Some("//=");
            }
        }
        _ => {}
    }

    None
}

fn check_double_slash(chars: &[char], cursor_pos: usize) -> Option<&'static str> {
    let len = chars.len();
    let current = chars[cursor_pos];

    if current != '/' {
        return None;
    }

    // Check if this is the first / of //
    if cursor_pos + 1 < len && chars[cursor_pos + 1] == '/' {
        // Make sure it's not //=
        if cursor_pos + 2 >= len || chars[cursor_pos + 2] != '=' {
            return Some("//");
        }
    }

    // Check if this is the second / of //
    if cursor_pos > 0 && chars[cursor_pos - 1] == '/' {
        // Make sure it's not //=
        if cursor_pos + 1 >= len || chars[cursor_pos + 1] != '=' {
            return Some("//");
        }
    }

    None
}

fn check_pipe_equals(chars: &[char], cursor_pos: usize) -> Option<&'static str> {
    let len = chars.len();
    let current = chars[cursor_pos];

    match current {
        '|' => {
            // Check if | is followed by =
            if cursor_pos + 1 < len && chars[cursor_pos + 1] == '=' {
                return Some("|=");
            }
        }
        '=' => {
            // Check if = is preceded by |
            if cursor_pos > 0 && chars[cursor_pos - 1] == '|' {
                return Some("|=");
            }
        }
        _ => {}
    }

    None
}

fn check_double_dot(chars: &[char], cursor_pos: usize) -> Option<&'static str> {
    let len = chars.len();
    let current = chars[cursor_pos];

    if current != '.' {
        return None;
    }

    // Check if this is the first . of ..
    if cursor_pos + 1 < len && chars[cursor_pos + 1] == '.' {
        // Make sure it's not ... (three dots)
        if cursor_pos + 2 >= len || chars[cursor_pos + 2] != '.' {
            // Also make sure there's no dot before (not part of ...)
            if cursor_pos == 0 || chars[cursor_pos - 1] != '.' {
                return Some("..");
            }
        }
    }

    // Check if this is the second . of ..
    if cursor_pos > 0 && chars[cursor_pos - 1] == '.' {
        // Make sure it's not ... (three dots)
        if cursor_pos + 1 >= len || chars[cursor_pos + 1] != '.' {
            // Also make sure there's no dot before the first dot (not part of ...)
            if cursor_pos < 2 || chars[cursor_pos - 2] != '.' {
                return Some("..");
            }
        }
    }

    None
}

#[cfg(test)]
#[path = "detector_tests.rs"]
mod detector_tests;
