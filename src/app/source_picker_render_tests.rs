use super::*;

use crate::app::App;
use crate::app::app_render_tests::render_to_string;
use crate::config::Config;
use crate::input::SourcePickerState;
use crate::input::loader::ClipboardPeek;

/// Concatenate the textual content of every span in a `Line` so tests
/// can assert on the rendered string regardless of styling.
fn line_text(line: &Line<'static>) -> String {
    line.spans
        .iter()
        .map(|s| s.content.as_ref())
        .collect::<String>()
}

/// Build an `App` parked in the source-picker state from a clipboard
/// peek outcome, using the same harness shape as the sibling render
/// tests.
fn picker_app(peek: ClipboardPeek) -> App {
    App::new_with_source_picker(SourcePickerState::from_peek(peek), &Config::default())
}

const PREVIEW_JSON: &str = "{\"name\":\"Alice\",\"age\":30}";

#[test]
fn fmt_bytes_picks_si_unit_at_each_boundary() {
    assert_eq!(fmt_bytes(512), "512 B");
    assert_eq!(fmt_bytes(1023), "1023 B");
    assert_eq!(fmt_bytes(1024), "1.0 KB");
    assert_eq!(fmt_bytes(1536), "1.5 KB");
    assert_eq!(fmt_bytes(1024 * 1024), "1.0 MB");
    // One byte below the MB boundary must still render as KB, never MB.
    let just_under_mb = fmt_bytes(1024 * 1024 - 1);
    assert!(
        just_under_mb.ends_with(" KB"),
        "expected KB suffix just below the MB boundary, got {just_under_mb}"
    );
}

#[test]
fn clipboard_preview_tail_pluralizes_and_picks_format_arm() {
    // (a) Many lines elided -> plural "more lines" tail.
    let five_lines = "a\nb\nc\nd\ne";
    let out = clipboard_preview_lines(five_lines, 2);
    let tail = line_text(out.last().unwrap());
    assert!(
        tail.contains("3 more lines"),
        "expected plural lines tail, got {tail:?}"
    );

    // (b) Exactly one line elided -> singular "more line" (no trailing s).
    let two_lines = "a\nb";
    let out = clipboard_preview_lines(two_lines, 1);
    let tail = line_text(out.last().unwrap());
    assert!(
        tail.contains("1 more line") && !tail.contains("1 more lines"),
        "expected singular line tail, got {tail:?}"
    );

    // (c) Content fits entirely -> no tail line appended.
    let fits = "a\nb\nc";
    let out = clipboard_preview_lines(fits, 10);
    assert_eq!(
        out.len(),
        3,
        "fully visible content must not append a truncation tail"
    );
    for (i, expected) in ["a", "b", "c"].iter().enumerate() {
        assert_eq!(&line_text(&out[i]), expected);
    }
}

#[test]
fn clipboard_preview_caps_scan_at_4kb_on_char_boundary() {
    // Construct a string whose byte 4096 lands inside a multibyte char.
    // 'é' is 2 bytes; placing it so it straddles PREVIEW_MAX_BYTES forces
    // the char-boundary backoff loop to slice safely instead of panicking.
    let mut s = "a".repeat(4095);
    s.push('é'); // bytes 4095..4097 -> boundary at 4096 is mid-char
    s.push_str(&"b".repeat(2000));
    assert!(s.len() > PREVIEW_MAX_BYTES);
    assert!(!s.is_char_boundary(PREVIEW_MAX_BYTES));

    // Single logical line, generous max_lines: truncation must come from
    // the byte cap, not the line cap. The key guarantee is no panic.
    let out = clipboard_preview_lines(&s, 100);
    let tail = line_text(out.last().unwrap());
    assert!(
        tail.starts_with('…'),
        "expected an ellipsis truncation tail from the byte cap, got {tail:?}"
    );
}

#[test]
fn bottom_hints_enter_label_tracks_selection() {
    let clip = line_text(&bottom_hints(SourceChoice::Clipboard));
    assert!(
        clip.contains("Load"),
        "Clipboard Enter hint should say 'Load', got {clip:?}"
    );
    assert!(
        !clip.contains("Open paste editor"),
        "Clipboard hint must not advertise the paste editor, got {clip:?}"
    );

    let paste = line_text(&bottom_hints(SourceChoice::Paste));
    assert!(
        paste.contains("Open paste editor"),
        "Paste Enter hint should say 'Open paste editor', got {paste:?}"
    );
}

#[test]
fn render_falls_back_to_hint_when_terminal_too_small() {
    let mut app = picker_app(ClipboardPeek::Usable(PREVIEW_JSON.to_string()));

    // Below the 6-row / 40-col threshold -> too-small fallback.
    let small = render_to_string(&mut app, 30, 5);
    assert!(
        small.contains("Terminal too small"),
        "tiny terminal should show the too-small fallback, got:\n{small}"
    );
    assert!(
        small.contains("--clipboard"),
        "fallback should point at the escape-hatch flags, got:\n{small}"
    );
    assert!(
        !small.contains("Choose JSON input source"),
        "fallback must replace the banner, not draw it"
    );

    // Comfortable terminal -> the real picker banner instead.
    let full = render_to_string(&mut app, 80, 24);
    assert!(
        full.contains("Choose JSON input source"),
        "full-size terminal should render the picker banner, got:\n{full}"
    );
    assert!(
        !full.contains("Terminal too small"),
        "full-size terminal must not show the fallback"
    );
}

#[test]
fn render_shows_clipboard_preview_only_when_clipboard_selected() {
    // Usable peek defaults to Clipboard selection -> preview pane shows.
    let mut clip_app = picker_app(ClipboardPeek::Usable(PREVIEW_JSON.to_string()));
    let clip_out = render_to_string(&mut clip_app, 80, 24);
    assert!(
        clip_out.contains("Clipboard preview"),
        "Clipboard selection should render the preview pane, got:\n{clip_out}"
    );
    assert!(
        clip_out.contains("Alice"),
        "preview pane should show the cached clipboard JSON, got:\n{clip_out}"
    );

    // Switch the highlight to Paste -> banner stays, preview pane is gone.
    if let Some(picker) = clip_app.source_picker.as_mut() {
        picker.select_next();
    }
    let paste_out = render_to_string(&mut clip_app, 80, 24);
    assert!(
        paste_out.contains("Choose JSON input source"),
        "banner must still render with Paste highlighted, got:\n{paste_out}"
    );
    assert!(
        !paste_out.contains("Clipboard preview"),
        "Paste selection must suppress the clipboard preview pane, got:\n{paste_out}"
    );
}
