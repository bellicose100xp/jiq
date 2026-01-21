use crate::app::app_render_tests::render_to_string;
use crate::app::app_state::Focus;
use crate::editor::EditorMode;
use crate::test_utils::test_helpers::test_app;
use insta::assert_snapshot;

const TEST_WIDTH: u16 = 80;
const TEST_HEIGHT: u16 = 24;

#[test]
fn snapshot_initial_ui_empty_query() {
    let json = r#"{"name": "Alice", "age": 30}"#;
    let mut app = test_app(json);

    let output = render_to_string(&mut app, TEST_WIDTH, TEST_HEIGHT);
    assert_snapshot!(output);
}

#[test]
fn snapshot_ui_with_query() {
    let json = r#"{"name": "Alice", "age": 30}"#;
    let mut app = test_app(json);
    app.input.textarea.insert_str(".name");
    app.query.as_mut().unwrap().execute(".name");
    app.update_stats();

    let output = render_to_string(&mut app, TEST_WIDTH, TEST_HEIGHT);
    assert_snapshot!(output);
}

#[test]
fn snapshot_ui_with_array_data() {
    let json = r#"[{"name": "Alice"}, {"name": "Bob"}, {"name": "Charlie"}]"#;
    let mut app = test_app(json);
    app.input.textarea.insert_str(".[].name");
    app.query.as_mut().unwrap().execute(".[].name");
    app.update_stats();

    let output = render_to_string(&mut app, TEST_WIDTH, TEST_HEIGHT);
    assert_snapshot!(output);
}

#[test]
fn snapshot_ui_input_focused() {
    let json = r#"{"test": true}"#;
    let mut app = test_app(json);
    app.focus = Focus::InputField;

    let output = render_to_string(&mut app, TEST_WIDTH, TEST_HEIGHT);
    assert_snapshot!(output);
}

#[test]
fn snapshot_ui_results_focused() {
    let json = r#"{"test": true}"#;
    let mut app = test_app(json);
    app.focus = Focus::ResultsPane;

    let output = render_to_string(&mut app, TEST_WIDTH, TEST_HEIGHT);
    assert_snapshot!(output);
}

#[test]
fn snapshot_ui_insert_mode() {
    let json = r#"{"test": true}"#;
    let mut app = test_app(json);
    app.input.editor_mode = EditorMode::Insert;

    let output = render_to_string(&mut app, TEST_WIDTH, TEST_HEIGHT);
    assert_snapshot!(output);
}

#[test]
fn snapshot_ui_normal_mode() {
    let json = r#"{"test": true}"#;
    let mut app = test_app(json);
    app.input.editor_mode = EditorMode::Normal;

    let output = render_to_string(&mut app, TEST_WIDTH, TEST_HEIGHT);
    assert_snapshot!(output);
}

#[test]
fn snapshot_ui_operator_mode() {
    let json = r#"{"test": true}"#;
    let mut app = test_app(json);
    app.input.editor_mode = EditorMode::Operator('d');

    let output = render_to_string(&mut app, TEST_WIDTH, TEST_HEIGHT);
    assert_snapshot!(output);
}

#[test]
fn snapshot_ui_with_error() {
    let json = r#"5"#;
    let mut app = test_app(json);
    app.input.textarea.insert_str(".foo");
    app.query.as_mut().unwrap().execute(".foo");

    let output = render_to_string(&mut app, TEST_WIDTH, TEST_HEIGHT);
    assert_snapshot!(output);
}

#[test]
fn snapshot_ui_error_overlay_visible() {
    let json = r#"5"#;
    let mut app = test_app(json);
    app.input.textarea.insert_str(".foo");
    app.query.as_mut().unwrap().execute(".foo");
    app.error_overlay_visible = true;

    let output = render_to_string(&mut app, TEST_WIDTH, TEST_HEIGHT);
    assert_snapshot!(output);
}

#[test]
fn snapshot_ui_small_terminal() {
    let json = r#"{"name": "Alice"}"#;
    let mut app = test_app(json);

    let output = render_to_string(&mut app, 40, 10);
    assert_snapshot!(output);
}

#[test]
fn snapshot_ui_wide_terminal() {
    let json = r#"{"name": "Alice"}"#;
    let mut app = test_app(json);

    let output = render_to_string(&mut app, 120, 30);
    assert_snapshot!(output);
}

#[test]
fn snapshot_error_overlay() {
    let json = r#"{"test": true}"#;
    let mut app = test_app(json);

    app.query.as_mut().unwrap().result =
        Err("jq: compile error: syntax error at line 1".to_string());
    app.error_overlay_visible = true;

    let output = render_to_string(&mut app, TEST_WIDTH, TEST_HEIGHT);
    assert_snapshot!(output);
}

#[test]
fn snapshot_results_pane_with_syntax_error_unfocused() {
    let json = r#"{"name": "Alice", "age": 30}"#;
    let mut app = test_app(json);

    app.input.textarea.insert_str(".name");
    app.query.as_mut().unwrap().execute(".name");
    app.update_stats();

    app.input.textarea.delete_line_by_head();
    app.input.textarea.insert_str(".invalid[");
    app.query.as_mut().unwrap().execute(".invalid[");

    app.focus = Focus::InputField;

    let output = render_to_string(&mut app, TEST_WIDTH, TEST_HEIGHT);
    assert_snapshot!(output);
}

#[test]
fn snapshot_results_pane_with_syntax_error_focused() {
    let json = r#"{"name": "Alice", "age": 30}"#;
    let mut app = test_app(json);

    app.input.textarea.insert_str(".name");
    app.query.as_mut().unwrap().execute(".name");
    app.update_stats();

    app.input.textarea.delete_line_by_head();
    app.input.textarea.insert_str(".invalid[");
    app.query.as_mut().unwrap().execute(".invalid[");

    app.focus = Focus::ResultsPane;

    let output = render_to_string(&mut app, TEST_WIDTH, TEST_HEIGHT);
    assert_snapshot!(output);
}

#[test]
fn snapshot_results_pane_with_success_unfocused() {
    let json = r#"{"name": "Alice", "age": 30}"#;
    let mut app = test_app(json);

    app.input.textarea.insert_str(".name");
    app.query.as_mut().unwrap().execute(".name");
    app.update_stats();

    app.focus = Focus::InputField;

    let output = render_to_string(&mut app, TEST_WIDTH, TEST_HEIGHT);
    assert_snapshot!(output);
}

#[test]
fn snapshot_results_pane_with_success_focused() {
    let json = r#"{"name": "Alice", "age": 30}"#;
    let mut app = test_app(json);

    app.input.textarea.insert_str(".name");
    app.query.as_mut().unwrap().execute(".name");
    app.update_stats();

    app.focus = Focus::ResultsPane;

    let output = render_to_string(&mut app, TEST_WIDTH, TEST_HEIGHT);
    assert_snapshot!(output);
}

#[test]
fn snapshot_stats_bar_array_focused() {
    let json = r#"[{"id": 1}, {"id": 2}, {"id": 3}]"#;
    let mut app = test_app(json);

    app.query.as_mut().unwrap().execute(".");

    app.focus = Focus::ResultsPane;

    let output = render_to_string(&mut app, TEST_WIDTH, TEST_HEIGHT);
    assert_snapshot!(output);
}

#[test]
fn snapshot_stats_bar_object_unfocused() {
    let json = r#"{"name": "Alice", "age": 30}"#;
    let mut app = test_app(json);

    app.query.as_mut().unwrap().execute(".");

    app.focus = Focus::InputField;

    let output = render_to_string(&mut app, TEST_WIDTH, TEST_HEIGHT);
    assert_snapshot!(output);
}

#[test]
fn snapshot_stats_bar_error_shows_last_stats() {
    let json = r#"[1, 2, 3, 4, 5]"#;
    let mut app = test_app(json);

    app.query.as_mut().unwrap().execute(".");

    app.input.textarea.insert_str(".invalid[");
    app.query.as_mut().unwrap().execute(".invalid[");

    app.focus = Focus::InputField;

    let output = render_to_string(&mut app, TEST_WIDTH, TEST_HEIGHT);
    assert_snapshot!(output);
}
