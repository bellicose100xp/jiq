//! AI suggestion selection integration tests

use super::*;
use proptest::prelude::*;

#[test]
fn test_ai_suggestion_selection_complete_flow() {
    // Test complete flow: AI response → navigation → apply → execute
    use crate::ai::suggestion::{Suggestion, SuggestionType};

    let mut app = app_with_query(".initial");
    app.focus = Focus::InputField;
    app.input.editor_mode = EditorMode::Insert;

    // Set up AI popup with suggestions
    app.ai.visible = true;
    app.ai.enabled = true;
    app.ai.suggestions = vec![
        Suggestion {
            query: ".name".to_string(),
            description: "Get name field".to_string(),
            suggestion_type: SuggestionType::Next,
        },
        Suggestion {
            query: ".value".to_string(),
            description: "Get value field".to_string(),
            suggestion_type: SuggestionType::Optimize,
        },
    ];

    // Navigate down to select first suggestion
    app.handle_key_event(key_with_mods(KeyCode::Down, KeyModifiers::ALT));
    assert_eq!(app.ai.selection.get_selected(), Some(0));
    assert!(app.ai.selection.is_navigation_active());

    // Press Enter to apply
    app.handle_key_event(key(KeyCode::Enter));

    // Query should be replaced
    assert_eq!(app.query(), ".name");

    // Selection should be cleared
    assert!(app.ai.selection.get_selected().is_none());
    assert!(!app.ai.selection.is_navigation_active());

    // Should NOT quit
    assert!(!app.should_quit);

    // Query should have been executed (result should be updated)
    assert!(app.query.result.is_ok());
}

#[test]
fn test_ai_suggestion_direct_selection_alt_1() {
    // Test direct selection with Alt+1
    use crate::ai::suggestion::{Suggestion, SuggestionType};

    let mut app = app_with_query(".initial");
    app.focus = Focus::InputField;

    // Set up AI popup with suggestions
    app.ai.visible = true;
    app.ai.enabled = true;
    app.ai.suggestions = vec![
        Suggestion {
            query: ".first".to_string(),
            description: "First suggestion".to_string(),
            suggestion_type: SuggestionType::Next,
        },
        Suggestion {
            query: ".second".to_string(),
            description: "Second suggestion".to_string(),
            suggestion_type: SuggestionType::Fix,
        },
    ];

    // Press Alt+1 to select first suggestion
    app.handle_key_event(key_with_mods(KeyCode::Char('1'), KeyModifiers::ALT));

    // Query should be replaced with first suggestion
    assert_eq!(app.query(), ".first");
    assert!(!app.should_quit);
}

#[test]
fn test_ai_suggestion_direct_selection_alt_2() {
    // Test direct selection with Alt+2
    use crate::ai::suggestion::{Suggestion, SuggestionType};

    let mut app = app_with_query(".initial");
    app.focus = Focus::InputField;

    // Set up AI popup with suggestions
    app.ai.visible = true;
    app.ai.enabled = true;
    app.ai.suggestions = vec![
        Suggestion {
            query: ".first".to_string(),
            description: "First suggestion".to_string(),
            suggestion_type: SuggestionType::Next,
        },
        Suggestion {
            query: ".second".to_string(),
            description: "Second suggestion".to_string(),
            suggestion_type: SuggestionType::Fix,
        },
    ];

    // Press Alt+2 to select second suggestion
    app.handle_key_event(key_with_mods(KeyCode::Char('2'), KeyModifiers::ALT));

    // Query should be replaced with second suggestion
    assert_eq!(app.query(), ".second");
    assert!(!app.should_quit);
}

#[test]
fn test_ai_suggestion_multiple_selections_in_sequence() {
    // Test multiple selections in sequence
    use crate::ai::suggestion::{Suggestion, SuggestionType};

    let mut app = app_with_query(".initial");
    app.focus = Focus::InputField;

    // Set up AI popup with suggestions
    app.ai.visible = true;
    app.ai.enabled = true;
    app.ai.suggestions = vec![
        Suggestion {
            query: ".first".to_string(),
            description: "First suggestion".to_string(),
            suggestion_type: SuggestionType::Next,
        },
        Suggestion {
            query: ".second".to_string(),
            description: "Second suggestion".to_string(),
            suggestion_type: SuggestionType::Fix,
        },
    ];

    // First selection: Alt+1
    app.handle_key_event(key_with_mods(KeyCode::Char('1'), KeyModifiers::ALT));
    assert_eq!(app.query(), ".first");

    // Second selection: Alt+2
    app.handle_key_event(key_with_mods(KeyCode::Char('2'), KeyModifiers::ALT));
    assert_eq!(app.query(), ".second");

    // Third selection via navigation: Alt+Down + Enter
    app.handle_key_event(key_with_mods(KeyCode::Down, KeyModifiers::ALT));
    app.handle_key_event(key(KeyCode::Enter));
    assert_eq!(app.query(), ".first"); // Wraps to first

    assert!(!app.should_quit);
}

#[test]
fn test_ai_suggestion_selection_hides_autocomplete() {
    // Test that selecting AI suggestion hides autocomplete
    use crate::ai::suggestion::{Suggestion, SuggestionType};

    let mut app = app_with_query(".na");
    app.focus = Focus::InputField;
    app.input.editor_mode = EditorMode::Insert;

    // Set up autocomplete
    let autocomplete_suggestions = vec![crate::autocomplete::Suggestion::new(
        "name",
        crate::autocomplete::SuggestionType::Field,
    )];
    app.autocomplete
        .update_suggestions(autocomplete_suggestions);
    assert!(app.autocomplete.is_visible());

    // Set up AI popup with suggestions
    app.ai.visible = true;
    app.ai.enabled = true;
    app.ai.suggestions = vec![Suggestion {
        query: ".name | length".to_string(),
        description: "Get name length".to_string(),
        suggestion_type: SuggestionType::Next,
    }];

    // Press Alt+1 to select AI suggestion
    app.handle_key_event(key_with_mods(KeyCode::Char('1'), KeyModifiers::ALT));

    // Query should be replaced
    assert_eq!(app.query(), ".name | length");

    // Autocomplete should be hidden
    assert!(!app.autocomplete.is_visible());
}

#[test]
fn test_ai_suggestion_selection_ignored_when_popup_hidden() {
    // Test that selection is ignored when AI popup is hidden
    use crate::ai::suggestion::{Suggestion, SuggestionType};

    let mut app = app_with_query(".initial");
    app.focus = Focus::InputField;

    // AI popup is hidden
    app.ai.visible = false;
    app.ai.enabled = true;
    app.ai.suggestions = vec![Suggestion {
        query: ".should_not_apply".to_string(),
        description: "Should not apply".to_string(),
        suggestion_type: SuggestionType::Next,
    }];

    // Press Alt+1 - should be ignored
    app.handle_key_event(key_with_mods(KeyCode::Char('1'), KeyModifiers::ALT));

    // Query should be unchanged
    assert_eq!(app.query(), ".initial");
}

#[test]
fn test_ai_suggestion_selection_ignored_when_no_suggestions() {
    // Test that selection is ignored when no suggestions
    let mut app = app_with_query(".initial");
    app.focus = Focus::InputField;

    // AI popup is visible but has no suggestions
    app.ai.visible = true;
    app.ai.enabled = true;
    app.ai.suggestions = vec![];

    // Press Alt+1 - should be ignored
    app.handle_key_event(key_with_mods(KeyCode::Char('1'), KeyModifiers::ALT));

    // Query should be unchanged
    assert_eq!(app.query(), ".initial");
}

#[test]
fn test_ai_suggestion_invalid_selection_ignored() {
    // Test that invalid selection (Ctrl+3 when only 2 suggestions) is ignored
    use crate::ai::suggestion::{Suggestion, SuggestionType};

    let mut app = app_with_query(".initial");
    app.focus = Focus::InputField;

    // Set up AI popup with only 2 suggestions
    app.ai.visible = true;
    app.ai.enabled = true;
    app.ai.suggestions = vec![
        Suggestion {
            query: ".first".to_string(),
            description: "First".to_string(),
            suggestion_type: SuggestionType::Next,
        },
        Suggestion {
            query: ".second".to_string(),
            description: "Second".to_string(),
            suggestion_type: SuggestionType::Fix,
        },
    ];

    // Press Alt+3 - should be ignored (only 2 suggestions)
    app.handle_key_event(key_with_mods(KeyCode::Char('3'), KeyModifiers::ALT));

    // Query should be unchanged
    assert_eq!(app.query(), ".initial");
}

#[test]
fn test_ai_suggestion_navigation_wrapping() {
    // Test navigation wrapping behavior
    use crate::ai::suggestion::{Suggestion, SuggestionType};

    let mut app = app_with_query(".initial");
    app.focus = Focus::InputField;

    // Set up AI popup with 3 suggestions
    app.ai.visible = true;
    app.ai.enabled = true;
    app.ai.suggestions = vec![
        Suggestion {
            query: ".first".to_string(),
            description: "First".to_string(),
            suggestion_type: SuggestionType::Next,
        },
        Suggestion {
            query: ".second".to_string(),
            description: "Second".to_string(),
            suggestion_type: SuggestionType::Fix,
        },
        Suggestion {
            query: ".third".to_string(),
            description: "Third".to_string(),
            suggestion_type: SuggestionType::Optimize,
        },
    ];

    // Navigate down 4 times (should wrap: 0 -> 1 -> 2 -> 0)
    app.handle_key_event(key_with_mods(KeyCode::Down, KeyModifiers::ALT)); // 0
    app.handle_key_event(key_with_mods(KeyCode::Down, KeyModifiers::ALT)); // 1
    app.handle_key_event(key_with_mods(KeyCode::Down, KeyModifiers::ALT)); // 2
    app.handle_key_event(key_with_mods(KeyCode::Down, KeyModifiers::ALT)); // 0 (wrap)

    assert_eq!(app.ai.selection.get_selected(), Some(0));

    // Press Enter to apply
    app.handle_key_event(key(KeyCode::Enter));
    assert_eq!(app.query(), ".first");
}

#[test]
fn test_ai_suggestion_enter_without_navigation_exits() {
    // Test that Enter without navigation exits the app
    use crate::ai::suggestion::{Suggestion, SuggestionType};

    let mut app = app_with_query(".");
    app.focus = Focus::InputField;

    // Set up AI popup with suggestions but NO navigation
    app.ai.visible = true;
    app.ai.enabled = true;
    app.ai.suggestions = vec![Suggestion {
        query: ".name".to_string(),
        description: "Get name".to_string(),
        suggestion_type: SuggestionType::Next,
    }];

    // Ensure no navigation has occurred
    assert!(!app.ai.selection.is_navigation_active());

    // Press Enter - should exit app (not apply suggestion)
    app.handle_key_event(key(KeyCode::Enter));

    // Should quit with Results output mode
    assert!(app.should_quit);
    assert_eq!(app.output_mode, Some(OutputMode::Results));

    // Query should be unchanged
    assert_eq!(app.query(), ".");
}

// **Feature: ai-assistant-phase3, Property 7: Mode independence**
// *For any* editor mode (Normal, Insert, Operator) and any suggestion, applying the
// suggestion should replace the query input and preserve the current editor mode unchanged.
// **Validates: Requirements 6.1, 6.2, 6.3, 6.4**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop_mode_independence_for_ai_suggestion_selection(
        // Test with different editor modes
        mode_idx in 0usize..3,
        // Test with different selection methods (Alt+1-5)
        selection_digit in 1usize..=5,
        // Test with different suggestion queries
        suggestion_query in prop_oneof![
            Just(".name"),
            Just(".value"),
            Just(".items[]"),
            Just(".users | length"),
        ],
    ) {
        use crate::ai::suggestion::{Suggestion, SuggestionType};

        let mut app = app_with_query(".existing");
        app.focus = Focus::InputField;

        // Set editor mode based on index
        let mode = match mode_idx {
            0 => EditorMode::Normal,
            1 => EditorMode::Insert,
            _ => EditorMode::Operator('d'),
        };
        app.input.editor_mode = mode;

        // Set up AI popup with suggestions
        app.ai.visible = true;
        app.ai.enabled = true;
        app.ai.suggestions = (0..5).map(|i| Suggestion {
            query: if i == selection_digit - 1 {
                suggestion_query.to_string()
            } else {
                format!(".field{}", i)
            },
            description: format!("Description {}", i),
            suggestion_type: SuggestionType::Next,
        }).collect();

        // Only test if selection is valid
        prop_assume!(selection_digit <= app.ai.suggestions.len());

        // Press Alt+N to select suggestion
        let key = key_with_mods(
            KeyCode::Char(char::from_digit(selection_digit as u32, 10).unwrap()),
            KeyModifiers::ALT,
        );
        app.handle_key_event(key);

        // Property 1: Query should be replaced with suggestion
        prop_assert_eq!(
            app.query(),
            suggestion_query,
            "Query should be replaced with selected suggestion"
        );

        // Property 2: Editor mode should be preserved
        prop_assert_eq!(
            app.input.editor_mode,
            mode,
            "Editor mode should be preserved after suggestion selection"
        );

        // Property 3: Should NOT quit
        prop_assert!(
            !app.should_quit,
            "Should not quit after selecting AI suggestion"
        );
    }

    #[test]
    fn prop_mode_independence_for_ai_navigation_selection(
        // Test with different editor modes
        mode_idx in 0usize..3,
        // Test with different navigation steps
        nav_steps in 1usize..5,
        // Test with different suggestion queries
        suggestion_query in prop_oneof![
            Just(".name"),
            Just(".value"),
            Just(".items[]"),
        ],
    ) {
        use crate::ai::suggestion::{Suggestion, SuggestionType};

        let mut app = app_with_query(".existing");
        app.focus = Focus::InputField;

        // Set editor mode based on index
        let mode = match mode_idx {
            0 => EditorMode::Normal,
            1 => EditorMode::Insert,
            _ => EditorMode::Operator('d'),
        };
        app.input.editor_mode = mode;

        // Set up AI popup with suggestions
        app.ai.visible = true;
        app.ai.enabled = true;
        app.ai.suggestions = vec![
            Suggestion {
                query: suggestion_query.to_string(),
                description: "First suggestion".to_string(),
                suggestion_type: SuggestionType::Next,
            },
            Suggestion {
                query: ".other".to_string(),
                description: "Second suggestion".to_string(),
                suggestion_type: SuggestionType::Fix,
            },
        ];

        // Navigate with Alt+Down
        for _ in 0..nav_steps {
            app.handle_key_event(key_with_mods(KeyCode::Down, KeyModifiers::ALT));
        }

        // Press Enter to apply
        app.handle_key_event(key(KeyCode::Enter));

        // Property 1: Query should be replaced with one of the suggestions
        let query = app.query();
        prop_assert!(
            query == suggestion_query || query == ".other",
            "Query '{}' should be one of the suggestions",
            query
        );

        // Property 2: Editor mode should be preserved
        prop_assert_eq!(
            app.input.editor_mode,
            mode,
            "Editor mode should be preserved after navigation selection"
        );

        // Property 3: Should NOT quit
        prop_assert!(
            !app.should_quit,
            "Should not quit after selecting AI suggestion via navigation"
        );
    }
}

#[test]
fn test_ctrl_a_triggers_ai_request_when_becoming_visible() {
    // Test that pressing Ctrl+A triggers an AI request when popup becomes visible
    // This validates the fix for enabled=false but configured=true scenario
    let mut app = app_with_query(".");
    app.focus = Focus::InputField;

    // Start with AI hidden but configured (simulating enabled=false in config)
    app.ai.visible = false;
    app.ai.enabled = false;
    app.ai.configured = true;

    // Set up AI channel (simulating worker is running because configured=true)
    let (tx, rx) = std::sync::mpsc::channel();
    let (_response_tx, response_rx) = std::sync::mpsc::channel();
    app.ai.request_tx = Some(tx);
    app.ai.response_rx = Some(response_rx);

    // Set initial query hash to ensure query appears changed
    app.ai.set_last_query_hash(".initial");

    // Press Ctrl+A to show AI box
    app.handle_key_event(key_with_mods(KeyCode::Char('a'), KeyModifiers::CONTROL));

    // Verify popup is now visible
    assert!(app.ai.visible, "AI popup should be visible after Ctrl+A");

    // Verify AI request was sent
    let mut found_request = false;
    while let Ok(msg) = rx.try_recv() {
        if matches!(msg, crate::ai::ai_state::AiRequest::Query { .. }) {
            found_request = true;
            break;
        }
    }
    assert!(
        found_request,
        "Should have sent AI request when popup became visible"
    );
}

#[test]
fn test_ctrl_a_no_request_when_not_configured() {
    // Test that pressing Ctrl+A does NOT trigger AI request when not configured
    let mut app = app_with_query(".");
    app.focus = Focus::InputField;

    // Start with AI hidden and NOT configured
    app.ai.visible = false;
    app.ai.enabled = false;
    app.ai.configured = false;

    // No channel setup (no worker running)
    app.ai.request_tx = None;
    app.ai.response_rx = None;

    // Press Ctrl+A to show AI box
    app.handle_key_event(key_with_mods(KeyCode::Char('a'), KeyModifiers::CONTROL));

    // Verify popup is visible (toggle still works)
    assert!(app.ai.visible, "AI popup should be visible after Ctrl+A");

    // No request was sent (because not configured, so no crash)
    // This test mainly verifies no panic occurs
}

#[test]
fn test_ctrl_a_toggles_off_no_request() {
    // Test that toggling AI popup OFF does not trigger a request
    let mut app = app_with_query(".");
    app.focus = Focus::InputField;

    // Start with AI visible and configured
    app.ai.visible = true;
    app.ai.enabled = true;
    app.ai.configured = true;

    // Set up AI channel
    let (tx, rx) = std::sync::mpsc::channel();
    let (_response_tx, response_rx) = std::sync::mpsc::channel();
    app.ai.request_tx = Some(tx);
    app.ai.response_rx = Some(response_rx);

    // Clear any pending messages
    while rx.try_recv().is_ok() {}

    // Press Ctrl+A to HIDE AI box
    app.handle_key_event(key_with_mods(KeyCode::Char('a'), KeyModifiers::CONTROL));

    // Verify popup is now hidden
    assert!(!app.ai.visible, "AI popup should be hidden after Ctrl+A");

    // Verify NO AI request was sent when hiding
    let mut found_request = false;
    while let Ok(msg) = rx.try_recv() {
        if matches!(msg, crate::ai::ai_state::AiRequest::Query { .. }) {
            found_request = true;
            break;
        }
    }
    assert!(
        !found_request,
        "Should NOT send AI request when hiding popup"
    );
}
