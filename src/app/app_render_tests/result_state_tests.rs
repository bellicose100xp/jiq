use crate::app::app_render_tests::render_to_string;
use crate::app::app_state::Focus;
use crate::test_utils::test_helpers::test_app;
use insta::assert_snapshot;

const TEST_WIDTH: u16 = 120;
const TEST_HEIGHT: u16 = 30;

#[test]
fn snapshot_results_success_focused() {
    let json = r#"[{"name": "svc1"}, {"name": "svc2"}, {"name": "svc3"}]"#;
    let mut app = test_app(json);
    app.input.textarea.insert_str(".[].name");
    app.query.as_mut().unwrap().execute(".[].name");
    app.update_stats();
    app.focus = Focus::ResultsPane;

    let output = render_to_string(&mut app, TEST_WIDTH, TEST_HEIGHT);
    assert_snapshot!(output);
}

#[test]
fn snapshot_results_success_unfocused() {
    let json = r#"[{"name": "svc1"}, {"name": "svc2"}, {"name": "svc3"}]"#;
    let mut app = test_app(json);
    app.input.textarea.insert_str(".[].name");
    app.query.as_mut().unwrap().execute(".[].name");
    app.update_stats();
    app.focus = Focus::InputField;

    let output = render_to_string(&mut app, TEST_WIDTH, TEST_HEIGHT);
    assert_snapshot!(output);
}

#[test]
fn snapshot_results_empty_focused() {
    let json = r#"[{"name": "svc1"}, {"name": "svc2"}]"#;
    let mut app = test_app(json);

    app.input
        .textarea
        .insert_str(".[] | select(.name == \"nonexistent\")");
    app.query
        .as_mut()
        .unwrap()
        .execute(".[] | select(.name == \"nonexistent\")");
    app.update_stats();
    app.focus = Focus::ResultsPane;

    let output = render_to_string(&mut app, TEST_WIDTH, TEST_HEIGHT);
    assert_snapshot!(output);
}

#[test]
fn snapshot_results_empty_unfocused() {
    let json = r#"[{"name": "svc1"}, {"name": "svc2"}]"#;
    let mut app = test_app(json);

    app.input
        .textarea
        .insert_str(".[] | select(.name == \"nonexistent\")");
    app.query
        .as_mut()
        .unwrap()
        .execute(".[] | select(.name == \"nonexistent\")");
    app.update_stats();
    app.focus = Focus::InputField;

    let output = render_to_string(&mut app, TEST_WIDTH, TEST_HEIGHT);
    assert_snapshot!(output);
}

#[test]
fn snapshot_results_error_focused() {
    let json = r#"{"test": true}"#;
    let mut app = test_app(json);

    app.input.textarea.insert_str(".invalid syntax here");
    app.query.as_mut().unwrap().execute(".invalid syntax here");
    app.update_stats();
    app.focus = Focus::ResultsPane;

    let output = render_to_string(&mut app, TEST_WIDTH, TEST_HEIGHT);
    assert_snapshot!(output);
}

#[test]
fn snapshot_results_error_unfocused() {
    let json = r#"{"test": true}"#;
    let mut app = test_app(json);

    app.input.textarea.insert_str(".invalid syntax here");
    app.query.as_mut().unwrap().execute(".invalid syntax here");
    app.update_stats();
    app.focus = Focus::InputField;

    let output = render_to_string(&mut app, TEST_WIDTH, TEST_HEIGHT);
    assert_snapshot!(output);
}
