//! Post-process AI-generated jq queries to enforce valid bracket notation
//! for non-ASCII keys.
//!
//! jq's `.field` shorthand only accepts ASCII identifiers. Smaller models
//! (e.g., Haiku) frequently emit `.名前` despite explicit prompt rules,
//! which produces a syntax error when executed. This sanitizer walks each
//! suggested query and rewrites any `.X` where X starts with a non-ASCII
//! character into `.["X"]`, making the correction independent of model
//! compliance.

/// Whether `name` qualifies as a jq `.field` shorthand — ASCII identifier
/// matching `[A-Za-z_][A-Za-z_0-9]*`.
fn is_ascii_jq_identifier(name: &str) -> bool {
    let mut chars = name.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

const SEPARATORS: &[char] = &[
    '.', '[', ']', '{', '}', '(', ')', '|', ';', ':', ',', '"', ' ', '\t', '\n', '?', '+', '-',
    '*', '=', '<', '>', '!', '@', '/',
];

/// Rewrite `.X` fragments where X begins with a non-ASCII character into
/// `.["X"]`. Preserves string literals, existing bracket notation, and all
/// other jq syntax untouched.
pub fn sanitize_jq_query(query: &str) -> String {
    let mut out = String::new();
    let mut chars = query.chars().peekable();
    let mut in_string = false;
    let mut escape = false;

    while let Some(c) = chars.next() {
        if escape {
            out.push(c);
            escape = false;
            continue;
        }
        if in_string {
            out.push(c);
            match c {
                '\\' => escape = true,
                '"' => in_string = false,
                _ => {}
            }
            continue;
        }
        if c == '"' {
            out.push(c);
            in_string = true;
            continue;
        }
        if c == '.'
            && let Some(&next) = chars.peek()
            && !SEPARATORS.contains(&next)
        {
            let mut name = String::new();
            while let Some(&nc) = chars.peek() {
                if SEPARATORS.contains(&nc) {
                    break;
                }
                name.push(nc);
                chars.next();
            }
            if is_ascii_jq_identifier(&name) {
                out.push('.');
                out.push_str(&name);
            } else {
                let escaped = name.replace('\\', "\\\\").replace('"', "\\\"");
                out.push_str(".[\"");
                out.push_str(&escaped);
                out.push_str("\"]");
            }
            continue;
        }
        out.push(c);
    }

    out
}

#[cfg(test)]
#[path = "sanitizer_tests.rs"]
mod sanitizer_tests;
