//! Tests for clipboard/backend

use super::*;

#[test]
fn test_copy_to_clipboard_osc52_backend() {
    let result = copy_to_clipboard("test", ClipboardBackend::Osc52);
    assert!(result.is_ok());
}

#[test]
fn test_copy_to_clipboard_system_backend() {
    let result = copy_to_clipboard("test", ClipboardBackend::System);
    assert!(result.is_ok() || matches!(result, Err(ClipboardError::SystemUnavailable)));
}

#[test]
fn test_copy_to_clipboard_auto_backend() {
    let result = copy_to_clipboard("test", ClipboardBackend::Auto);
    assert!(result.is_ok());
}

#[test]
fn test_copy_to_clipboard_empty_string() {
    let result = copy_to_clipboard("", ClipboardBackend::Osc52);
    assert!(result.is_ok());
}

#[test]
fn test_copy_to_clipboard_unicode() {
    let result = copy_to_clipboard("日本語 🎉", ClipboardBackend::Osc52);
    assert!(result.is_ok());
}
