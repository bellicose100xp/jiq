use std::collections::HashSet;
use std::process::{Command, Stdio};
use std::sync::{Arc, OnceLock};
use std::thread::sleep;
use std::time::Duration;

use serde_json::Value;
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
    /// Lazily parsed JSON input, cached for autocomplete navigation.
    /// Uses OnceLock for thread-safe one-time initialization.
    json_input_parsed: OnceLock<Option<Arc<Value>>>,
    /// All unique field names from the JSON, collected recursively.
    /// Cached for non-deterministic autocomplete fallback.
    all_field_names: OnceLock<Arc<HashSet<String>>>,
}

impl JqExecutor {
    /// Create a new JQ executor with JSON input
    pub fn new(json_input: String) -> Self {
        Self {
            json_input: Arc::new(json_input),
            json_input_parsed: OnceLock::new(),
            all_field_names: OnceLock::new(),
        }
    }

    /// Get a reference to the JSON input
    pub fn json_input(&self) -> &str {
        &self.json_input
    }

    /// Get the parsed JSON input, lazily parsing on first access.
    ///
    /// Returns the original input JSON as a parsed Value, cached for repeated access.
    /// This is the true original file input that never changes during the session.
    /// Used by autocomplete to navigate nested structures.
    ///
    /// Returns `None` if the JSON input is invalid.
    pub fn json_input_parsed(&self) -> Option<Arc<Value>> {
        self.json_input_parsed
            .get_or_init(|| serde_json::from_str(&self.json_input).ok().map(Arc::new))
            .clone()
    }

    /// Get all unique field names from the JSON, collected recursively.
    ///
    /// Returns a cached set of all field names found anywhere in the JSON tree.
    /// Used for non-deterministic autocomplete fallback when path navigation fails.
    pub fn all_field_names(&self) -> Arc<HashSet<String>> {
        self.all_field_names
            .get_or_init(|| {
                let mut fields = HashSet::new();
                if let Some(parsed) = self.json_input_parsed() {
                    Self::collect_fields_recursive(&parsed, &mut fields);
                }
                Arc::new(fields)
            })
            .clone()
    }

    fn collect_fields_recursive(value: &Value, fields: &mut HashSet<String>) {
        match value {
            Value::Object(map) => {
                for (key, val) in map {
                    fields.insert(key.clone());
                    Self::collect_fields_recursive(val, fields);
                }
            }
            Value::Array(arr) => {
                if let Some(first) = arr.first() {
                    Self::collect_fields_recursive(first, fields);
                }
            }
            _ => {}
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

        // Galaxy theme colors for jq output (using true color ANSI codes)
        // Format: null:false:true:numbers:strings:arrays:objects:keys
        // Each segment is an ANSI SGR code (38;2;R;G;B for true color)
        let jq_colors = [
            "38;2;130;133;158",  // null - muted gray
            "38;2;224;108;117",  // false - soft red
            "38;2;107;203;119",  // true - fresh green
            "38;2;189;147;249",  // numbers - purple
            "38;2;107;203;119",  // strings - fresh green
            "1;38;2;0;217;255",  // arrays - bold electric cyan
            "1;38;2;0;217;255",  // objects - bold electric cyan
            "1;38;2;255;217;61", // keys - bold golden yellow
        ]
        .join(":");

        // Spawn jq process with custom colors
        let mut child = Command::new("jq")
            .env("JQ_COLORS", jq_colors)
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
