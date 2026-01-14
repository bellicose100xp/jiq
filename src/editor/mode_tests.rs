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

#[test]
fn test_char_search_mode_display() {
    assert_eq!(
        EditorMode::CharSearch(SearchDirection::Forward, SearchType::Find).display(),
        "CHAR(f)"
    );
    assert_eq!(
        EditorMode::CharSearch(SearchDirection::Backward, SearchType::Find).display(),
        "CHAR(F)"
    );
    assert_eq!(
        EditorMode::CharSearch(SearchDirection::Forward, SearchType::Till).display(),
        "CHAR(t)"
    );
    assert_eq!(
        EditorMode::CharSearch(SearchDirection::Backward, SearchType::Till).display(),
        "CHAR(T)"
    );
}

#[test]
fn test_text_object_mode_display() {
    assert_eq!(
        EditorMode::TextObject('d', TextObjectScope::Inner).display(),
        "di…"
    );
    assert_eq!(
        EditorMode::TextObject('d', TextObjectScope::Around).display(),
        "da…"
    );
    assert_eq!(
        EditorMode::TextObject('c', TextObjectScope::Inner).display(),
        "ci…"
    );
    assert_eq!(
        EditorMode::TextObject('c', TextObjectScope::Around).display(),
        "ca…"
    );
}
