//! Tests for editor/mode

use super::*;

#[test]
fn test_default_mode() {
    assert_eq!(EditorMode::default(), EditorMode::Insert);
}

#[test]
fn test_mode_display() {
    assert_eq!(EditorMode::Insert.display(), "INSERT");
    assert_eq!(EditorMode::Normal.display(), "NORMAL");
    assert_eq!(EditorMode::Operator('d').display(), "OPERATOR(d)");
    assert_eq!(EditorMode::Operator('c').display(), "OPERATOR(c)");
}
