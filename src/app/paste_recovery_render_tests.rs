use crate::app::App;
use crate::app::app_render_tests::render_to_string;
use crate::editor::EditorMode;
use crate::input::PasteRecoveryState;
use crate::test_utils::test_helpers::test_app;

/// Build an App parked in paste-recovery with the given state, with the
/// notification timer suppressed so renders are deterministic.
fn app_in_paste_recovery(state: PasteRecoveryState) -> App {
    let mut app = test_app(crate::test_utils::test_helpers::TEST_JSON);
    app.notification = crate::notification::NotificationState::new();
    app.paste_recovery = Some(state);
    app
}

#[test]
fn paste_recovery_explicit_empty_suppresses_top_block() {
    // Explicit mode with an empty message: top_block_height() == 0, so
    // render() early-returns and only the full-screen textarea shows.
    let mut app = app_in_paste_recovery(PasteRecoveryState::new_explicit());
    let output = render_to_string(&mut app, 80, 24);

    assert!(
        output.contains("Paste JSON"),
        "explicit empty mode should still render the textarea title"
    );
    assert!(
        !output.contains("No JSON loaded"),
        "explicit mode must not show the Recovery error title"
    );
    assert!(
        !output.contains(" Info "),
        "explicit empty mode must suppress the info box entirely"
    );
}

#[test]
fn paste_recovery_explicit_with_context_shows_info_block() {
    // Explicit mode with a context line: the top block renders with the
    // neutral " Info " title (not Recovery's "No JSON loaded") and shows
    // the context text.
    let mut app = app_in_paste_recovery(PasteRecoveryState::new_explicit_with_context(Some(
        "Clipboard is empty - paste below to load.",
    )));
    let output = render_to_string(&mut app, 80, 24);

    assert!(
        output.contains("Info"),
        "explicit-with-context should title the top block 'Info'"
    );
    assert!(
        output.contains("paste below to load"),
        "explicit-with-context should render the context message"
    );
    assert!(
        !output.contains("No JSON loaded"),
        "explicit mode must not reuse the Recovery title"
    );
}

#[test]
fn paste_recovery_mid_operator_drops_toggle_hint() {
    // While mid-operator the bottom hints keep "Load JSON" but drop the
    // Insert/Normal toggle hint (bottom_hints _ => None), and mode_color
    // resolves the Operator arm without panicking.
    let mut app = app_in_paste_recovery(PasteRecoveryState::new("No JSON loaded yet."));
    app.input.textarea.insert_str("{not json");
    app.input.editor_mode = EditorMode::Operator('d');
    let output = render_to_string(&mut app, 80, 24);

    assert!(
        output.contains("Load JSON"),
        "mid-operator hints should still advertise Enter Load JSON"
    );
    assert!(
        !output.contains("Esc Normal"),
        "mid-operator must not surface the Esc Normal toggle hint"
    );
    assert!(
        !output.contains("i Insert"),
        "mid-operator must not surface the i Insert toggle hint"
    );
}
