use super::render_to_string;
use crate::app::App;
use crate::config::Config;
use crate::input::FileLoader;
use crate::input::loader::{LoaderSource, LoadingState};
use insta::assert_snapshot;
use std::sync::mpsc::channel;

fn app_in_recovery() -> App {
    let (tx, rx) = channel();
    let _ = tx.send(Err(crate::error::JiqError::Io(
        "No input provided. Clipboard does not contain valid JSON.\n\nUsage:\n  jiq <file>"
            .to_string(),
    )));
    let loader = FileLoader {
        state: LoadingState::Error(crate::error::JiqError::Io("err".to_string())),
        rx: Some(rx),
        source: LoaderSource::Clipboard,
    };
    let mut app = App::new_with_loader(loader, &Config::default());
    app.poll_file_loader();
    app
}

#[test]
fn snapshot_paste_recovery_initial() {
    let mut app = app_in_recovery();
    // Suppress notification timer so snapshot is stable.
    app.notification = crate::notification::NotificationState::new();
    let output = render_to_string(&mut app, 80, 24);
    assert_snapshot!(output);
}

#[test]
fn snapshot_paste_recovery_after_invalid_paste() {
    let mut app = app_in_recovery();
    app.notification = crate::notification::NotificationState::new();
    app.input.textarea.insert_str("{not json");
    let raw = app.input.textarea.lines().join("\n");
    if let Some(r) = app.paste_recovery.as_mut() {
        let _ = r.try_submit(&raw);
    }
    let output = render_to_string(&mut app, 80, 24);
    assert_snapshot!(output);
}

#[test]
fn snapshot_paste_recovery_with_typed_content() {
    let mut app = app_in_recovery();
    app.notification = crate::notification::NotificationState::new();
    app.input
        .textarea
        .insert_str("{\n  \"name\": \"Alice\",\n  \"age\": 30\n}");
    let output = render_to_string(&mut app, 80, 24);
    assert_snapshot!(output);
}

#[test]
fn snapshot_paste_recovery_in_normal_mode_shows_only_insert_toggle() {
    // While in Normal mode the bottom-border hints should advertise the
    // Insert toggle, NOT both. This snapshot pins that.
    let mut app = app_in_recovery();
    app.notification = crate::notification::NotificationState::new();
    app.input.textarea.insert_str(r#"{"a":1}"#);
    app.input.editor_mode = crate::editor::EditorMode::Normal;
    let output = render_to_string(&mut app, 80, 24);
    assert_snapshot!(output);
}

#[test]
fn snapshot_paste_recovery_narrow_width_does_not_panic() {
    let mut app = app_in_recovery();
    app.notification = crate::notification::NotificationState::new();
    let _ = render_to_string(&mut app, 40, 16);
}

#[test]
fn paste_recovery_does_not_render_normal_input_or_results() {
    let mut app = app_in_recovery();
    let output = render_to_string(&mut app, 80, 24);
    // Normal layout has " Query [" mode indicator on the input field
    // and a results border. None of those should appear in recovery.
    assert!(
        !output.contains("Query [INSERT"),
        "recovery view must hide the normal Query input"
    );
    assert!(
        !output.contains("Showing last successful result"),
        "recovery view must hide the results pane"
    );
    assert!(
        output.contains("Paste"),
        "recovery view should mention 'Paste'"
    );
}

#[test]
fn paste_recovery_clears_after_accept() {
    let mut app = app_in_recovery();
    app.accept_paste_recovery_json(r#"{"name": "Bob"}"#.to_string());
    assert!(app.paste_recovery.is_none());
    // Now the normal layout should render.
    let output = render_to_string(&mut app, 80, 24);
    // Normal layout renders the Query input title.
    assert!(output.contains("Query"));
}
