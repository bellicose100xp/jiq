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
                let start = i;
                while i < chars.len() && chars[i].is_whitespace() {
                    i += 1;
                }
                spans.push(Span::raw(chars[start..i].iter().collect::<String>()));
                continue;
            }
            if chars[i] == '"' {
                let start = i;
                i += 1;
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
                spans.push(Span::styled(
                    chars[start..i].iter().collect::<String>(),
                    Style::default().fg(Color::Green),
                ));
                continue;
            }
            if chars[i].is_ascii_digit()
                || (chars[i] == '-' && i + 1 < chars.len() && chars[i + 1].is_ascii_digit())
            {
                let start = i;
                if chars[i] == '-' {
                    i += 1;
                }
                while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') {
                    i += 1;
                }
                spans.push(Span::styled(
                    chars[start..i].iter().collect::<String>(),
                    Style::default().fg(Color::Cyan),
                ));
                continue;
            }

            if is_operator(chars[i]) {
                let mut op = String::from(chars[i]);
                i += 1;

                if i < chars.len() {
                    let two_char = format!("{}{}", op, chars[i]);
                    if is_two_char_operator(&two_char) {
                        op = two_char;
                        i += 1;
                    }
                }

                spans.push(Span::styled(op, Style::default().fg(Color::Magenta)));
                continue;
            }
            if chars[i].is_alphabetic() || chars[i] == '_' || chars[i] == '.' || chars[i] == '$' {
                let start = i;

                let starts_with_dot = chars[i] == '.';

                while i < chars.len()
                    && (chars[i].is_alphanumeric()
                        || chars[i] == '_'
                        || chars[i] == '.'
                        || chars[i] == '$')
                {
                    i += 1;
                }

                let word = chars[start..i].iter().collect::<String>();

                let is_object_field = !starts_with_dot && i < chars.len() && {
                    let mut j = i;
                    while j < chars.len() && chars[j].is_whitespace() {
                        j += 1;
                    }
                    j < chars.len() && chars[j] == ':'
                };
                if is_keyword(&word) {
                    spans.push(Span::styled(word, Style::default().fg(Color::Yellow)));
                } else if is_builtin_function(&word) {
                    spans.push(Span::styled(word, Style::default().fg(Color::Blue)));
                } else if is_object_field {
                    spans.push(Span::styled(word, Style::default().fg(Color::Cyan)));
                } else {
                    spans.push(Span::raw(word));
                }
                continue;
            }
            spans.push(Span::raw(chars[i].to_string()));
            i += 1;
        }

        spans
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

#[cfg(test)]
#[path = "syntax_highlight_tests.rs"]
mod syntax_highlight_tests;

#[cfg(test)]
pub mod snapshot_helpers {
    pub use super::syntax_highlight_tests::serialize_spans;
}
