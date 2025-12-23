use crate::app::app_render_tests::render_to_string;
use crate::test_utils::test_helpers::test_app;
use insta::assert_snapshot;

const TEST_WIDTH: u16 = 80;
const TEST_HEIGHT: u16 = 24;

#[test]
fn snapshot_processing_query_spinner_frame_0() {
    let json = r#"{"name": "Alice", "age": 30}"#;
    let mut app = test_app(json);

    app.input.textarea.insert_str(".name");
    if let Some(query_state) = &mut app.query {
        query_state.execute_async(".name");
    }

    app.frame_count = 0;

    let output = render_to_string(&mut app, TEST_WIDTH, TEST_HEIGHT);
    assert_snapshot!(output);
}

#[test]
fn snapshot_processing_query_spinner_frame_8() {
    let json = r#"{"name": "Alice", "age": 30}"#;
    let mut app = test_app(json);

    app.input.textarea.insert_str(".name");
    if let Some(query_state) = &mut app.query {
        query_state.execute_async(".name");
    }

    app.frame_count = 8;

    let output = render_to_string(&mut app, TEST_WIDTH, TEST_HEIGHT);
    assert_snapshot!(output);
}

#[test]
fn snapshot_processing_query_spinner_frame_64() {
    let json = r#"{"name": "Alice", "age": 30}"#;
    let mut app = test_app(json);

    app.input.textarea.insert_str(".name");
    if let Some(query_state) = &mut app.query {
        query_state.execute_async(".name");
    }

    app.frame_count = 64;

    let output = render_to_string(&mut app, TEST_WIDTH, TEST_HEIGHT);
    assert_snapshot!(output);
}

#[test]
fn snapshot_processing_with_previous_result() {
    let json = r#"{"name": "Alice", "age": 30}"#;
    let mut app = test_app(json);

    app.input.textarea.insert_str(".name");
    app.query.as_mut().unwrap().execute(".name");
    app.update_stats();

    app.input.textarea.delete_line_by_head();
    app.input.textarea.insert_str(".age");
    if let Some(query_state) = &mut app.query {
        query_state.execute_async(".age");
    }

    app.frame_count = 16;

    let output = render_to_string(&mut app, TEST_WIDTH, TEST_HEIGHT);
    assert_snapshot!(output);
}

#[test]
fn snapshot_processing_with_syntax_error() {
    let json = r#"{"name": "Alice", "age": 30}"#;
    let mut app = test_app(json);

    app.input.textarea.insert_str(".name");
    app.query.as_mut().unwrap().execute(".name");
    app.update_stats();

    app.input.textarea.delete_line_by_head();
    app.input.textarea.insert_str(".invalid[");
    app.query.as_mut().unwrap().execute(".invalid[");

    if let Some(query_state) = &mut app.query {
        query_state.execute_async(".invalid[");
    }

    app.frame_count = 24;

    let output = render_to_string(&mut app, TEST_WIDTH, TEST_HEIGHT);
    assert_snapshot!(output);
}
