use super::*;

fn usable(text: &str) -> ClipboardPeek {
    ClipboardPeek::Usable(text.to_string())
}

#[test]
fn from_peek_usable_clipboard_preselects_clipboard_and_caches_bytes() {
    let state = SourcePickerState::from_peek(usable(r#"{"a": 1}"#));
    assert_eq!(state.selection, SourceChoice::Clipboard);
    assert_eq!(state.clipboard_cache.as_deref(), Some(r#"{"a": 1}"#));
}

#[test]
fn from_peek_unreadable_preselects_paste_and_no_cache() {
    let state = SourcePickerState::from_peek(ClipboardPeek::Unreadable);
    assert_eq!(state.selection, SourceChoice::Paste);
    assert!(state.clipboard_cache.is_none());
}

#[test]
fn from_peek_empty_preselects_paste() {
    let state = SourcePickerState::from_peek(ClipboardPeek::Empty);
    assert_eq!(state.selection, SourceChoice::Paste);
    assert!(state.clipboard_cache.is_none());
}

#[test]
fn from_peek_invalid_preselects_paste() {
    let state = SourcePickerState::from_peek(ClipboardPeek::Invalid);
    assert_eq!(state.selection, SourceChoice::Paste);
}

#[test]
fn from_peek_primitive_preselects_paste() {
    let state = SourcePickerState::from_peek(ClipboardPeek::Primitive);
    assert_eq!(state.selection, SourceChoice::Paste);
}

#[test]
fn select_next_cycles_clipboard_to_paste_to_clipboard() {
    let mut state = SourcePickerState::from_peek(usable(r#"{}"#));
    assert_eq!(state.selection, SourceChoice::Clipboard);
    state.select_next();
    assert_eq!(state.selection, SourceChoice::Paste);
    state.select_next();
    assert_eq!(state.selection, SourceChoice::Clipboard);
}

#[test]
fn select_previous_is_inverse_cycle() {
    let mut state = SourcePickerState::from_peek(usable(r#"{}"#));
    state.select_previous();
    assert_eq!(state.selection, SourceChoice::Paste);
    state.select_previous();
    assert_eq!(state.selection, SourceChoice::Clipboard);
}

#[test]
fn failure_context_only_when_unusable() {
    assert!(usable(r#"{}"#).failure_context().is_none());
    assert!(ClipboardPeek::Empty.failure_context().is_some());
    assert!(ClipboardPeek::Invalid.failure_context().is_some());
    assert!(ClipboardPeek::Primitive.failure_context().is_some());
    assert!(ClipboardPeek::Unreadable.failure_context().is_some());
}
