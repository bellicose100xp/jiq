//! Tests for toggle key handling (Ctrl+A visibility toggle)

use super::*;

#[test]
fn test_ctrl_a_toggles_visibility_on() {
    let mut ai_state = AiState::new(true);
    assert!(!ai_state.visible);

    let handled = handle_toggle_key(
        key_with_mods(KeyCode::Char('a'), KeyModifiers::CONTROL),
        &mut ai_state,
    );

    assert!(handled);
    assert!(ai_state.visible);
}

#[test]
fn test_ctrl_a_toggles_visibility_off() {
    let mut ai_state = AiState::new(true);
    ai_state.visible = true;

    let handled = handle_toggle_key(
        key_with_mods(KeyCode::Char('a'), KeyModifiers::CONTROL),
        &mut ai_state,
    );

    assert!(handled);
    assert!(!ai_state.visible);
}

#[test]
fn test_plain_a_not_handled() {
    let mut ai_state = AiState::new(true);

    let handled = handle_toggle_key(key(KeyCode::Char('a')), &mut ai_state);

    assert!(!handled);
    assert!(!ai_state.visible);
}

#[test]
fn test_ctrl_other_key_not_handled() {
    let mut ai_state = AiState::new(true);

    let handled = handle_toggle_key(
        key_with_mods(KeyCode::Char('b'), KeyModifiers::CONTROL),
        &mut ai_state,
    );

    assert!(!handled);
    assert!(!ai_state.visible);
}

#[test]
fn test_esc_closes_visible_popup() {
    let mut ai_state = AiState::new(true);
    ai_state.visible = true;

    let handled = handle_close_key(key(KeyCode::Esc), &mut ai_state);

    assert!(handled);
    assert!(!ai_state.visible);
}

#[test]
fn test_esc_not_handled_when_popup_hidden() {
    let mut ai_state = AiState::new(true);
    assert!(!ai_state.visible);

    let handled = handle_close_key(key(KeyCode::Esc), &mut ai_state);

    assert!(!handled);
}

#[test]
fn test_other_key_not_handled_for_close() {
    let mut ai_state = AiState::new(true);
    ai_state.visible = true;

    let handled = handle_close_key(key(KeyCode::Enter), &mut ai_state);

    assert!(!handled);
    assert!(ai_state.visible);
}
