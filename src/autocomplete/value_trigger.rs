//! Trigger classifier for string-value autocomplete.
//!
//! Decides whether the cursor sits inside an unclosed `"..."` literal at a
//! value-comparison position. When it does, returns enough context to look up
//! candidate values from the loaded JSON. Pure function operating on byte
//! offsets so callers can feed it `cursor_pos` directly from the editor.
//!
//! Returns `None` for regex argument functions (`test`, `match`, `scan`,
//! `splits`, `sub`, `gsub`), strings containing `\(` interpolation, and
//! strings already terminated before the cursor.
//!
//! Path folding: when a trigger fires inside `select(...)` or after a pipe
//! `<expr> | <trigger>`, the classifier folds the surrounding context into a
//! single absolute path rooted at the JSON root. So
//! `.users[] | select(.role == "` produces `lhs_path = .users[].role` rather
//! than just `.role`. Folding recognizes `select` (identity on input) and
//! `map` (iterates input). Anything else (`reduce`, `foreach`, user-defined
//! functions, `sort_by`, etc.) causes folding to give up and return `None`,
//! and the dispatcher falls back to the global string list.

use super::scan_state::ScanState;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TriggerKind {
    Eq,
    Neq,
    Contains,
    StartsWith,
    EndsWith,
    Inside,
    In,
    HasOrIn,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ValueTrigger {
    pub kind: TriggerKind,
    pub lhs_path: Option<String>,
    pub partial: String,
    pub quote_open_byte: usize,
}

const REGEX_FUNCTIONS: &[&str] = &["test", "match", "scan", "splits", "sub", "gsub"];

const STRING_PREDICATES: &[(&str, TriggerKind)] = &[
    ("contains", TriggerKind::Contains),
    ("startswith", TriggerKind::StartsWith),
    ("endswith", TriggerKind::EndsWith),
    ("inside", TriggerKind::Inside),
];

/// Wrappers we treat as identity-on-input when folding paths. These functions
/// either pass their input through (`select`) or filter without transforming
/// (`first`, `last`, `values`, etc.). For our purposes they don't change the
/// path that should be walked from the JSON root.
const IDENTITY_WRAPPERS: &[&str] = &[
    "select",
    "first",
    "last",
    "limit",
    "values",
    "nulls",
    "booleans",
    "numbers",
    "strings",
    "arrays",
    "objects",
    "iterables",
    "scalars",
    "any",
    "all",
];

/// Wrappers that iterate their input (each element of an input array becomes
/// the `.` for the body). When folding through one, we prepend `[]` because
/// the body operates on each element.
const ITERATING_WRAPPERS: &[&str] = &["map", "map_values"];

pub(crate) fn classify(query: &str, cursor_byte: usize) -> Option<ValueTrigger> {
    let cursor = clamp_to_char_boundary(query, cursor_byte);
    let prefix = &query[..cursor];

    let (quote_open_byte, partial_text) = locate_active_string(prefix)?;
    if partial_contains_interpolation(&partial_text) {
        return None;
    }

    let before_quote = prefix[..quote_open_byte].trim_end_matches(is_h_ws);

    if let Some(kind) = match_eq_neq(before_quote) {
        let op_len = 2;
        let before_op = &before_quote[..before_quote.len() - op_len];
        let before_op_trimmed = before_op.trim_end_matches(is_h_ws);
        let inner_lhs = extract_trailing_path(before_op_trimmed);
        let lhs_start = match &inner_lhs {
            Some(p) => before_op_trimmed.len().saturating_sub(p.len()),
            None => before_op_trimmed.len(),
        };
        let lhs_path = fold_to_absolute_path(prefix, lhs_start, inner_lhs);
        return Some(ValueTrigger {
            kind,
            lhs_path,
            partial: partial_text,
            quote_open_byte,
        });
    }

    let call = enclosing_function_call(prefix, quote_open_byte)?;

    if REGEX_FUNCTIONS.iter().any(|n| *n == call.name) {
        return None;
    }

    if let Some((_, kind)) = STRING_PREDICATES.iter().find(|(n, _)| *n == call.name) {
        let (inner_lhs, lhs_start) = extract_pre_call_path(prefix, call.name_start);
        let lhs_path = fold_to_absolute_path(prefix, lhs_start, inner_lhs);
        return Some(ValueTrigger {
            kind: *kind,
            lhs_path,
            partial: partial_text,
            quote_open_byte,
        });
    }

    if call.name == "IN" {
        let inner_lhs = extract_in_lhs_path(&call.arg_text_until_quote);
        // For IN, fold from the position immediately before the call name —
        // that's where any `<path> | IN(...)` would be.
        let lhs_path = fold_to_absolute_path(prefix, call.name_start, inner_lhs);
        return Some(ValueTrigger {
            kind: TriggerKind::In,
            lhs_path,
            partial: partial_text,
            quote_open_byte,
        });
    }

    if call.name == "has" || call.name == "in" {
        let (inner_lhs, lhs_start) = extract_pre_call_path(prefix, call.name_start);
        let lhs_path = fold_to_absolute_path(prefix, lhs_start, inner_lhs);
        return Some(ValueTrigger {
            kind: TriggerKind::HasOrIn,
            lhs_path,
            partial: partial_text,
            quote_open_byte,
        });
    }

    None
}

/// Fold the inner relative path into an absolute path rooted at the JSON root
/// by walking outward through enclosing `select`/`map`/`<path> | <here>`
/// constructs. Returns `None` if the surrounding context can't be folded
/// safely (reduce, foreach, user-defined functions, etc.).
fn fold_to_absolute_path(prefix: &str, start: usize, inner: Option<String>) -> Option<String> {
    let mut accumulated = inner.unwrap_or_default();
    let mut cursor = start;
    let mut hops = 0;

    loop {
        if hops > 32 {
            return None;
        }
        hops += 1;

        match find_enclosing_construct(prefix, cursor) {
            EnclosingConstruct::None => break,
            EnclosingConstruct::Pipe { left_end } => {
                let left_path = extract_path_chain_ending_at(prefix, left_end)?;
                let left_path_start = left_end
                    .saturating_sub(skip_ws_back(prefix, left_end))
                    .saturating_sub(left_path.len());
                accumulated = concat_paths(&left_path, &accumulated);
                cursor = left_path_start;
            }
            EnclosingConstruct::IdentityWrapper { call_start } => {
                cursor = call_start;
            }
            EnclosingConstruct::IteratingWrapper { call_start } => {
                accumulated = concat_paths("[]", &accumulated);
                cursor = call_start;
            }
            EnclosingConstruct::Unknown => return None,
        }
    }

    if accumulated.is_empty() {
        return None;
    }
    canonicalize_absolute(&accumulated)
}

/// Concatenate two path segments. Both may or may not start with `.` or `[`.
fn concat_paths(left: &str, right: &str) -> String {
    if right.is_empty() {
        return left.to_string();
    }
    if left.is_empty() {
        return right.to_string();
    }
    // Avoid `.` between `[]` and the next segment when the next starts with `.`.
    // Just concatenate; `parse_path` handles either form.
    format!("{}{}", left, right)
}

fn canonicalize_absolute(path: &str) -> Option<String> {
    if path.starts_with('.') {
        Some(path.to_string())
    } else {
        Some(format!(".{}", path))
    }
}

#[derive(Debug)]
enum EnclosingConstruct {
    None,
    /// `<left_path> | <here>` — `left_end` is the byte position of the `|`.
    Pipe {
        left_end: usize,
    },
    /// `select(<here>)` and other identity-on-input wrappers — `call_start`
    /// is the byte position of the first character of the call name.
    IdentityWrapper {
        call_start: usize,
    },
    /// `map(<here>)` and other iterating wrappers — `call_start` is the byte
    /// position of the first character of the call name.
    IteratingWrapper {
        call_start: usize,
    },
    /// Some other enclosing construct we can't safely fold across (reduce,
    /// foreach, user-defined functions, sort_by, etc.).
    Unknown,
}

fn find_enclosing_construct(prefix: &str, cursor: usize) -> EnclosingConstruct {
    let bytes = prefix.as_bytes();
    let cursor = cursor.min(bytes.len());

    // Walk back through whitespace.
    let mut i = cursor;
    while i > 0 && is_h_ws(bytes[i - 1] as char) {
        i -= 1;
    }
    if i == 0 {
        return EnclosingConstruct::None;
    }

    // If we're directly preceded by `|`, that's a pipe boundary. Exclude `|=`
    // (update assignment).
    if bytes[i - 1] == b'|' {
        if i >= 2 && bytes[i - 2] == b'|' {
            return EnclosingConstruct::Unknown;
        }
        return EnclosingConstruct::Pipe { left_end: i - 1 };
    }

    // Otherwise look for an enclosing unclosed paren whose call name we
    // recognize. The paren must be the *immediate* enclosing context — i.e.
    // there must be only whitespace between `(` and the cursor.
    if bytes[i - 1] != b'(' {
        return EnclosingConstruct::None;
    }
    let between = &prefix[..cursor];
    let paren_pos = match find_innermost_unclosed_paren(between) {
        Some(p) => p,
        None => return EnclosingConstruct::None,
    };
    if paren_pos != i - 1 {
        return EnclosingConstruct::None;
    }

    let before_paren = &between[..paren_pos];
    let trimmed = before_paren.trim_end_matches(is_h_ws);
    let name_end = trimmed.len();
    let name_start = identifier_start(trimmed, name_end);
    if name_start == name_end {
        return EnclosingConstruct::Unknown;
    }
    let name = &trimmed[name_start..name_end];

    if IDENTITY_WRAPPERS.contains(&name) {
        return EnclosingConstruct::IdentityWrapper {
            call_start: name_start,
        };
    }
    if ITERATING_WRAPPERS.contains(&name) {
        return EnclosingConstruct::IteratingWrapper {
            call_start: name_start,
        };
    }
    EnclosingConstruct::Unknown
}

fn extract_path_chain_ending_at(prefix: &str, end: usize) -> Option<String> {
    let bytes = prefix.as_bytes();
    let mut i = end;
    while i > 0 && is_h_ws(bytes[i - 1] as char) {
        i -= 1;
    }
    let chain_end = i;
    while i > 0 {
        let b = bytes[i - 1];
        if is_path_byte(b) {
            i -= 1;
        } else {
            break;
        }
    }
    let candidate = &prefix[i..chain_end];
    let trimmed = candidate.trim();
    if trimmed.is_empty() || !trimmed.starts_with('.') {
        return None;
    }
    Some(trimmed.to_string())
}

fn skip_ws_back(prefix: &str, pos: usize) -> usize {
    let bytes = prefix.as_bytes();
    let pos = pos.min(bytes.len());
    let mut i = pos;
    while i > 0 && is_h_ws(bytes[i - 1] as char) {
        i -= 1;
    }
    pos - i
}

fn clamp_to_char_boundary(s: &str, byte: usize) -> usize {
    let mut clamped = byte.min(s.len());
    while clamped > 0 && !s.is_char_boundary(clamped) {
        clamped -= 1;
    }
    clamped
}

fn is_h_ws(c: char) -> bool {
    c == ' ' || c == '\t'
}

fn locate_active_string(prefix: &str) -> Option<(usize, String)> {
    let mut state = ScanState::default();
    let mut quote_byte: Option<usize> = None;

    for (idx, ch) in prefix.char_indices() {
        let was_in = state.is_in_string();
        let next = state.advance(ch);
        let now_in = next.is_in_string();
        if !was_in && now_in {
            quote_byte = Some(idx);
        } else if was_in && !now_in {
            quote_byte = None;
        }
        state = next;
    }

    if !state.is_in_string() {
        return None;
    }

    let qb = quote_byte?;
    let partial = prefix[qb + 1..].to_string();
    Some((qb, partial))
}

fn partial_contains_interpolation(partial: &str) -> bool {
    let bytes = partial.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'\\' {
            if i + 1 < bytes.len() && bytes[i + 1] == b'(' {
                return true;
            }
            i += 2;
            continue;
        }
        i += 1;
    }
    false
}

fn match_eq_neq(before_quote: &str) -> Option<TriggerKind> {
    if before_quote.ends_with("==") {
        Some(TriggerKind::Eq)
    } else if before_quote.ends_with("!=") {
        Some(TriggerKind::Neq)
    } else {
        None
    }
}

#[derive(Debug)]
struct EnclosingCall {
    name: String,
    name_start: usize,
    arg_text_until_quote: String,
}

fn enclosing_function_call(prefix: &str, quote_open_byte: usize) -> Option<EnclosingCall> {
    let between = &prefix[..quote_open_byte];
    let paren_pos = find_innermost_unclosed_paren(between)?;

    let after_paren = &between[paren_pos + 1..];
    let arg_text_until_quote = after_paren.trim().to_string();

    let before_paren = &between[..paren_pos];
    let trimmed = before_paren.trim_end_matches(is_h_ws);
    let name_end = trimmed.len();
    let name_start = identifier_start(trimmed, name_end);
    if name_start == name_end {
        return None;
    }
    let name = trimmed[name_start..name_end].to_string();

    Some(EnclosingCall {
        name,
        name_start,
        arg_text_until_quote,
    })
}

fn find_innermost_unclosed_paren(text: &str) -> Option<usize> {
    let mut state = ScanState::default();
    let mut stack: Vec<usize> = Vec::new();
    for (idx, ch) in text.char_indices() {
        let was_in = state.is_in_string();
        let next = state.advance(ch);
        if !was_in && !next.is_in_string() {
            match ch {
                '(' => stack.push(idx),
                ')' => {
                    stack.pop();
                }
                _ => {}
            }
        }
        state = next;
    }
    stack.last().copied()
}

fn identifier_start(text: &str, end: usize) -> usize {
    let bytes = text.as_bytes();
    let mut i = end;
    while i > 0 && is_ident_byte(bytes[i - 1]) {
        i -= 1;
    }
    i
}

fn is_ident_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

/// Returns `(inner_lhs, position_where_the_inner_path_starts_in_prefix)`.
/// When there's no `<path> |` immediately before the call, returns
/// `(None, name_start)` so callers can still fold from the call's own
/// position.
fn extract_pre_call_path(prefix: &str, name_start: usize) -> (Option<String>, usize) {
    let bytes = prefix.as_bytes();
    let mut i = name_start;
    while i > 0 && is_h_ws(bytes[i - 1] as char) {
        i -= 1;
    }
    if i == 0 || bytes[i - 1] != b'|' {
        return (None, name_start);
    }
    let pipe_pos = i - 1;
    let mut j = pipe_pos;
    while j > 0 && is_h_ws(bytes[j - 1] as char) {
        j -= 1;
    }
    let chain_end = j;
    while j > 0 {
        let b = bytes[j - 1];
        if is_path_byte(b) {
            j -= 1;
        } else {
            break;
        }
    }
    let candidate = &prefix[j..chain_end];
    if candidate.starts_with('.') {
        (Some(candidate.to_string()), j)
    } else {
        (None, name_start)
    }
}

fn extract_in_lhs_path(arg_text: &str) -> Option<String> {
    let semi = arg_text.find(';')?;
    let lhs = arg_text[..semi].trim();
    if lhs.is_empty() {
        return None;
    }
    canonicalize_path(lhs)
}

/// Walks backwards from the end of `text`, collecting only path-relevant
/// characters (`.`, identifier chars, `?`, `[`, `]`, digits, `-`). Returns the
/// resulting path if it begins with `.`.
fn extract_trailing_path(text: &str) -> Option<String> {
    if text.is_empty() {
        return None;
    }
    let bytes = text.as_bytes();
    let mut i = bytes.len();
    while i > 0 {
        let b = bytes[i - 1];
        if is_path_byte(b) {
            i -= 1;
        } else {
            break;
        }
    }
    let candidate = &text[i..];
    canonicalize_path(candidate)
}

fn is_path_byte(b: u8) -> bool {
    b == b'.' || b == b'?' || b == b'[' || b == b']' || b == b'-' || is_ident_byte(b)
}

fn canonicalize_path(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() || !trimmed.starts_with('.') {
        return None;
    }
    Some(trimmed.to_string())
}

#[cfg(test)]
#[path = "value_trigger_tests.rs"]
mod value_trigger_tests;
