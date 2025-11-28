//! Clipboard backend selection and error types
//!
//! This module provides the main entry point for clipboard operations,
//! selecting the appropriate backend based on configuration.

use crate::config::ClipboardBackend;

use super::{osc52, system};

/// Result type for clipboard operations
pub type ClipboardResult = Result<(), ClipboardError>;

/// Errors that can occur during clipboard operations
#[derive(Debug)]
pub enum ClipboardError {
    /// System clipboard is not available
    SystemUnavailable,
    /// Error writing to clipboard
    WriteError,
}

/// Copy text to clipboard using the specified backend
///
/// # Arguments
/// * `text` - The text to copy to the clipboard
/// * `backend` - The clipboard backend to use
///
/// # Backend Selection
/// - `System`: Uses only the OS clipboard API (via arboard)
/// - `Osc52`: Uses only OSC 52 escape sequences
/// - `Auto`: Tries system clipboard first, falls back to OSC 52 if unavailable
///
/// # Returns
/// - `Ok(())` if the copy operation succeeded
/// - `Err(ClipboardError)` if the operation failed
pub fn copy_to_clipboard(text: &str, backend: ClipboardBackend) -> ClipboardResult {
    match backend {
        ClipboardBackend::System => system::copy(text),
        ClipboardBackend::Osc52 => osc52::copy(text),
        ClipboardBackend::Auto => {
            // Try system first, fall back to OSC 52 if unavailable
            system::copy(text).or_else(|_| osc52::copy(text))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_copy_to_clipboard_osc52_backend() {
        // OSC 52 should always succeed (writes to stdout)
        let result = copy_to_clipboard("test", ClipboardBackend::Osc52);
        assert!(result.is_ok());
    }

    #[test]
    fn test_copy_to_clipboard_system_backend() {
        // System clipboard may or may not be available depending on environment
        let result = copy_to_clipboard("test", ClipboardBackend::System);
        // We just verify it returns a valid result type
        assert!(
            result.is_ok() || matches!(result, Err(ClipboardError::SystemUnavailable))
        );
    }

    #[test]
    fn test_copy_to_clipboard_auto_backend() {
        // Auto mode should always succeed because it falls back to OSC 52
        let result = copy_to_clipboard("test", ClipboardBackend::Auto);
        assert!(result.is_ok());
    }

    #[test]
    fn test_copy_to_clipboard_empty_string() {
        // Should handle empty strings
        let result = copy_to_clipboard("", ClipboardBackend::Osc52);
        assert!(result.is_ok());
    }

    #[test]
    fn test_copy_to_clipboard_unicode() {
        // Should handle unicode content
        let result = copy_to_clipboard("日本語 🎉", ClipboardBackend::Osc52);
        assert!(result.is_ok());
    }
}
