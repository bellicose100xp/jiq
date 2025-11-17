use std::fs::File;
use std::io::{self, Read};
use std::path::Path;
use serde_json::Value;
use crate::error::JiqError;

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
}
