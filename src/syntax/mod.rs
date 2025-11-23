use ratatui::style::{Color, Style};
use ratatui::text::Span;

/// Simple regex-free jq syntax highlighter
/// This provides basic keyword, operator, and literal highlighting
pub struct JqHighlighter;

impl JqHighlighter {
    /// Highlight a jq query string and return styled spans
    pub fn highlight(text: &str) -> Vec<Span<'static>> {
        let mut spans = Vec::new();
        let chars: Vec<char> = text.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            // Skip whitespace (keep it unstyled)
            if chars[i].is_whitespace() {
                let start = i;
                while i < chars.len() && chars[i].is_whitespace() {
                    i += 1;
                }
                spans.push(Span::raw(chars[start..i].iter().collect::<String>()));
                continue;
            }

            // String literals (double-quoted)
            if chars[i] == '"' {
                let start = i;
                i += 1;
                while i < chars.len() {
                    if chars[i] == '\\' && i + 1 < chars.len() {
                        i += 2; // Skip escaped character
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

            // Numbers
            if chars[i].is_ascii_digit() || (chars[i] == '-' && i + 1 < chars.len() && chars[i + 1].is_ascii_digit()) {
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

            // Operators and special characters
            if is_operator(chars[i]) {
                let mut op = String::from(chars[i]);
                i += 1;

                // Check for multi-character operators (==, !=, <=, >=, //)
                if i < chars.len() {
                    let two_char = format!("{}{}", op, chars[i]);
                    if is_two_char_operator(&two_char) {
                        op = two_char;
                        i += 1;
                    }
                }

                spans.push(Span::styled(
                    op,
                    Style::default().fg(Color::Magenta),
                ));
                continue;
            }

            // Keywords and identifiers
            if chars[i].is_alphabetic() || chars[i] == '_' || chars[i] == '.' || chars[i] == '$' {
                let start = i;

                // Check if this is a field accessor (starts with .)
                let starts_with_dot = chars[i] == '.';

                while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_' || chars[i] == '.' || chars[i] == '$') {
                    i += 1;
                }

                let word = chars[start..i].iter().collect::<String>();

                // Check if this identifier is followed by ':' (field name in object constructor)
                let is_object_field = !starts_with_dot && i < chars.len() && {
                    // Skip whitespace to check for ':'
                    let mut j = i;
                    while j < chars.len() && chars[j].is_whitespace() {
                        j += 1;
                    }
                    j < chars.len() && chars[j] == ':'
                };

                // Check if it's a keyword
                if is_keyword(&word) {
                    spans.push(Span::styled(
                        word,
                        Style::default().fg(Color::Yellow),
                    ));
                } else if is_builtin_function(&word) {
                    spans.push(Span::styled(
                        word,
                        Style::default().fg(Color::Blue),
                    ));
                } else if is_object_field {
                    // Field name in object constructor {name: value}
                    spans.push(Span::styled(
                        word,
                        Style::default().fg(Color::Cyan),
                    ));
                } else {
                    // Field accessors (.name) and regular identifiers - default color
                    spans.push(Span::raw(word));
                }
                continue;
            }

            // Single character we don't recognize
            spans.push(Span::raw(chars[i].to_string()));
            i += 1;
        }

        spans
    }
}

/// Check if a character is an operator
fn is_operator(ch: char) -> bool {
    matches!(
        ch,
        '|' | '=' | '!' | '<' | '>' | '+' | '-' | '*' | '/' | '%' |
        '(' | ')' | '[' | ']' | '{' | '}' | ',' | ';' | ':' | '?' | '@'
    )
}

/// Check if a two-character string is a multi-character operator
fn is_two_char_operator(op: &str) -> bool {
    matches!(
        op,
        "==" | "!=" | "<=" | ">=" | "//"
    )
}

/// Check if a word is a jq keyword
fn is_keyword(word: &str) -> bool {
    matches!(
        word,
        "if" | "then" | "else" | "elif" | "end" |
        "and" | "or" | "not" |
        "as" | "def" | "reduce" | "foreach" |
        "try" | "catch" |
        "import" | "include" | "module" |
        "empty" | "null" | "true" | "false"
    )
}

/// Check if a word is a built-in jq function
fn is_builtin_function(word: &str) -> bool {
    matches!(
        word,
        // Type and path functions
        "type" | "length" | "keys" | "keys_unsorted" | "values" | "empty" |
        "has" | "in" | "contains" | "inside" | "getpath" | "setpath" | "delpaths" |

        // Array functions
        "map" | "select" | "sort" | "sort_by" | "reverse" | "unique" | "unique_by" |
        "group_by" | "min" | "max" | "min_by" | "max_by" | "add" | "any" | "all" |
        "flatten" | "range" | "first" | "last" | "nth" | "indices" | "index" | "rindex" |

        // Object functions
        "to_entries" | "from_entries" | "with_entries" |

        // String functions
        "tostring" | "tonumber" | "toarray" | "split" | "join" | "ltrimstr" | "rtrimstr" |
        "startswith" | "endswith" | "test" | "match" | "capture" | "sub" | "gsub" |
        "ascii_downcase" | "ascii_upcase" |

        // Math functions
        "floor" | "ceil" | "round" | "sqrt" | "pow" |

        // Date functions
        "now" | "fromdateiso8601" | "todateiso8601" | "fromdate" | "todate" |

        // I/O functions
        "input" | "inputs" | "debug" | "error" |

        // Other
        "recurse" | "walk" | "paths" | "leaf_paths" |
        "limit" | "until" | "while" | "repeat"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_highlight_empty() {
        let spans = JqHighlighter::highlight("");
        assert_eq!(spans.len(), 0);
    }

    #[test]
    fn test_highlight_simple_field() {
        let spans = JqHighlighter::highlight(".name");
        assert_eq!(spans.len(), 1);
        // Field accessors are now default color (white)
        assert_eq!(spans[0].style.fg, None);
    }

    #[test]
    fn test_highlight_keyword() {
        let spans = JqHighlighter::highlight("if");
        assert_eq!(spans.len(), 1);
        // Keyword should be yellow
        assert_eq!(spans[0].style.fg, Some(Color::Yellow));
    }

    #[test]
    fn test_highlight_string() {
        let spans = JqHighlighter::highlight(r#""hello""#);
        assert_eq!(spans.len(), 1);
        // String should be green
        assert_eq!(spans[0].style.fg, Some(Color::Green));
    }

    #[test]
    fn test_highlight_number() {
        let spans = JqHighlighter::highlight("123");
        assert_eq!(spans.len(), 1);
        // Number should be cyan
        assert_eq!(spans[0].style.fg, Some(Color::Cyan));
    }

    #[test]
    fn test_highlight_function() {
        let spans = JqHighlighter::highlight("map");
        assert_eq!(spans.len(), 1);
        // Function should be blue
        assert_eq!(spans[0].style.fg, Some(Color::Blue));
    }

    #[test]
    fn test_highlight_operator() {
        let spans = JqHighlighter::highlight("|");
        assert_eq!(spans.len(), 1);
        // Operator should be magenta
        assert_eq!(spans[0].style.fg, Some(Color::Magenta));
    }

    #[test]
    fn test_highlight_complex_query() {
        let spans = JqHighlighter::highlight(r#".users[] | select(.active == true) | .name"#);
        // Should have multiple spans
        assert!(spans.len() > 5);
    }

    #[test]
    fn test_highlight_with_whitespace() {
        let spans = JqHighlighter::highlight("  map  ");
        assert!(spans.len() >= 2); // Whitespace + function + whitespace
    }

    // --- NEW COMPREHENSIVE EDGE CASE TESTS ---

    #[test]
    fn test_unterminated_string() {
        // Should not panic, just consume to end
        let spans = JqHighlighter::highlight(r#""unterminated"#);
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].style.fg, Some(Color::Green));
        assert_eq!(spans[0].content, r#""unterminated"#);
    }

    #[test]
    fn test_string_with_escapes() {
        let spans = JqHighlighter::highlight(r#""hello \"world\"""#);
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].style.fg, Some(Color::Green));
    }

    #[test]
    fn test_negative_number() {
        let spans = JqHighlighter::highlight("-123");
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].style.fg, Some(Color::Cyan));
        assert_eq!(spans[0].content, "-123");
    }

    #[test]
    fn test_decimal_number() {
        let spans = JqHighlighter::highlight("3.14");
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].style.fg, Some(Color::Cyan));
        assert_eq!(spans[0].content, "3.14");
    }

    #[test]
    fn test_two_char_operators() {
        // Test ==
        let spans = JqHighlighter::highlight("==");
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].content, "==");
        assert_eq!(spans[0].style.fg, Some(Color::Magenta));

        // Test !=
        let spans = JqHighlighter::highlight("!=");
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].content, "!=");

        // Test <=
        let spans = JqHighlighter::highlight("<=");
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].content, "<=");

        // Test >=
        let spans = JqHighlighter::highlight(">=");
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].content, ">=");

        // Test //
        let spans = JqHighlighter::highlight("//");
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].content, "//");
    }

    #[test]
    fn test_nested_field_path() {
        let spans = JqHighlighter::highlight(".foo.bar.baz");
        assert_eq!(spans.len(), 1);
        // Field accessors are now default color (white)
        assert_eq!(spans[0].style.fg, None);
        assert_eq!(spans[0].content, ".foo.bar.baz");
    }

    #[test]
    fn test_just_dot() {
        let spans = JqHighlighter::highlight(".");
        assert_eq!(spans.len(), 1);
        // Identity filter is default color (white)
        assert_eq!(spans[0].style.fg, None);
    }

    #[test]
    fn test_variable_reference() {
        let spans = JqHighlighter::highlight("$foo");
        assert_eq!(spans.len(), 1);
        // Should be treated as regular identifier (no color)
        assert_eq!(spans[0].style.fg, None);
    }

    #[test]
    fn test_keywords_and_or() {
        let spans = JqHighlighter::highlight("and");
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].style.fg, Some(Color::Yellow));

        let spans = JqHighlighter::highlight("or");
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].style.fg, Some(Color::Yellow));
    }

    #[test]
    fn test_comparison_in_context() {
        let spans = JqHighlighter::highlight(".age >= 18");
        // Should have: .age (cyan), space, >= (magenta), space, 18 (cyan)
        assert!(spans.len() >= 5);
        // Check the >= operator
        let op_span = spans.iter().find(|s| s.content == ">=");
        assert!(op_span.is_some());
        assert_eq!(op_span.unwrap().style.fg, Some(Color::Magenta));
    }

    #[test]
    fn test_empty_keyword() {
        // "empty" is both a keyword and a function - should be keyword
        let spans = JqHighlighter::highlight("empty");
        assert_eq!(spans.len(), 1);
        // Keywords are checked before functions, so should be yellow
        assert_eq!(spans[0].style.fg, Some(Color::Yellow));
    }

    #[test]
    fn test_unicode_in_string() {
        let spans = JqHighlighter::highlight(r#""hello 世界""#);
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].style.fg, Some(Color::Green));
    }

    #[test]
    fn test_array_indexing() {
        let spans = JqHighlighter::highlight(".items[0]");
        // Should highlight .items as field, [0] as operator+number
        assert!(spans.len() >= 3);
    }

    #[test]
    fn test_keywords_inside_strings_not_highlighted() {
        // Keywords inside strings should NOT be highlighted - entire string is green
        let spans = JqHighlighter::highlight(r#""if then else""#);
        assert_eq!(spans.len(), 1, "String should be a single span");
        assert_eq!(spans[0].content, r#""if then else""#);
        assert_eq!(spans[0].style.fg, Some(Color::Green), "Entire string should be green, keywords not highlighted");
    }

    #[test]
    fn test_query_with_string_containing_keywords() {
        let spans = JqHighlighter::highlight(r#"select(.status == "if")"#);

        // Find the string span
        let string_span = spans.iter().find(|s| s.content == r#""if""#);
        assert!(string_span.is_some(), "String should be present");
        assert_eq!(string_span.unwrap().style.fg, Some(Color::Green), "String 'if' should be green, not yellow");

        // Find the select keyword
        let select_span = spans.iter().find(|s| s.content == "select");
        assert!(select_span.is_some(), "select keyword should be present");
        assert_eq!(select_span.unwrap().style.fg, Some(Color::Blue), "select should be blue (function)");
    }

    #[test]
    fn test_object_field_names_highlighted() {
        // Test simple object constructor
        let spans = JqHighlighter::highlight("{name: .name}");

        // Find the field name (before :)
        let field_span = spans.iter().find(|s| s.content == "name");
        assert!(field_span.is_some(), "Field name 'name' should be present");
        assert_eq!(field_span.unwrap().style.fg, Some(Color::Cyan), "Object field name should be cyan");

        // The field accessor .name should be white (default)
        let accessor_span = spans.iter().find(|s| s.content == ".name");
        assert!(accessor_span.is_some(), "Field accessor '.name' should be present");
        assert_eq!(accessor_span.unwrap().style.fg, None, "Field accessor should be default color");
    }

    #[test]
    fn test_object_with_multiple_fields() {
        let spans = JqHighlighter::highlight("{firstName: .first, lastName: .last, age: .age}");

        // Check that object field names are cyan
        for field_name in ["firstName", "lastName", "age"] {
            let field_span = spans.iter().find(|s| s.content == field_name);
            assert!(field_span.is_some(), "Field '{}' should be present", field_name);
            assert_eq!(
                field_span.unwrap().style.fg,
                Some(Color::Cyan),
                "Object field '{}' should be cyan",
                field_name
            );
        }

        // Check that field accessors are white
        for accessor in [".first", ".last", ".age"] {
            let accessor_span = spans.iter().find(|s| s.content == accessor);
            assert!(accessor_span.is_some(), "Accessor '{}' should be present", accessor);
            assert_eq!(
                accessor_span.unwrap().style.fg,
                None,
                "Field accessor '{}' should be default color",
                accessor
            );
        }
    }

    #[test]
    fn test_object_field_with_whitespace_before_colon() {
        // Test that field names are detected even with whitespace before ':'
        let spans = JqHighlighter::highlight("{name : .value}");

        let field_span = spans.iter().find(|s| s.content == "name");
        assert!(field_span.is_some(), "Field name should be present");
        assert_eq!(field_span.unwrap().style.fg, Some(Color::Cyan), "Field name should be cyan even with whitespace");
    }
}
