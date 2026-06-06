//! Tests for tooltip/tooltip_render

use super::*;
use crate::autocomplete::jq_functions::JQ_FUNCTION_METADATA;
use crate::test_utils::test_helpers::test_app;
use crate::tooltip::operator_content::OPERATOR_CONTENT;
use proptest::prelude::*;
use ratatui::Terminal;
use ratatui::backend::TestBackend;

/// Render `render_popup` for an App that already has its tooltip state set,
/// returning both the popup area it reported and the rendered screen text.
///
/// Reuses the same TestBackend + `terminal.draw(|f| render_popup(...))` pattern
/// the autocomplete render tests use, so individual tests only describe state.
fn render_tooltip(app: &App, width: u16, height: u16) -> (Option<Rect>, String) {
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).unwrap();
    // Place the input row near the bottom so the popup, drawn above it, has room
    // for a non-zero height (popup_area.height = popup_height.min(input_area.y)).
    let input_area = Rect::new(0, height.saturating_sub(3), width, 3);

    let mut popup_area = None;
    terminal
        .draw(|f| {
            popup_area = render_popup(app, f, input_area);
        })
        .unwrap();

    (popup_area, terminal.backend().to_string())
}

/// Build an App with the given function name set as the active tooltip function.
fn app_with_tooltip_function(name: &str) -> App {
    let mut app = test_app("{}");
    app.tooltip.set_current_function(Some(name.to_string()));
    app
}

#[test]
fn test_format_tooltip_title_function() {
    assert_eq!(format_tooltip_title(true, "select"), "fn: select");
    assert_eq!(format_tooltip_title(true, "map"), "fn: map");
    assert_eq!(format_tooltip_title(true, "sort_by"), "fn: sort_by");
}

#[test]
fn test_format_tooltip_title_operator() {
    assert_eq!(format_tooltip_title(false, "//"), "operator: //");
    assert_eq!(format_tooltip_title(false, "|="), "operator: |=");
    assert_eq!(format_tooltip_title(false, "//="), "operator: //=");
    assert_eq!(format_tooltip_title(false, ".."), "operator: ..");
}

// **Feature: operator-tooltips, Property 6: Title format correctness**
// *For any* function name, the title generator SHALL produce `fn: <name>`.
// *For any* operator, the title generator SHALL produce `operator: <op>`.
// **Validates: Requirements 1.3, 2.3, 3.3, 4.3, 5.1, 5.2**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop_function_title_format(func_index in 0usize..JQ_FUNCTION_METADATA.len()) {
        let func = &JQ_FUNCTION_METADATA[func_index];
        let func_name = func.name;

        let title = format_tooltip_title(true, func_name);

        // Title should start with "fn: "
        prop_assert!(
            title.starts_with("fn: "),
            "Function title '{}' should start with 'fn: '",
            title
        );

        // Title should end with the function name
        prop_assert!(
            title.ends_with(func_name),
            "Function title '{}' should end with function name '{}'",
            title,
            func_name
        );

        // Title should be exactly "fn: <name>"
        let expected = format!("fn: {}", func_name);
        prop_assert_eq!(
            title,
            expected,
            "Function title should be exactly 'fn: {}'",
            func_name
        );
    }

    #[test]
    fn prop_operator_title_format(op_index in 0usize..OPERATOR_CONTENT.len()) {
        let op = &OPERATOR_CONTENT[op_index];
        let op_name = op.function;

        let title = format_tooltip_title(false, op_name);

        // Title should start with "operator: "
        prop_assert!(
            title.starts_with("operator: "),
            "Operator title '{}' should start with 'operator: '",
            title
        );

        // Title should end with the operator
        prop_assert!(
            title.ends_with(op_name),
            "Operator title '{}' should end with operator '{}'",
            title,
            op_name
        );

        // Title should be exactly "operator: <op>"
        let expected = format!("operator: {}", op_name);
        prop_assert_eq!(
            title,
            expected,
            "Operator title should be exactly 'operator: {}'",
            op_name
        );
    }
}

#[test]
fn test_wrap_text_truncates_to_two_lines() {
    // Test that wrap_text truncates to max 2 lines
    let long_text = "This is a very long text that will definitely wrap into more than two lines when we have a small max width like twenty characters total width";
    let result = wrap_text(long_text, 20);
    assert!(result.len() <= 2, "Should truncate to max 2 lines");
}

#[test]
fn test_wrap_text_short_text() {
    let short_text = "Hello";
    let result = wrap_text(short_text, 50);
    assert_eq!(result.len(), 1);
    assert_eq!(result[0], "Hello");
}

#[test]
fn test_wrap_text_exactly_two_lines() {
    // Text that wraps to exactly 2 lines shouldn't truncate
    let text = "First line text here and second line";
    let result = wrap_text(text, 20);
    assert!(!result.is_empty());
}

// =========================================================================
// render_popup early-return guards and content-driven render branches
// =========================================================================

#[test]
fn render_popup_returns_none_for_missing_content() {
    // Case A: a function name with no matching tooltip content -> None (line 62).
    let app = app_with_tooltip_function("definitely_not_a_real_jq_fn");
    let (area, _screen) = render_tooltip(&app, 80, 24);
    assert!(
        area.is_none(),
        "render_popup must return None when the function has no tooltip content"
    );

    // Case B: an operator with no matching content -> None (line 68).
    let mut app = test_app("{}");
    app.tooltip
        .set_current_operator(Some("@@nope@@".to_string()));
    let (area, _screen) = render_tooltip(&app, 80, 24);
    assert!(
        area.is_none(),
        "render_popup must return None when the operator has no tooltip content"
    );

    // Case C: neither function nor operator set -> None (line 71).
    let app = test_app("{}");
    assert!(app.tooltip.current_function.is_none());
    assert!(app.tooltip.current_operator.is_none());
    let (area, _screen) = render_tooltip(&app, 80, 24);
    assert!(
        area.is_none(),
        "render_popup must return None when no function or operator is active"
    );
}

#[test]
fn render_popup_renders_example_without_description() {
    // group_by ships its second example with no '#' comment
    // ("group_by(.type) | map({type: .[0].type, count: length})"). That example
    // exercises the ('', code) parse arm (line 82), the desc.is_empty() width calc
    // (line 100), and the single-Span code-only render branch (lines 171-175) -
    // it renders as a plain "  {code}" line with no "code │ desc" separator column.
    let app = app_with_tooltip_function("group_by");
    let (area, screen) = render_tooltip(&app, 80, 24);

    assert!(
        area.is_some(),
        "render_popup should return a popup area for a real function"
    );

    // "group_by(.type)" appears only in the no-'#' example, so finding it proves
    // the code-only render arm drew that example. If the desc.is_empty() branch
    // regressed (panicked or mis-parsed the empty desc), this would not render.
    assert!(
        screen.contains("group_by(.type)"),
        "code-only example should be rendered verbatim, got:\n{screen}"
    );
}

#[test]
fn render_popup_renders_multiline_wrapped_tip() {
    // getpath's tip ("Use .a.b.c for static paths; getpath for dynamic/computed
    // paths") is long enough that it wraps to two lines at getpath's popup width.
    // The continuation ("dynamic/computed paths") is rendered by the
    // wrapped_tip_lines.iter().skip(1) loop, closing at line 208 - and unlike the
    // first line it is short enough to render fully (not truncated by the border).
    let tip = "Use .a.b.c for static paths; getpath for dynamic/computed paths";

    // Precondition: at getpath's tip-available width the tip must wrap to 2 lines,
    // so the skip(1) continuation loop is guaranteed to execute. (Popup inner
    // width for getpath is ~56 columns; the tip is 63 chars.)
    let wrapped = wrap_text(tip, 56);
    assert_eq!(
        wrapped.len(),
        2,
        "getpath tip should wrap to exactly 2 lines at the popup's tip width"
    );

    let app = app_with_tooltip_function("getpath");
    let (area, screen) = render_tooltip(&app, 80, 24);

    assert!(area.is_some(), "render_popup should render getpath");

    // The continuation line text must appear on its own (indented) row, proving
    // the skip(1) loop ran rather than dropping the tip remainder.
    assert!(
        screen.contains("dynamic/computed paths"),
        "wrapped tip continuation line should be rendered, got:\n{screen}"
    );
}
