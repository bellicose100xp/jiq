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
    pub state: LoadingState,
    pub rx: Option<Receiver<Result<String, JiqError>>>,
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

    /// Load from the system clipboard synchronously.
    ///
    /// Used when jiq is launched with no file argument and no piped stdin.
    /// Runs on the main thread *before* the event loop starts, because the OSC
    /// 52 fallback reads the terminal's reply on stdin and would race the main
    /// crossterm reader if it ran in a worker thread. macOS / desktop Linux
    /// hits the fast `arboard` path and adds no latency; the up-to-1s OSC 52
    /// timeout only kicks in on remote SSH sessions where arboard fails first.
    /// The OSC 52 path picks up content copied *inside* the remote session
    /// (e.g. tmux selection buffers); content copied on the host workstation
    /// generally cannot reach jiq because most terminals refuse to forward
    /// host-clipboard reads back through the SSH tunnel for security reasons.
    pub fn load_clipboard_blocking() -> Self {
        let result = load_clipboard_sync();
        let state = match &result {
            Ok(json) => LoadingState::Complete(json.clone()),
            Err(e) => LoadingState::Error(e.clone()),
        };

        let (tx, rx) = channel();
        let _ = tx.send(result);
        Self {
            state,
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
                    log::error!("File loader thread disconnected");
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

/// Validate that content is valid JSON or JSONL
///
/// Uses StreamDeserializer to handle both single JSON values and JSONL (multiple values).
fn validate_json_or_jsonl(content: &str) -> Result<(), JiqError> {
    let deserializer = serde_json::Deserializer::from_str(content).into_iter::<serde_json::Value>();
    let mut count = 0;
    for result in deserializer {
        result.map_err(|e| JiqError::InvalidJson(e.to_string()))?;
        count += 1;
    }
    if count == 0 {
        return Err(JiqError::InvalidJson("Empty input".to_string()));
    }
    log::debug!("JSON validation: {} top-level value(s)", count);
    Ok(())
}

/// Synchronous file loading (runs in background thread)
///
/// Reads the file from disk and validates that it contains valid JSON or JSONL.
fn load_file_sync(path: &Path) -> Result<String, JiqError> {
    use std::fs::File;
    use std::io::Read;

    log::debug!("Loading file: {:?}", path);
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    log::debug!("File read: {} bytes", contents.len());

    validate_json_or_jsonl(&contents)?;

    Ok(contents)
}

/// Synchronous stdin loading (runs in background thread)
///
/// Reads from stdin and validates that it contains valid JSON or JSONL.
fn load_stdin_sync() -> Result<String, JiqError> {
    use std::io::{self, IsTerminal, Read};

    log::debug!("Loading from stdin");
    if io::stdin().is_terminal() {
        log::debug!("stdin is a terminal, no piped input");
        return Err(input_load_error(InputErrorReason::NoStdin));
    }

    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer)?;
    log::debug!("Stdin read: {} bytes", buffer.len());

    validate_json_or_jsonl(&buffer)?;

    Ok(buffer)
}

/// Synchronous clipboard loading.
///
/// Tries the system clipboard first via `arboard`, which is the fast path on
/// macOS, desktop Linux, Windows, and WSL. When that fails - typically a
/// remote SSH session without X11/Wayland - falls back to OSC 52 read.
///
/// What OSC 52 read actually picks up depends on the terminal stack: tmux
/// selection buffers and OSC-52-written content from peer apps in the same
/// session usually round-trip cleanly, while content copied on the host
/// workstation (Cmd+C in a Mac browser) typically does not, because most
/// terminals do not forward remote OSC 52 read replies for security reasons.
///
/// Every failure mode below collapses into the multi-line usage hint produced
/// by `input_load_error`. Raw `arboard` error text is logged at debug level
/// for diagnosis but never surfaced to the user.
fn load_clipboard_sync() -> Result<String, JiqError> {
    log::debug!("Loading from clipboard");

    let contents = match read_clipboard_text() {
        Ok(text) => text,
        Err(reason) => {
            log::debug!("Clipboard unavailable: {}", reason);
            return Err(input_load_error(InputErrorReason::ClipboardUnreadable));
        }
    };
    log::debug!("Clipboard read: {} bytes", contents.len());

    if contents.trim().is_empty() {
        return Err(input_load_error(InputErrorReason::ClipboardEmpty));
    }

    if validate_json_or_jsonl(&contents).is_err() {
        log::debug!("Clipboard contents are not valid JSON or JSONL");
        return Err(input_load_error(InputErrorReason::ClipboardInvalidJson));
    }

    Ok(contents)
}

/// Try the system clipboard via `arboard`, then OSC 52 if that fails.
fn read_clipboard_text() -> Result<String, String> {
    match arboard::Clipboard::new().and_then(|mut c| c.get_text()) {
        Ok(text) => Ok(text),
        Err(arboard_err) => {
            log::debug!("arboard failed ({}), trying OSC 52 read", arboard_err);
            match crate::clipboard::osc52::read_with_timeout(std::time::Duration::from_millis(1000))
            {
                Ok(text) => {
                    log::debug!("OSC 52 read succeeded: {} bytes", text.len());
                    Ok(text)
                }
                Err(osc_err) => Err(format!("arboard: {}; osc52: {:?}", arboard_err, osc_err)),
            }
        }
    }
}

/// Reasons the input-load step can fail. Drives the multi-line error text.
///
/// Each variant maps to a single-sentence diagnosis on the first line of the
/// error, so users can tell apart "we never reached the clipboard at all"
/// from "we read the clipboard but its contents weren't usable".
#[derive(Debug, Clone, Copy)]
pub(crate) enum InputErrorReason {
    /// Both `arboard` and the OSC 52 fallback failed - we never got bytes.
    ClipboardUnreadable,
    /// Clipboard read succeeded but the buffer was empty or whitespace-only.
    ClipboardEmpty,
    /// Clipboard had non-empty content but it didn't parse as JSON/JSONL.
    ClipboardInvalidJson,
    /// Piped stdin was actually a terminal (no real data to read).
    NoStdin,
}

pub(crate) fn input_load_error(reason: InputErrorReason) -> JiqError {
    JiqError::Io(format_input_load_error(reason))
}

fn format_input_load_error(reason: InputErrorReason) -> String {
    let detail = match reason {
        InputErrorReason::ClipboardUnreadable => {
            "No input provided. Could not read the system clipboard."
        }
        InputErrorReason::ClipboardEmpty => "No input provided. Clipboard is empty.",
        InputErrorReason::ClipboardInvalidJson => {
            "No input provided. Clipboard does not contain valid JSON."
        }
        InputErrorReason::NoStdin => "No input provided.",
    };

    format!(
        "{}\n\nUsage:\n  jiq <file>            # load from file\n  cat data.json | jiq   # load from piped stdin\n  jiq                   # load from system clipboard",
        detail
    )
}

#[cfg(test)]
#[path = "loader_tests.rs"]
mod loader_tests;
