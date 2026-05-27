use std::fs;
use std::sync::Arc;

use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tempfile::TempDir;

use crate::save::save_state::SaveMode;
use crate::test_utils::test_helpers::{TEST_JSON, key, test_app};

fn key_ctrl(c: char) -> KeyEvent {
    KeyEvent::new(KeyCode::Char(c), KeyModifiers::CONTROL)
}

fn install_result(app: &mut crate::app::App, text: &str) {
    let qs = app
        .query
        .as_mut()
        .expect("test_app should provide a query state");
    qs.last_successful_result_unformatted = Some(Arc::from(text.to_string()));
}

fn install_no_result(app: &mut crate::app::App) {
    if let Some(qs) = app.query.as_mut() {
        qs.last_successful_result_unformatted = None;
    }
}

#[test]
fn open_save_popup_with_result_advances_to_filename() {
    let mut app = test_app(TEST_JSON);
    install_result(&mut app, "{\"a\":1}");

    super::open_save_popup(&mut app);

    assert!(app.save.is_visible());
    assert_eq!(app.save.mode(), &SaveMode::EnterFilename);
}

#[test]
fn open_save_popup_with_no_result_shows_notification() {
    let mut app = test_app(TEST_JSON);
    install_no_result(&mut app);

    super::open_save_popup(&mut app);

    assert!(!app.save.is_visible(), "popup must not open without result");
    assert_eq!(app.notification.current_message(), Some("Nothing to save"));
}

#[test]
fn open_save_popup_with_empty_result_shows_notification() {
    let mut app = test_app(TEST_JSON);
    install_result(&mut app, "");

    super::open_save_popup(&mut app);

    assert!(!app.save.is_visible());
    assert_eq!(app.notification.current_message(), Some("Nothing to save"));
}

#[test]
fn esc_in_filename_closes_popup() {
    let mut app = test_app(TEST_JSON);
    install_result(&mut app, "x");
    super::open_save_popup(&mut app);

    super::handle_save_popup_key(&mut app, key(KeyCode::Esc));

    assert!(!app.save.is_visible());
}

#[test]
fn typing_marks_filename_dirty() {
    let mut app = test_app(TEST_JSON);
    install_result(&mut app, "x");
    super::open_save_popup(&mut app);

    assert!(!app.save.filename_dirty());
    super::handle_save_popup_key(&mut app, key(KeyCode::Char('a')));
    assert!(app.save.filename_dirty());
}

#[test]
fn enter_writes_file_and_shows_success_notification() {
    let dir = TempDir::new().unwrap();
    let target = dir.path().join("out.json");

    let mut app = test_app(TEST_JSON);
    install_result(&mut app, "{\"k\":1}");
    super::open_save_popup(&mut app);
    set_filename(&mut app, target.to_string_lossy().as_ref());

    super::handle_save_popup_key(&mut app, key(KeyCode::Enter));

    assert!(target.exists());
    assert_eq!(fs::read_to_string(&target).unwrap(), "{\"k\":1}");
    let msg = app.notification.current_message().unwrap_or("");
    assert!(
        msg.starts_with("Saved to "),
        "expected success notification, got {:?}",
        msg
    );
    assert!(!app.save.is_visible(), "popup should close after success");
}

#[test]
fn enter_on_existing_file_overwrites_directly() {
    let dir = TempDir::new().unwrap();
    let target = dir.path().join("out.json");
    fs::write(&target, "old").unwrap();

    let mut app = test_app(TEST_JSON);
    install_result(&mut app, "new");
    super::open_save_popup(&mut app);
    set_filename(&mut app, target.to_string_lossy().as_ref());

    super::handle_save_popup_key(&mut app, key(KeyCode::Enter));

    // Live preview already warned the user; Enter writes (popup's the warning).
    assert!(
        !app.save.is_visible(),
        "popup closes after successful write"
    );
    assert_eq!(fs::read_to_string(&target).unwrap(), "new");
}

#[test]
fn esc_before_enter_does_not_overwrite_existing_file() {
    let dir = TempDir::new().unwrap();
    let target = dir.path().join("out.json");
    fs::write(&target, "old").unwrap();

    let mut app = test_app(TEST_JSON);
    install_result(&mut app, "new");
    super::open_save_popup(&mut app);
    set_filename(&mut app, target.to_string_lossy().as_ref());

    super::handle_save_popup_key(&mut app, key(KeyCode::Esc));

    assert!(!app.save.is_visible());
    assert_eq!(fs::read_to_string(&target).unwrap(), "old");
}

#[test]
fn empty_filename_surfaces_error_notification_and_keeps_popup_open() {
    let mut app = test_app(TEST_JSON);
    install_result(&mut app, "x");
    super::open_save_popup(&mut app);
    set_filename(&mut app, "");

    super::handle_save_popup_key(&mut app, key(KeyCode::Enter));

    assert!(app.save.is_visible());
    let msg = app.notification.current_message().unwrap_or("");
    assert!(
        msg.starts_with("Save failed:"),
        "expected error, got {:?}",
        msg
    );
}

#[test]
fn write_failure_keeps_popup_open_in_filename_mode() {
    let mut app = test_app(TEST_JSON);
    install_result(&mut app, "x");
    super::open_save_popup(&mut app);
    // Force a write failure by targeting a non-existent parent directory.
    set_filename(&mut app, "/tmp/jiq-no-such-dir-xyz123/out.json");

    super::handle_save_popup_key(&mut app, key(KeyCode::Enter));

    assert!(app.save.is_visible());
    let msg = app.notification.current_message().unwrap_or("");
    assert!(msg.starts_with("Save failed:"));
}

#[test]
fn ctrl_modifier_keys_in_filename_route_to_textarea() {
    let mut app = test_app(TEST_JSON);
    install_result(&mut app, "x");
    super::open_save_popup(&mut app);
    let initial = app.save.current_filename_text();

    // Ctrl+u (kill-line-back) should clear the filename buffer in tui-textarea.
    super::handle_save_popup_key(&mut app, key_ctrl('u'));

    let after = app.save.current_filename_text();
    assert_ne!(after, initial, "Ctrl+U should mutate filename");
}

fn set_filename(app: &mut crate::app::App, text: &str) {
    let ta = app.save.filename_mut();
    while !ta.is_empty() {
        ta.delete_char();
    }
    ta.insert_str(text);
}
