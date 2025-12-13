use crate::error::JiqError;
use serde_json::Value;
use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

/// Read JSON from stdin or a file
pub struct InputReader;

impl InputReader {
    /// Read JSON from stdin or file path
    ///
    /// # Arguments
    /// * `path` - Optional file path. If None, reads from stdin.
    ///
    /// # Returns
    /// * `Ok(String)` - Valid JSON string
    /// * `Err(JiqError)` - If JSON is invalid or IO error occurs
    pub fn read_json(path: Option<&Path>) -> Result<String, JiqError> {
        let json_str = match path {
            Some(file_path) => {
                // Read from file
                let mut file = File::open(file_path)?;
                let mut contents = String::new();
                file.read_to_string(&mut contents)?;
                contents
            }
            None => {
                // Read from stdin
                let mut buffer = String::new();
                io::stdin().read_to_string(&mut buffer)?;
                buffer
            }
        };

        // Validate JSON syntax
        serde_json::from_str::<Value>(&json_str)
            .map_err(|e| JiqError::InvalidJson(e.to_string()))?;

        Ok(json_str)
    }

    /// Read and validate JSON from a string (used for testing)
    #[cfg(test)]
    fn read_json_from_string(json_str: &str) -> Result<String, JiqError> {
        // Validate JSON syntax
        serde_json::from_str::<Value>(json_str)
            .map_err(|e| JiqError::InvalidJson(e.to_string()))?;

        Ok(json_str.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_read_valid_json_from_file() {
        let path = PathBuf::from("tests/fixtures/simple.json");
        let result = InputReader::read_json(Some(&path));

        assert!(result.is_ok());
        let json = result.unwrap();
        assert!(json.contains("Alice"));
        assert!(json.contains("Seattle"));
    }

    #[test]
    fn test_read_array_json_from_file() {
        let path = PathBuf::from("tests/fixtures/array.json");
        let result = InputReader::read_json(Some(&path));

        assert!(result.is_ok());
        let json = result.unwrap();
        assert!(json.contains("Alice"));
        assert!(json.contains("Bob"));
        assert!(json.contains("Charlie"));
    }

    #[test]
    fn test_read_nested_json_from_file() {
        let path = PathBuf::from("tests/fixtures/nested.json");
        let result = InputReader::read_json(Some(&path));

        assert!(result.is_ok());
        let json = result.unwrap();
        assert!(json.contains("TechCorp"));
        assert!(json.contains("engineering"));
    }

    #[test]
    fn test_invalid_json_returns_error() {
        let path = PathBuf::from("tests/fixtures/invalid.json");
        let result = InputReader::read_json(Some(&path));

        assert!(result.is_err());
        match result {
            Err(JiqError::InvalidJson(_)) => {
                // Expected error type
            }
            _ => panic!("Expected InvalidJson error"),
        }
    }

    #[test]
    fn test_file_not_found_returns_error() {
        let path = PathBuf::from("tests/fixtures/nonexistent.json");
        let result = InputReader::read_json(Some(&path));

        assert!(result.is_err());
        match result {
            Err(JiqError::Io(_)) => {
                // Expected IO error
            }
            _ => panic!("Expected IO error"),
        }
    }

    #[test]
    fn test_valid_json_string() {
        let json = r#"{"name": "Test", "value": 42}"#;
        let result = InputReader::read_json_from_string(json);

        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output, json);
    }

    #[test]
    fn test_invalid_json_string() {
        let json = r#"{"name": "Test", invalid}"#;
        let result = InputReader::read_json_from_string(json);

        assert!(result.is_err());
        match result {
            Err(JiqError::InvalidJson(_)) => {
                // Expected error type
            }
            _ => panic!("Expected InvalidJson error"),
        }
    }

    #[test]
    fn test_empty_json_object() {
        let json = "{}";
        let result = InputReader::read_json_from_string(json);

        assert!(result.is_ok());
    }

    #[test]
    fn test_empty_json_array() {
        let json = "[]";
        let result = InputReader::read_json_from_string(json);

        assert!(result.is_ok());
    }
}
