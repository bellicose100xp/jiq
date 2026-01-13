use super::scan_state::ScanState;
use std::collections::HashSet;

const BUILTIN_VARIABLES: &[&str] = &["$ENV", "$__loc__"];

/// Extracts all unique variable names defined in the query.
/// Returns a deduplicated list including built-in variables.
pub fn extract_variables(query: &str) -> Vec<String> {
    let mut variables: HashSet<String> = HashSet::new();

    for var in extract_user_variables(query) {
        variables.insert(var);
    }

    for builtin in BUILTIN_VARIABLES {
        variables.insert((*builtin).to_string());
    }

    let mut result: Vec<String> = variables.into_iter().collect();
    result.sort();
    result
}

/// Extracts user-defined variables from the query, skipping those inside strings.
fn extract_user_variables(query: &str) -> Vec<String> {
    let mut variables = Vec::new();
    let chars: Vec<char> = query.chars().collect();
    let mut scan_state = ScanState::Normal;
    let mut i = 0;

    while i < chars.len() {
        let ch = chars[i];
        let prev_state = scan_state;
        scan_state = scan_state.advance(ch);

        if prev_state.is_in_string() {
            i += 1;
            continue;
        }

        if is_variable_definition_keyword_at(&chars, i)
            && let Some((var_names, end_pos)) = extract_variables_after_keyword(&chars, i)
        {
            for var_name in var_names {
                variables.push(var_name);
            }
            i = end_pos;
            continue;
        }

        i += 1;
    }

    variables
}

/// Checks if we're at a variable definition keyword (as, label).
fn is_variable_definition_keyword_at(chars: &[char], pos: usize) -> bool {
    is_keyword_at(chars, pos, "as") || is_keyword_at(chars, pos, "label")
}

/// Checks if a specific keyword exists at position with proper word boundaries.
fn is_keyword_at(chars: &[char], pos: usize, keyword: &str) -> bool {
    let keyword_chars: Vec<char> = keyword.chars().collect();

    if pos > 0 && is_identifier_char(chars[pos - 1]) {
        return false;
    }

    if pos + keyword_chars.len() > chars.len() {
        return false;
    }

    for (j, kc) in keyword_chars.iter().enumerate() {
        if chars[pos + j] != *kc {
            return false;
        }
    }

    let after_pos = pos + keyword_chars.len();
    if after_pos < chars.len() && is_identifier_char(chars[after_pos]) {
        return false;
    }

    true
}

/// Extracts variable names after a definition keyword (as, label).
/// Returns the variables found and the position after extraction.
fn extract_variables_after_keyword(
    chars: &[char],
    keyword_pos: usize,
) -> Option<(Vec<String>, usize)> {
    let keyword_len = if is_keyword_at(chars, keyword_pos, "label") {
        5
    } else {
        2
    };

    let mut pos = keyword_pos + keyword_len;

    pos = skip_whitespace(chars, pos);

    if pos >= chars.len() {
        return None;
    }

    let ch = chars[pos];

    if ch == '$' {
        if let Some((var_name, end_pos)) = extract_single_variable(chars, pos) {
            return Some((vec![var_name], end_pos));
        }
    } else if ch == '[' {
        return extract_array_destructure_variables(chars, pos);
    } else if ch == '{' {
        return extract_object_destructure_variables(chars, pos);
    }

    None
}

/// Extracts a single variable name starting with $.
fn extract_single_variable(chars: &[char], pos: usize) -> Option<(String, usize)> {
    if pos >= chars.len() || chars[pos] != '$' {
        return None;
    }

    let mut end = pos + 1;
    while end < chars.len() && is_identifier_char(chars[end]) {
        end += 1;
    }

    if end == pos + 1 {
        return None;
    }

    let var_name: String = chars[pos..end].iter().collect();
    Some((var_name, end))
}

/// Extracts variables from array destructuring pattern: [$a, $b].
fn extract_array_destructure_variables(chars: &[char], pos: usize) -> Option<(Vec<String>, usize)> {
    if pos >= chars.len() || chars[pos] != '[' {
        return None;
    }

    let mut variables = Vec::new();
    let mut i = pos + 1;
    let mut depth = 1;

    while i < chars.len() && depth > 0 {
        let ch = chars[i];

        match ch {
            '[' => depth += 1,
            ']' => depth -= 1,
            '$' if depth >= 1 => {
                if let Some((var_name, end_pos)) = extract_single_variable(chars, i) {
                    variables.push(var_name);
                    i = end_pos;
                    continue;
                }
            }
            _ => {}
        }

        i += 1;
    }

    Some((variables, i))
}

/// Extracts variables from object destructuring pattern: {key: $a, other: $b}.
fn extract_object_destructure_variables(
    chars: &[char],
    pos: usize,
) -> Option<(Vec<String>, usize)> {
    if pos >= chars.len() || chars[pos] != '{' {
        return None;
    }

    let mut variables = Vec::new();
    let mut i = pos + 1;
    let mut depth = 1;

    while i < chars.len() && depth > 0 {
        let ch = chars[i];

        match ch {
            '{' => depth += 1,
            '}' => depth -= 1,
            '$' if depth >= 1 => {
                if let Some((var_name, end_pos)) = extract_single_variable(chars, i) {
                    variables.push(var_name);
                    i = end_pos;
                    continue;
                }
            }
            _ => {}
        }

        i += 1;
    }

    Some((variables, i))
}

/// Skips whitespace characters and returns the new position.
fn skip_whitespace(chars: &[char], mut pos: usize) -> usize {
    while pos < chars.len() && chars[pos].is_whitespace() {
        pos += 1;
    }
    pos
}

/// Checks if a character is valid in an identifier (alphanumeric or underscore).
fn is_identifier_char(ch: char) -> bool {
    ch.is_alphanumeric() || ch == '_'
}

#[cfg(test)]
#[path = "variable_extractor_tests.rs"]
mod variable_extractor_tests;
