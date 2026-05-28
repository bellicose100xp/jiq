//! Insertion of selected string values into the query.
//!
//! Replaces the partial inside the active `"..."` with the selected value
//! (jq-escaped) and ensures exactly one closing `"`. Cursor lands on the byte
//! immediately after the closing `"`.

use tui_textarea::TextArea;

use super::value_trigger::ValueTrigger;
use crate::str_utils::byte_pos_to_char_pos;

/// Build the new query buffer after inserting a value.
/// Returns `(new_query, cursor_byte_after_close)`.
pub(crate) fn build_inserted(
    query: &str,
    cursor_byte: usize,
    trigger: &ValueTrigger,
    value: &str,
) -> (String, usize) {
    let _ = cursor_byte;
    let escaped = escape_jq_string(value);

    let body_start = trigger.quote_open_byte + 1;
    let prefix = &query[..body_start];

    // Suffix begins after the close quote of the active string. If a close
    // quote exists anywhere from body_start onward, skip past it; otherwise
    // the string runs to end-of-query and there is no suffix.
    let suffix_start = match find_close_quote(query, body_start) {
        Some(close) => close + 1,
        None => query.len(),
    };

    let suffix = &query[suffix_start..];
    let mut new_query = String::with_capacity(prefix.len() + escaped.len() + 1 + suffix.len());
    new_query.push_str(prefix);
    new_query.push_str(&escaped);
    new_query.push('"');
    let cursor_after = new_query.len();
    new_query.push_str(suffix);
    (new_query, cursor_after)
}

pub(crate) fn apply_to_textarea(
    textarea: &mut TextArea<'_>,
    query: &str,
    cursor_byte: usize,
    trigger: &ValueTrigger,
    value: &str,
) {
    let (new_query, cursor_after) = build_inserted(query, cursor_byte, trigger, value);
    let cursor_char = byte_pos_to_char_pos(&new_query, cursor_after);

    textarea.delete_line_by_head();
    textarea.delete_line_by_end();
    textarea.insert_str(&new_query);
    super::insertion::move_cursor_to_column(textarea, cursor_char);
}

fn escape_jq_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                out.push_str(&format!("\\u{:04x}", c as u32));
            }
            c => out.push(c),
        }
    }
    out
}

/// Locate the next unescaped `"` from `from`. Skips over `\\"` correctly.
fn find_close_quote(query: &str, from: usize) -> Option<usize> {
    let bytes = query.as_bytes();
    let mut i = from;
    while i < bytes.len() {
        match bytes[i] {
            b'\\' => i += 2,
            b'"' => return Some(i),
            _ => i += 1,
        }
    }
    None
}

#[cfg(test)]
#[path = "value_insertion_tests.rs"]
mod value_insertion_tests;
