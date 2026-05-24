//! Shared JSON path emission and lookup utilities.
//!
//! Centralizes the canonical "format a key as jq access" rules so that any
//! caller that needs to emit a jq path (autocomplete, path-at-cursor, future
//! AI prompt construction) reaches for the same source of truth. Also
//! provides [`path_at_line`], which maps a 0-indexed line number in jq's
//! default pretty-printed output of a [`serde_json::Value`] to the jq path
//! that selects the value rendered on that line.
//!
//! The walker traverses keys in input order, which requires `serde_json` to
//! be compiled with the `preserve_order` feature (see `Cargo.toml`).

use serde_json::Value;

/// Check if a field name can use jq's simple dot syntax (e.g., `.foo`).
///
/// Per the jq manual, simple-dot keys must be ASCII alphanumeric or `_`,
/// and must not start with a digit. Names that don't fit require quoted
/// access (`."field-name"`) or bracket access (`.["field-name"]`).
pub fn is_simple_jq_identifier(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }
    let first_char = name.chars().next().unwrap();
    !first_char.is_ascii_digit() && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// Format a bracket-notation key access for jq, used for keys that don't
/// fit simple dot syntax. Uses `serde_json::to_string` so embedded `"`,
/// `\`, and control characters in keys are correctly escaped.
pub fn format_bracket_access(key: &str) -> String {
    let escaped = serde_json::to_string(key).unwrap_or_else(|_| format!("\"{}\"", key));
    format!("[{}]", escaped)
}

/// Format a field name for jq syntax, using bracket notation for keys that
/// don't fit simple dot syntax.
pub fn format_field_name(prefix: &str, name: &str) -> String {
    if is_simple_jq_identifier(name) {
        format!("{}{}", prefix, name)
    } else {
        format!("{}{}", prefix, format_bracket_access(name))
    }
}

/// One step in a structured JSON path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JsonPathStep {
    Key(String),
    Index(usize),
    /// Iterate-all marker (`[]` in jq). Produced by the `*` chord when
    /// turning a specific array index into a "match every element" step.
    Splat,
}

/// A structured path inside a JSON document.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct JsonPath {
    steps: Vec<JsonPathStep>,
}

impl JsonPath {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push_key(&mut self, key: impl Into<String>) {
        self.steps.push(JsonPathStep::Key(key.into()));
    }

    pub fn push_index(&mut self, index: usize) {
        self.steps.push(JsonPathStep::Index(index));
    }

    pub fn pop(&mut self) {
        self.steps.pop();
    }

    /// Replace the rightmost `Index` step with [`JsonPathStep::Splat`] and
    /// return `true`. If the path contains no `Index`, leave it unchanged
    /// and return `false`. Used by the `*` (iterate) chord to turn
    /// `.users[2].tags[1]` into `.users[2].tags[]`.
    pub fn splat_nearest_index(&mut self) -> bool {
        for step in self.steps.iter_mut().rev() {
            if let JsonPathStep::Index(_) = step {
                *step = JsonPathStep::Splat;
                return true;
            }
        }
        false
    }

    #[allow(dead_code)]
    pub fn steps(&self) -> &[JsonPathStep] {
        &self.steps
    }

    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.steps.is_empty()
    }

    /// Render as a jq path expression: `.users[2].profile["zip-code"]`.
    /// Always begins with `.` (representing the root identity).
    pub fn to_jq(&self) -> String {
        if self.steps.is_empty() {
            return String::from(".");
        }
        let mut out = String::from(".");
        for step in &self.steps {
            match step {
                JsonPathStep::Key(k) => {
                    if is_simple_jq_identifier(k) {
                        if !out.ends_with('.') {
                            out.push('.');
                        }
                        out.push_str(k);
                    } else {
                        out.push_str(&format_bracket_access(k));
                    }
                }
                JsonPathStep::Index(i) => {
                    if out.ends_with('.') && out.len() > 1 {
                        out.pop();
                    }
                    out.push_str(&format!("[{}]", i));
                }
                JsonPathStep::Splat => {
                    if out.ends_with('.') && out.len() > 1 {
                        out.pop();
                    }
                    out.push_str("[]");
                }
            }
        }
        out
    }
}

/// Parse a jq path expression that this module's `to_jq` could have
/// produced — `.`, `.foo`, `.["foo-bar"]`, `[5]`, `[]`, and concatenations
/// thereof — into a [`JsonPath`]. Returns `None` for any input outside
/// that grammar (pipes, filters, function calls, etc.).
///
/// Used by the `[` (step out) chord, which parses the user's *current
/// query* (or the trailing segment after the last pipe), drops the last
/// step, and renders back. The parser deliberately does NOT try to
/// understand full jq programs — anything it can't recognize bails out
/// and the chord becomes a no-op with a "can't step out of complex
/// query" notification.
pub fn parse_jq_path(input: &str) -> Option<JsonPath> {
    let s = input.trim();
    if s.is_empty() || s == "." {
        return Some(JsonPath::new());
    }
    let mut path = JsonPath::new();
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'.' => {
                i += 1;
                if i >= bytes.len() {
                    // Trailing dot is not part of our emitted syntax.
                    return None;
                }
                if bytes[i] == b'[' {
                    // `.[...]` — fall through to the bracket branch.
                    continue;
                }
                let start = i;
                while i < bytes.len() {
                    let c = bytes[i];
                    if c.is_ascii_alphanumeric() || c == b'_' {
                        i += 1;
                    } else {
                        break;
                    }
                }
                if start == i {
                    return None;
                }
                let key = std::str::from_utf8(&bytes[start..i]).ok()?;
                if !is_simple_jq_identifier(key) {
                    return None;
                }
                path.push_key(key);
            }
            b'[' => {
                i += 1;
                if i >= bytes.len() {
                    return None;
                }
                if bytes[i] == b']' {
                    path.steps.push(JsonPathStep::Splat);
                    i += 1;
                } else if bytes[i] == b'"' {
                    // Quoted key, JSON string semantics. Find the matching
                    // closing quote with backslash-escape awareness.
                    let key_start = i;
                    i += 1;
                    while i < bytes.len() && bytes[i] != b'"' {
                        if bytes[i] == b'\\' {
                            i += 1;
                            if i >= bytes.len() {
                                return None;
                            }
                        }
                        i += 1;
                    }
                    if i >= bytes.len() {
                        return None;
                    }
                    let key_end = i + 1; // include the closing quote
                    let raw = std::str::from_utf8(&bytes[key_start..key_end]).ok()?;
                    let key: String = serde_json::from_str(raw).ok()?;
                    i += 1;
                    if i >= bytes.len() || bytes[i] != b']' {
                        return None;
                    }
                    i += 1;
                    path.push_key(key);
                } else if bytes[i].is_ascii_digit() {
                    let start = i;
                    while i < bytes.len() && bytes[i].is_ascii_digit() {
                        i += 1;
                    }
                    if i >= bytes.len() || bytes[i] != b']' {
                        return None;
                    }
                    let idx_str = std::str::from_utf8(&bytes[start..i]).ok()?;
                    let idx: usize = idx_str.parse().ok()?;
                    i += 1;
                    path.push_index(idx);
                } else {
                    return None;
                }
            }
            _ => return None,
        }
    }
    Some(path)
}

/// Locate the path of the value pretty-printed on a given line.
///
/// The line layout matches `serde_json::to_string_pretty` with the default
/// 2-space indent. Walks `O(target_line)` and stops as soon as the target
/// row is reached. On lines holding only a closing bracket, the path
/// returned is the parent container's path. Returns `None` if `target_line`
/// is past the end of the rendered value.
pub fn path_at_line(value: &Value, target_line: usize) -> Option<JsonPath> {
    let mut walker = LineWalker::new(target_line);
    walker.walk(value, &mut JsonPath::new());
    walker.found
}

/// Count how many lines `serde_json::to_string_pretty` would emit for `value`.
///
/// Public for future stream-aware path lookups; currently consumed only by
/// the test suite to lock the walker's line-counting invariant against
/// `serde_json` updates.
#[allow(dead_code)]
pub fn pretty_line_count(value: &Value) -> usize {
    let mut counter = LineCounter::default();
    counter.count(value);
    counter.lines + 1
}

#[derive(Default)]
#[allow(dead_code)]
struct LineCounter {
    lines: usize,
}

#[allow(dead_code)]
impl LineCounter {
    fn count(&mut self, value: &Value) {
        match value {
            Value::Array(arr) => {
                if arr.is_empty() {
                    return;
                }
                for v in arr {
                    self.lines += 1;
                    self.count(v);
                }
                self.lines += 1;
            }
            Value::Object(map) => {
                if map.is_empty() {
                    return;
                }
                for (_k, v) in map {
                    self.lines += 1;
                    self.count(v);
                }
                self.lines += 1;
            }
            _ => {}
        }
    }
}

struct LineWalker {
    target: usize,
    cursor: usize,
    found: Option<JsonPath>,
}

impl LineWalker {
    fn new(target: usize) -> Self {
        Self {
            target,
            cursor: 0,
            found: None,
        }
    }

    fn check(&mut self, path: &JsonPath) -> bool {
        if self.found.is_some() {
            return true;
        }
        if self.cursor == self.target {
            self.found = Some(path.clone());
            return true;
        }
        false
    }

    fn walk(&mut self, value: &Value, path: &mut JsonPath) {
        if self.check(path) {
            return;
        }
        match value {
            Value::Array(arr) => {
                if arr.is_empty() {
                    return;
                }
                for (i, v) in arr.iter().enumerate() {
                    self.cursor += 1;
                    path.push_index(i);
                    if self.check(path) {
                        path.pop();
                        return;
                    }
                    self.walk(v, path);
                    path.pop();
                    if self.found.is_some() {
                        return;
                    }
                }
                self.cursor += 1;
                let _ = self.check(path);
            }
            Value::Object(map) => {
                if map.is_empty() {
                    return;
                }
                for (k, v) in map {
                    self.cursor += 1;
                    path.push_key(k);
                    if self.check(path) {
                        path.pop();
                        return;
                    }
                    self.walk(v, path);
                    path.pop();
                    if self.found.is_some() {
                        return;
                    }
                }
                self.cursor += 1;
                let _ = self.check(path);
            }
            _ => {}
        }
    }
}

#[cfg(test)]
#[path = "json_path_tests.rs"]
mod json_path_tests;
