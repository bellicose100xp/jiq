use std::sync::Arc;
use std::sync::mpsc::{Receiver, Sender, channel};
use tokio_util::sync::CancellationToken;

use ansi_to_tui::IntoText;
use ratatui::text::{Line, Span, Text};

use crate::query::executor::JqExecutor;
use crate::query::worker::preprocess::strip_ansi_codes;
use crate::query::worker::types::RenderedLine;
use crate::query::worker::{QueryRequest, QueryResponse, spawn_worker};
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
    /// Cached last successful result with ANSI colors (for rendering on error)
    /// Uses Arc to make cloning cheap - autocomplete clones this on every keystroke!
    pub last_successful_result: Option<Arc<String>>,
    /// Unformatted result without ANSI codes (for autosuggestion analysis)
    /// Uses Arc to make cloning cheap - autocomplete clones this on every keystroke!
    pub last_successful_result_unformatted: Option<Arc<String>>,
    /// Parsed JSON value of last successful result (for autocomplete field extraction)
    /// Uses Arc to avoid re-parsing large files on every keystroke!
    /// This is THE critical optimization for large files.
    pub last_successful_result_parsed: Option<Arc<Value>>,
    /// Pre-rendered Text<'static> for display
    /// Avoids expensive into_text() conversion in render loop (~10x/sec)
    pub last_successful_result_rendered: Option<Text<'static>>,
    /// Cached processed result for AI context (minified/truncated)
    /// Updated only when last_successful_result_unformatted changes
    pub last_successful_result_for_context: Option<Arc<String>>,
    /// Base query that produced the last successful result (for suggestions)
    pub base_query_for_suggestions: Option<String>,
    /// Type of the last successful result (for type-aware suggestions)
    pub base_type_for_suggestions: Option<ResultType>,
    /// Cached line count (computed once per result, not per render)
    pub(crate) cached_line_count: u32,
    /// Cached max line width (computed once per result, not per render)
    pub(crate) cached_max_line_width: u16,
    /// Whether current result is null/empty (valid query but no results)
    pub is_empty_result: bool,

    // Async execution support
    /// Channel to send query requests to worker
    request_tx: Option<Sender<QueryRequest>>,
    /// Channel to receive query responses from worker
    response_rx: Option<Receiver<QueryResponse>>,
    /// Current request ID counter (starts at 1, 0 reserved for worker errors)
    next_request_id: u64,
    /// ID of currently in-flight request, if any
    in_flight_request_id: Option<u64>,
    /// Cancellation token for current request
    current_cancel_token: Option<CancellationToken>,
}

impl QueryState {
    /// Create a new QueryState with the given JSON input
    ///
    /// Spawns a background worker thread for async query execution.
    pub fn new(json_input: String) -> Self {
        let executor = JqExecutor::new(json_input.clone());
        let cancel_token = CancellationToken::new();
        let result = executor
            .execute_with_cancel(".", &cancel_token)
            .map_err(|e| e.to_string());
        let last_successful_result = result.as_ref().ok().map(|s| Arc::new(s.clone()));
        let last_successful_result_unformatted = last_successful_result
            .as_ref()
            .map(|s| Arc::new(strip_ansi_codes(s)));

        // Pre-process for AI context
        let last_successful_result_for_context =
            last_successful_result_unformatted.as_ref().map(|s| {
                Arc::new(crate::ai::context::prepare_json_for_context(
                    s,
                    crate::ai::context::MAX_JSON_SAMPLE_LENGTH,
                ))
            });

        let base_query_for_suggestions = Some(".".to_string());
        let base_type_for_suggestions = last_successful_result_unformatted
            .as_ref()
            .map(|s| Self::detect_result_type(s));

        // Avoid re-parsing on every keystroke
        let last_successful_result_parsed = last_successful_result_unformatted
            .as_ref()
            .and_then(|s| Self::parse_first_value(s))
            .map(Arc::new);

        // Pre-render result to avoid expensive conversion in render loop
        let last_successful_result_rendered = last_successful_result.clone().map(|s| {
            s.as_bytes()
                .to_vec()
                .into_text()
                .unwrap_or_else(|_| Text::raw(s.to_string()))
        });

        // Cache line count and max width for initial result
        let (cached_line_count, cached_max_line_width) = last_successful_result
            .as_ref()
            .map(|s| {
                let line_count = s.lines().count() as u32;
                let max_width = s
                    .lines()
                    .map(|l| l.len())
                    .max()
                    .unwrap_or(0)
                    .min(u16::MAX as usize) as u16;
                (line_count, max_width)
            })
            .unwrap_or((0, 0));

        let (request_tx, request_rx) = channel();
        let (response_tx, response_rx) = channel();

        spawn_worker(json_input, request_rx, response_tx);

        Self {
            executor,
            result,
            last_successful_result,
            last_successful_result_unformatted,
            last_successful_result_parsed,
            last_successful_result_rendered,
            last_successful_result_for_context,
            base_query_for_suggestions,
            base_type_for_suggestions,
            cached_line_count,
            cached_max_line_width,
            is_empty_result: false,
            request_tx: Some(request_tx),
            response_rx: Some(response_rx),
            next_request_id: 1, // Reserve 0 for worker errors
            in_flight_request_id: None,
            current_cancel_token: None,
        }
    }

    /// Execute a query and update results
    /// Only caches non-null results for autosuggestions
    pub fn execute(&mut self, query: &str) {
        let cancel_token = CancellationToken::new();
        self.result = self
            .executor
            .execute_with_cancel(query, &cancel_token)
            .map_err(|e| e.to_string());
        if let Ok(result) = &self.result {
            self.update_successful_result(result.clone(), query);
        }
    }

    /// Update cached results for autosuggestions
    ///
    /// Only caches non-null results to avoid polluting suggestions with partial queries.
    fn update_successful_result(&mut self, output: String, query: &str) {
        // Partial queries like ".s" return "null"; keep last meaningful result for suggestions
        let unformatted = strip_ansi_codes(&output);

        let is_only_nulls = unformatted
            .lines()
            .filter(|line| !line.trim().is_empty())
            .all(|line| line.trim() == "null");

        self.is_empty_result = is_only_nulls;

        if !is_only_nulls {
            // Cache line count and max width BEFORE moving output
            let cached_line_count = output.lines().count() as u32;
            let cached_max_line_width = output
                .lines()
                .map(|l| l.len())
                .max()
                .unwrap_or(0)
                .min(u16::MAX as usize) as u16;

            // Pre-render result BEFORE moving output into Arc
            // This avoids expensive into_text() conversion in render loop
            let rendered = output
                .as_bytes()
                .to_vec()
                .into_text()
                .unwrap_or_else(|_| Text::raw(output.clone()));

            self.last_successful_result_rendered = Some(rendered);
            self.last_successful_result = Some(Arc::new(output));
            self.last_successful_result_unformatted = Some(Arc::new(unformatted.clone()));

            // Pre-process for AI context (minified/truncated)
            self.last_successful_result_for_context =
                Some(Arc::new(crate::ai::context::prepare_json_for_context(
                    &unformatted,
                    crate::ai::context::MAX_JSON_SAMPLE_LENGTH,
                )));

            // Critical: prevents re-parsing large files on EVERY keystroke
            self.last_successful_result_parsed =
                Self::parse_first_value(&unformatted).map(Arc::new);

            // Trim trailing whitespace/incomplete operators: ".services | ." → ".services"
            let base_query = Self::normalize_base_query(query);
            self.base_query_for_suggestions = Some(base_query);
            self.base_type_for_suggestions = Some(Self::detect_result_type(&unformatted));

            self.cached_line_count = cached_line_count;
            self.cached_max_line_width = cached_max_line_width;
        }
    }

    /// Execute query asynchronously
    ///
    /// Sends query to worker thread and returns immediately.
    /// Call poll_response() in main event loop to get results.
    ///
    /// Automatically cancels any in-flight request before starting new one.
    pub fn execute_async(&mut self, query: &str) {
        // Cancel any existing request
        self.cancel_in_flight();

        // Allocate new request ID
        let request_id = self.next_request_id;
        self.next_request_id = self.next_request_id.wrapping_add(1);

        // Skip 0 on wrap (reserved for worker errors)
        if self.next_request_id == 0 {
            self.next_request_id = 1;
        }

        // Create cancellation token
        let cancel_token = CancellationToken::new();
        self.current_cancel_token = Some(cancel_token.clone());
        self.in_flight_request_id = Some(request_id);

        // Send request to worker
        if let Some(ref tx) = self.request_tx {
            let request = QueryRequest {
                query: query.to_string(),
                request_id,
                cancel_token,
            };

            // If send fails, worker died - clear channels
            if tx.send(request).is_err() {
                log::error!("Query worker disconnected - send failed");
                self.request_tx = None;
                self.response_rx = None;
                self.in_flight_request_id = None;
                self.current_cancel_token = None;
                self.result = Err("Query worker disconnected".to_string());
            }
        } else {
            log::error!("No request channel available");
        }
    }

    /// Cancel in-flight request if any
    pub fn cancel_in_flight(&mut self) {
        if let Some(token) = self.current_cancel_token.take() {
            token.cancel();
        }
        self.in_flight_request_id = None;
    }

    /// Poll for query responses (non-blocking)
    ///
    /// Call this in main event loop to check for completed queries.
    /// Returns the query that produced the last completed result (for AI context), or None if no update.
    pub fn poll_response(&mut self) -> Option<String> {
        let mut completed_query: Option<String> = None;

        // Take the receiver temporarily to avoid borrow checker issues
        let rx = self.response_rx.take()?;

        // Process all available responses
        // Keep last completed query (if multiple responses, most recent wins)
        loop {
            match rx.try_recv() {
                Ok(response) => {
                    if let Some(query) = self.process_response(response) {
                        completed_query = Some(query);
                    }
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {
                    // Put receiver back and break
                    self.response_rx = Some(rx);
                    break;
                }
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    log::error!("Query worker disconnected in poll_response");
                    self.request_tx = None;
                    if self.in_flight_request_id.is_some() {
                        self.result = Err("Query worker disconnected".to_string());
                        self.in_flight_request_id = None;
                        self.current_cancel_token = None;
                        completed_query = Some(String::new());
                    }
                    // Don't put receiver back - it's disconnected
                    break;
                }
            }
        }

        completed_query
    }

    /// Process a single response
    ///
    /// Returns the query that produced this result (for AI context updates).
    /// Returns None if response was stale/cancelled (no state change).
    fn process_response(&mut self, response: QueryResponse) -> Option<String> {
        let current_request_id = self.in_flight_request_id;

        match response {
            QueryResponse::ProcessedSuccess {
                processed,
                request_id,
            } => {
                // Ignore stale responses
                if Some(request_id) != current_request_id {
                    return None;
                }

                // Check if result is only nulls (same logic as sync path)
                let is_only_nulls = processed
                    .unformatted
                    .lines()
                    .filter(|line| !line.trim().is_empty())
                    .all(|line| line.trim() == "null");

                self.is_empty_result = is_only_nulls;

                // Clear in-flight tracking immediately
                self.in_flight_request_id = None;
                self.current_cancel_token = None;

                // Only update cache if result is not null (same as sync path)
                if !is_only_nulls {
                    // Convert rendered lines to Text (fast - just allocations)
                    let rendered = Self::rendered_lines_to_text(processed.rendered_lines);

                    // Update result and all caches
                    self.result = Ok(processed.output.as_ref().clone());
                    self.last_successful_result = Some(processed.output);
                    self.last_successful_result_unformatted = Some(processed.unformatted.clone());
                    self.last_successful_result_rendered = Some(rendered);
                    self.last_successful_result_parsed = processed.parsed;
                    // Pre-process for AI context
                    self.last_successful_result_for_context =
                        Some(Arc::new(crate::ai::context::prepare_json_for_context(
                            &processed.unformatted,
                            crate::ai::context::MAX_JSON_SAMPLE_LENGTH,
                        )));
                    self.cached_line_count = processed.line_count;
                    self.cached_max_line_width = processed.max_width;
                    self.base_query_for_suggestions = Some(processed.query.clone());
                    self.base_type_for_suggestions = Some(processed.result_type);
                } else {
                    // Null result - preserve ALL cache including rendered output
                    // Only update self.result so it shows as "null" in error state
                    self.result = Ok(processed.output.as_ref().clone());
                }

                Some(processed.query)
            }
            QueryResponse::Error {
                message,
                query,
                request_id,
            } => {
                // Worker-level errors (request_id == 0) always apply
                // Request-level errors only apply if they match current request
                if request_id == 0 || Some(request_id) == current_request_id {
                    self.in_flight_request_id = None;
                    self.current_cancel_token = None;
                    self.result = Err(message);
                    self.is_empty_result = false;
                    // Return the query that produced this error for AI context
                    return Some(query);
                }

                None
            }
            QueryResponse::Cancelled { request_id } => {
                // Only clear in-flight if it matches
                if Some(request_id) == current_request_id {
                    self.in_flight_request_id = None;
                    self.current_cancel_token = None;
                }
                None
            }
        }
    }

    /// Check if a query is currently pending
    pub fn is_pending(&self) -> bool {
        self.in_flight_request_id.is_some()
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

    /// Parse first JSON value from result text
    ///
    /// Handles both single values and destructured output (multiple JSON values).
    /// For destructured results like `{"a":1}\n{"b":2}`, parses just the first value.
    fn parse_first_value(text: &str) -> Option<Value> {
        let text = text.trim();
        if text.is_empty() {
            return None;
        }

        // Try to parse the entire text first (common case: single value)
        if let Ok(value) = serde_json::from_str(text) {
            return Some(value);
        }

        // Fallback: use streaming parser to get first value (handles destructured output)
        let mut deserializer = serde_json::Deserializer::from_str(text).into_iter();
        deserializer.next().and_then(|r| r.ok())
    }

    /// Convert pre-rendered lines to Text for display
    ///
    /// Takes Vec<RenderedLine> from worker and converts to ratatui Text.
    /// This is a simple allocation operation - much faster than ANSI parsing.
    fn rendered_lines_to_text(lines: Vec<RenderedLine>) -> Text<'static> {
        Text::from(
            lines
                .into_iter()
                .map(|line| {
                    Line::from(
                        line.spans
                            .into_iter()
                            .map(|s| Span::styled(s.content, s.style))
                            .collect::<Vec<_>>(),
                    )
                })
                .collect::<Vec<_>>(),
        )
    }

    /// Get the total number of lines in the current results
    /// Note: Returns u32 to handle large files (>65K lines) correctly
    /// Always uses cached value computed when result changes
    pub fn line_count(&self) -> u32 {
        self.cached_line_count
    }

    /// Get the maximum line width in the current results (for horizontal scrolling)
    /// Always uses cached value computed when result changes
    pub fn max_line_width(&self) -> u16 {
        self.cached_max_line_width
    }
}

#[cfg(test)]
#[path = "query_state_tests.rs"]
mod query_state_tests;
