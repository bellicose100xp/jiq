//! System clipboard backend
//!
//! Provides clipboard access via the operating system's native clipboard API
//! using the arboard crate.

use arboard::Clipboard;

use super::backend::{ClipboardError, ClipboardResult};

/// Copy text to system clipboard using arboard
///
/// This function attempts to access the system clipboard and set its contents
/// to the provided text. It handles cases where the clipboard is unavailable
/// (e.g., in headless environments or when no display server is running).
pub fn copy(text: &str) -> ClipboardResult {
    let mut clipboard =
        Clipboard::new().map_err(|_| ClipboardError::SystemUnavailable)?;

    clipboard
        .set_text(text)
        .map_err(|_| ClipboardError::WriteError)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_copy_returns_result() {
        // This test verifies the function signature and basic behavior.
        // The actual clipboard operation may fail in CI environments
        // without a display server, which is expected behavior.
        let result = copy("test");
        // We just verify it returns a Result, not that it succeeds,
        // since clipboard availability depends on the environment.
        assert!(result.is_ok() || matches!(result, Err(ClipboardError::SystemUnavailable)));
    }
}
