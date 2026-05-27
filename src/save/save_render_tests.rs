use std::fs;

use insta::assert_snapshot;
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::layout::Rect;
use tempfile::TempDir;

use super::*;

const TS: &str = "20260527-104522";

fn render_to_string(state: &mut SaveState, width: u16, height: u16) -> String {
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).unwrap();
    let area = Rect {
        x: 0,
        y: 0,
        width,
        height,
    };
    terminal
        .draw(|f| {
            render_save_popup(f, area, state);
        })
        .unwrap();
    terminal.backend().to_string()
}

fn open_state_with_filename(text: &str) -> SaveState {
    let mut s = SaveState::new();
    s.open(TS.to_string());
    let ta = s.filename_mut();
    while !ta.is_empty() {
        ta.delete_char();
    }
    ta.insert_str(text);
    s
}

#[test]
fn snapshot_closed_state_renders_nothing() {
    let mut s = SaveState::new();
    let out = render_to_string(&mut s, 60, 10);
    assert_snapshot!(out);
}

#[test]
fn snapshot_enter_filename_user_typed_absolute() {
    let mut s = open_state_with_filename("/tmp/jiq-custom.json");
    let out = render_to_string(&mut s, 80, 12);
    assert_snapshot!(out);
}

#[test]
fn snapshot_enter_filename_narrow_terminal() {
    let mut s = open_state_with_filename("/tmp/jiq-x.json");
    let out = render_to_string(&mut s, 50, 10);
    assert_snapshot!(out);
}

#[test]
fn snapshot_preview_warns_when_file_exists() {
    let dir = TempDir::new().unwrap();
    let target = dir.path().join("collide.json");
    fs::write(&target, "old").unwrap();
    // We can't snapshot a tempdir path stably; render and verify the warning
    // glyph + 'Overwrite' label appear in the rendered text.
    let mut s = open_state_with_filename(target.to_string_lossy().as_ref());
    let out = render_to_string(&mut s, 100, 12);
    assert!(
        out.contains('\u{26A0}'),
        "expected warning glyph in render: {}",
        out
    );
    assert!(
        out.contains("Overwrite"),
        "expected 'Overwrite' button label: {}",
        out
    );
    assert!(
        out.contains("File exists:"),
        "expected 'File exists:' message: {}",
        out
    );
}

#[test]
fn snapshot_preview_shows_save_label_when_file_does_not_exist() {
    // /tmp/jiq-nonexistent-... is unlikely to exist.
    let mut s = open_state_with_filename("/tmp/jiq-nonexistent-snapshot-test-999.json");
    let _ = fs::remove_file("/tmp/jiq-nonexistent-snapshot-test-999.json");
    let out = render_to_string(&mut s, 80, 12);
    assert_snapshot!(out);
}

#[test]
fn snapshot_preview_shows_error_for_unset_env_var() {
    unsafe {
        std::env::remove_var("JIQ_RENDER_TEST_MISSING");
    }
    let mut s = open_state_with_filename("$JIQ_RENDER_TEST_MISSING/x.json");
    let out = render_to_string(&mut s, 80, 12);
    assert_snapshot!(out);
}

#[test]
fn snapshot_preview_shows_error_for_empty_input() {
    let mut s = open_state_with_filename("");
    let out = render_to_string(&mut s, 80, 12);
    assert_snapshot!(out);
}

#[test]
fn truncate_front_returns_path_unchanged_when_it_fits() {
    let result = super::truncate_front("/short.json", 30, 2);
    assert_eq!(result, "/short.json");
}

#[test]
fn truncate_front_replaces_leading_chars_with_ellipsis() {
    let result = super::truncate_front("/very/long/nested/path/file.json", 12, 2);
    // budget = 10. Leading … plus 9 cells of suffix.
    assert!(result.starts_with('\u{2026}'));
    assert!(result.ends_with("file.json"));
    assert!(result.chars().count() <= 10);
}

#[test]
fn truncate_front_preserves_filename_at_end() {
    let result = super::truncate_front("/home/user/some/deep/path/jiq-result.json", 25, 2);
    assert!(result.ends_with("jiq-result.json"));
}

#[test]
fn truncate_front_returns_empty_when_prefix_eats_all_budget() {
    assert_eq!(super::truncate_front("anything", 2, 5), "");
    assert_eq!(super::truncate_front("anything", 0, 0), "");
}
