//! Tests for model name display and truncation logic

use super::*;
use crate::ai::ai_state::lifecycle::TEST_MAX_CONTEXT_LENGTH;
use proptest::prelude::*;

/// Test that short model names are not truncated
#[test]
fn test_model_name_no_truncation_short() {
    let model_name = "gpt-4o-mini".to_string();
    let popup_width: u16 = 100;
    let max_model_width = (popup_width / 2).saturating_sub(2);

    let model_display = if model_name.len() > max_model_width as usize {
        format!(
            "{}...",
            &model_name[..max_model_width.saturating_sub(3) as usize]
        )
    } else {
        model_name.clone()
    };

    assert_eq!(model_display, "gpt-4o-mini");
}

/// Test that long model names are truncated with ellipsis
#[test]
fn test_model_name_truncation_long() {
    let model_name = "anthropic.claude-3-5-sonnet-20241022-v2:0".to_string();
    let popup_width: u16 = 50;
    let max_model_width = (popup_width / 2).saturating_sub(2);

    let model_display = if model_name.len() > max_model_width as usize {
        format!(
            "{}...",
            &model_name[..max_model_width.saturating_sub(3) as usize]
        )
    } else {
        model_name.clone()
    };

    assert!(model_display.ends_with("..."));
    assert!(model_display.len() <= max_model_width as usize);
}

/// Test edge case: exactly at max width
#[test]
fn test_model_name_exactly_at_max() {
    let popup_width: u16 = 50;
    let max_model_width = (popup_width / 2).saturating_sub(2);
    let model_name = "a".repeat(max_model_width as usize);

    let model_display = if model_name.len() > max_model_width as usize {
        format!(
            "{}...",
            &model_name[..max_model_width.saturating_sub(3) as usize]
        )
    } else {
        model_name.clone()
    };

    assert_eq!(model_display.len(), max_model_width as usize);
    assert!(!model_display.ends_with("..."));
}

/// Test edge case: one character over max width
#[test]
fn test_model_name_one_over_max() {
    let popup_width: u16 = 50;
    let max_model_width = (popup_width / 2).saturating_sub(2);
    let model_name = "a".repeat(max_model_width as usize + 1);

    let model_display = if model_name.len() > max_model_width as usize {
        format!(
            "{}...",
            &model_name[..max_model_width.saturating_sub(3) as usize]
        )
    } else {
        model_name.clone()
    };

    assert!(model_display.ends_with("..."));
    assert_eq!(model_display.len(), max_model_width as usize);
}

/// Test empty model name
#[test]
fn test_model_name_empty() {
    let model_name = String::new();
    let popup_width: u16 = 100;
    let max_model_width = (popup_width / 2).saturating_sub(2);

    let model_display = if model_name.len() > max_model_width as usize {
        format!(
            "{}...",
            &model_name[..max_model_width.saturating_sub(3) as usize]
        )
    } else {
        model_name.clone()
    };

    assert_eq!(model_display, "");
}

/// Test very small popup width
#[test]
fn test_model_name_small_popup() {
    let model_name = "claude-3-5-sonnet".to_string();
    let popup_width: u16 = 20;
    let max_model_width = (popup_width / 2).saturating_sub(2);

    let model_display = if model_name.len() > max_model_width as usize {
        format!(
            "{}...",
            &model_name[..max_model_width.saturating_sub(3) as usize]
        )
    } else {
        model_name.clone()
    };

    assert!(model_display.ends_with("..."));
    assert!(model_display.len() <= max_model_width as usize);
}

// Property test: truncated display never exceeds max width
// Using ASCII-only strings to avoid UTF-8 character boundary issues
proptest! {
    #[test]
    fn prop_model_name_never_exceeds_max(
        model_name in "[a-zA-Z0-9\\-_.:/]{1,100}",
        popup_width in 40u16..200u16
    ) {
        let max_model_width = (popup_width / 2).saturating_sub(2);

        let model_display = if model_name.len() > max_model_width as usize {
            format!("{}...", &model_name[..max_model_width.saturating_sub(3) as usize])
        } else {
            model_name.clone()
        };

        prop_assert!(model_display.len() <= max_model_width as usize);
    }
}

// Property test: truncation always adds ellipsis when needed
// Using ASCII-only strings to avoid UTF-8 character boundary issues
proptest! {
    #[test]
    fn prop_model_name_ellipsis_when_truncated(
        model_name in "[a-zA-Z0-9\\-_.:/]{1,100}",
        popup_width in 40u16..200u16
    ) {
        // Skip the far-fetched scenario where model name naturally ends with "..."
        if model_name.ends_with("...") {
            return Ok(());
        }

        let max_model_width = (popup_width / 2).saturating_sub(2);

        let model_display = if model_name.len() > max_model_width as usize {
            format!("{}...", &model_name[..max_model_width.saturating_sub(3) as usize])
        } else {
            model_name.clone()
        };

        if model_name.len() > max_model_width as usize {
            prop_assert!(model_display.ends_with("..."));
        } else {
            prop_assert!(!model_display.ends_with("..."));
        }
    }
}

// Property test: max width is always 50% of popup minus 2
proptest! {
    #[test]
    fn prop_max_width_calculation(popup_width in 10u16..200u16) {
        let max_model_width = (popup_width / 2).saturating_sub(2);
        prop_assert!(max_model_width <= popup_width / 2);
        // max_model_width is u16, so >= 0 always true (removed useless comparison)
    }
}

/// Snapshot test: verify model name display in popup
#[test]
fn snapshot_model_name_display() {
    use insta::assert_snapshot;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;
    use ratatui::layout::Rect;

    let mut state = AiState::new_with_config(
        true,
        true,
        "Anthropic".to_string(),
        "claude-3-5-sonnet-20241022".to_string(),
        TEST_MAX_CONTEXT_LENGTH,
    );
    state.visible = true;
    state.response = "Test response".to_string();

    let mut terminal = Terminal::new(TestBackend::new(100, 30)).unwrap();
    terminal
        .draw(|f| {
            let input_area = Rect {
                x: 0,
                y: 26,
                width: 100,
                height: 3,
            };
            crate::ai::ai_render::render_popup(&mut state, f, input_area);
        })
        .unwrap();

    let output = terminal.backend().to_string();
    assert_snapshot!(output);
}

/// Snapshot test: verify long model name truncation
#[test]
fn snapshot_long_model_name_truncation() {
    use insta::assert_snapshot;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;
    use ratatui::layout::Rect;

    let mut state = AiState::new_with_config(
        true,
        true,
        "Bedrock".to_string(),
        "anthropic.claude-3-5-sonnet-20241022-v2:0-super-long-model-name".to_string(),
        TEST_MAX_CONTEXT_LENGTH,
    );
    state.visible = true;
    state.response = "Test response".to_string();

    let mut terminal = Terminal::new(TestBackend::new(80, 30)).unwrap();
    terminal
        .draw(|f| {
            let input_area = Rect {
                x: 0,
                y: 26,
                width: 80,
                height: 3,
            };
            crate::ai::ai_render::render_popup(&mut state, f, input_area);
        })
        .unwrap();

    let output = terminal.backend().to_string();
    assert_snapshot!(output);
}
