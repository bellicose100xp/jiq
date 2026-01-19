//! Autocomplete acceptance tests

use super::*;

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
        app.query.as_ref().unwrap().base_query_for_suggestions,
        Some(".".to_string()),
        "base_query should remain '.' since .na returns null"
    );
    assert_eq!(
        app.query.as_ref().unwrap().base_type_for_suggestions,
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
        app.query.as_ref().unwrap().base_query_for_suggestions,
        Some(".services".to_string()),
        "base_query should be '.services'"
    );
    assert_eq!(
        app.query.as_ref().unwrap().base_type_for_suggestions,
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
        app.query.as_ref().unwrap().base_query_for_suggestions,
        Some(".services".to_string())
    );
    assert_eq!(
        app.query.as_ref().unwrap().base_type_for_suggestions,
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
        app.query.as_ref().unwrap().base_query_for_suggestions,
        Some(".items[].tags".to_string()),
        "base_query should be '.items[].tags'"
    );
    assert_eq!(
        app.query.as_ref().unwrap().base_type_for_suggestions,
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

// ========== Unit Tests for Enter Key Autocomplete ==========

#[test]
fn test_enter_tab_equivalence_for_autocomplete() {
    let suggestions = [
        ("name", crate::autocomplete::SuggestionType::Field),
        ("age", crate::autocomplete::SuggestionType::Field),
        ("length", crate::autocomplete::SuggestionType::Function),
        ("keys", crate::autocomplete::SuggestionType::Function),
    ];

    for (text, stype) in suggestions {
        let mut app_enter = app_with_query(".");
        app_enter.input.editor_mode = EditorMode::Insert;
        app_enter.focus = Focus::InputField;

        let mut app_tab = app_with_query(".");
        app_tab.input.editor_mode = EditorMode::Insert;
        app_tab.focus = Focus::InputField;

        let suggestion = crate::autocomplete::Suggestion::new(text, stype.clone());
        app_enter
            .autocomplete
            .update_suggestions(vec![suggestion.clone()]);
        app_tab.autocomplete.update_suggestions(vec![suggestion]);

        assert!(app_enter.autocomplete.is_visible());
        assert!(app_tab.autocomplete.is_visible());

        app_enter.handle_key_event(key(KeyCode::Enter));
        app_tab.handle_key_event(key(KeyCode::Tab));

        assert_eq!(
            app_enter.query(),
            app_tab.query(),
            "Enter and Tab should produce identical query strings for '{}'",
            text
        );

        assert!(!app_enter.autocomplete.is_visible());
        assert!(!app_tab.autocomplete.is_visible());
    }
}

#[test]
fn test_enter_accepts_autocomplete_and_closes_popup() {
    let suggestions = ["name", "age", "city", "services", "items"];

    for text in suggestions {
        let mut app = app_with_query(".");
        app.input.editor_mode = EditorMode::Insert;
        app.focus = Focus::InputField;

        let suggestion =
            crate::autocomplete::Suggestion::new(text, crate::autocomplete::SuggestionType::Field);
        app.autocomplete.update_suggestions(vec![suggestion]);

        assert!(app.autocomplete.is_visible());

        app.handle_key_event(key(KeyCode::Enter));

        assert!(
            !app.autocomplete.is_visible(),
            "Autocomplete should be hidden after Enter"
        );
        assert!(
            app.query().contains(text),
            "Query '{}' should contain suggestion text '{}'",
            app.query(),
            text
        );
        assert!(
            !app.should_quit,
            "Should not quit when accepting autocomplete"
        );
    }
}

#[test]
fn test_enter_exits_when_autocomplete_not_visible() {
    let focus_states = [Focus::InputField, Focus::ResultsPane];
    let editor_modes = [EditorMode::Insert, EditorMode::Normal];

    for focus in focus_states {
        for mode in editor_modes {
            let mut app = app_with_query(".");
            app.focus = focus;
            app.input.editor_mode = mode;

            app.autocomplete.hide();
            assert!(!app.autocomplete.is_visible());

            app.handle_key_event(key(KeyCode::Enter));

            assert!(
                app.should_quit,
                "Should quit when Enter pressed without autocomplete (focus: {:?}, mode: {:?})",
                focus, mode
            );
            assert_eq!(
                app.output_mode,
                Some(OutputMode::Results),
                "Output mode should be Results"
            );
        }
    }
}
