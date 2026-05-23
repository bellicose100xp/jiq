//! UTF-8 safe conversions between character indices and byte offsets.
//!
//! `tui_textarea` reports cursor positions as character indices, but Rust
//! string slicing requires byte offsets. Using these helpers at module
//! boundaries lets downstream code treat positions uniformly as byte offsets.

/// Convert a character index to its byte offset in `s`.
///
/// If `char_pos` is at or past the end of the string, returns `s.len()`.
pub fn char_pos_to_byte_pos(s: &str, char_pos: usize) -> usize {
    s.char_indices()
        .nth(char_pos)
        .map(|(b, _)| b)
        .unwrap_or(s.len())
}

/// Convert a byte offset to its character index in `s`.
///
/// If `byte_pos` is at or past the end of the string, returns the total
/// character count. If `byte_pos` falls inside a multi-byte character,
/// returns the character index of the *next* character boundary.
/// In practice callers pass byte positions that lie on char boundaries.
pub fn byte_pos_to_char_pos(s: &str, byte_pos: usize) -> usize {
    if byte_pos >= s.len() {
        return s.chars().count();
    }
    s.char_indices().take_while(|(b, _)| *b < byte_pos).count()
}

/// Truncate `s` from the front, keeping the trailing characters that fit
/// within `max_width` display columns and prefixing with `…` when content
/// is dropped. Display width is counted via `unicode_width` so CJK and
/// emoji widths are handled correctly.
pub fn head_truncate_to_width(s: &str, max_width: usize) -> String {
    use unicode_width::UnicodeWidthChar;

    if max_width == 0 {
        return String::new();
    }

    let mut total: usize = 0;
    for ch in s.chars() {
        total += UnicodeWidthChar::width(ch).unwrap_or(0);
    }
    if total <= max_width {
        return s.to_string();
    }

    let budget = max_width.saturating_sub(1);
    let mut accumulated: usize = 0;
    let mut start_byte = s.len();
    for (idx, ch) in s.char_indices().rev() {
        let w = UnicodeWidthChar::width(ch).unwrap_or(0);
        if accumulated + w > budget {
            break;
        }
        accumulated += w;
        start_byte = idx;
    }

    let mut out = String::with_capacity(s.len() - start_byte + 4);
    out.push('…');
    out.push_str(&s[start_byte..]);
    out
}

#[cfg(test)]
#[path = "str_utils_tests.rs"]
mod str_utils_tests;
