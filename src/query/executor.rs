use std::io::Write;
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;

use tokio_util::sync::CancellationToken;

use crate::query::worker::types::QueryError;

/// Execute jq queries against JSON input
///
/// Uses Arc<String> to enable cheap cloning when spawning worker threads.
/// Without Arc, each query execution would copy the entire JSON input (O(n)),
/// causing typing lag on large files. With Arc, cloning is just a reference
/// count increment (O(1)).
pub struct JqExecutor {
    json_input: Arc<String>,
}

impl JqExecutor {
    /// Create a new JQ executor with JSON input
    pub fn new(json_input: String) -> Self {
        Self {
            json_input: Arc::new(json_input),
        }
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
        use std::io::Read;
        use std::sync::mpsc::channel;

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

        // Spawn thread to write JSON to stdin
        // This prevents deadlock if JSON is large (>64KB) and jq is slow to read
        // Arc::clone is O(1) - just increments reference count, no data copying
        let json_input = Arc::clone(&self.json_input);
        if let Some(stdin) = child.stdin.take() {
            std::thread::spawn(move || {
                use std::io::Write;
                let mut stdin = stdin;
                let _ = stdin.write_all(json_input.as_bytes());
                // stdin is dropped here, closing the pipe
            });
        }

        // Spawn threads to read stdout/stderr concurrently
        // This prevents pipe buffer deadlock on large outputs
        let (stdout_tx, stdout_rx) = channel();
        let (stderr_tx, stderr_rx) = channel();

        if let Some(mut stdout) = child.stdout.take() {
            std::thread::spawn(move || {
                let mut buffer = Vec::new();
                let _ = stdout.read_to_end(&mut buffer);
                let _ = stdout_tx.send(buffer);
            });
        }

        if let Some(mut stderr) = child.stderr.take() {
            std::thread::spawn(move || {
                let mut buffer = Vec::new();
                let _ = stderr.read_to_end(&mut buffer);
                let _ = stderr_tx.send(buffer);
            });
        }

        // Poll for completion or cancellation
        const POLL_INTERVAL_MS: u64 = 10;
        let status = loop {
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
                Some(s) => break s,
                None => {
                    // Process still running - sleep briefly
                    sleep(Duration::from_millis(POLL_INTERVAL_MS));
                }
            }
        };

        // Process has exited - collect output from reader threads
        let stdout_data = stdout_rx
            .recv()
            .map_err(|_| QueryError::OutputReadFailed("Failed to read stdout".to_string()))?;
        let stderr_data = stderr_rx
            .recv()
            .map_err(|_| QueryError::OutputReadFailed("Failed to read stderr".to_string()))?;

        if status.success() {
            Ok(String::from_utf8_lossy(&stdout_data).to_string())
        } else {
            Err(QueryError::ExecutionFailed(
                String::from_utf8_lossy(&stderr_data).to_string(),
            ))
        }
    }
}

#[cfg(test)]
#[path = "executor_tests.rs"]
mod executor_tests;
