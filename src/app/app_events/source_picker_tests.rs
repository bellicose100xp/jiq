//! Tests for the source-picker key router (`handle_key`).
//!
//! These exercise the key-code-to-action MAPPING in `handle_key`
//! (Esc=quit, Enter=confirm-with-guard, prev/next navigation, and the
//! catch-all swallow), distinct from the `select_*` cycle mechanics
//! already covered in `src/input/source_picker_tests.rs`.

use super::*;

use crate::config::Config;
use crate::input::SourcePickerState;
use crate::input::loader::ClipboardPeek;
use crate::test_utils::test_helpers::key;
use ratatui::crossterm::event::KeyCode;

/// Build an App opened on the source picker from a launch-time peek.
/// `ClipboardPeek::Usable` preselects Clipboard (with cached bytes);
/// anything else preselects Paste (no cache).
fn picker_app(peek: ClipboardPeek) -> App {
    App::new_with_source_picker(SourcePickerState::from_peek(peek), &Config::default())
}

#[test]
fn esc_sets_should_quit() {
    let mut app = picker_app(ClipboardPeek::Empty);

    let handled = handle_key(&mut app, key(KeyCode::Esc));

    assert!(handled, "Esc must be handled by the picker router");
    assert!(
        app.should_quit,
        "Esc must set should_quit so the user can exit"
    );
}

#[test]
fn enter_confirms_paste_selection_enters_paste_recovery() {
    // Empty peek preselects Paste with no clipboard cache.
    let mut app = picker_app(ClipboardPeek::Empty);
    assert_eq!(
        app.source_picker.as_ref().unwrap().selection,
        SourceChoice::Paste
    );

    let handled = handle_key(&mut app, key(KeyCode::Enter));

    assert!(handled, "Enter must be handled");
    assert!(
        app.source_picker.is_none(),
        "confirm_source_picker must take the picker off screen"
    );
    assert!(
        app.paste_recovery.is_some(),
        "Paste confirm must enter the paste-recovery editor"
    );
}

#[test]
fn enter_on_clipboard_without_cache_is_swallowed_not_confirmed() {
    // Empty peek => Paste preselected, cache None. Force the bad state:
    // user manually toggled to Clipboard while no usable bytes exist.
    let mut app = picker_app(ClipboardPeek::Empty);
    {
        let state = app.source_picker.as_mut().unwrap();
        state.selection = SourceChoice::Clipboard;
        assert!(
            state.clipboard_cache.is_none(),
            "fixture must have no cache"
        );
    }

    let handled = handle_key(&mut app, key(KeyCode::Enter));

    assert!(handled, "Enter is always handled (swallowed) by the picker");
    assert!(
        app.source_picker.is_some(),
        "can_confirm()==false must keep the picker on screen (no confirm)"
    );
    assert!(
        app.file_loader.is_none(),
        "nothing should have been committed to the file loader"
    );
    assert!(
        app.paste_recovery.is_none(),
        "Clipboard-without-cache must not enter paste recovery"
    );
}

#[test]
fn backtab_and_h_select_previous_option() {
    // Usable peek preselects Clipboard; previous-nav must flip to Paste.
    for code in [KeyCode::BackTab, KeyCode::Char('h')] {
        let mut app = picker_app(ClipboardPeek::Usable("{}".into()));
        assert_eq!(
            app.source_picker.as_ref().unwrap().selection,
            SourceChoice::Clipboard,
            "fixture must start on Clipboard"
        );

        let handled = handle_key(&mut app, key(code));

        assert!(handled, "{code:?} must be handled");
        assert_eq!(
            app.source_picker.as_ref().unwrap().selection,
            SourceChoice::Paste,
            "{code:?} must route to select_previous"
        );
    }
}

#[test]
fn tab_and_l_select_next_option() {
    // Usable peek preselects Clipboard; next-nav must flip to Paste.
    for code in [KeyCode::Tab, KeyCode::Char('l')] {
        let mut app = picker_app(ClipboardPeek::Usable("{}".into()));
        assert_eq!(
            app.source_picker.as_ref().unwrap().selection,
            SourceChoice::Clipboard,
            "fixture must start on Clipboard"
        );

        let handled = handle_key(&mut app, key(code));

        assert!(handled, "{code:?} must be handled");
        assert_eq!(
            app.source_picker.as_ref().unwrap().selection,
            SourceChoice::Paste,
            "{code:?} must route to select_next"
        );
    }
}

#[test]
fn unmapped_key_is_swallowed_without_side_effects() {
    let mut app = picker_app(ClipboardPeek::Empty);
    let selection_before = app.source_picker.as_ref().unwrap().selection;

    let handled = handle_key(&mut app, key(KeyCode::Char('x')));

    assert!(handled, "unmapped keys must be swallowed (return true)");
    assert!(!app.should_quit, "stray key must not quit");
    assert!(app.source_picker.is_some(), "picker must stay on screen");
    assert_eq!(
        app.source_picker.as_ref().unwrap().selection,
        selection_before,
        "stray key must not change the selection"
    );
    assert!(app.query.is_none(), "stray key must not load a query");
    assert!(
        app.file_loader.is_none(),
        "stray key must not commit a loader"
    );
    assert!(
        app.paste_recovery.is_none(),
        "stray key must not enter paste recovery"
    );
}
