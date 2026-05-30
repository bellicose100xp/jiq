use std::collections::{HashMap, HashSet};
use std::process::{Command, Stdio};
use std::sync::{Arc, OnceLock};
use std::thread::sleep;
use std::time::Duration;

use serde_json::Value;
use tokio_util::sync::CancellationToken;

use crate::autocomplete::json_navigator::DEFAULT_ARRAY_SAMPLE_SIZE;
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
    /// All distinct string VALUES from the JSON, collected recursively,
    /// sorted by descending frequency (alphabetical tiebreaker), capped at
    /// `MAX_GLOBAL_STRING_VALUES`. Used as the last-resort fallback by
    /// string-value autocomplete.
    all_string_values: OnceLock<Arc<Vec<String>>>,
    array_sample_size: usize,
}

/// Cap on distinct values returned by `all_string_values`. Keeps the lazy
/// precompute bounded on pathological JSON (e.g. log dumps with millions of
/// unique IDs).
pub const MAX_GLOBAL_STRING_VALUES: usize = 10_000;

impl JqExecutor {
    /// Create a new JQ executor with JSON input and default sample size
    pub fn new(json_input: String) -> Self {
        Self::new_with_sample_size(json_input, DEFAULT_ARRAY_SAMPLE_SIZE)
    }

    /// Create a new JQ executor with JSON input and custom array sample size
    pub fn new_with_sample_size(json_input: String, array_sample_size: usize) -> Self {
        Self {
            json_input: Arc::new(json_input),
            json_input_parsed: OnceLock::new(),
            all_field_names: OnceLock::new(),
            all_string_values: OnceLock::new(),
            array_sample_size,
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
        let sample_size = self.array_sample_size;
        self.all_field_names
            .get_or_init(|| {
                let mut fields = HashSet::new();
                if let Some(parsed) = self.json_input_parsed() {
                    Self::collect_fields_recursive(&parsed, &mut fields, sample_size);
                }
                Arc::new(fields)
            })
            .clone()
    }

    fn collect_fields_recursive(
        value: &Value,
        fields: &mut HashSet<String>,
        array_sample_size: usize,
    ) {
        match value {
            Value::Object(map) => {
                for (key, val) in map {
                    fields.insert(key.clone());
                    Self::collect_fields_recursive(val, fields, array_sample_size);
                }
            }
            Value::Array(arr) => {
                for element in arr.iter().take(array_sample_size) {
                    Self::collect_fields_recursive(element, fields, array_sample_size);
                }
            }
            _ => {}
        }
    }

    /// Get all distinct string VALUES from the JSON, sorted by descending
    /// frequency (alphabetical tiebreaker), capped at `MAX_GLOBAL_STRING_VALUES`.
    ///
    /// Lazy precompute via `OnceLock`. The first caller pays the walk cost
    /// once per session; subsequent callers get an `Arc::clone`. Mirrors
    /// `all_field_names()` but for string values rather than keys.
    pub fn all_string_values(&self) -> Arc<Vec<String>> {
        self.all_string_values
            .get_or_init(|| {
                let mut counts: HashMap<String, u32> = HashMap::new();
                if let Some(parsed) = self.json_input_parsed() {
                    Self::collect_string_values_recursive(&parsed, &mut counts);
                }
                Arc::new(sort_and_cap_strings(counts, MAX_GLOBAL_STRING_VALUES))
            })
            .clone()
    }

    fn collect_string_values_recursive(value: &Value, counts: &mut HashMap<String, u32>) {
        // Iterative DFS to avoid stack overflow on deeply nested JSON.
        let mut stack: Vec<&Value> = vec![value];
        while let Some(node) = stack.pop() {
            if counts.len() >= MAX_GLOBAL_STRING_VALUES {
                // Don't add NEW distinct values past the cap, but keep walking
                // so we accumulate frequency for already-seen strings. (Walking
                // unbounded JSON for this is intentional — caller is gated by
                // OnceLock so this only ever runs once.)
            }
            match node {
                Value::String(s) => {
                    if let Some(c) = counts.get_mut(s.as_str()) {
                        *c += 1;
                    } else if counts.len() < MAX_GLOBAL_STRING_VALUES {
                        counts.insert(s.clone(), 1);
                    }
                }
                Value::Array(arr) => {
                    for element in arr {
                        stack.push(element);
                    }
                }
                Value::Object(map) => {
                    for (_, v) in map {
                        stack.push(v);
                    }
                }
                _ => {}
            }
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

        let _t = crate::Timer::new("jq query");

        // Empty query defaults to identity filter
        let query = if query.trim().is_empty() { "." } else { query };
        log::debug!(
            "jq query: {:?} (input: {} bytes)",
            query,
            self.json_input.len()
        );

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
        let poll_start = std::time::Instant::now();
        let mut slow_warned = false;
        let status = loop {
            // Check cancellation first
            if cancel_token.is_cancelled() {
                log::debug!("jq process killed due to cancellation");
                let _ = child.kill();
                return Err(QueryError::Cancelled);
            }

            // Warn once if jq is taking a long time
            if !slow_warned && poll_start.elapsed() > Duration::from_secs(5) {
                log::warn!("jq process still running after 5s for query {:?}", query);
                slow_warned = true;
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
            log::debug!("jq succeeded: {} bytes output", stdout_data.len());
            Ok(String::from_utf8_lossy(&stdout_data).to_string())
        } else {
            let stderr_str = String::from_utf8_lossy(&stderr_data).to_string();
            log::debug!("jq failed (exit {:?}): {}", status.code(), stderr_str);
            Err(QueryError::ExecutionFailed(stderr_str))
        }
    }
}

fn sort_and_cap_strings(counts: HashMap<String, u32>, cap: usize) -> Vec<String> {
    let mut entries: Vec<(String, u32)> = counts.into_iter().collect();
    entries.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    entries.truncate(cap);
    entries.into_iter().map(|(s, _)| s).collect()
}

#[cfg(test)]
#[path = "executor_tests.rs"]
mod executor_tests;
