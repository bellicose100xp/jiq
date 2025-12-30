use super::render_to_string;
use crate::app::App;
use crate::config::Config;
use crate::input::FileLoader;
use crate::test_utils::test_helpers::test_app;
use insta::assert_snapshot;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;

#[test]
fn test_ai_popup_not_rendered_when_file_load_fails() {
    let config = Config::default();
    let loader = FileLoader::spawn_load(PathBuf::from("/nonexistent/file.json"));
    let mut app = App::new_with_loader(loader, &config);

    app.ai.visible = true;
    app.ai.configured = true;

    thread::sleep(Duration::from_millis(100));
    app.poll_file_loader();

    assert!(
        app.query.is_none(),
        "Query should be None when file load fails"
    );

    let output = render_to_string(&mut app, 80, 24);

    assert!(
        !output.contains("Anthropic") && !output.contains("Bedrock") && !output.contains("OpenAI"),
        "AI popup should not render when query is None. Output:\n{}",
        output
    );
}

#[test]
fn test_ai_popup_not_rendered_when_query_none() {
    let mut app = test_app(r#"{"test": true}"#);

    app.query = None;
    app.ai.visible = true;
    app.ai.configured = true;

    let output = render_to_string(&mut app, 80, 24);

    assert!(
        !output.contains("Anthropic") && !output.contains("Bedrock") && !output.contains("OpenAI"),
        "AI popup should not render when query is None"
    );
}

#[test]
fn test_ai_popup_renders_when_query_exists() {
    let mut app = test_app(r#"{"test": true}"#);

    app.ai.visible = true;
    app.ai.configured = true;

    let output = render_to_string(&mut app, 120, 30);

    assert!(
        output.contains("Not Configured"),
        "AI popup should render when query exists (showing 'Not Configured' since no provider is set)"
    );
}

#[test]
fn snapshot_file_load_error_with_notification() {
    let config = Config::default();
    let loader = FileLoader::spawn_load(PathBuf::from("/nonexistent/file.json"));
    let mut app = App::new_with_loader(loader, &config);

    thread::sleep(Duration::from_millis(100));
    app.poll_file_loader();

    let output = render_to_string(&mut app, 80, 24);
    assert_snapshot!(output);
}

#[test]
fn snapshot_file_load_error_full_details_in_results_area() {
    let config = Config::default();
    let loader = FileLoader::spawn_load(PathBuf::from("/nonexistent/file.json"));
    let mut app = App::new_with_loader(loader, &config);

    thread::sleep(Duration::from_millis(100));
    app.poll_file_loader();

    let output = render_to_string(&mut app, 120, 30);

    assert!(
        output.contains("Failed to load file"),
        "Results area should show error message"
    );
    assert!(
        output.contains("Error"),
        "Results area should have Error title"
    );

    assert_snapshot!(output);
}

#[test]
fn test_notification_shows_brief_error_message() {
    let config = Config::default();
    let loader = FileLoader::spawn_load(PathBuf::from("/nonexistent/file.json"));
    let mut app = App::new_with_loader(loader, &config);

    thread::sleep(Duration::from_millis(100));
    app.poll_file_loader();

    let notification = app.notification.current();
    assert!(
        notification.is_some(),
        "Notification should be shown on file load error"
    );

    if let Some(notif) = notification {
        assert_eq!(
            notif.message, "Failed to load file",
            "Notification should show brief error message"
        );
    }
}
