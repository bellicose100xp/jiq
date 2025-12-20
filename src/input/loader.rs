//! File Loader Module
//!
//! Handles asynchronous file loading in a background thread to avoid blocking the UI.
//! Uses channels for thread communication following the pattern established by the AI worker.

use std::path::{Path, PathBuf};
use std::sync::mpsc::{Receiver, channel};

use crate::error::JiqError;

/// Represents the current state of file loading
#[derive(Debug, Clone, PartialEq)]
pub enum LoadingState {
    Loading,
    Complete(String),
    Error(JiqError),
}

/// Manages asynchronous file loading in a background thread
pub struct FileLoader {
    state: LoadingState,
    rx: Option<Receiver<Result<String, JiqError>>>,
}

impl FileLoader {
    /// Spawn a background thread to load a file
    ///
    /// Creates a background thread that reads the file, validates JSON,
    /// and sends the result back via a channel.
    ///
    /// # Arguments
    /// * `path` - Path to the JSON file to load
    pub fn spawn_load(path: PathBuf) -> Self {
        let (tx, rx) = channel();

        std::thread::spawn(move || {
            let result = load_file_sync(&path);
            let _ = tx.send(result);
        });

        Self {
            state: LoadingState::Loading,
            rx: Some(rx),
        }
    }

    /// Spawn a background thread to load from stdin
    ///
    /// Creates a background thread that reads from stdin, validates JSON,
    /// and sends the result back via a channel.
    pub fn spawn_load_stdin() -> Self {
        let (tx, rx) = channel();

        std::thread::spawn(move || {
            let result = load_stdin_sync();
            let _ = tx.send(result);
        });

        Self {
            state: LoadingState::Loading,
            rx: Some(rx),
        }
    }

    /// Poll for loading completion (non-blocking)
    ///
    /// Checks the channel for results without blocking. Returns None if still loading,
    /// or Some with the result when complete.
    pub fn poll(&mut self) -> Option<Result<String, JiqError>> {
        if let Some(rx) = &self.rx {
            match rx.try_recv() {
                Ok(result) => {
                    self.rx = None;
                    self.state = match &result {
                        Ok(json) => LoadingState::Complete(json.clone()),
                        Err(e) => LoadingState::Error(e.clone()),
                    };
                    Some(result)
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => None,
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    self.rx = None;
                    let err = JiqError::Io("File loader thread disconnected".to_string());
                    self.state = LoadingState::Error(err.clone());
                    Some(Err(err))
                }
            }
        } else {
            None
        }
    }

    /// Get the current loading state
    pub fn state(&self) -> &LoadingState {
        &self.state
    }

    /// Check if currently loading
    pub fn is_loading(&self) -> bool {
        matches!(self.state, LoadingState::Loading)
    }
}

/// Synchronous file loading (runs in background thread)
///
/// Reads the file from disk and validates that it contains valid JSON.
fn load_file_sync(path: &Path) -> Result<String, JiqError> {
    use std::fs::File;
    use std::io::Read;

    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    // Validate JSON
    serde_json::from_str::<serde_json::Value>(&contents)
        .map_err(|e| JiqError::InvalidJson(e.to_string()))?;

    Ok(contents)
}

/// Synchronous stdin loading (runs in background thread)
///
/// Reads from stdin and validates that it contains valid JSON.
fn load_stdin_sync() -> Result<String, JiqError> {
    use std::io::{self, Read};

    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer)?;

    // Validate JSON
    serde_json::from_str::<serde_json::Value>(&buffer)
        .map_err(|e| JiqError::InvalidJson(e.to_string()))?;

    Ok(buffer)
}

#[cfg(test)]
#[path = "loader_tests.rs"]
mod loader_tests;
