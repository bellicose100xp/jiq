//! Syntax highlighting for jq query expressions.
//!
//! This module provides syntax highlighting for jq queries by tokenizing the input
//! and applying color styles based on token types:
//! - Keywords (if, then, else, etc.) → Yellow
//! - Built-in functions (map, select, etc.) → Blue
//! - Variables ($foo, $x, etc.) → Red
//! - Object field names (in {name: value}) → Cyan
//! - Numbers → Cyan
//! - Strings → Green
//! - Operators (|, ==, +, etc.) → Magenta

pub mod overlay;

use ratatui::style::{Color, Style};
use ratatui::text::Span;

pub struct JqHighlighter;

impl JqHighlighter {
    pub fn highlight(text: &str) -> Vec<Span<'static>> {
        let mut spans = Vec::new();
        let chars: Vec<char> = text.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            if chars[i].is_whitespace() {
                let (content, new_i) = parse_whitespace(&chars, i);
                spans.push(Span::raw(content));
                i = new_i;
                continue;
            }

            if chars[i] == '"' {
                let (content, new_i) = parse_string(&chars, i);
                spans.push(Span::styled(content, Style::default().fg(Color::Green)));
                i = new_i;
                continue;
            }

            if chars[i].is_ascii_digit()
                || (chars[i] == '-' && i + 1 < chars.len() && chars[i + 1].is_ascii_digit())
            {
                let (content, new_i) = parse_number(&chars, i);
                spans.push(Span::styled(content, Style::default().fg(Color::Cyan)));
                i = new_i;
                continue;
            }

            if is_operator(chars[i]) {
                let (content, new_i) = parse_operator(&chars, i);
                spans.push(Span::styled(content, Style::default().fg(Color::Magenta)));
                i = new_i;
                continue;
            }

            if chars[i].is_alphabetic() || chars[i] == '_' || chars[i] == '.' || chars[i] == '$' {
                let (word, new_i, starts_with_dot) = parse_identifier(&chars, i);
                let is_object_field = !starts_with_dot && is_followed_by_colon(&chars, new_i);
                let style = classify_word(&word, is_object_field);
                spans.push(Span::styled(word, style));
                i = new_i;
                continue;
            }

            spans.push(Span::raw(chars[i].to_string()));
            i += 1;
        }

        spans
    }
}

/// Parses consecutive whitespace characters starting at position `i`.
///
/// # Parameters
/// - `chars`: Character array of the query text
/// - `i`: Starting index of the whitespace
///
/// # Returns
/// Tuple of (whitespace_string, new_index)
fn parse_whitespace(chars: &[char], i: usize) -> (String, usize) {
    let start = i;
    let mut pos = i;
    while pos < chars.len() && chars[pos].is_whitespace() {
        pos += 1;
    }
    (chars[start..pos].iter().collect(), pos)
}

/// Parses a string literal starting at the opening quote.
///
/// Handles escape sequences by skipping over escaped characters.
/// Continues until the closing quote or end of input.
///
/// # Parameters
/// - `chars`: Character array of the query text
/// - `start`: Index of the opening quote character
///
/// # Returns
/// Tuple of (string_content, end_index)
fn parse_string(chars: &[char], start: usize) -> (String, usize) {
    let mut i = start + 1;
    while i < chars.len() {
        if chars[i] == '\\' && i + 1 < chars.len() {
            i += 2;
        } else if chars[i] == '"' {
            i += 1;
            break;
        } else {
            i += 1;
        }
    }
    (chars[start..i].iter().collect(), i)
}

/// Parses a number (including negative and decimal).
///
/// # Parameters
/// - `chars`: Character array of the query text
/// - `start`: Index where the number starts
///
/// # Returns
/// Tuple of (number_string, end_index)
fn parse_number(chars: &[char], start: usize) -> (String, usize) {
    let mut i = start;
    if chars[i] == '-' {
        i += 1;
    }
    while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') {
        i += 1;
    }
    (chars[start..i].iter().collect(), i)
}

/// Parses an operator (single or two-character).
///
/// Checks for two-character operators (==, !=, <=, >=, //) and falls back
/// to single-character operators.
///
/// # Parameters
/// - `chars`: Character array of the query text
/// - `i`: Index of the operator character
///
/// # Returns
/// Tuple of (operator_string, new_index)
fn parse_operator(chars: &[char], i: usize) -> (String, usize) {
    let mut op = String::from(chars[i]);
    let mut pos = i + 1;

    if pos < chars.len() {
        let two_char = format!("{}{}", op, chars[pos]);
        if is_two_char_operator(&two_char) {
            op = two_char;
            pos += 1;
        }
    }

    (op, pos)
}

/// Parses an identifier (word starting with letter, _, ., or $).
///
/// Continues parsing while characters are alphanumeric, underscore, dot, or dollar sign.
///
/// # Parameters
/// - `chars`: Character array of the query text
/// - `start`: Index where the identifier starts
///
/// # Returns
/// Tuple of (word, end_index, starts_with_dot)
fn parse_identifier(chars: &[char], start: usize) -> (String, usize, bool) {
    let starts_with_dot = chars[start] == '.';
    let mut i = start;

    while i < chars.len()
        && (chars[i].is_alphanumeric() || chars[i] == '_' || chars[i] == '.' || chars[i] == '$')
    {
        i += 1;
    }

    let word = chars[start..i].iter().collect();
    (word, i, starts_with_dot)
}

/// Checks if an identifier is followed by a colon (object field context).
///
/// Skips whitespace before checking for the colon character.
///
/// # Parameters
/// - `chars`: Character array of the query text
/// - `pos`: Position after the identifier
///
/// # Returns
/// true if a colon follows (making this an object field name)
fn is_followed_by_colon(chars: &[char], pos: usize) -> bool {
    if pos >= chars.len() {
        return false;
    }

    let mut j = pos;
    while j < chars.len() && chars[j].is_whitespace() {
        j += 1;
    }
    j < chars.len() && chars[j] == ':'
}

/// Determines the style for a word based on its classification.
///
/// Classification order (important - checked in sequence):
/// 1. Keywords (if, then, else, etc.) → Yellow
/// 2. Built-in functions (map, select, etc.) → Blue
/// 3. Variables (starts with $) → Red
/// 4. Object field names (followed by :) → Cyan
/// 5. Default (field accessors like .name) → No color
///
/// # Parameters
/// - `word`: The identifier text
/// - `is_object_field`: Whether this identifier is followed by a colon
///
/// # Returns
/// Style with appropriate color applied
fn classify_word(word: &str, is_object_field: bool) -> Style {
    if is_keyword(word) {
        Style::default().fg(Color::Yellow)
    } else if is_builtin_function(word) {
        Style::default().fg(Color::Blue)
    } else if is_variable(word) {
        Style::default().fg(Color::Red)
    } else if is_object_field {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default()
    }
}

fn is_operator(ch: char) -> bool {
    matches!(
        ch,
        '|' | '='
            | '!'
            | '<'
            | '>'
            | '+'
            | '-'
            | '*'
            | '/'
            | '%'
            | '('
            | ')'
            | '['
            | ']'
            | '{'
            | '}'
            | ','
            | ';'
            | ':'
            | '?'
            | '@'
    )
}

fn is_two_char_operator(op: &str) -> bool {
    matches!(op, "==" | "!=" | "<=" | ">=" | "//")
}

fn is_keyword(word: &str) -> bool {
    matches!(
        word,
        "if" | "then"
            | "else"
            | "elif"
            | "end"
            | "and"
            | "or"
            | "not"
            | "as"
            | "def"
            | "reduce"
            | "foreach"
            | "try"
            | "catch"
            | "import"
            | "include"
            | "module"
            | "empty"
            | "null"
            | "true"
            | "false"
    )
}

fn is_builtin_function(word: &str) -> bool {
    matches!(
        word,
        "type"
            | "length"
            | "keys"
            | "keys_unsorted"
            | "values"
            | "empty"
            | "has"
            | "in"
            | "contains"
            | "inside"
            | "getpath"
            | "setpath"
            | "delpaths"
            | "map"
            | "select"
            | "sort"
            | "sort_by"
            | "reverse"
            | "unique"
            | "unique_by"
            | "group_by"
            | "min"
            | "max"
            | "min_by"
            | "max_by"
            | "add"
            | "any"
            | "all"
            | "flatten"
            | "range"
            | "first"
            | "last"
            | "nth"
            | "indices"
            | "index"
            | "rindex"
            | "to_entries"
            | "from_entries"
            | "with_entries"
            | "tostring"
            | "tonumber"
            | "toarray"
            | "split"
            | "join"
            | "ltrimstr"
            | "rtrimstr"
            | "startswith"
            | "endswith"
            | "test"
            | "match"
            | "capture"
            | "sub"
            | "gsub"
            | "ascii_downcase"
            | "ascii_upcase"
            | "floor"
            | "ceil"
            | "round"
            | "sqrt"
            | "pow"
            | "now"
            | "fromdateiso8601"
            | "todateiso8601"
            | "fromdate"
            | "todate"
            | "input"
            | "inputs"
            | "debug"
            | "error"
            | "recurse"
            | "walk"
            | "paths"
            | "leaf_paths"
            | "limit"
            | "until"
            | "while"
            | "repeat"
    )
}

/// Checks if a word is a jq variable (starts with $).
///
/// # Parameters
/// - `word`: The identifier text
///
/// # Returns
/// true if the word starts with the $ character
fn is_variable(word: &str) -> bool {
    word.starts_with('$')
}

#[cfg(test)]
#[path = "syntax_highlight_tests.rs"]
mod syntax_highlight_tests;

#[cfg(test)]
pub mod snapshot_helpers {
    pub use super::syntax_highlight_tests::serialize_spans;
}
