//! Unit tests for the error-overlay line builders.

use super::*;
use crate::test_utils::test_helpers::{TEST_JSON, test_app};
use ratatui::Terminal;
use ratatui::backend::TestBackend;

/// Concatenate a line's span contents into a plain string.
fn line_text(line: &Line<'_>) -> String {
    line.spans.iter().map(|s| s.content.as_ref()).collect()
}

/// Render the full app to a plain string, mirroring the harness used by the
/// other `*_render_tests.rs` modules. The `app::app_render_tests` helper of the
/// same name is module-private, so each render-test module defines its own.
fn render_to_string(app: &mut App, width: u16, height: u16) -> String {
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal.draw(|f| app.render(f)).unwrap();
    terminal.backend().to_string()
}

/// Build an app whose query result is the given error and open the Ctrl+E
/// error overlay, so a full render exercises `render_error_overlay`.
fn app_with_error_overlay(error: &str) -> App {
    let mut app = test_app(TEST_JSON);
    app.query.as_mut().unwrap().result = Err(error.to_string());
    app.error_overlay_visible = true;
    app
}

#[test]
fn wrap_plain_keeps_short_text_on_one_line() {
    assert_eq!(wrap_plain("hello world", 40), vec!["hello world"]);
}

#[test]
fn wrap_plain_breaks_on_word_boundaries() {
    let wrapped = wrap_plain("one two three four", 8);
    // Each line stays within the 8-column budget.
    for line in &wrapped {
        assert!(line.len() <= 8, "line too wide: {line:?}");
    }
    assert_eq!(wrapped.join(" "), "one two three four");
}

#[test]
fn wrap_plain_emits_overlong_token_alone() {
    let wrapped = wrap_plain("supercalifragilistic x", 8);
    assert_eq!(wrapped[0], "supercalifragilistic");
    assert_eq!(wrapped[1], "x");
}

#[test]
fn wrap_plain_zero_width_returns_text_unsplit() {
    assert_eq!(wrap_plain("abc def", 0), vec!["abc def"]);
}

#[test]
fn wrap_plain_empty_text_yields_one_empty_line() {
    assert_eq!(wrap_plain("", 10), vec![String::new()]);
}

#[test]
fn enhanced_lines_include_summary_hint_and_location() {
    let enhanced = EnhancedError {
        summary: "Can't index an array with a field name.".to_string(),
        hint: Some("Use .[0] or .[].".to_string()),
        location: Some("line 1, column 5".to_string()),
    };
    let lines = build_enhanced_error_lines(&enhanced, 60);
    let joined: String = lines
        .iter()
        .map(|l| line_text(l))
        .collect::<Vec<_>>()
        .join("\n");

    assert!(joined.contains("Can't index an array with a field name."));
    assert!(joined.contains("Try: Use .[0] or .[]."));
    assert!(joined.contains("jq: line 1, column 5"));
    // Blank separator line between summary and hint.
    assert!(lines.iter().any(|l| line_text(l).is_empty()));
}

#[test]
fn enhanced_lines_omit_hint_and_location_when_absent() {
    let enhanced = EnhancedError {
        summary: "Boom".to_string(),
        hint: None,
        location: None,
    };
    let lines = build_enhanced_error_lines(&enhanced, 60);
    let joined: String = lines
        .iter()
        .map(|l| line_text(l))
        .collect::<Vec<_>>()
        .join("\n");
    assert_eq!(joined, "Boom");
}

#[test]
fn enhanced_summary_with_newline_wraps_into_multiple_lines() {
    let enhanced = EnhancedError {
        summary: "first part\nsecond part".to_string(),
        hint: None,
        location: None,
    };
    let lines = build_enhanced_error_lines(&enhanced, 60);
    let texts: Vec<String> = lines.iter().map(|l| line_text(l)).collect();
    assert!(texts.contains(&"first part".to_string()));
    assert!(texts.contains(&"second part".to_string()));
}

#[test]
fn raw_lines_preserve_each_input_line() {
    let raw = "line one\nline two";
    let lines = build_raw_error_lines(raw, 60);
    let texts: Vec<String> = lines.iter().map(|l| line_text(l)).collect();
    assert_eq!(texts, vec!["line one".to_string(), "line two".to_string()]);
}

#[test]
fn enhanced_lines_indent_wrapped_hint_continuation() {
    // A long hint at a narrow width wraps to multiple lines so the
    // continuation branch (i != 0) is exercised.
    let enhanced = EnhancedError {
        summary: "Boom".to_string(),
        hint: Some("alpha beta gamma delta epsilon zeta eta theta iota kappa".to_string()),
        location: None,
    };
    let lines = build_enhanced_error_lines(&enhanced, 20);
    let texts: Vec<String> = lines.iter().map(|l| line_text(l)).collect();

    // First hint line carries the bold "Try: " label.
    assert!(
        texts.iter().any(|t| t.starts_with("Try: ")),
        "expected a hint line prefixed with the \"Try: \" label, got {texts:?}"
    );
    // At least one continuation line is indented with the five-space pad and
    // is not the blank separator line.
    assert!(
        texts
            .iter()
            .any(|t| t.starts_with("     ") && !t.trim().is_empty()),
        "expected a wrapped continuation line indented by five spaces, got {texts:?}"
    );
}

#[test]
fn overlay_falls_back_to_raw_text_for_unrecognized_error() {
    // Not a recognizable jq error, so enhance_jq_error returns None and the
    // overlay must render the raw text verbatim via build_raw_error_lines.
    let mut app = app_with_error_overlay("some internal jiq failure not in jq format");
    let output = render_to_string(&mut app, 80, 24);

    assert!(
        output.contains("internal jiq failure"),
        "overlay should show the raw unrecognized error text. Output:\n{output}"
    );
    assert!(
        output.contains("Error"),
        "overlay should carry the Error title. Output:\n{output}"
    );
}
