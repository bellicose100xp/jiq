use crate::query::executor::JqExecutor;
use serde_json::Value;

/// Type of result returned by a jq query
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResultType {
    /// Array containing objects: [{"a": 1}, {"b": 2}]
    ArrayOfObjects,
    /// Multiple objects from destructuring: {"a": 1}\n{"b": 2}
    DestructuredObjects,
    /// Single object: {"a": 1}
    Object,
    /// Array of primitives: [1, 2, 3]
    Array,
    /// String value: "hello"
    String,
    /// Numeric value: 42, 3.14
    Number,
    /// Boolean value: true, false
    Boolean,
    /// Null value
    Null,
}

/// Type of character that precedes the trigger character
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CharType {
    PipeOperator,    // |
    Semicolon,       // ;
    Comma,           // ,
    Colon,           // :
    OpenParen,       // (
    OpenBracket,     // [
    OpenBrace,       // {
    CloseBracket,    // ]
    CloseBrace,      // }
    CloseParen,      // )
    QuestionMark,    // ?
    Dot,             // .
    NoOp,            // Regular identifier character
}

/// Query execution state
pub struct QueryState {
    pub executor: JqExecutor,
    pub result: Result<String, String>,
    pub last_successful_result: Option<String>,
    /// Unformatted result without ANSI codes (for autosuggestion analysis)
    pub last_successful_result_unformatted: Option<String>,
    /// Base query that produced the last successful result (for suggestions)
    pub base_query_for_suggestions: Option<String>,
    /// Type of the last successful result (for type-aware suggestions)
    pub base_type_for_suggestions: Option<ResultType>,
}

impl QueryState {
    /// Create a new QueryState with the given JSON input
    pub fn new(json_input: String) -> Self {
        let executor = JqExecutor::new(json_input);
        let result = executor.execute(".");
        let last_successful_result = result.as_ref().ok().cloned();
        let last_successful_result_unformatted = last_successful_result
            .as_ref()
            .map(|s| Self::strip_ansi_codes(s));

        let base_query_for_suggestions = Some(".".to_string());
        let base_type_for_suggestions = last_successful_result_unformatted
            .as_ref()
            .map(|s| Self::detect_result_type(s));

        Self {
            executor,
            result,
            last_successful_result,
            last_successful_result_unformatted,
            base_query_for_suggestions,
            base_type_for_suggestions,
        }
    }

    /// Execute a query and update results
    /// Only caches non-null results for autosuggestions
    pub fn execute(&mut self, query: &str) {
        self.result = self.executor.execute(query);
        if let Ok(result) = &self.result {
            // Only cache non-null results for autosuggestions
            // When typing partial queries like ".s", jq returns "null" (potentially with ANSI codes)
            // For array iterations, may return multiple nulls: "null\nnull\nnull\n"
            // We want to keep the last meaningful result for suggestions
            let unformatted = Self::strip_ansi_codes(result);

            // Check if result contains only nulls and whitespace
            let is_only_nulls = unformatted
                .lines()
                .filter(|line| !line.trim().is_empty())
                .all(|line| line.trim() == "null");

            if !is_only_nulls {
                self.last_successful_result = Some(result.clone());
                self.last_successful_result_unformatted = Some(unformatted.clone());

                // Cache base query and result type for type-aware suggestions
                // Trim trailing whitespace and incomplete operators/dots
                // Examples to strip:
                //   ".services | ." → ".services"
                //   ".services[]." → ".services[]"
                //   ".user " → ".user"
                let base_query = Self::normalize_base_query(query);
                self.base_query_for_suggestions = Some(base_query);
                self.base_type_for_suggestions = Some(Self::detect_result_type(&unformatted));
            }
        }
    }

    /// Normalize base query by stripping trailing incomplete operations
    ///
    /// Strips patterns like:
    /// - " | ." → pipe with identity (will be re-added by PipeOperator formula)
    /// - "." at end → trailing dot (incomplete field access)
    /// - Trailing whitespace
    ///
    /// Examples:
    /// - ".services | ." → ".services"
    /// - ".services[]." → ".services[]"
    /// - ".user " → ".user"
    /// - "." → "." (keep root as-is)
    fn normalize_base_query(query: &str) -> String {
        let mut base = query.trim_end().to_string();

        // Strip trailing " | ." pattern (pipe followed by identity)
        // The PipeOperator formula will re-add " | " with proper spacing
        if base.ends_with(" | .") {
            base = base[..base.len() - 4].trim_end().to_string();
        }
        // Strip trailing " | " (incomplete pipe without operand)
        else if base.ends_with(" |") {
            base = base[..base.len() - 2].trim_end().to_string();
        }
        // Strip trailing "." if it's incomplete field access
        // But preserve "." if it's the root query
        else if base.ends_with('.') && base.len() > 1 {
            base = base[..base.len() - 1].to_string();
        }

        base
    }

    /// Detect the type of a query result for type-aware autosuggestions
    ///
    /// Examines the structure of the result to determine:
    /// - Is it an array? Are elements objects or primitives?
    /// - Is it multiple values (destructured)?
    /// - Is it a single value? What type?
    fn detect_result_type(result: &str) -> ResultType {
        use serde_json::Deserializer;

        // Use streaming parser to read first value
        let mut deserializer = Deserializer::from_str(result).into_iter();

        let first_value = match deserializer.next() {
            Some(Ok(v)) => v,
            _ => return ResultType::Null,
        };

        // Check if there's a second value (indicates destructured output)
        let has_multiple_values = deserializer.next().is_some();

        // Determine type based on first value and whether there are more
        match first_value {
            Value::Object(_) if has_multiple_values => ResultType::DestructuredObjects,
            Value::Object(_) => ResultType::Object,
            Value::Array(ref arr) => {
                if arr.is_empty() {
                    ResultType::Array
                } else if matches!(arr[0], Value::Object(_)) {
                    ResultType::ArrayOfObjects
                } else {
                    ResultType::Array
                }
            }
            Value::String(_) => ResultType::String,
            Value::Number(_) => ResultType::Number,
            Value::Bool(_) => ResultType::Boolean,
            Value::Null => ResultType::Null,
        }
    }

    /// Classify a character into its CharType
    pub fn classify_char(ch: Option<char>) -> CharType {
        match ch {
            Some('|') => CharType::PipeOperator,
            Some(';') => CharType::Semicolon,
            Some(',') => CharType::Comma,
            Some(':') => CharType::Colon,
            Some('(') => CharType::OpenParen,
            Some('[') => CharType::OpenBracket,
            Some('{') => CharType::OpenBrace,
            Some(']') => CharType::CloseBracket,
            Some('}') => CharType::CloseBrace,
            Some(')') => CharType::CloseParen,
            Some('?') => CharType::QuestionMark,
            Some('.') => CharType::Dot,
            Some(_) => CharType::NoOp,
            None => CharType::NoOp,
        }
    }

    /// Strip ANSI escape codes from a string
    ///
    /// jq outputs colored results with ANSI codes like:
    /// - `\x1b[0m` (reset)
    /// - `\x1b[1;39m` (bold)
    /// - `\x1b[0;32m` (green)
    fn strip_ansi_codes(s: &str) -> String {
        let mut result = String::with_capacity(s.len());
        let mut chars = s.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == '\x1b' {
                // Found escape character, skip until 'm' (end of ANSI sequence)
                if chars.peek() == Some(&'[') {
                    chars.next(); // consume '['
                    while let Some(c) = chars.next() {
                        if c == 'm' {
                            break;
                        }
                    }
                }
            } else {
                result.push(ch);
            }
        }

        result
    }

    /// Get the total number of lines in the current results
    /// Note: Returns u32 to handle large files (>65K lines) correctly
    /// When there's an error, uses last_successful_result since that's what gets rendered
    pub fn line_count(&self) -> u32 {
        match &self.result {
            Ok(result) => result.lines().count() as u32,
            Err(_) => {
                self.last_successful_result
                    .as_ref()
                    .map(|r| r.lines().count() as u32)
                    .unwrap_or(0)
            }
        }
    }

    /// Get the maximum line width in the current results (for horizontal scrolling)
    pub fn max_line_width(&self) -> u16 {
        let content = match &self.result {
            Ok(result) => result,
            Err(_) => self.last_successful_result.as_deref().unwrap_or(""),
        };
        content
            .lines()
            .map(|l| l.len())
            .max()
            .unwrap_or(0)
            .min(u16::MAX as usize) as u16
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_query_state() {
        let json = r#"{"name": "test"}"#;
        let state = QueryState::new(json.to_string());

        assert!(state.result.is_ok());
        assert!(state.last_successful_result.is_some());
    }

    #[test]
    fn test_execute_updates_result() {
        let json = r#"{"name": "test", "age": 30}"#;
        let mut state = QueryState::new(json.to_string());

        state.execute(".name");
        assert!(state.result.is_ok());
        assert!(state.last_successful_result.is_some());
    }

    #[test]
    fn test_execute_caches_successful_result() {
        let json = r#"{"value": 42}"#;
        let mut state = QueryState::new(json.to_string());

        state.execute(".value");
        let cached = state.last_successful_result.clone();
        assert!(cached.is_some());

        // Execute invalid query (syntax error)
        state.execute(".[invalid syntax");
        assert!(state.result.is_err());

        // Last successful result should still be cached
        assert_eq!(state.last_successful_result, cached);
    }

    #[test]
    fn test_line_count_with_ok_result() {
        let json = r#"{"test": true}"#;
        let mut state = QueryState::new(json.to_string());

        let content: String = (0..50).map(|i| format!("line{}\n", i)).collect();
        state.result = Ok(content);

        assert_eq!(state.line_count(), 50);
    }

    #[test]
    fn test_line_count_uses_cached_on_error() {
        let json = r#"{"test": true}"#;
        let mut state = QueryState::new(json.to_string());

        let valid_result: String = (0..30).map(|i| format!("line{}\n", i)).collect();
        state.result = Ok(valid_result.clone());
        state.last_successful_result = Some(valid_result);

        // Now set an error
        state.result = Err("syntax error".to_string());

        // Should use cached result line count
        assert_eq!(state.line_count(), 30);
    }

    #[test]
    fn test_line_count_zero_on_error_without_cache() {
        let json = r#"{"test": true}"#;
        let mut state = QueryState::new(json.to_string());

        state.result = Err("error".to_string());
        state.last_successful_result = None;

        assert_eq!(state.line_count(), 0);
    }

    #[test]
    fn test_null_results_dont_overwrite_cache() {
        let json = r#"{"name": "test", "age": 30}"#;
        let mut state = QueryState::new(json.to_string());

        // Initial state: should have cached the root object
        let initial_cache = state.last_successful_result.clone();
        let initial_unformatted = state.last_successful_result_unformatted.clone();
        assert!(initial_cache.is_some());
        assert!(initial_unformatted.is_some());

        // Execute a query that returns null (like typing partial field ".s")
        state.execute(".nonexistent");
        assert!(state.result.is_ok());
        // jq returns "null" with ANSI codes, just check it contains "null"
        assert!(state.result.as_ref().unwrap().contains("null"));

        // Both caches should NOT be updated - should still have the root object
        assert_eq!(state.last_successful_result, initial_cache);
        assert_eq!(state.last_successful_result_unformatted, initial_unformatted);

        // Execute a valid query that returns data
        state.execute(".name");
        // jq returns with ANSI codes, just check it contains "test"
        assert!(state.result.as_ref().unwrap().contains("test"));

        // Both caches should now be updated
        assert_ne!(state.last_successful_result, initial_cache);
        assert_ne!(state.last_successful_result_unformatted, initial_unformatted);
        assert!(state.last_successful_result.as_ref().unwrap().contains("test"));

        // Unformatted should not have ANSI codes
        let unformatted = state.last_successful_result_unformatted.as_ref().unwrap();
        assert!(!unformatted.contains("\x1b"));
        assert!(unformatted.contains("test"));
    }

    #[test]
    fn test_strip_ansi_codes_simple() {
        let input = "\x1b[0m{\x1b[1;39m\"name\"\x1b[0m: \x1b[0;32m\"test\"\x1b[0m}";
        let output = QueryState::strip_ansi_codes(input);
        assert_eq!(output, r#"{"name": "test"}"#);
        assert!(!output.contains("\x1b"));
    }

    #[test]
    fn test_strip_ansi_codes_complex() {
        let input = "\x1b[1;39m{\x1b[0m\n  \x1b[0;34m\"key\"\x1b[0m: \x1b[0;32m\"value\"\x1b[0m\n\x1b[1;39m}\x1b[0m";
        let output = QueryState::strip_ansi_codes(input);
        assert!(output.contains(r#""key""#));
        assert!(output.contains(r#""value""#));
        assert!(!output.contains("\x1b"));
    }

    #[test]
    fn test_strip_ansi_codes_no_codes() {
        let input = r#"{"name": "plain"}"#;
        let output = QueryState::strip_ansi_codes(input);
        assert_eq!(output, input);
    }

    #[test]
    fn test_strip_ansi_codes_null_with_color() {
        let input = "\x1b[0;90mnull\x1b[0m";
        let output = QueryState::strip_ansi_codes(input);
        assert_eq!(output, "null");
    }

    #[test]
    fn test_unformatted_result_stored_on_execute() {
        let json = r#"{"name": "test"}"#;
        let mut state = QueryState::new(json.to_string());

        // Execute a query
        state.execute(".name");

        // Both formatted and unformatted should be cached
        assert!(state.last_successful_result.is_some());
        assert!(state.last_successful_result_unformatted.is_some());

        // Unformatted should not contain ANSI codes
        let unformatted = state.last_successful_result_unformatted.as_ref().unwrap();
        assert!(!unformatted.contains("\x1b"));
    }

    #[test]
    fn test_unformatted_result_handles_multiline_objects() {
        let json = r#"{"items": [{"id": 1, "name": "a"}, {"id": 2, "name": "b"}]}"#;
        let mut state = QueryState::new(json.to_string());

        // Execute query that returns multiple objects (pretty-printed)
        state.execute(".items[]");

        // Unformatted result should be parseable
        let unformatted = state.last_successful_result_unformatted.as_ref().unwrap();
        assert!(!unformatted.contains("\x1b"));

        // Should contain the field names
        assert!(unformatted.contains("id"));
        assert!(unformatted.contains("name"));
    }

    #[test]
    fn test_multiple_nulls_dont_overwrite_cache() {
        let json = r#"{"items": [{"id": 1}, {"id": 2}, {"id": 3}]}"#;
        let mut state = QueryState::new(json.to_string());

        // Execute query that returns multiple objects
        state.execute(".items[]");
        let cached_after_items = state.last_successful_result_unformatted.clone();
        assert!(cached_after_items.is_some());
        assert!(cached_after_items.as_ref().unwrap().contains("id"));

        // Execute query that returns multiple nulls (nonexistent field)
        state.execute(".items[].nonexistent");

        // Result should contain "null\n" repeated
        assert!(state.result.as_ref().unwrap().contains("null"));

        // Cache should NOT be updated - should still have the objects from .items[]
        assert_eq!(state.last_successful_result_unformatted, cached_after_items);
    }

    // ============================================================================
    // ResultType Detection Tests
    // ============================================================================

    #[test]
    fn test_detect_array_of_objects() {
        let result = r#"[{"id": 1, "name": "a"}, {"id": 2, "name": "b"}]"#;
        assert_eq!(QueryState::detect_result_type(result), ResultType::ArrayOfObjects);
    }

    #[test]
    fn test_detect_empty_array() {
        let result = "[]";
        assert_eq!(QueryState::detect_result_type(result), ResultType::Array);
    }

    #[test]
    fn test_detect_array_of_primitives() {
        let result = "[1, 2, 3, 4, 5]";
        assert_eq!(QueryState::detect_result_type(result), ResultType::Array);

        let result = r#"["a", "b", "c"]"#;
        assert_eq!(QueryState::detect_result_type(result), ResultType::Array);

        let result = "[true, false, true]";
        assert_eq!(QueryState::detect_result_type(result), ResultType::Array);
    }

    #[test]
    fn test_detect_destructured_objects() {
        // Multiple objects on separate lines (from .[] iteration)
        let result = r#"{"id": 1, "name": "a"}
{"id": 2, "name": "b"}
{"id": 3, "name": "c"}"#;
        assert_eq!(QueryState::detect_result_type(result), ResultType::DestructuredObjects);
    }

    #[test]
    fn test_detect_destructured_objects_pretty_printed() {
        // Pretty-printed multi-value output
        let result = r#"{
  "id": 1,
  "name": "a"
}
{
  "id": 2,
  "name": "b"
}"#;
        assert_eq!(QueryState::detect_result_type(result), ResultType::DestructuredObjects);
    }

    #[test]
    fn test_detect_single_object() {
        let result = r#"{"name": "test", "age": 30}"#;
        assert_eq!(QueryState::detect_result_type(result), ResultType::Object);
    }

    #[test]
    fn test_detect_single_object_pretty_printed() {
        let result = r#"{
  "name": "test",
  "age": 30
}"#;
        assert_eq!(QueryState::detect_result_type(result), ResultType::Object);
    }

    #[test]
    fn test_detect_string() {
        let result = r#""hello world""#;
        assert_eq!(QueryState::detect_result_type(result), ResultType::String);
    }

    #[test]
    fn test_detect_number() {
        let result = "42";
        assert_eq!(QueryState::detect_result_type(result), ResultType::Number);

        let result = "3.14159";
        assert_eq!(QueryState::detect_result_type(result), ResultType::Number);

        let result = "-100";
        assert_eq!(QueryState::detect_result_type(result), ResultType::Number);
    }

    #[test]
    fn test_detect_boolean() {
        let result = "true";
        assert_eq!(QueryState::detect_result_type(result), ResultType::Boolean);

        let result = "false";
        assert_eq!(QueryState::detect_result_type(result), ResultType::Boolean);
    }

    #[test]
    fn test_detect_null() {
        let result = "null";
        assert_eq!(QueryState::detect_result_type(result), ResultType::Null);
    }

    #[test]
    fn test_detect_invalid_json_returns_null() {
        let result = "not valid json";
        assert_eq!(QueryState::detect_result_type(result), ResultType::Null);
    }

    #[test]
    fn test_detect_multiple_primitives() {
        // Multiple strings (destructured)
        let result = r#""value1"
"value2"
"value3""#;
        // First value is string, has multiple values, but not objects
        // So it's just String (we don't have "DestructuredStrings")
        assert_eq!(QueryState::detect_result_type(result), ResultType::String);
    }

    // ============================================================================
    // CharType Classification Tests
    // ============================================================================

    #[test]
    fn test_classify_pipe_operator() {
        assert_eq!(QueryState::classify_char(Some('|')), CharType::PipeOperator);
    }

    #[test]
    fn test_classify_semicolon() {
        assert_eq!(QueryState::classify_char(Some(';')), CharType::Semicolon);
    }

    #[test]
    fn test_classify_comma() {
        assert_eq!(QueryState::classify_char(Some(',')), CharType::Comma);
    }

    #[test]
    fn test_classify_colon() {
        assert_eq!(QueryState::classify_char(Some(':')), CharType::Colon);
    }

    #[test]
    fn test_classify_open_paren() {
        assert_eq!(QueryState::classify_char(Some('(')), CharType::OpenParen);
    }

    #[test]
    fn test_classify_close_paren() {
        assert_eq!(QueryState::classify_char(Some(')')), CharType::CloseParen);
    }

    #[test]
    fn test_classify_open_bracket() {
        assert_eq!(QueryState::classify_char(Some('[')), CharType::OpenBracket);
    }

    #[test]
    fn test_classify_close_bracket() {
        assert_eq!(QueryState::classify_char(Some(']')), CharType::CloseBracket);
    }

    #[test]
    fn test_classify_open_brace() {
        assert_eq!(QueryState::classify_char(Some('{')), CharType::OpenBrace);
    }

    #[test]
    fn test_classify_close_brace() {
        assert_eq!(QueryState::classify_char(Some('}')), CharType::CloseBrace);
    }

    #[test]
    fn test_classify_question_mark() {
        assert_eq!(QueryState::classify_char(Some('?')), CharType::QuestionMark);
    }

    #[test]
    fn test_classify_dot() {
        assert_eq!(QueryState::classify_char(Some('.')), CharType::Dot);
    }

    #[test]
    fn test_classify_no_op_characters() {
        // Regular identifier characters
        assert_eq!(QueryState::classify_char(Some('a')), CharType::NoOp);
        assert_eq!(QueryState::classify_char(Some('Z')), CharType::NoOp);
        assert_eq!(QueryState::classify_char(Some('5')), CharType::NoOp);
        assert_eq!(QueryState::classify_char(Some('_')), CharType::NoOp);
    }

    #[test]
    fn test_classify_none() {
        // None (at start of query)
        assert_eq!(QueryState::classify_char(None), CharType::NoOp);
    }

    // ============================================================================
    // Base Query Normalization Tests
    // ============================================================================

    #[test]
    fn test_normalize_strips_pipe_with_identity() {
        // ".services | ." should strip " | ."
        assert_eq!(QueryState::normalize_base_query(".services | ."), ".services");
        assert_eq!(QueryState::normalize_base_query(".items[] | ."), ".items[]");
    }

    #[test]
    fn test_normalize_strips_incomplete_pipe() {
        // Trailing " | " without operand
        assert_eq!(QueryState::normalize_base_query(".services |"), ".services");
        assert_eq!(QueryState::normalize_base_query(".items[] | "), ".items[]");
    }

    #[test]
    fn test_normalize_strips_trailing_dot() {
        // Trailing dot (incomplete field access)
        assert_eq!(QueryState::normalize_base_query(".services."), ".services");
        assert_eq!(QueryState::normalize_base_query(".services[]."), ".services[]");
        assert_eq!(QueryState::normalize_base_query(".user.profile."), ".user.profile");
    }

    #[test]
    fn test_normalize_strips_trailing_whitespace() {
        // Trailing whitespace
        assert_eq!(QueryState::normalize_base_query(".services "), ".services");
        assert_eq!(QueryState::normalize_base_query(".services  "), ".services");
        assert_eq!(QueryState::normalize_base_query(".services\t"), ".services");
    }

    #[test]
    fn test_normalize_preserves_root() {
        // Root query "." should be preserved
        assert_eq!(QueryState::normalize_base_query("."), ".");
    }

    #[test]
    fn test_normalize_preserves_complete_queries() {
        // Complete queries should not be modified
        assert_eq!(QueryState::normalize_base_query(".services"), ".services");
        assert_eq!(QueryState::normalize_base_query(".services[]"), ".services[]");
        assert_eq!(QueryState::normalize_base_query(".user.name"), ".user.name");
    }

    #[test]
    fn test_normalize_handles_complex_patterns() {
        // Complex incomplete patterns
        assert_eq!(QueryState::normalize_base_query(".a | .b | ."), ".a | .b");
        assert_eq!(QueryState::normalize_base_query(".services[].config | ."), ".services[].config");
    }
}
