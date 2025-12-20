use std::io::Write;
use std::process::{Command, Stdio};
use std::thread::sleep;
use std::time::Duration;

use tokio_util::sync::CancellationToken;

use crate::query::worker::types::QueryError;

/// Execute jq queries against JSON input
pub struct JqExecutor {
    json_input: String,
}

impl JqExecutor {
    /// Create a new JQ executor with JSON input
    pub fn new(json_input: String) -> Self {
        Self { json_input }
    }

    /// Get a reference to the JSON input
    pub fn json_input(&self) -> &str {
        &self.json_input
    }

    /// Execute a jq query and return results or error
    ///
    /// # Arguments
    /// * `query` - The jq filter expression (e.g., ".items[]")
    ///
    /// # Returns
    /// * `Ok(String)` - Filtered JSON output with colors preserved
    /// * `Err(String)` - jq error message
    pub fn execute(&self, query: &str) -> Result<String, String> {
        // Empty query defaults to identity filter
        let query = if query.trim().is_empty() { "." } else { query };

        // Spawn jq process with color output
        let mut child = Command::new("jq")
            .arg("--color-output")
            .arg(query)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to spawn jq: {}", e))?;

        // Write JSON to jq's stdin
        if let Some(mut stdin) = child.stdin.take() {
            stdin
                .write_all(self.json_input.as_bytes())
                .map_err(|e| format!("Failed to write to jq stdin: {}", e))?;
        }

        // Wait for jq to finish and capture output
        let output = child
            .wait_with_output()
            .map_err(|e| format!("Failed to read jq output: {}", e))?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).to_string())
        }
    }

    /// Execute a jq query with cancellation support
    ///
    /// Uses polling approach with try_wait() to check for cancellation
    /// and process completion. This avoids blocking the worker thread
    /// while still allowing cancellation.
    ///
    /// # Arguments
    /// * `query` - The jq filter expression
    /// * `cancel_token` - Token for cancelling execution
    ///
    /// # Returns
    /// * `Ok(String)` - Filtered JSON output with colors
    /// * `Err(QueryError)` - Error or cancellation
    pub fn execute_with_cancel(
        &self,
        query: &str,
        cancel_token: &CancellationToken,
    ) -> Result<String, QueryError> {
        // Empty query defaults to identity filter
        let query = if query.trim().is_empty() { "." } else { query };

        // Spawn jq process
        let mut child = Command::new("jq")
            .arg("--color-output")
            .arg(query)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| QueryError::SpawnFailed(e.to_string()))?;

        // Write JSON to jq's stdin
        if let Some(mut stdin) = child.stdin.take() {
            stdin
                .write_all(self.json_input.as_bytes())
                .map_err(|e| QueryError::StdinWriteFailed(e.to_string()))?;
        }

        // Poll for completion or cancellation
        const POLL_INTERVAL_MS: u64 = 10;
        loop {
            // Check cancellation first
            if cancel_token.is_cancelled() {
                let _ = child.kill();
                return Err(QueryError::Cancelled);
            }

            // Check if process finished
            match child
                .try_wait()
                .map_err(|e| QueryError::OutputReadFailed(e.to_string()))?
            {
                Some(status) => {
                    // Process finished - get output
                    let output = child
                        .wait_with_output()
                        .map_err(|e| QueryError::OutputReadFailed(e.to_string()))?;

                    if status.success() {
                        return Ok(String::from_utf8_lossy(&output.stdout).to_string());
                    } else {
                        return Err(QueryError::ExecutionFailed(
                            String::from_utf8_lossy(&output.stderr).to_string(),
                        ));
                    }
                }
                None => {
                    // Process still running - sleep briefly
                    sleep(Duration::from_millis(POLL_INTERVAL_MS));
                }
            }
        }
    }
}

#[cfg(test)]
#[path = "executor_tests.rs"]
mod executor_tests;
