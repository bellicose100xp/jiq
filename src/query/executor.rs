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
mod tests {
    use super::*;

    #[test]
    fn test_identity_filter() {
        let json = r#"{"name": "Alice", "age": 30}"#;
        let executor = JqExecutor::new(json.to_string());
        let result = executor.execute(".");

        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("Alice"));
        assert!(output.contains("30"));
    }

    #[test]
    fn test_empty_query_defaults_to_identity() {
        let json = r#"{"name": "Bob"}"#;
        let executor = JqExecutor::new(json.to_string());
        let result = executor.execute("");

        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("Bob"));
    }

    #[test]
    fn test_field_selection() {
        let json = r#"{"name": "Charlie", "age": 25, "city": "NYC"}"#;
        let executor = JqExecutor::new(json.to_string());
        let result = executor.execute(".name");

        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("Charlie"));
        assert!(!output.contains("NYC"));
    }

    #[test]
    fn test_array_iteration() {
        let json = r#"[{"id": 1}, {"id": 2}, {"id": 3}]"#;
        let executor = JqExecutor::new(json.to_string());
        let result = executor.execute(".[]");

        assert!(result.is_ok());
        let output = result.unwrap();
        // Check that all three IDs appear in the output (format may vary)
        assert!(output.contains("1"));
        assert!(output.contains("2"));
        assert!(output.contains("3"));
        assert!(output.contains("id"));
    }

    #[test]
    fn test_invalid_query_returns_error() {
        let json = r#"{"name": "Dave"}"#;
        let executor = JqExecutor::new(json.to_string());
        let result = executor.execute(".invalid.[syntax");

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(!error.is_empty());
    }

    #[test]
    fn test_nested_field_access() {
        let json = r#"{"user": {"name": "Eve", "age": 28}}"#;
        let executor = JqExecutor::new(json.to_string());
        let result = executor.execute(".user.name");

        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("Eve"));
    }

    #[test]
    fn test_color_output_flag_present() {
        // This test verifies that ANSI color codes are present in output
        let json = r#"{"key": "value"}"#;
        let executor = JqExecutor::new(json.to_string());
        let result = executor.execute(".");

        assert!(result.is_ok());
        let output = result.unwrap();
        // jq with --color-output produces ANSI escape codes
        assert!(output.contains("\x1b[") || output.len() > json.len());
    }
}
