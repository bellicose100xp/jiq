//! Autocomplete acceptance tests

use super::*;
use proptest::prelude::*;

// ========== Tab Autocomplete Acceptance Tests ==========

#[test]
fn test_tab_accepts_field_suggestion_replaces_from_dot() {
    // Field suggestions should replace from the last dot
    let mut app = app_with_query(".na");
    app.input.editor_mode = EditorMode::Insert;
    app.focus = Focus::InputField;

    // Validate base state
    // .na returns null, so base_query stays at "." (from App::new())
    use crate::query::ResultType;
    assert_eq!(
        app.query.base_query_for_suggestions,
        Some(".".to_string()),
        "base_query should remain '.' since .na returns null"
    );
    assert_eq!(
        app.query.base_type_for_suggestions,
        Some(ResultType::Object),
        "base_type should be Object (root object)"
    );

    // Suggestion should be "name" (no leading dot) since after Dot (CharType::Dot)
    let suggestions = vec![crate::autocomplete::Suggestion::new(
        "name",
        crate::autocomplete::SuggestionType::Field,
    )];
    app.autocomplete.update_suggestions(suggestions);

    app.handle_key_event(key(KeyCode::Tab));

    // Formula for Dot: base + suggestion = "." + "name" = ".name" ✅
    assert_eq!(app.query(), ".name");
    assert!(!app.autocomplete.is_visible());
}

#[test]
fn test_tab_accepts_array_suggestion_appends() {
    // Array suggestions should APPEND when no partial exists
    let mut app = app_with_query(".services");
    app.input.editor_mode = EditorMode::Insert;
    app.focus = Focus::InputField;

    // Validate base state was set up by app_with_query
    use crate::query::ResultType;
    assert_eq!(
        app.query.base_query_for_suggestions,
        Some(".services".to_string()),
        "base_query should be '.services'"
    );
    assert_eq!(
        app.query.base_type_for_suggestions,
        Some(ResultType::ArrayOfObjects),
        "base_type should be ArrayOfObjects"
    );

    // Verify cursor is at end
    assert_eq!(app.input.textarea.cursor().1, 9); // After ".services"

    let suggestions = vec![crate::autocomplete::Suggestion::new(
        "[].name",
        crate::autocomplete::SuggestionType::Field,
    )];
    app.autocomplete.update_suggestions(suggestions);

    app.handle_key_event(key(KeyCode::Tab));

    // Should append: .services → .services[].name
    assert_eq!(app.query(), ".services[].name");
    assert!(!app.autocomplete.is_visible());
}

#[test]
fn test_tab_accepts_array_suggestion_replaces_short_partial() {
    // Array suggestions should replace short partials (1-3 chars)
    // First execute base query to set up state
    let mut app = app_with_query(".services");
    app.input.editor_mode = EditorMode::Insert;
    app.focus = Focus::InputField;

    // Validate base state
    use crate::query::ResultType;
    assert_eq!(
        app.query.base_query_for_suggestions,
        Some(".services".to_string())
    );
    assert_eq!(
        app.query.base_type_for_suggestions,
        Some(ResultType::ArrayOfObjects)
    );

    // Now add the partial to textarea
    app.input.textarea.insert_str(".s");

    let suggestions = vec![crate::autocomplete::Suggestion::new(
        "[].serviceArn",
        crate::autocomplete::SuggestionType::Field,
    )];
    app.autocomplete.update_suggestions(suggestions);

    app.handle_key_event(key(KeyCode::Tab));

    // Should replace: base + suggestion = ".services" + "[].serviceArn"
    assert_eq!(app.query(), ".services[].serviceArn");
    assert!(!app.autocomplete.is_visible());
}

#[test]
fn test_tab_accepts_nested_array_suggestion() {
    // Nested array access: user types dot after .items[].tags to trigger autocomplete
    let mut app = app_with_query(".items[].tags");
    app.input.editor_mode = EditorMode::Insert;
    app.focus = Focus::InputField;

    // Validate base state
    use crate::query::ResultType;
    assert_eq!(
        app.query.base_query_for_suggestions,
        Some(".items[].tags".to_string()),
        "base_query should be '.items[].tags'"
    );
    assert_eq!(
        app.query.base_type_for_suggestions,
        Some(ResultType::ArrayOfObjects),
        "base_type should be ArrayOfObjects"
    );

    // User types "." to trigger autocomplete
    app.input.textarea.insert_char('.');

    // Suggestion is "[].name" (no leading dot since after NoOp 's')
    let suggestions = vec![crate::autocomplete::Suggestion::new(
        "[].name",
        crate::autocomplete::SuggestionType::Field,
    )];
    app.autocomplete.update_suggestions(suggestions);

    app.handle_key_event(key(KeyCode::Tab));

    // Formula for NoOp: base + suggestion
    // ".items[].tags" + "[].name" = ".items[].tags[].name" ✅
    assert_eq!(app.query(), ".items[].tags[].name");
    assert!(!app.autocomplete.is_visible());
}

// ========== Enter Key Autocomplete Tests ==========

#[test]
fn test_enter_accepts_suggestion_when_autocomplete_visible() {
    // Test Enter accepts suggestion when autocomplete visible
    let mut app = app_with_query(".na");
    app.input.editor_mode = EditorMode::Insert;
    app.focus = Focus::InputField;

    let suggestions = vec![crate::autocomplete::Suggestion::new(
        "name",
        crate::autocomplete::SuggestionType::Field,
    )];
    app.autocomplete.update_suggestions(suggestions);
    assert!(app.autocomplete.is_visible());

    app.handle_key_event(key(KeyCode::Enter));

    // Should accept suggestion, not exit
    assert!(!app.should_quit);
    assert!(app.output_mode.is_none());
    assert_eq!(app.query(), ".name");
}

#[test]
fn test_enter_closes_autocomplete_popup_after_selection() {
    // Test Enter closes autocomplete popup after selection
    let mut app = app_with_query(".na");
    app.input.editor_mode = EditorMode::Insert;
    app.focus = Focus::InputField;

    let suggestions = vec![crate::autocomplete::Suggestion::new(
        "name",
        crate::autocomplete::SuggestionType::Field,
    )];
    app.autocomplete.update_suggestions(suggestions);
    assert!(app.autocomplete.is_visible());

    app.handle_key_event(key(KeyCode::Enter));

    // Autocomplete should be hidden after selection
    assert!(!app.autocomplete.is_visible());
}

#[test]
fn test_enter_exits_application_when_autocomplete_not_visible() {
    // Test Enter exits application when autocomplete not visible
    let mut app = app_with_query(".");
    app.input.editor_mode = EditorMode::Insert;
    app.focus = Focus::InputField;

    // Ensure autocomplete is not visible
    assert!(!app.autocomplete.is_visible());

    app.handle_key_event(key(KeyCode::Enter));

    // Should exit with results
    assert!(app.should_quit);
    assert_eq!(app.output_mode, Some(OutputMode::Results));
}

#[test]
fn test_enter_with_shift_modifier_bypasses_autocomplete_check() {
    // Test Enter with Shift modifier bypasses autocomplete check
    let mut app = app_with_query(".na");
    app.input.editor_mode = EditorMode::Insert;
    app.focus = Focus::InputField;

    let suggestions = vec![crate::autocomplete::Suggestion::new(
        "name",
        crate::autocomplete::SuggestionType::Field,
    )];
    app.autocomplete.update_suggestions(suggestions);
    assert!(app.autocomplete.is_visible());

    // Shift+Enter should output query, not accept autocomplete
    app.handle_key_event(key_with_mods(KeyCode::Enter, KeyModifiers::SHIFT));

    // Should exit with query output mode (bypassing autocomplete)
    assert!(app.should_quit);
    assert_eq!(app.output_mode, Some(OutputMode::Query));
}

#[test]
fn test_enter_with_alt_modifier_bypasses_autocomplete_check() {
    // Test Enter with Alt modifier bypasses autocomplete check
    let mut app = app_with_query(".na");
    app.input.editor_mode = EditorMode::Insert;
    app.focus = Focus::InputField;

    let suggestions = vec![crate::autocomplete::Suggestion::new(
        "name",
        crate::autocomplete::SuggestionType::Field,
    )];
    app.autocomplete.update_suggestions(suggestions);
    assert!(app.autocomplete.is_visible());

    // Alt+Enter should output query, not accept autocomplete
    app.handle_key_event(key_with_mods(KeyCode::Enter, KeyModifiers::ALT));

    // Should exit with query output mode (bypassing autocomplete)
    assert!(app.should_quit);
    assert_eq!(app.output_mode, Some(OutputMode::Query));
}

// ========== Property-Based Tests for Enter Key Autocomplete ==========

// Feature: enter-key-autocomplete, Property 1: Enter and Tab equivalence for autocomplete selection
// *For any* application state where the autocomplete popup is visible with at least one suggestion,
// pressing Enter should produce the exact same query string as pressing Tab.
// **Validates: Requirements 3.1**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop_enter_tab_equivalence_for_autocomplete(
        // Generate different suggestion types
        suggestion_type in prop_oneof![
            Just(crate::autocomplete::SuggestionType::Field),
            Just(crate::autocomplete::SuggestionType::Function),
        ],
        // Generate different suggestion texts
        suggestion_text in prop_oneof![
            Just("name"),
            Just("age"),
            Just("city"),
            Just("length"),
            Just("keys"),
        ],
    ) {
        // Create two identical app instances
        let mut app_enter = app_with_query(".");
        app_enter.input.editor_mode = EditorMode::Insert;
        app_enter.focus = Focus::InputField;

        let mut app_tab = app_with_query(".");
        app_tab.input.editor_mode = EditorMode::Insert;
        app_tab.focus = Focus::InputField;

        // Set up identical autocomplete suggestions
        let suggestion = crate::autocomplete::Suggestion::new(suggestion_text, suggestion_type.clone());
        app_enter.autocomplete.update_suggestions(vec![suggestion.clone()]);
        app_tab.autocomplete.update_suggestions(vec![suggestion]);

        // Verify both have visible autocomplete
        prop_assert!(app_enter.autocomplete.is_visible());
        prop_assert!(app_tab.autocomplete.is_visible());

        // Press Enter on one, Tab on the other
        app_enter.handle_key_event(key(KeyCode::Enter));
        app_tab.handle_key_event(key(KeyCode::Tab));

        // Both should produce the same query string
        prop_assert_eq!(
            app_enter.query(),
            app_tab.query(),
            "Enter and Tab should produce identical query strings"
        );

        // Both should have autocomplete hidden
        prop_assert!(
            !app_enter.autocomplete.is_visible(),
            "Autocomplete should be hidden after Enter"
        );
        prop_assert!(
            !app_tab.autocomplete.is_visible(),
            "Autocomplete should be hidden after Tab"
        );
    }
}

// Feature: enter-key-autocomplete, Property 2: Enter accepts autocomplete and closes popup
// *For any* application state where the autocomplete popup is visible,
// pressing Enter should result in the autocomplete popup being hidden
// and the selected suggestion text appearing in the query.
// **Validates: Requirements 1.1, 1.2**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop_enter_accepts_autocomplete_and_closes_popup(
        // Generate different suggestion texts
        suggestion_text in prop_oneof![
            Just("name"),
            Just("age"),
            Just("city"),
            Just("services"),
            Just("items"),
        ],
    ) {
        let mut app = app_with_query(".");
        app.input.editor_mode = EditorMode::Insert;
        app.focus = Focus::InputField;

        // Set up autocomplete with a suggestion
        let suggestion = crate::autocomplete::Suggestion::new(
            suggestion_text,
            crate::autocomplete::SuggestionType::Field,
        );
        app.autocomplete.update_suggestions(vec![suggestion]);

        // Verify autocomplete is visible
        prop_assert!(app.autocomplete.is_visible());

        // Press Enter
        app.handle_key_event(key(KeyCode::Enter));

        // Autocomplete should be hidden
        prop_assert!(
            !app.autocomplete.is_visible(),
            "Autocomplete should be hidden after Enter"
        );

        // Query should contain the suggestion text
        prop_assert!(
            app.query().contains(suggestion_text),
            "Query '{}' should contain suggestion text '{}'",
            app.query(),
            suggestion_text
        );

        // Should NOT have quit (autocomplete acceptance, not exit)
        prop_assert!(
            !app.should_quit,
            "Should not quit when accepting autocomplete"
        );
    }
}

// Feature: enter-key-autocomplete, Property 3: Enter exits when autocomplete not visible
// *For any* application state where the autocomplete popup is not visible,
// pressing Enter should set the should_quit flag to true and output_mode to Results.
// **Validates: Requirements 2.1**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop_enter_exits_when_autocomplete_not_visible(
        // Test with different focus states
        focus_on_input in any::<bool>(),
        // Test with different editor modes
        insert_mode in any::<bool>(),
    ) {
        let mut app = app_with_query(".");
        app.focus = if focus_on_input {
            Focus::InputField
        } else {
            Focus::ResultsPane
        };
        app.input.editor_mode = if insert_mode {
            EditorMode::Insert
        } else {
            EditorMode::Normal
        };

        // Ensure autocomplete is NOT visible
        app.autocomplete.hide();
        prop_assert!(!app.autocomplete.is_visible());

        // Press Enter
        app.handle_key_event(key(KeyCode::Enter));

        // Should quit with Results output mode
        prop_assert!(
            app.should_quit,
            "Should quit when Enter pressed without autocomplete"
        );
        prop_assert_eq!(
            app.output_mode,
            Some(OutputMode::Results),
            "Output mode should be Results"
        );
    }
}
