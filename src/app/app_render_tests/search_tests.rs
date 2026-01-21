use crate::app::app_render_tests::render_to_string;
use crate::app::app_state::Focus;
use crate::test_utils::test_helpers::test_app;
use insta::assert_snapshot;

const TEST_WIDTH: u16 = 80;
const TEST_HEIGHT: u16 = 24;

#[test]
fn snapshot_search_bar_visible() {
    let json = r#"{"name": "Alice", "email": "alice@example.com", "role": "admin"}"#;
    let mut app = test_app(json);

    app.query.as_mut().unwrap().execute(".");

    app.search.open();
    app.search.search_textarea_mut().insert_str("alice");

    if let Some(content) = &app
        .query
        .as_ref()
        .unwrap()
        .last_successful_result_unformatted
    {
        app.search.update_matches(content);
    }

    app.focus = Focus::ResultsPane;

    let output = render_to_string(&mut app, TEST_WIDTH, TEST_HEIGHT);
    assert_snapshot!(output);
}

#[test]
fn snapshot_search_bar_with_match_count() {
    let json = r#"[{"name": "alice"}, {"name": "bob"}, {"name": "alice"}]"#;
    let mut app = test_app(json);

    app.query.as_mut().unwrap().execute(".");

    app.search.open();
    app.search.search_textarea_mut().insert_str("alice");

    if let Some(content) = &app
        .query
        .as_ref()
        .unwrap()
        .last_successful_result_unformatted
    {
        app.search.update_matches(content);
    }

    app.focus = Focus::ResultsPane;

    let output = render_to_string(&mut app, TEST_WIDTH, TEST_HEIGHT);
    assert_snapshot!(output);
}

#[test]
fn snapshot_search_bar_no_matches() {
    let json = r#"{"name": "Alice", "age": 30}"#;
    let mut app = test_app(json);

    app.query.as_mut().unwrap().execute(".");

    app.search.open();
    app.search.search_textarea_mut().insert_str("xyz");

    if let Some(content) = &app
        .query
        .as_ref()
        .unwrap()
        .last_successful_result_unformatted
    {
        app.search.update_matches(content);
    }

    app.focus = Focus::ResultsPane;

    let output = render_to_string(&mut app, TEST_WIDTH, TEST_HEIGHT);
    assert_snapshot!(output);
}

#[test]
fn snapshot_search_with_highlighted_matches() {
    let json =
        r#"[{"name": "alice", "email": "alice@test.com"}, {"name": "bob"}, {"name": "alice"}]"#;
    let mut app = test_app(json);

    app.query.as_mut().unwrap().execute(".");

    app.search.open();
    app.search.search_textarea_mut().insert_str("alice");

    if let Some(content) = &app
        .query
        .as_ref()
        .unwrap()
        .last_successful_result_unformatted
    {
        app.search.update_matches(content);
    }

    app.search.next_match();

    app.focus = Focus::ResultsPane;

    let output = render_to_string(&mut app, TEST_WIDTH, TEST_HEIGHT);
    assert_snapshot!(output);
}

#[test]
fn snapshot_search_with_horizontal_scroll() {
    let long_value = format!("{}match_here", " ".repeat(150));
    let json = format!(
        r#"{{"short": "value", "very_long_field": "{}"}}"#,
        long_value
    );
    let mut app = test_app(&json);

    app.query.as_mut().unwrap().execute(".");

    app.results_scroll.viewport_width = 80;
    app.results_scroll.viewport_height = 20;

    app.search.open();
    app.search.search_textarea_mut().insert_str("match_here");

    if let Some(content) = &app
        .query
        .as_ref()
        .unwrap()
        .last_successful_result_unformatted
    {
        app.search.update_matches(content);

        let max_line_width = content.lines().map(|l| l.len()).max().unwrap_or(0) as u16;
        app.results_scroll
            .update_h_bounds(max_line_width, app.results_scroll.viewport_width);
    }

    app.search.confirm();
    if let Some(line) = app.search.next_match()
        && let Some(current_match) = app.search.current_match()
    {
        let target_col = current_match.col;
        let match_len = current_match.len;
        let h_offset = app.results_scroll.h_offset;
        let max_h_offset = app.results_scroll.max_h_offset;
        let viewport_width = app.results_scroll.viewport_width;

        if max_h_offset > 0 && viewport_width > 0 {
            let match_end = target_col.saturating_add(match_len);
            let visible_h_start = h_offset;
            let visible_h_end = h_offset.saturating_add(viewport_width);

            if target_col < visible_h_start || match_end > visible_h_end {
                let left_margin: u16 = 10;
                let new_h_offset = target_col.saturating_sub(left_margin);
                app.results_scroll.h_offset = new_h_offset.min(max_h_offset);
            }
        }

        let target_line = line.min(u16::MAX as u32) as u16;
        let viewport_height = app.results_scroll.viewport_height;
        let current_offset = app.results_scroll.offset;
        let max_offset = app.results_scroll.max_offset;

        if viewport_height > 0 && max_offset > 0 {
            let visible_start = current_offset;
            let visible_end = current_offset.saturating_add(viewport_height);

            if target_line < visible_start || target_line >= visible_end {
                let half_viewport = viewport_height / 2;
                let new_offset = target_line.saturating_sub(half_viewport);
                app.results_scroll.offset = new_offset.min(max_offset);
            }
        }
    }

    app.focus = Focus::ResultsPane;

    let output = render_to_string(&mut app, TEST_WIDTH, TEST_HEIGHT);
    assert_snapshot!(output);
}

#[test]
fn snapshot_search_bar_active_state() {
    let json = r#"{"name": "Alice", "email": "alice@example.com"}"#;
    let mut app = test_app(json);

    app.query.as_mut().unwrap().execute(".");

    app.search.open();
    app.search.search_textarea_mut().insert_str("alice");

    if let Some(content) = &app
        .query
        .as_ref()
        .unwrap()
        .last_successful_result_unformatted
    {
        app.search.update_matches(content);
    }

    app.focus = Focus::ResultsPane;

    let output = render_to_string(&mut app, TEST_WIDTH, TEST_HEIGHT);
    assert_snapshot!(output);
}

#[test]
fn snapshot_search_bar_inactive_state() {
    let json = r#"{"name": "Alice", "email": "alice@example.com"}"#;
    let mut app = test_app(json);

    app.query.as_mut().unwrap().execute(".");

    app.search.open();
    app.search.search_textarea_mut().insert_str("alice");

    if let Some(content) = &app
        .query
        .as_ref()
        .unwrap()
        .last_successful_result_unformatted
    {
        app.search.update_matches(content);
    }

    app.search.confirm();

    app.focus = Focus::ResultsPane;

    let output = render_to_string(&mut app, TEST_WIDTH, TEST_HEIGHT);
    assert_snapshot!(output);
}
