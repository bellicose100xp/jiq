use super::*;

#[test]
fn test_new_snippet_state() {
    let state = SnippetState::new();
    assert!(!state.is_visible());
    assert_eq!(*state.mode(), SnippetMode::Browse);
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

#[test]
fn test_mode_default_is_browse() {
    let mode = SnippetMode::default();
    assert_eq!(mode, SnippetMode::Browse);
}

#[test]
fn test_snippet_mode_eq() {
    assert_eq!(SnippetMode::Browse, SnippetMode::Browse);
    assert_eq!(SnippetMode::CreateName, SnippetMode::CreateName);
    assert_ne!(SnippetMode::Browse, SnippetMode::CreateName);
}

#[test]
fn test_snippet_mode_clone() {
    let mode = SnippetMode::CreateName;
    let cloned = mode.clone();
    assert_eq!(mode, cloned);
}

#[test]
fn test_snippet_mode_create_description_eq() {
    assert_eq!(
        SnippetMode::CreateDescription,
        SnippetMode::CreateDescription
    );
    assert_ne!(SnippetMode::CreateDescription, SnippetMode::CreateName);
    assert_ne!(SnippetMode::CreateDescription, SnippetMode::Browse);
}

#[test]
fn test_snippet_mode_create_description_clone() {
    let mode = SnippetMode::CreateDescription;
    let cloned = mode.clone();
    assert_eq!(mode, cloned);
}
