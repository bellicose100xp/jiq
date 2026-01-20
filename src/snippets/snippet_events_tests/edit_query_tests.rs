use crate::editor::EditorMode;
use crate::snippets::{Snippet, SnippetMode};
use crate::test_utils::test_helpers::{app_with_query, key, key_with_mods};
use crossterm::event::{KeyCode, KeyModifiers};

#[test]
fn test_e_key_enters_edit_mode() {
    let mut app = app_with_query(".test");
    app.input.editor_mode = EditorMode::Insert;
    app.snippets.disable_persistence();

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));
    app.snippets.set_snippets(vec![Snippet {
        name: "My Snippet".to_string(),
        query: ".test | keys".to_string(),
        description: None,
    }]);
    app.snippets.on_search_input_changed();

    app.handle_key_event(key(KeyCode::Char('e')));

    assert!(matches!(
        app.snippets.mode(),
        SnippetMode::EditName { original_name } if original_name == "My Snippet"
    ));
}

#[test]
fn test_e_key_with_no_snippets_does_nothing() {
    let mut app = app_with_query(".test");
    app.input.editor_mode = EditorMode::Insert;
    app.snippets.disable_persistence();

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));
    app.snippets.set_snippets(vec![]);
    app.snippets.on_search_input_changed();

    app.handle_key_event(key(KeyCode::Char('e')));

    assert_eq!(*app.snippets.mode(), SnippetMode::Browse);
}

#[test]
fn test_esc_in_edit_mode_cancels() {
    let mut app = app_with_query(".test");
    app.input.editor_mode = EditorMode::Insert;
    app.snippets.disable_persistence();

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));
    app.snippets.set_snippets(vec![Snippet {
        name: "My Snippet".to_string(),
        query: ".test".to_string(),
        description: None,
    }]);
    app.snippets.on_search_input_changed();
    app.handle_key_event(key(KeyCode::Char('e')));

    app.handle_key_event(key(KeyCode::Esc));

    assert_eq!(*app.snippets.mode(), SnippetMode::Browse);
    assert!(app.snippets.is_visible());
}

#[test]
fn test_tab_in_edit_name_mode_saves_and_navigates_to_query() {
    let mut app = app_with_query(".test");
    app.input.editor_mode = EditorMode::Insert;
    app.snippets.disable_persistence();

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));
    app.snippets.set_snippets(vec![Snippet {
        name: "Old".to_string(),
        query: ".old".to_string(),
        description: None,
    }]);
    app.snippets.on_search_input_changed();
    app.handle_key_event(key(KeyCode::Char('e')));

    for _ in 0..3 {
        app.handle_key_event(key(KeyCode::Backspace));
    }
    app.handle_key_event(key(KeyCode::Char('N')));
    app.handle_key_event(key(KeyCode::Char('e')));
    app.handle_key_event(key(KeyCode::Char('w')));

    assert!(matches!(app.snippets.mode(), SnippetMode::EditName { .. }));
    app.handle_key_event(key(KeyCode::Tab));

    assert!(matches!(app.snippets.mode(), SnippetMode::EditQuery { .. }));
    assert_eq!(app.snippets.snippets()[0].name, "New");
}

#[test]
fn test_typing_in_edit_name_mode_updates_name() {
    let mut app = app_with_query(".test");
    app.input.editor_mode = EditorMode::Insert;
    app.snippets.disable_persistence();

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));
    app.snippets.set_snippets(vec![Snippet {
        name: "Old".to_string(),
        query: ".test".to_string(),
        description: None,
    }]);
    app.snippets.on_search_input_changed();
    app.handle_key_event(key(KeyCode::Char('e')));

    for _ in 0..3 {
        app.handle_key_event(key(KeyCode::Backspace));
    }
    app.handle_key_event(key(KeyCode::Char('N')));
    app.handle_key_event(key(KeyCode::Char('e')));
    app.handle_key_event(key(KeyCode::Char('w')));

    assert_eq!(app.snippets.name_input(), "New");
}

#[test]
fn test_enter_in_edit_name_mode_saves_and_exits() {
    let mut app = app_with_query(".test");
    app.input.editor_mode = EditorMode::Insert;
    app.snippets.disable_persistence();

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));
    app.snippets.set_snippets(vec![Snippet {
        name: "Old".to_string(),
        query: ".test".to_string(),
        description: None,
    }]);
    app.snippets.on_search_input_changed();
    app.handle_key_event(key(KeyCode::Char('e')));

    for _ in 0..3 {
        app.handle_key_event(key(KeyCode::Backspace));
    }
    app.handle_key_event(key(KeyCode::Char('N')));
    app.handle_key_event(key(KeyCode::Char('e')));
    app.handle_key_event(key(KeyCode::Char('w')));
    app.handle_key_event(key(KeyCode::Enter));

    assert_eq!(*app.snippets.mode(), SnippetMode::Browse);
    assert_eq!(app.snippets.snippets()[0].name, "New");
}

#[test]
fn test_edit_name_empty_shows_error() {
    let mut app = app_with_query(".test");
    app.input.editor_mode = EditorMode::Insert;
    app.snippets.disable_persistence();

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));
    app.snippets.set_snippets(vec![Snippet {
        name: "Old".to_string(),
        query: ".test".to_string(),
        description: None,
    }]);
    app.snippets.on_search_input_changed();
    app.handle_key_event(key(KeyCode::Char('e')));

    for _ in 0..3 {
        app.handle_key_event(key(KeyCode::Backspace));
    }
    app.handle_key_event(key(KeyCode::Enter));

    assert!(matches!(app.snippets.mode(), SnippetMode::EditName { .. }));
    assert!(app.notification.current().is_some());
    let notification = app.notification.current().unwrap();
    assert!(notification.message.contains("empty"));
}

#[test]
fn test_is_editing_true_in_edit_mode() {
    let mut app = app_with_query(".test");
    app.input.editor_mode = EditorMode::Insert;
    app.snippets.disable_persistence();

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));
    app.snippets.set_snippets(vec![Snippet {
        name: "My Snippet".to_string(),
        query: ".test".to_string(),
        description: None,
    }]);
    app.snippets.on_search_input_changed();
    assert!(!app.snippets.is_editing());

    app.handle_key_event(key(KeyCode::Char('e')));
    assert!(app.snippets.is_editing());
}

#[test]
fn test_question_mark_blocked_in_edit_mode() {
    let mut app = app_with_query(".test");
    app.input.editor_mode = EditorMode::Insert;
    app.snippets.disable_persistence();

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));
    app.snippets.set_snippets(vec![Snippet {
        name: "My Snippet".to_string(),
        query: ".old".to_string(),
        description: None,
    }]);
    app.snippets.on_search_input_changed();
    app.handle_key_event(key(KeyCode::Char('e')));

    app.handle_key_event(key(KeyCode::Char('?')));

    assert!(!app.help.visible);
    assert!(app.snippets.name_input().contains('?'));
}

#[test]
fn test_full_edit_flow_name_query_description() {
    let mut app = app_with_query(".test");
    app.input.editor_mode = EditorMode::Insert;
    app.snippets.disable_persistence();

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));
    app.snippets.set_snippets(vec![Snippet {
        name: "OldName".to_string(),
        query: ".old".to_string(),
        description: Some("Old desc".to_string()),
    }]);
    app.snippets.on_search_input_changed();

    // Enter edit mode (starts in EditName)
    app.handle_key_event(key(KeyCode::Char('e')));
    assert!(matches!(app.snippets.mode(), SnippetMode::EditName { .. }));

    // Edit name and go to query with Tab
    for _ in 0..7 {
        app.handle_key_event(key(KeyCode::Backspace));
    }
    app.handle_key_event(key(KeyCode::Char('N')));
    app.handle_key_event(key(KeyCode::Char('e')));
    app.handle_key_event(key(KeyCode::Char('w')));
    app.handle_key_event(key(KeyCode::Tab)); // Save name and go to query

    assert!(matches!(app.snippets.mode(), SnippetMode::EditQuery { .. }));
    assert_eq!(app.snippets.snippets()[0].name, "New");

    // Edit query and go to description with Tab
    for _ in 0..4 {
        app.handle_key_event(key(KeyCode::Backspace));
    }
    app.handle_key_event(key(KeyCode::Char('.')));
    app.handle_key_event(key(KeyCode::Char('n')));
    app.handle_key_event(key(KeyCode::Char('e')));
    app.handle_key_event(key(KeyCode::Char('w')));
    app.handle_key_event(key(KeyCode::Tab)); // Save query and go to description

    assert!(matches!(
        app.snippets.mode(),
        SnippetMode::EditDescription { .. }
    ));
    assert_eq!(app.snippets.snippets()[0].query, ".new");

    // Edit description and save with Enter
    for _ in 0..8 {
        app.handle_key_event(key(KeyCode::Backspace));
    }
    app.handle_key_event(key(KeyCode::Char('N')));
    app.handle_key_event(key(KeyCode::Char('e')));
    app.handle_key_event(key(KeyCode::Char('w')));
    app.handle_key_event(key(KeyCode::Enter));

    assert_eq!(*app.snippets.mode(), SnippetMode::Browse);
    assert_eq!(
        app.snippets.snippets()[0].description,
        Some("New".to_string())
    );
}
