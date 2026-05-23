//! Integration tests for paste-recovery integration with App state.

use crate::app::App;
use crate::config::Config;
use crate::input::FileLoader;
use crate::input::loader::{LoaderSource, LoadingState};
use std::sync::mpsc::channel;

fn loader_with_error(source: LoaderSource) -> FileLoader {
    let (tx, rx) = channel();
    let _ = tx.send(Err(crate::error::JiqError::Io(
        "No input provided. Clipboard does not contain valid JSON.\n\nUsage:\n  jiq <file>"
            .to_string(),
    )));
    FileLoader {
        state: LoadingState::Error(crate::error::JiqError::Io("err".to_string())),
        rx: Some(rx),
        source,
    }
}

#[test]
fn clipboard_load_failure_enters_paste_recovery() {
    let mut app = App::new_with_loader(
        loader_with_error(LoaderSource::Clipboard),
        &Config::default(),
    );
    app.poll_file_loader();

    assert!(app.paste_recovery.is_some());
    assert!(app.file_loader.is_none());
    let recovery = app.paste_recovery.as_ref().unwrap();
    // Only the diagnosis line should bubble through, not the whole "Usage:" block.
    assert_eq!(
        recovery.error_message,
        "No input provided. Clipboard does not contain valid JSON."
    );
}

#[test]
fn clipboard_load_failure_does_not_show_redundant_notification() {
    // The recovery panel itself carries the "Paste JSON … press Enter"
    // instruction inline; a transient toast would just flash and
    // disappear over the same text.
    let mut app = App::new_with_loader(
        loader_with_error(LoaderSource::Clipboard),
        &Config::default(),
    );
    app.poll_file_loader();

    assert!(
        app.notification.current().is_none(),
        "no notification should fire when entering paste recovery"
    );
}

#[test]
fn file_load_failure_does_not_enter_paste_recovery() {
    let mut app = App::new_with_loader(loader_with_error(LoaderSource::File), &Config::default());
    app.poll_file_loader();

    assert!(app.paste_recovery.is_none());
    // file_loader stays for results-pane error display (existing behavior).
    assert!(app.file_loader.is_some());
}

#[test]
fn stdin_load_failure_does_not_enter_paste_recovery() {
    let mut app = App::new_with_loader(loader_with_error(LoaderSource::Stdin), &Config::default());
    app.poll_file_loader();

    assert!(app.paste_recovery.is_none());
    assert!(app.file_loader.is_some());
}

#[test]
fn accept_paste_recovery_json_initializes_query_state() {
    let mut app = App::new_with_loader(
        loader_with_error(LoaderSource::Clipboard),
        &Config::default(),
    );
    app.poll_file_loader();
    assert!(app.query.is_none(), "no query before accept");

    app.accept_paste_recovery_json(r#"{"name": "Alice"}"#.to_string());

    assert!(app.query.is_some(), "query initialised after accept");
    assert!(app.paste_recovery.is_none(), "recovery cleared");
    assert!(app.file_loader.is_none(), "file_loader cleared");
}

#[test]
fn accept_paste_recovery_json_populates_input_json_schema() {
    let mut app = App::new_with_loader(
        loader_with_error(LoaderSource::Clipboard),
        &Config::default(),
    );
    app.poll_file_loader();

    app.accept_paste_recovery_json(r#"{"name": "Alice", "age": 30}"#.to_string());

    assert!(
        app.input_json_schema.is_some(),
        "schema should be derived from accepted JSON"
    );
}

#[test]
fn accept_paste_recovery_json_shows_loaded_notification() {
    let mut app = App::new_with_loader(
        loader_with_error(LoaderSource::Clipboard),
        &Config::default(),
    );
    app.poll_file_loader();

    let json = r#"{"name": "Alice"}"#.to_string();
    let bytes = json.len();
    app.accept_paste_recovery_json(json);

    let notif = app.notification.current().expect("notification expected");
    assert!(
        notif.message.contains("Loaded"),
        "expected loaded notification, got: {}",
        notif.message
    );
    assert!(
        notif.message.contains(&bytes.to_string()),
        "notification should mention byte count, got: {}",
        notif.message
    );
}

#[test]
fn accept_paste_recovery_json_marks_dirty() {
    let mut app = App::new_with_loader(
        loader_with_error(LoaderSource::Clipboard),
        &Config::default(),
    );
    app.poll_file_loader();
    app.clear_dirty();

    app.accept_paste_recovery_json(r#"{"name": "Alice"}"#.to_string());

    assert!(app.should_render(), "render must be requested after accept");
}

#[test]
fn poll_file_loader_marks_dirty_on_clipboard_failure() {
    let mut app = App::new_with_loader(
        loader_with_error(LoaderSource::Clipboard),
        &Config::default(),
    );
    app.clear_dirty();

    app.poll_file_loader();

    assert!(app.should_render());
}
