use crate::editor::EditorMode;
use crate::snippets::Snippet;
use crate::test_utils::test_helpers::{app_with_query, key, key_with_mods};
use crossterm::event::{KeyCode, KeyModifiers};

#[test]
fn test_typing_filters_snippets() {
    let mut app = app_with_query("");
    app.input.editor_mode = EditorMode::Insert;

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));

    app.snippets.set_snippets(vec![
        Snippet {
            name: "flat array".to_string(),
            query: "flatten".to_string(),
            description: None,
        },
        Snippet {
            name: "sort data".to_string(),
            query: "sort".to_string(),
            description: None,
        },
        Snippet {
            name: "flat map".to_string(),
            query: "map(flatten)".to_string(),
            description: None,
        },
    ]);

    assert_eq!(app.snippets.filtered_count(), 3);

    app.handle_key_event(key(KeyCode::Char('f')));
    app.handle_key_event(key(KeyCode::Char('l')));
    app.handle_key_event(key(KeyCode::Char('a')));
    app.handle_key_event(key(KeyCode::Char('t')));

    assert_eq!(app.snippets.search_query(), "flat");
    assert_eq!(app.snippets.filtered_count(), 2);
}

#[test]
fn test_search_then_select_applies_filtered_snippet() {
    let mut app = app_with_query("");
    app.input.editor_mode = EditorMode::Insert;

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));

    app.snippets.set_snippets(vec![
        Snippet {
            name: "Flatten arrays".to_string(),
            query: "flatten".to_string(),
            description: None,
        },
        Snippet {
            name: "First data".to_string(),
            query: "first".to_string(),
            description: None,
        },
    ]);

    app.handle_key_event(key(KeyCode::Char('f')));
    app.handle_key_event(key(KeyCode::Char('i')));
    app.handle_key_event(key(KeyCode::Char('s')));
    app.handle_key_event(key(KeyCode::Char('t')));

    assert_eq!(app.snippets.filtered_count(), 1);

    app.handle_key_event(key(KeyCode::Enter));

    assert!(!app.snippets.is_visible());
    assert_eq!(app.input.query(), "first");
}

#[test]
fn test_backspace_updates_search() {
    let mut app = app_with_query("");
    app.input.editor_mode = EditorMode::Insert;

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));

    app.handle_key_event(key(KeyCode::Char('f')));
    app.handle_key_event(key(KeyCode::Char('l')));
    app.handle_key_event(key(KeyCode::Char('a')));
    app.handle_key_event(key(KeyCode::Char('t')));
    assert_eq!(app.snippets.search_query(), "flat");

    app.handle_key_event(key(KeyCode::Backspace));
    assert_eq!(app.snippets.search_query(), "fla");
}

#[test]
fn test_search_clears_when_popup_closes() {
    let mut app = app_with_query("");
    app.input.editor_mode = EditorMode::Insert;

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));

    app.snippets.set_snippets(vec![Snippet {
        name: "test".to_string(),
        query: ".".to_string(),
        description: None,
    }]);

    app.handle_key_event(key(KeyCode::Char('z')));
    assert_eq!(app.snippets.search_query(), "z");

    app.handle_key_event(key(KeyCode::Esc));
    assert!(!app.snippets.is_visible());

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));
    assert_eq!(app.snippets.search_query(), "");
}
