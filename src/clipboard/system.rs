use arboard::Clipboard;

use super::backend::{ClipboardError, ClipboardResult};

pub fn copy(text: &str) -> ClipboardResult {
    let mut clipboard = Clipboard::new().map_err(|_| ClipboardError::SystemUnavailable)?;

    clipboard
        .set_text(text)
        .map_err(|_| ClipboardError::WriteError)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_copy_returns_result() {
        let result = copy("test");
        assert!(result.is_ok() || matches!(result, Err(ClipboardError::SystemUnavailable)));
    }
}
