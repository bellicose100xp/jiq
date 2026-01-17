//! Path parser for jq field access expressions.
//!
//! Parses paths like `.user.profile.name` into segments for JSON navigation.
//! Used by the autocomplete system to navigate nested structures.

/// A segment in a jq path expression.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PathSegment {
    /// Field access: `.name`
    Field(String),
    /// Optional field access: `.name?`
    OptionalField(String),
    /// Array iteration: `.[]`
    ArrayIterator,
    /// Array index access: `.[0]`, `.[-1]`
    ArrayIndex(i64),
}

/// Result of parsing a jq path expression.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedPath {
    /// Completed path segments
    pub segments: Vec<PathSegment>,
    /// Incomplete field name being typed (empty if cursor is after a `.`)
    pub partial: String,
}

impl ParsedPath {
    pub fn new(segments: Vec<PathSegment>, partial: String) -> Self {
        Self { segments, partial }
    }

    pub fn empty() -> Self {
        Self {
            segments: Vec::new(),
            partial: String::new(),
        }
    }
}

/// Parse a jq path expression into segments.
///
/// # Examples
/// ```text
/// parse_path(".user.profile.") → segments: [Field("user"), Field("profile")], partial: ""
/// parse_path(".user.prof") → segments: [Field("user")], partial: "prof"
/// parse_path(".[].name") → segments: [ArrayIterator, Field("name")], partial: ""
/// parse_path(".[0].data") → segments: [ArrayIndex(0), Field("data")], partial: ""
/// ```
pub fn parse_path(input: &str) -> ParsedPath {
    if input.is_empty() {
        return ParsedPath::empty();
    }

    let mut segments = Vec::new();
    let mut chars = input.chars().peekable();
    let mut partial = String::new();

    while let Some(ch) = chars.next() {
        match ch {
            '.' => {
                if !partial.is_empty() {
                    segments.push(field_segment(&partial, false));
                    partial.clear();
                }

                match chars.peek() {
                    Some('[') => {
                        chars.next();
                        if let Some(segment) = parse_bracket_content(&mut chars) {
                            segments.push(segment);
                        }
                    }
                    Some(&c) if is_field_start_char(c) => {
                        partial = collect_field_name(&mut chars);
                    }
                    Some('.') => {
                        // Skip recursive descent `..` - not supported
                    }
                    Some(_) | None => {
                        // Trailing dot or dot followed by non-field char
                    }
                }
            }
            '[' => {
                if !partial.is_empty() {
                    segments.push(field_segment(&partial, false));
                    partial.clear();
                }
                if let Some(segment) = parse_bracket_content(&mut chars) {
                    segments.push(segment);
                }
            }
            '?' => {
                if !partial.is_empty() {
                    segments.push(field_segment(&partial, true));
                    partial.clear();
                }
            }
            '(' => {
                // Function call: identifier followed by parens (e.g., select(...), map(...))
                // Skip the entire function call and continue parsing
                partial.clear();
                skip_function_call(&mut chars);
            }
            c if is_field_char(c) => {
                partial.push(c);
            }
            _ => {
                // Unknown character, stop parsing
                break;
            }
        }
    }

    ParsedPath::new(segments, partial)
}

fn field_segment(name: &str, optional: bool) -> PathSegment {
    if optional {
        PathSegment::OptionalField(name.to_string())
    } else {
        PathSegment::Field(name.to_string())
    }
}

fn is_field_start_char(c: char) -> bool {
    c.is_alphabetic() || c == '_'
}

fn is_field_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

fn collect_field_name(chars: &mut std::iter::Peekable<std::str::Chars>) -> String {
    let mut name = String::new();
    while let Some(&c) = chars.peek() {
        if is_field_char(c) {
            name.push(c);
            chars.next();
        } else {
            break;
        }
    }
    name
}

/// Parse content inside brackets: `[]`, `[0]`, `[-1]`, `["field"]`
fn parse_bracket_content(chars: &mut std::iter::Peekable<std::str::Chars>) -> Option<PathSegment> {
    match chars.peek() {
        Some(']') => {
            chars.next();
            skip_optional_marker(chars);
            Some(PathSegment::ArrayIterator)
        }
        Some('"') => {
            chars.next();
            let field_name = collect_quoted_string(chars);
            skip_closing_bracket(chars);
            Some(PathSegment::Field(field_name))
        }
        Some(&c) if c.is_ascii_digit() || c == '-' => {
            let index = collect_number(chars);
            skip_closing_bracket(chars);
            Some(PathSegment::ArrayIndex(index))
        }
        _ => None,
    }
}

fn collect_quoted_string(chars: &mut std::iter::Peekable<std::str::Chars>) -> String {
    let mut result = String::new();
    let mut escaped = false;

    for c in chars.by_ref() {
        if escaped {
            result.push(c);
            escaped = false;
        } else if c == '\\' {
            escaped = true;
        } else if c == '"' {
            break;
        } else {
            result.push(c);
        }
    }

    result
}

fn collect_number(chars: &mut std::iter::Peekable<std::str::Chars>) -> i64 {
    let mut num_str = String::new();

    while let Some(&c) = chars.peek() {
        if c.is_ascii_digit() || c == '-' {
            num_str.push(c);
            chars.next();
        } else {
            break;
        }
    }

    num_str.parse().unwrap_or(0)
}

fn skip_closing_bracket(chars: &mut std::iter::Peekable<std::str::Chars>) {
    if chars.peek() == Some(&']') {
        chars.next();
        skip_optional_marker(chars);
    }
}

fn skip_optional_marker(chars: &mut std::iter::Peekable<std::str::Chars>) {
    if chars.peek() == Some(&'?') {
        chars.next();
    }
}

/// Skip a function call from opening `(` to matching `)`.
///
/// Handles nested parentheses and string literals within the function call.
/// After skipping, also consumes any trailing `?` (optional operator).
///
/// # Examples
/// - `(...)` → skipped
/// - `(.x == "y")` → skipped (handles strings)
/// - `(foo(bar))` → skipped (handles nesting)
/// - `(...)?` → skipped including the `?`
fn skip_function_call(chars: &mut std::iter::Peekable<std::str::Chars>) {
    let mut depth = 1;

    while depth > 0 {
        match chars.next() {
            Some('(') => depth += 1,
            Some(')') => depth -= 1,
            Some('"') => skip_string_in_function(chars),
            Some(_) => {} // Skip any other character
            None => break,
        }
    }

    skip_optional_marker(chars);
}

/// Skip a string literal inside a function call.
///
/// Called after consuming the opening `"`. Consumes characters
/// until the closing `"`, respecting escape sequences.
fn skip_string_in_function(chars: &mut std::iter::Peekable<std::str::Chars>) {
    let mut escaped = false;

    for c in chars.by_ref() {
        if escaped {
            escaped = false;
        } else if c == '\\' {
            escaped = true;
        } else if c == '"' {
            break;
        }
    }
}
