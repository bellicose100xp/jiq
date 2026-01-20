use crate::editor::EditorMode;
use crate::snippets::SnippetMode;
use crate::test_utils::test_helpers::{app_with_query, key, key_with_mods};
use crossterm::event::{KeyCode, KeyModifiers};

#[test]
fn test_tab_in_create_name_mode_moves_to_query() {
    let mut app = app_with_query(".test");
    app.input.editor_mode = EditorMode::Insert;
    app.snippets.disable_persistence();

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));
    app.handle_key_event(key_with_mods(KeyCode::Char('n'), KeyModifiers::CONTROL));
    assert_eq!(*app.snippets.mode(), SnippetMode::CreateName);

    app.handle_key_event(key(KeyCode::Tab));

    assert_eq!(*app.snippets.mode(), SnippetMode::CreateQuery);
}

#[test]
fn test_tab_in_create_query_mode_moves_to_description() {
    let mut app = app_with_query(".test");
    app.input.editor_mode = EditorMode::Insert;
    app.snippets.disable_persistence();

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));
    app.handle_key_event(key_with_mods(KeyCode::Char('n'), KeyModifiers::CONTROL));
    app.handle_key_event(key(KeyCode::Tab)); // Name -> Query
    assert_eq!(*app.snippets.mode(), SnippetMode::CreateQuery);

    app.handle_key_event(key(KeyCode::Tab));

    assert_eq!(*app.snippets.mode(), SnippetMode::CreateDescription);
}

#[test]
fn test_typing_in_create_description_mode() {
    let mut app = app_with_query(".test");
    app.input.editor_mode = EditorMode::Insert;
    app.snippets.disable_persistence();

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));
    app.handle_key_event(key_with_mods(KeyCode::Char('n'), KeyModifiers::CONTROL));
    app.handle_key_event(key(KeyCode::Tab)); // Name -> Query
    app.handle_key_event(key(KeyCode::Tab)); // Query -> Description
    assert_eq!(*app.snippets.mode(), SnippetMode::CreateDescription);

    app.handle_key_event(key(KeyCode::Char('D')));
    app.handle_key_event(key(KeyCode::Char('e')));
    app.handle_key_event(key(KeyCode::Char('s')));
    app.handle_key_event(key(KeyCode::Char('c')));

    assert_eq!(app.snippets.description_input(), "Desc");
}

#[test]
fn test_shift_tab_in_description_mode_goes_back_to_query() {
    let mut app = app_with_query(".test");
    app.input.editor_mode = EditorMode::Insert;
    app.snippets.disable_persistence();

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));
    app.handle_key_event(key_with_mods(KeyCode::Char('n'), KeyModifiers::CONTROL));
    app.handle_key_event(key(KeyCode::Tab)); // Name -> Query
    app.handle_key_event(key(KeyCode::Tab)); // Query -> Description
    assert_eq!(*app.snippets.mode(), SnippetMode::CreateDescription);

    app.handle_key_event(key(KeyCode::BackTab));

    assert_eq!(*app.snippets.mode(), SnippetMode::CreateQuery);
}

#[test]
fn test_tab_in_description_mode_cycles_to_name() {
    let mut app = app_with_query(".test");
    app.input.editor_mode = EditorMode::Insert;
    app.snippets.disable_persistence();

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));
    app.handle_key_event(key_with_mods(KeyCode::Char('n'), KeyModifiers::CONTROL));
    app.handle_key_event(key(KeyCode::Tab)); // Name -> Query
    app.handle_key_event(key(KeyCode::Tab)); // Query -> Description
    assert_eq!(*app.snippets.mode(), SnippetMode::CreateDescription);

    app.handle_key_event(key(KeyCode::Tab));

    assert_eq!(*app.snippets.mode(), SnippetMode::CreateName);
}

#[test]
fn test_shift_tab_in_name_mode_cycles_to_description() {
    let mut app = app_with_query(".test");
    app.input.editor_mode = EditorMode::Insert;
    app.snippets.disable_persistence();

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));
    app.handle_key_event(key_with_mods(KeyCode::Char('n'), KeyModifiers::CONTROL));
    assert_eq!(*app.snippets.mode(), SnippetMode::CreateName);

    app.handle_key_event(key(KeyCode::BackTab));

    assert_eq!(*app.snippets.mode(), SnippetMode::CreateDescription);
}

#[test]
fn test_enter_in_description_mode_saves_snippet_with_description() {
    let mut app = app_with_query(".test | keys");
    app.input.editor_mode = EditorMode::Insert;
    app.snippets.disable_persistence();

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));
    app.snippets.set_snippets(vec![]);
    app.handle_key_event(key_with_mods(KeyCode::Char('n'), KeyModifiers::CONTROL));

    app.handle_key_event(key(KeyCode::Char('M')));
    app.handle_key_event(key(KeyCode::Char('y')));
    app.handle_key_event(key(KeyCode::Char(' ')));
    app.handle_key_event(key(KeyCode::Char('S')));
    app.handle_key_event(key(KeyCode::Char('n')));
    app.handle_key_event(key(KeyCode::Char('i')));
    app.handle_key_event(key(KeyCode::Char('p')));
    app.handle_key_event(key(KeyCode::Tab)); // Name -> Query
    app.handle_key_event(key(KeyCode::Tab)); // Query -> Description

    app.handle_key_event(key(KeyCode::Char('T')));
    app.handle_key_event(key(KeyCode::Char('e')));
    app.handle_key_event(key(KeyCode::Char('s')));
    app.handle_key_event(key(KeyCode::Char('t')));
    app.handle_key_event(key(KeyCode::Enter));

    assert_eq!(*app.snippets.mode(), SnippetMode::Browse);
    assert_eq!(app.snippets.snippets().len(), 1);
    assert_eq!(app.snippets.snippets()[0].name, "My Snip");
    assert_eq!(
        app.snippets.snippets()[0].description,
        Some("Test".to_string())
    );
}

#[test]
fn test_esc_in_description_mode_cancels() {
    let mut app = app_with_query(".test");
    app.input.editor_mode = EditorMode::Insert;
    app.snippets.disable_persistence();

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));
    app.handle_key_event(key_with_mods(KeyCode::Char('n'), KeyModifiers::CONTROL));
    app.handle_key_event(key(KeyCode::Tab)); // Name -> Query
    app.handle_key_event(key(KeyCode::Tab)); // Query -> Description
    assert_eq!(*app.snippets.mode(), SnippetMode::CreateDescription);

    app.handle_key_event(key(KeyCode::Esc));

    assert_eq!(*app.snippets.mode(), SnippetMode::Browse);
    assert!(app.snippets.is_visible());
}

#[test]
fn test_question_mark_blocked_in_description_mode() {
    let mut app = app_with_query(".test");
    app.input.editor_mode = EditorMode::Insert;
    app.snippets.disable_persistence();

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));
    app.handle_key_event(key_with_mods(KeyCode::Char('n'), KeyModifiers::CONTROL));
    app.handle_key_event(key(KeyCode::Tab)); // Name -> Query
    app.handle_key_event(key(KeyCode::Tab)); // Query -> Description

    app.handle_key_event(key(KeyCode::Char('?')));

    assert!(!app.help.visible);
    assert_eq!(app.snippets.description_input(), "?");
}

#[test]
fn test_empty_name_in_description_mode_shows_error() {
    let mut app = app_with_query(".test");
    app.input.editor_mode = EditorMode::Insert;
    app.snippets.disable_persistence();

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));
    app.snippets.set_snippets(vec![]);
    app.handle_key_event(key_with_mods(KeyCode::Char('n'), KeyModifiers::CONTROL));
    app.handle_key_event(key(KeyCode::Tab)); // Name -> Query
    app.handle_key_event(key(KeyCode::Tab)); // Query -> Description
    app.handle_key_event(key(KeyCode::Enter));

    assert_eq!(*app.snippets.mode(), SnippetMode::CreateDescription);
    assert!(app.notification.current().is_some());
    let notification = app.notification.current().unwrap();
    assert!(notification.message.contains("Name cannot be empty"));
}

#[test]
fn test_is_editing_true_in_description_mode() {
    let mut app = app_with_query(".test");
    app.input.editor_mode = EditorMode::Insert;

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));
    app.handle_key_event(key_with_mods(KeyCode::Char('n'), KeyModifiers::CONTROL));
    app.handle_key_event(key(KeyCode::Tab)); // Name -> Query
    app.handle_key_event(key(KeyCode::Tab)); // Query -> Description

    assert!(app.snippets.is_editing());
}

#[test]
fn test_backspace_in_description_mode() {
    let mut app = app_with_query(".test");
    app.input.editor_mode = EditorMode::Insert;
    app.snippets.disable_persistence();

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));
    app.handle_key_event(key_with_mods(KeyCode::Char('n'), KeyModifiers::CONTROL));
    app.handle_key_event(key(KeyCode::Tab)); // Name -> Query
    app.handle_key_event(key(KeyCode::Tab)); // Query -> Description

    app.handle_key_event(key(KeyCode::Char('A')));
    app.handle_key_event(key(KeyCode::Char('B')));
    app.handle_key_event(key(KeyCode::Char('C')));
    assert_eq!(app.snippets.description_input(), "ABC");

    app.handle_key_event(key(KeyCode::Backspace));
    assert_eq!(app.snippets.description_input(), "AB");
}

#[test]
fn test_save_snippet_with_optional_description() {
    let mut app = app_with_query(".test");
    app.input.editor_mode = EditorMode::Insert;
    app.snippets.disable_persistence();

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));
    app.snippets.set_snippets(vec![]);
    app.handle_key_event(key_with_mods(KeyCode::Char('n'), KeyModifiers::CONTROL));

    app.handle_key_event(key(KeyCode::Char('T')));
    app.handle_key_event(key(KeyCode::Char('e')));
    app.handle_key_event(key(KeyCode::Char('s')));
    app.handle_key_event(key(KeyCode::Char('t')));
    app.handle_key_event(key(KeyCode::Tab)); // Name -> Query
    app.handle_key_event(key(KeyCode::Tab)); // Query -> Description
    app.handle_key_event(key(KeyCode::Enter));

    assert_eq!(app.snippets.snippets().len(), 1);
    assert_eq!(app.snippets.snippets()[0].description, None);
}
