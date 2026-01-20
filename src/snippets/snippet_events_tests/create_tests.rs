use crate::editor::EditorMode;
use crate::snippets::{Snippet, SnippetMode};
use crate::test_utils::test_helpers::{app_with_query, key, key_with_mods};
use crossterm::event::{KeyCode, KeyModifiers};

#[test]
fn test_n_key_enters_create_mode() {
    let mut app = app_with_query(".test | keys");
    app.input.editor_mode = EditorMode::Insert;

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));
    assert!(app.snippets.is_visible());
    assert_eq!(*app.snippets.mode(), SnippetMode::Browse);

    app.handle_key_event(key_with_mods(KeyCode::Char('n'), KeyModifiers::CONTROL));
    assert_eq!(*app.snippets.mode(), SnippetMode::CreateName);
    assert_eq!(app.snippets.pending_query(), ".test | keys");
}

#[test]
fn test_create_mode_captures_current_query() {
    let mut app = app_with_query(".foo | .bar");
    app.input.editor_mode = EditorMode::Insert;

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));
    app.handle_key_event(key_with_mods(KeyCode::Char('n'), KeyModifiers::CONTROL));

    assert_eq!(*app.snippets.mode(), SnippetMode::CreateName);
    assert_eq!(app.snippets.pending_query(), ".foo | .bar");
}

#[test]
fn test_esc_in_create_mode_cancels() {
    let mut app = app_with_query(".test");
    app.input.editor_mode = EditorMode::Insert;

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));
    app.handle_key_event(key_with_mods(KeyCode::Char('n'), KeyModifiers::CONTROL));
    assert_eq!(*app.snippets.mode(), SnippetMode::CreateName);

    app.handle_key_event(key(KeyCode::Esc));
    assert_eq!(*app.snippets.mode(), SnippetMode::Browse);
    assert!(app.snippets.is_visible());
}

#[test]
fn test_typing_in_create_mode_updates_name() {
    let mut app = app_with_query(".test");
    app.input.editor_mode = EditorMode::Insert;
    app.snippets.disable_persistence();

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));
    app.handle_key_event(key_with_mods(KeyCode::Char('n'), KeyModifiers::CONTROL));
    assert_eq!(*app.snippets.mode(), SnippetMode::CreateName);

    app.handle_key_event(key(KeyCode::Char('T')));
    app.handle_key_event(key(KeyCode::Char('e')));
    app.handle_key_event(key(KeyCode::Char('s')));
    app.handle_key_event(key(KeyCode::Char('t')));

    assert_eq!(app.snippets.name_input(), "Test");
}

#[test]
fn test_enter_in_create_name_mode_saves_snippet() {
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

    app.handle_key_event(key(KeyCode::Enter));

    assert_eq!(*app.snippets.mode(), SnippetMode::Browse);
    assert_eq!(app.snippets.snippets().len(), 1);
    assert_eq!(app.snippets.snippets()[0].name, "My Snip");
}

#[test]
fn test_enter_in_create_description_mode_saves_snippet() {
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
    app.handle_key_event(key(KeyCode::Enter)); // Name -> Query
    app.handle_key_event(key(KeyCode::Enter)); // Query -> Description
    app.handle_key_event(key(KeyCode::Enter)); // Save

    assert_eq!(*app.snippets.mode(), SnippetMode::Browse);
    assert_eq!(app.snippets.snippets().len(), 1);
    assert_eq!(app.snippets.snippets()[0].name, "My Snip");
    assert_eq!(app.snippets.snippets()[0].query, ".test | keys");
}

#[test]
fn test_enter_with_empty_name_shows_error() {
    let mut app = app_with_query(".test");
    app.input.editor_mode = EditorMode::Insert;
    app.snippets.disable_persistence();

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));
    app.snippets.set_snippets(vec![]);
    app.handle_key_event(key_with_mods(KeyCode::Char('n'), KeyModifiers::CONTROL));

    app.handle_key_event(key(KeyCode::Enter));

    assert_eq!(*app.snippets.mode(), SnippetMode::CreateName);
    assert_eq!(app.snippets.snippets().len(), 0);
    assert!(app.notification.current().is_some());
}

#[test]
fn test_create_mode_blocks_navigation_keys() {
    let mut app = app_with_query(".test");
    app.input.editor_mode = EditorMode::Insert;
    app.snippets.disable_persistence();

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));
    app.handle_key_event(key_with_mods(KeyCode::Char('n'), KeyModifiers::CONTROL));

    app.handle_key_event(key(KeyCode::Down));
    app.handle_key_event(key(KeyCode::Up));

    assert_eq!(*app.snippets.mode(), SnippetMode::CreateName);
}

#[test]
fn test_is_editing_true_in_create_mode() {
    let mut app = app_with_query(".test");
    app.input.editor_mode = EditorMode::Insert;

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));
    assert!(!app.snippets.is_editing());

    app.handle_key_event(key_with_mods(KeyCode::Char('n'), KeyModifiers::CONTROL));
    assert!(app.snippets.is_editing());
}

#[test]
fn test_question_mark_blocked_in_create_mode() {
    let mut app = app_with_query(".test");
    app.input.editor_mode = EditorMode::Insert;
    app.snippets.disable_persistence();

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));
    app.handle_key_event(key_with_mods(KeyCode::Char('n'), KeyModifiers::CONTROL));

    app.handle_key_event(key(KeyCode::Char('?')));

    assert!(!app.help.visible);
    assert_eq!(app.snippets.name_input(), "?");
}

#[test]
fn test_backspace_in_create_mode() {
    let mut app = app_with_query(".test");
    app.input.editor_mode = EditorMode::Insert;
    app.snippets.disable_persistence();

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));
    app.handle_key_event(key_with_mods(KeyCode::Char('n'), KeyModifiers::CONTROL));

    app.handle_key_event(key(KeyCode::Char('A')));
    app.handle_key_event(key(KeyCode::Char('B')));
    app.handle_key_event(key(KeyCode::Char('C')));
    assert_eq!(app.snippets.name_input(), "ABC");

    app.handle_key_event(key(KeyCode::Backspace));
    assert_eq!(app.snippets.name_input(), "AB");
}

#[test]
fn test_popup_closes_after_successful_save() {
    let mut app = app_with_query(".test");
    app.input.editor_mode = EditorMode::Insert;
    app.snippets.disable_persistence();

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));
    app.handle_key_event(key_with_mods(KeyCode::Char('n'), KeyModifiers::CONTROL));

    app.handle_key_event(key(KeyCode::Char('T')));
    app.handle_key_event(key(KeyCode::Char('e')));
    app.handle_key_event(key(KeyCode::Char('s')));
    app.handle_key_event(key(KeyCode::Char('t')));
    app.handle_key_event(key(KeyCode::Enter)); // Name -> Query

    assert!(app.snippets.is_visible());
}

#[test]
fn test_empty_name_shows_error_notification() {
    let mut app = app_with_query(".test");
    app.input.editor_mode = EditorMode::Insert;
    app.snippets.disable_persistence();

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));
    app.snippets.set_snippets(vec![]);
    app.handle_key_event(key_with_mods(KeyCode::Char('n'), KeyModifiers::CONTROL));
    app.handle_key_event(key(KeyCode::Enter)); // Try to create with empty name

    assert_eq!(*app.snippets.mode(), SnippetMode::CreateName);
    assert!(app.notification.current().is_some());
    let notification = app.notification.current().unwrap();
    assert!(notification.message.contains("Name cannot be empty"));
}

#[test]
fn test_empty_query_shows_error_notification() {
    let mut app = app_with_query("");
    app.input.editor_mode = EditorMode::Insert;
    app.snippets.disable_persistence();

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));
    app.snippets.set_snippets(vec![]);
    app.handle_key_event(key_with_mods(KeyCode::Char('n'), KeyModifiers::CONTROL));

    app.handle_key_event(key(KeyCode::Char('T')));
    app.handle_key_event(key(KeyCode::Char('e')));
    app.handle_key_event(key(KeyCode::Char('s')));
    app.handle_key_event(key(KeyCode::Char('t')));
    app.handle_key_event(key(KeyCode::Enter)); // Try to create with empty query

    assert_eq!(*app.snippets.mode(), SnippetMode::CreateName);
    assert!(app.notification.current().is_some());
    let notification = app.notification.current().unwrap();
    assert!(notification.message.contains("Query cannot be empty"));
}

#[test]
fn test_duplicate_name_shows_error_notification() {
    let mut app = app_with_query(".test");
    app.input.editor_mode = EditorMode::Insert;
    app.snippets.disable_persistence();

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));
    app.snippets.set_snippets(vec![Snippet {
        name: "Existing".to_string(),
        query: ".foo".to_string(),
        description: None,
    }]);
    app.handle_key_event(key_with_mods(KeyCode::Char('n'), KeyModifiers::CONTROL));

    app.handle_key_event(key(KeyCode::Char('E')));
    app.handle_key_event(key(KeyCode::Char('x')));
    app.handle_key_event(key(KeyCode::Char('i')));
    app.handle_key_event(key(KeyCode::Char('s')));
    app.handle_key_event(key(KeyCode::Char('t')));
    app.handle_key_event(key(KeyCode::Char('i')));
    app.handle_key_event(key(KeyCode::Char('n')));
    app.handle_key_event(key(KeyCode::Char('g')));
    app.handle_key_event(key(KeyCode::Enter)); // Try to create

    assert_eq!(*app.snippets.mode(), SnippetMode::CreateName);
    assert!(app.notification.current().is_some());
    let notification = app.notification.current().unwrap();
    assert!(notification.message.contains("already exists"));
}

#[test]
fn test_case_insensitive_duplicate_shows_notification() {
    let mut app = app_with_query(".test");
    app.input.editor_mode = EditorMode::Insert;
    app.snippets.disable_persistence();

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));
    app.snippets.set_snippets(vec![Snippet {
        name: "MySnippet".to_string(),
        query: ".foo".to_string(),
        description: None,
    }]);
    app.handle_key_event(key_with_mods(KeyCode::Char('n'), KeyModifiers::CONTROL));

    app.handle_key_event(key(KeyCode::Char('m')));
    app.handle_key_event(key(KeyCode::Char('y')));
    app.handle_key_event(key(KeyCode::Char('s')));
    app.handle_key_event(key(KeyCode::Char('n')));
    app.handle_key_event(key(KeyCode::Char('i')));
    app.handle_key_event(key(KeyCode::Char('p')));
    app.handle_key_event(key(KeyCode::Char('p')));
    app.handle_key_event(key(KeyCode::Char('e')));
    app.handle_key_event(key(KeyCode::Char('t')));
    app.handle_key_event(key(KeyCode::Enter)); // Try to create

    assert_eq!(*app.snippets.mode(), SnippetMode::CreateName);
    assert!(app.notification.current().is_some());
}

#[test]
fn test_new_snippets_appear_at_top_of_list() {
    let mut app = app_with_query(".test");
    app.input.editor_mode = EditorMode::Insert;
    app.snippets.disable_persistence();

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));
    app.snippets.set_snippets(vec![Snippet {
        name: "Old".to_string(),
        query: ".old".to_string(),
        description: None,
    }]);
    app.handle_key_event(key_with_mods(KeyCode::Char('n'), KeyModifiers::CONTROL));

    app.handle_key_event(key(KeyCode::Char('N')));
    app.handle_key_event(key(KeyCode::Char('e')));
    app.handle_key_event(key(KeyCode::Char('w')));
    app.handle_key_event(key(KeyCode::Enter)); // Create

    assert_eq!(app.snippets.snippets()[0].name, "New");
    assert_eq!(app.snippets.selected_index(), 0);
}
