use std::io::Write;
use std::process::{Command, Stdio};

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
}

#[cfg(test)]
#[path = "executor_tests.rs"]
mod executor_tests;
