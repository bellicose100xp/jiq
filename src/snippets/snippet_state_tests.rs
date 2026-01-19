use super::*;

#[test]
fn test_new_snippet_state() {
    let state = SnippetState::new();
    assert!(!state.is_visible());
}

#[test]
fn test_default_snippet_state() {
    let state = SnippetState::default();
    assert!(!state.is_visible());
}

#[test]
fn test_open_snippet_popup() {
    let mut state = SnippetState::new();
    assert!(!state.is_visible());

    state.open();
    assert!(state.is_visible());
}

#[test]
fn test_close_snippet_popup() {
    let mut state = SnippetState::new();
    state.open();
    assert!(state.is_visible());

    state.close();
    assert!(!state.is_visible());
}

#[test]
fn test_open_close_open() {
    let mut state = SnippetState::new();

    state.open();
    assert!(state.is_visible());

    state.close();
    assert!(!state.is_visible());

    state.open();
    assert!(state.is_visible());
}

#[test]
fn test_is_editing_returns_false_in_browse_mode() {
    let state = SnippetState::new();
    assert!(!state.is_editing());
}
