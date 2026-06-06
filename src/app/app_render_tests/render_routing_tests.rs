//! App::render branch-selection / region-capture integration tests.
//!
//! These exercise the wiring inside `App::render` -- which sub-view fires
//! and which `layout_regions` rects get populated for mouse hit-testing --
//! rather than the leaf render modules, which have their own unit tests.

use super::render_to_string;
use crate::app::App;
use crate::config::Config;
use crate::input::FileLoader;
use crate::input::loader::{ClipboardPeek, LoaderSource, LoadingState};
use crate::input::source_picker::SourcePickerState;
use crate::notification::NotificationState;
use crate::snippets::{Snippet, SnippetState};
use crate::test_utils::test_helpers::test_app;
use insta::assert_snapshot;
use std::sync::mpsc::channel;

const TEST_WIDTH: u16 = 80;
const TEST_HEIGHT: u16 = 24;
const PICKER_JSON: &str = r#"{"a":1}"#;

/// A source-picker App whose launch-time clipboard peek returned usable
/// JSON, with the notification timer reset for snapshot stability.
fn app_with_source_picker() -> App {
    let state = SourcePickerState::from_peek(ClipboardPeek::Usable(PICKER_JSON.to_string()));
    let mut app = App::new_with_source_picker(state, &Config::default());
    app.notification = NotificationState::new();
    app
}

/// A paste-recovery App (clipboard load failed), notification reset.
fn app_in_recovery() -> App {
    let (tx, rx) = channel();
    let _ = tx.send(Err(crate::error::JiqError::Io(
        "No input provided. Clipboard does not contain valid JSON.".to_string(),
    )));
    let loader = FileLoader {
        state: LoadingState::Error(crate::error::JiqError::Io("err".to_string())),
        rx: Some(rx),
        source: LoaderSource::Clipboard,
    };
    let mut app = App::new_with_loader(loader, &Config::default());
    app.poll_file_loader();
    app.notification = NotificationState::new();
    app
}

#[test]
fn snapshot_source_picker_renders_full_screen() {
    let mut app = app_with_source_picker();
    let output = render_to_string(&mut app, TEST_WIDTH, TEST_HEIGHT);

    // render() short-circuits to the picker and never lays out the normal
    // input pane, so the "Query [INSERT" mode indicator must be absent.
    assert!(
        !output.contains("Query [INSERT"),
        "source picker view must skip the normal Query input pane"
    );
    assert_snapshot!(output);
}

#[test]
fn source_picker_with_help_visible_captures_help_region() {
    let mut app = app_with_source_picker();
    app.help.visible = true;

    // No help region recorded before rendering.
    assert!(app.layout_regions.help_popup.is_none());

    let _ = render_to_string(&mut app, TEST_WIDTH, TEST_HEIGHT);

    // F1/help works from the picker: the help popup renders on top and its
    // rect is captured for mouse hit-testing.
    assert!(
        app.layout_regions.help_popup.is_some(),
        "help popup region must be captured when help is visible over the picker"
    );
}

#[test]
fn paste_recovery_with_help_visible_captures_help_region() {
    let mut app = app_in_recovery();
    app.help.visible = true;

    assert!(app.layout_regions.help_popup.is_none());

    let _ = render_to_string(&mut app, TEST_WIDTH, TEST_HEIGHT);

    // The help overlay composites over the recovery view and its rect is
    // captured (F1-from-recovery branch).
    assert!(
        app.layout_regions.help_popup.is_some(),
        "help popup region must be captured when help is visible over recovery"
    );
}

#[test]
fn snippets_popup_renders_and_captures_list_and_preview_regions() {
    let json = r#"{"name": "Alice", "age": 30}"#;
    let mut app = test_app(json);

    // Use the non-persisting constructor so open() does not reload the
    // developer's on-disk snippet file, keeping the snapshot deterministic.
    let mut snippets = SnippetState::new_without_persistence();
    snippets.set_snippets(vec![
        Snippet {
            name: "All keys".to_string(),
            query: "keys".to_string(),
            description: Some("List the object's keys".to_string()),
        },
        Snippet {
            name: "Names".to_string(),
            query: ".[].name".to_string(),
            description: None,
        },
    ]);
    snippets.open();
    app.snippets = snippets;
    app.notification = NotificationState::new();

    // Browse mode with populated snippets is the only path that captures
    // both the list and preview rects.
    assert!(app.snippets.is_visible());
    assert!(app.layout_regions.snippet_list.is_none());
    assert!(app.layout_regions.snippet_preview.is_none());

    let output = render_to_string(&mut app, TEST_WIDTH, TEST_HEIGHT);

    assert!(
        app.layout_regions.snippet_list.is_some(),
        "snippet list region must be captured for mouse hit-testing"
    );
    assert!(
        app.layout_regions.snippet_preview.is_some(),
        "snippet preview region must be captured for mouse hit-testing"
    );
    assert_snapshot!(output);
}

#[test]
fn save_popup_renders_over_results() {
    let json = r#"{"name": "Alice", "age": 30}"#;
    let mut app = test_app(json);
    app.query.as_mut().unwrap().execute(".");
    app.save.open("2026-01-01_00-00-00".to_string());
    app.notification = NotificationState::new();

    assert!(app.save.is_visible());

    let output = render_to_string(&mut app, TEST_WIDTH, TEST_HEIGHT);
    // The save overlay composites on top of the normal results layout. Assert
    // on the popup chrome rather than a full-screen snapshot, since the
    // resolved-path line is cwd-dependent and not portable across machines.
    assert!(
        output.contains("Save Result to file"),
        "save popup title must render over the results area"
    );
    assert!(
        output.contains("Enter Save"),
        "save popup action hints must render"
    );
    // The normal results pane is still laid out underneath the popup.
    assert!(
        output.contains("Object"),
        "results pane must still render beneath the save popup"
    );
}
