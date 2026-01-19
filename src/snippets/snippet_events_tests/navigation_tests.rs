use crate::editor::EditorMode;
use crate::snippets::Snippet;
use crate::test_utils::test_helpers::{app_with_query, key, key_with_mods};
use crossterm::event::{KeyCode, KeyModifiers};

#[test]
fn test_down_arrow_navigates_to_next_snippet() {
    let mut app = app_with_query("");
    app.input.editor_mode = EditorMode::Insert;

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));
    assert!(app.snippets.is_visible());

    app.snippets.set_snippets(vec![
        Snippet {
            name: "test1".to_string(),
            query: ".".to_string(),
            description: None,
        },
        Snippet {
            name: "test2".to_string(),
            query: ".".to_string(),
            description: None,
        },
    ]);
    assert_eq!(app.snippets.selected_index(), 0);

    app.handle_key_event(key(KeyCode::Down));
    assert_eq!(app.snippets.selected_index(), 1);
}

#[test]
fn test_up_arrow_navigates_to_prev_snippet() {
    let mut app = app_with_query("");
    app.input.editor_mode = EditorMode::Insert;

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));
    app.snippets.set_snippets(vec![
        Snippet {
            name: "test1".to_string(),
            query: ".".to_string(),
            description: None,
        },
        Snippet {
            name: "test2".to_string(),
            query: ".".to_string(),
            description: None,
        },
    ]);

    app.handle_key_event(key(KeyCode::Down));
    assert_eq!(app.snippets.selected_index(), 1);

    app.handle_key_event(key(KeyCode::Up));
    assert_eq!(app.snippets.selected_index(), 0);
}

#[test]
fn test_j_key_types_into_search() {
    let mut app = app_with_query("");
    app.input.editor_mode = EditorMode::Insert;

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));
    assert!(app.snippets.is_visible());

    app.handle_key_event(key(KeyCode::Char('j')));
    assert_eq!(app.snippets.search_query(), "j");
}

#[test]
fn test_k_key_types_into_search() {
    let mut app = app_with_query("");
    app.input.editor_mode = EditorMode::Insert;

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));
    assert!(app.snippets.is_visible());

    app.handle_key_event(key(KeyCode::Char('k')));
    assert_eq!(app.snippets.search_query(), "k");
}

#[test]
fn test_navigation_stops_at_last_item() {
    let mut app = app_with_query("");
    app.input.editor_mode = EditorMode::Insert;

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));
    app.snippets.set_snippets(vec![
        Snippet {
            name: "test1".to_string(),
            query: ".".to_string(),
            description: None,
        },
        Snippet {
            name: "test2".to_string(),
            query: ".".to_string(),
            description: None,
        },
    ]);

    app.handle_key_event(key(KeyCode::Down));
    assert_eq!(app.snippets.selected_index(), 1);

    app.handle_key_event(key(KeyCode::Down));
    assert_eq!(app.snippets.selected_index(), 1);
}

#[test]
fn test_navigation_stops_at_first_item() {
    let mut app = app_with_query("");
    app.input.editor_mode = EditorMode::Insert;

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));
    app.snippets.set_snippets(vec![
        Snippet {
            name: "test1".to_string(),
            query: ".".to_string(),
            description: None,
        },
        Snippet {
            name: "test2".to_string(),
            query: ".".to_string(),
            description: None,
        },
    ]);
    assert_eq!(app.snippets.selected_index(), 0);

    app.handle_key_event(key(KeyCode::Up));
    assert_eq!(app.snippets.selected_index(), 0);
}

#[test]
fn test_navigation_with_empty_list() {
    let mut app = app_with_query("");
    app.input.editor_mode = EditorMode::Insert;

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));
    assert!(app.snippets.is_visible());

    app.snippets.set_snippets(vec![]);
    assert_eq!(app.snippets.selected_index(), 0);

    app.handle_key_event(key(KeyCode::Down));
    assert_eq!(app.snippets.selected_index(), 0);

    app.handle_key_event(key(KeyCode::Up));
    assert_eq!(app.snippets.selected_index(), 0);
}
