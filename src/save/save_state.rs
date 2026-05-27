use std::path::PathBuf;

use tui_textarea::TextArea;

use super::save_io::{SaveError, expand_path, ext_for_result};

pub const DEFAULT_PATH_PATTERN: &str = "jiq-{timestamp}.json";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SaveMode {
    Closed,
    EnterFilename,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PathPreview {
    Ready { resolved: PathBuf, exists: bool },
    Error(String),
}

impl PathPreview {
    pub fn would_overwrite(&self) -> bool {
        matches!(self, Self::Ready { exists: true, .. })
    }
}

#[derive(Debug)]
pub enum WriteOutcome {
    ReadyToWrite(PathBuf),
    Error(SaveError),
}

pub struct SaveState {
    mode: SaveMode,
    filename: TextArea<'static>,
    filename_dirty: bool,
    locked_timestamp: String,
}

impl Default for SaveState {
    fn default() -> Self {
        Self::new()
    }
}

impl SaveState {
    pub fn new() -> Self {
        Self {
            mode: SaveMode::Closed,
            filename: TextArea::default(),
            filename_dirty: false,
            locked_timestamp: String::new(),
        }
    }

    pub fn open(&mut self, locked_timestamp: String) {
        self.mode = SaveMode::EnterFilename;
        self.filename_dirty = false;
        self.locked_timestamp = locked_timestamp;
        let initial = expand_initial_pattern(DEFAULT_PATH_PATTERN, &self.locked_timestamp);
        self.filename = make_textarea(&initial);
    }

    pub fn close(&mut self) {
        self.mode = SaveMode::Closed;
    }

    pub fn is_visible(&self) -> bool {
        !matches!(self.mode, SaveMode::Closed)
    }

    pub fn mode(&self) -> &SaveMode {
        &self.mode
    }

    pub fn filename_mut(&mut self) -> &mut TextArea<'static> {
        &mut self.filename
    }

    pub fn mark_filename_edited(&mut self) {
        self.filename_dirty = true;
    }

    pub fn current_filename_text(&self) -> String {
        self.filename.lines().join("")
    }

    #[cfg(test)]
    pub fn locked_timestamp(&self) -> &str {
        &self.locked_timestamp
    }

    #[cfg(test)]
    pub fn filename_dirty(&self) -> bool {
        self.filename_dirty
    }

    /// Compute what the popup should display under the input field.
    /// Pure function of the current input + locked timestamp; safe to call on
    /// every render (one stat() per call — caller may cache if needed).
    pub fn compute_preview(&self) -> PathPreview {
        let pattern = self.current_filename_text();
        if pattern.trim().is_empty() {
            return PathPreview::Error("filename is empty".into());
        }
        match expand_path(&pattern, ext_for_result(), &self.locked_timestamp) {
            Ok(resolved) => {
                let exists = resolved.exists();
                PathPreview::Ready { resolved, exists }
            }
            Err(e) => PathPreview::Error(e.to_string()),
        }
    }

    pub fn prepare_write(&self) -> WriteOutcome {
        let pattern = self.current_filename_text();
        if pattern.trim().is_empty() {
            return WriteOutcome::Error(SaveError::BadPath("filename is empty".into()));
        }
        match expand_path(&pattern, ext_for_result(), &self.locked_timestamp) {
            Ok(path) => WriteOutcome::ReadyToWrite(path),
            Err(e) => WriteOutcome::Error(e),
        }
    }
}

fn make_textarea(initial: &str) -> TextArea<'static> {
    let mut ta = TextArea::default();
    ta.insert_str(initial);
    ta
}

fn expand_initial_pattern(pattern: &str, timestamp: &str) -> String {
    pattern
        .replace("{timestamp}", timestamp)
        .replace("{ext}", ext_for_result())
}

#[cfg(test)]
#[path = "save_state_tests.rs"]
mod save_state_tests;
