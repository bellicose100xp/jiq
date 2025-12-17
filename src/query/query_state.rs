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
    PipeOperator, // |
    Semicolon,    // ;
    Comma,        // ,
    Colon,        // :
    OpenParen,    // (
    OpenBracket,  // [
    OpenBrace,    // {
    CloseBracket, // ]
    CloseBrace,   // }
    CloseParen,   // )
    QuestionMark, // ?
    Dot,          // .
    NoOp,         // Regular identifier character
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
                    for c in chars.by_ref() {
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
            Err(_) => self
                .last_successful_result
                .as_ref()
                .map(|r| r.lines().count() as u32)
                .unwrap_or(0),
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
#[path = "query_state_tests.rs"]
mod query_state_tests;
