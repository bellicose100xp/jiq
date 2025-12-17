//! Tests for clipboard/system

use super::*;

#[test]
fn test_copy_returns_result() {
    let result = copy("test");
    assert!(result.is_ok() || matches!(result, Err(ClipboardError::SystemUnavailable)));
}
