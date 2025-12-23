//! Query Worker Types
//!
//! Type definitions for the query worker thread communication.
//! These types enable request/response pattern with cancellation support.

use std::sync::Arc;

use tokio_util::sync::CancellationToken;

use crate::query::query_state::ResultType;

/// Request to execute a jq query
#[derive(Debug)]
pub struct QueryRequest {
    /// The jq query to execute (e.g., ".items[]")
    pub query: String,
    /// Unique ID for tracking this request
    pub request_id: u64,
    /// Token for cancelling this request
    pub cancel_token: CancellationToken,
}

/// Pre-rendered span with style (Send-safe)
#[derive(Debug, Clone)]
pub struct RenderedSpan {
    pub content: String,
    pub style: ratatui::style::Style,
}

/// Pre-rendered line
#[derive(Debug, Clone)]
pub struct RenderedLine {
    pub spans: Vec<RenderedSpan>,
}

/// Fully processed query result with all caches computed
#[derive(Debug)]
pub struct ProcessedResult {
    pub output: Arc<String>,
    pub unformatted: Arc<String>,
    pub rendered_lines: Vec<RenderedLine>,
    pub parsed: Option<Arc<serde_json::Value>>,
    pub line_count: u32,
    pub max_width: u16,
    pub result_type: ResultType,
    pub query: String,
}

/// Response from query execution
#[derive(Debug)]
pub enum QueryResponse {
    /// Query execution succeeded
    Success {
        /// Output from jq with ANSI colors
        output: String,
        /// The query that produced this output (for updating base_query_for_suggestions)
        query: String,
        /// Request ID this response belongs to
        request_id: u64,
    },
    /// Fully processed result with all caches computed (new async path)
    ProcessedSuccess {
        /// Pre-processed result with all caches
        processed: ProcessedResult,
        /// Request ID this response belongs to
        request_id: u64,
    },
    /// Query execution failed
    Error {
        /// Error message from jq stderr
        message: String,
        /// The query that produced this error (for AI context)
        query: String,
        /// Request ID this response belongs to
        /// Note: request_id = 0 indicates a worker-level error (applies immediately)
        request_id: u64,
    },
    /// Query execution was cancelled
    Cancelled {
        /// Request ID that was cancelled
        request_id: u64,
    },
}

/// Error types for query execution
#[derive(Debug, Clone)]
pub enum QueryError {
    /// Failed to spawn jq process
    SpawnFailed(String),
    /// Failed to read jq output
    OutputReadFailed(String),
    /// Query execution was cancelled
    Cancelled,
    /// jq returned non-zero exit code
    ExecutionFailed(String),
}

impl std::fmt::Display for QueryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QueryError::SpawnFailed(e) => write!(f, "Failed to spawn jq: {}", e),
            QueryError::OutputReadFailed(e) => write!(f, "Failed to read jq output: {}", e),
            QueryError::Cancelled => write!(f, "Query execution cancelled"),
            QueryError::ExecutionFailed(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for QueryError {}

#[cfg(test)]
#[path = "types_tests.rs"]
mod types_tests;
