use crate::app::App;
use crate::editor::EditorMode;
use crate::snippets::{Snippet, SnippetMode};
use crate::test_utils::test_helpers::{app_with_query, key, key_with_mods};
use crossterm::event::{KeyCode, KeyModifiers};

fn editor_with(query: &str, snippet: Snippet) -> App {
    let mut app = app_with_query(query);
    app.input.editor_mode = EditorMode::Insert;
    app.snippets.disable_persistence();

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));
    app.snippets.set_snippets(vec![snippet]);
    app.snippets.on_search_input_changed();
    app
}

fn enter_edit_query(query: &str, stored_query: &str) -> App {
    let mut app = editor_with(
        query,
        Snippet {
            name: "My Snippet".to_string(),
            query: stored_query.to_string(),
            description: Some("A desc".to_string()),
        },
    );
    app.handle_key_event(key_with_mods(KeyCode::Char('e'), KeyModifiers::CONTROL));
    app.handle_key_event(key(KeyCode::Tab)); // EditName -> EditQuery
    assert!(matches!(app.snippets.mode(), SnippetMode::EditQuery { .. }));
    app
}

fn enter_edit_description(query: &str, stored_query: &str) -> App {
    let mut app = enter_edit_query(query, stored_query);
    app.handle_key_event(key(KeyCode::Tab)); // EditQuery -> EditDescription
    assert!(matches!(
        app.snippets.mode(),
        SnippetMode::EditDescription { .. }
    ));
    app
}

fn enter_create_query(query: &str) -> App {
    let mut app = app_with_query(query);
    app.input.editor_mode = EditorMode::Insert;
    app.snippets.disable_persistence();

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));
    app.snippets.set_snippets(vec![]);
    app.handle_key_event(key_with_mods(KeyCode::Char('n'), KeyModifiers::CONTROL));
    app.handle_key_event(key(KeyCode::Tab)); // CreateName -> CreateQuery
    assert_eq!(*app.snippets.mode(), SnippetMode::CreateQuery);
    app
}

fn clear_query(app: &mut App) {
    while !app.snippets.query_input().is_empty() {
        app.handle_key_event(key(KeyCode::Backspace));
    }
}

#[test]
fn test_esc_in_edit_query_mode_cancels() {
    let mut app = enter_edit_query(".test", ".q");

    app.handle_key_event(key(KeyCode::Esc));

    assert_eq!(*app.snippets.mode(), SnippetMode::Browse);
    assert!(app.snippets.is_visible());
    assert_eq!(app.snippets.snippets()[0].query, ".q");
}

#[test]
fn test_enter_in_edit_query_mode_saves_and_exits() {
    let mut app = enter_edit_query(".test", ".old");

    clear_query(&mut app);
    app.handle_key_event(key(KeyCode::Char('.')));
    app.handle_key_event(key(KeyCode::Char('n')));
    app.handle_key_event(key(KeyCode::Char('e')));
    app.handle_key_event(key(KeyCode::Char('w')));
    app.handle_key_event(key(KeyCode::Enter));

    assert_eq!(*app.snippets.mode(), SnippetMode::Browse);
    assert_eq!(app.snippets.snippets()[0].query, ".new");
}

#[test]
fn test_enter_in_edit_query_empty_shows_error() {
    let mut app = enter_edit_query(".test", ".old");

    clear_query(&mut app);
    app.handle_key_event(key(KeyCode::Enter));

    assert!(matches!(app.snippets.mode(), SnippetMode::EditQuery { .. }));
    assert_eq!(app.snippets.snippets()[0].query, ".old");
    assert!(app.notification.current().is_some());
    let notification = app.notification.current().unwrap();
    assert!(notification.message.contains("Query cannot be empty"));
}

#[test]
fn test_backtab_in_edit_query_navigates_to_name() {
    let mut app = enter_edit_query(".test", ".q");

    app.handle_key_event(key(KeyCode::BackTab));

    assert!(matches!(
        app.snippets.mode(),
        SnippetMode::EditName { original_name } if original_name == "My Snippet"
    ));
    assert_eq!(app.snippets.snippets()[0].name, "My Snippet");
}

#[test]
fn test_tab_in_edit_query_empty_shows_error() {
    let mut app = enter_edit_query(".test", ".old");

    clear_query(&mut app);
    app.handle_key_event(key(KeyCode::Tab));

    assert!(matches!(app.snippets.mode(), SnippetMode::EditQuery { .. }));
    assert!(app.notification.current().is_some());
    let notification = app.notification.current().unwrap();
    assert!(notification.message.contains("Query cannot be empty"));
}

#[test]
fn test_esc_in_edit_description_mode_cancels() {
    let mut app = enter_edit_description(".test", ".q");

    app.handle_key_event(key(KeyCode::Esc));

    assert_eq!(*app.snippets.mode(), SnippetMode::Browse);
    assert!(app.snippets.is_visible());
}

#[test]
fn test_tab_in_edit_description_cycles_to_name() {
    let mut app = enter_edit_description(".test", ".q");

    app.handle_key_event(key(KeyCode::Tab));

    assert!(matches!(
        app.snippets.mode(),
        SnippetMode::EditName { original_name } if original_name == "My Snippet"
    ));
}

#[test]
fn test_backtab_in_edit_description_navigates_to_query() {
    let mut app = enter_edit_description(".test", ".q");

    app.handle_key_event(key(KeyCode::BackTab));

    assert!(matches!(
        app.snippets.mode(),
        SnippetMode::EditQuery { original_query } if original_query == ".q"
    ));
}

#[test]
fn test_backtab_in_edit_name_navigates_to_description() {
    let mut app = editor_with(
        ".test",
        Snippet {
            name: "My Snippet".to_string(),
            query: ".q".to_string(),
            description: Some("A desc".to_string()),
        },
    );
    app.handle_key_event(key_with_mods(KeyCode::Char('e'), KeyModifiers::CONTROL));
    assert!(matches!(app.snippets.mode(), SnippetMode::EditName { .. }));

    app.handle_key_event(key(KeyCode::BackTab));

    assert!(matches!(
        app.snippets.mode(),
        SnippetMode::EditDescription { .. }
    ));
    assert_eq!(app.snippets.snippets()[0].name, "My Snippet");
}

#[test]
fn test_tab_in_edit_name_empty_shows_error() {
    let mut app = editor_with(
        ".test",
        Snippet {
            name: "My Snippet".to_string(),
            query: ".q".to_string(),
            description: Some("A desc".to_string()),
        },
    );
    app.handle_key_event(key_with_mods(KeyCode::Char('e'), KeyModifiers::CONTROL));

    while !app.snippets.name_input().is_empty() {
        app.handle_key_event(key(KeyCode::Backspace));
    }
    app.handle_key_event(key(KeyCode::Tab));

    assert!(matches!(app.snippets.mode(), SnippetMode::EditName { .. }));
    assert!(app.notification.current().is_some());
    let notification = app.notification.current().unwrap();
    assert!(notification.message.contains("empty"));
}

#[test]
fn test_esc_in_create_query_mode_cancels() {
    let mut app = enter_create_query(".test");

    app.handle_key_event(key(KeyCode::Esc));

    assert_eq!(*app.snippets.mode(), SnippetMode::Browse);
    assert!(app.snippets.is_visible());
}

#[test]
fn test_typing_in_create_query_mode_edits_query() {
    let mut app = enter_create_query(".test");

    clear_query(&mut app);
    app.handle_key_event(key(KeyCode::Char('x')));

    assert_eq!(app.snippets.query_input(), "x");
    assert_eq!(*app.snippets.mode(), SnippetMode::CreateQuery);
}

#[test]
fn test_backtab_in_create_query_mode_goes_back_to_name() {
    let mut app = enter_create_query(".test");

    app.handle_key_event(key(KeyCode::BackTab));

    assert_eq!(*app.snippets.mode(), SnippetMode::CreateName);
}

#[test]
fn test_enter_in_create_query_empty_name_shows_error() {
    let mut app = enter_create_query(".test");

    app.handle_key_event(key(KeyCode::Enter));

    assert_eq!(*app.snippets.mode(), SnippetMode::CreateQuery);
    assert!(app.notification.current().is_some());
    let notification = app.notification.current().unwrap();
    assert!(notification.message.contains("Name cannot be empty"));
}
