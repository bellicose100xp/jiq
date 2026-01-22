//! Global key handler tests
//!
//! Tests for quit commands, output modes, focus switching, and general key handling

use super::*;
use crate::history::HistoryState;

// ========== Quit and Output Mode Tests ==========

#[test]
fn test_ctrl_c_sets_quit_flag() {
    let mut app = app_with_query(".");

    app.handle_key_event(key_with_mods(KeyCode::Char('c'), KeyModifiers::CONTROL));

    assert!(app.should_quit);
}

#[test]
fn test_q_sets_quit_flag_in_normal_mode() {
    let mut app = app_with_query(".");
    app.input.editor_mode = EditorMode::Normal;

    app.handle_key_event(key(KeyCode::Char('q')));

    assert!(app.should_quit);
}

#[test]
fn test_q_does_not_quit_in_insert_mode() {
    let mut app = app_with_query(".");
    app.input.editor_mode = EditorMode::Insert;

    app.handle_key_event(key(KeyCode::Char('q')));

    // Should NOT quit - 'q' should be typed instead
    assert!(!app.should_quit);
    assert_eq!(app.query(), ".q");
}

#[test]
fn test_enter_sets_results_output_mode() {
    let mut app = app_with_query(".");

    app.handle_key_event(key(KeyCode::Enter));

    assert_eq!(app.output_mode, Some(OutputMode::Results));
    assert!(app.should_quit);
}

#[test]
fn test_enter_saves_successful_query_to_history() {
    // CRITICAL: Enter should save query to history before exiting
    let mut app = app_with_query(".name");
    let initial_count = app.history.total_count();

    // Ensure query is successful
    assert!(app.query.as_ref().unwrap().result.is_ok());

    app.handle_key_event(key(KeyCode::Enter));

    // History should have one more entry
    assert_eq!(app.history.total_count(), initial_count + 1);
    assert!(app.should_quit);
}

#[test]
fn test_enter_does_not_save_failed_query_to_history() {
    // Failed queries should NOT be saved to history
    let mut app = test_app(r#"{"name": "test"}"#);
    app.input.editor_mode = EditorMode::Insert;

    // Type invalid query
    app.handle_key_event(key(KeyCode::Char('|')));
    // Flush debounced query to execute immediately
    flush_debounced_query(&mut app);
    let initial_count = app.history.total_count();

    // Ensure query failed
    assert!(app.query.as_ref().unwrap().result.is_err());

    app.handle_key_event(key(KeyCode::Enter));

    // History should NOT have changed
    assert_eq!(app.history.total_count(), initial_count);
    assert!(app.should_quit);
}

#[test]
fn test_enter_does_not_save_empty_query_to_history() {
    // Empty queries should NOT be saved to history
    let mut app = app_with_query("");
    let initial_count = app.history.total_count();

    app.handle_key_event(key(KeyCode::Enter));

    // History should NOT have changed
    assert_eq!(app.history.total_count(), initial_count);
    assert!(app.should_quit);
}

// ========== Ctrl+Q Tests ==========

#[test]
fn test_ctrl_q_outputs_query_and_saves_successful_query() {
    let mut app = app_with_query(".name");
    let initial_count = app.history.total_count();

    app.handle_key_event(key_with_mods(KeyCode::Char('q'), KeyModifiers::CONTROL));

    assert_eq!(app.history.total_count(), initial_count + 1);
    assert_eq!(app.output_mode, Some(OutputMode::Query));
    assert!(app.should_quit);
}

#[test]
fn test_ctrl_q_does_not_save_failed_query() {
    let mut app = test_app(TEST_JSON);
    app.input.editor_mode = EditorMode::Insert;
    app.history = HistoryState::empty();

    // Type invalid query
    app.handle_key_event(key(KeyCode::Char('|')));
    // Flush debounced query to execute immediately
    flush_debounced_query(&mut app);
    let initial_count = app.history.total_count();

    // Ensure query failed
    assert!(app.query.as_ref().unwrap().result.is_err());

    app.handle_key_event(key_with_mods(KeyCode::Char('q'), KeyModifiers::CONTROL));

    // Should NOT save to history
    assert_eq!(app.history.total_count(), initial_count);
    // But should still exit with query output mode
    assert_eq!(app.output_mode, Some(OutputMode::Query));
    assert!(app.should_quit);
}

// ========== Shift+Enter Tests ==========

#[test]
fn test_shift_enter_outputs_query_and_saves_successful_query() {
    let mut app = app_with_query(".name");
    let initial_count = app.history.total_count();

    app.handle_key_event(key_with_mods(KeyCode::Enter, KeyModifiers::SHIFT));

    assert_eq!(app.history.total_count(), initial_count + 1);
    assert_eq!(app.output_mode, Some(OutputMode::Query));
    assert!(app.should_quit);
}

#[test]
fn test_shift_enter_does_not_save_failed_query() {
    let mut app = test_app(TEST_JSON);
    app.input.editor_mode = EditorMode::Insert;
    app.history = HistoryState::empty();

    // Type invalid query
    app.handle_key_event(key(KeyCode::Char('|')));
    // Flush debounced query to execute immediately
    flush_debounced_query(&mut app);
    let initial_count = app.history.total_count();

    // Ensure query failed
    assert!(app.query.as_ref().unwrap().result.is_err());

    app.handle_key_event(key_with_mods(KeyCode::Enter, KeyModifiers::SHIFT));

    // Should NOT save to history
    assert_eq!(app.history.total_count(), initial_count);
    // But should still exit with query output mode
    assert_eq!(app.output_mode, Some(OutputMode::Query));
    assert!(app.should_quit);
}

// ========== Alt+Enter Tests ==========

#[test]
fn test_alt_enter_outputs_query_and_saves_successful_query() {
    let mut app = app_with_query(".name");
    let initial_count = app.history.total_count();

    app.handle_key_event(key_with_mods(KeyCode::Enter, KeyModifiers::ALT));

    assert_eq!(app.history.total_count(), initial_count + 1);
    assert_eq!(app.output_mode, Some(OutputMode::Query));
    assert!(app.should_quit);
}

#[test]
fn test_alt_enter_does_not_save_failed_query() {
    let mut app = test_app(TEST_JSON);
    app.input.editor_mode = EditorMode::Insert;
    app.history = HistoryState::empty();

    // Type invalid query
    app.handle_key_event(key(KeyCode::Char('|')));
    // Flush debounced query to execute immediately
    flush_debounced_query(&mut app);
    let initial_count = app.history.total_count();

    // Ensure query failed
    assert!(app.query.as_ref().unwrap().result.is_err());

    app.handle_key_event(key_with_mods(KeyCode::Enter, KeyModifiers::ALT));

    // Should NOT save to history
    assert_eq!(app.history.total_count(), initial_count);
    // But should still exit with query output mode
    assert_eq!(app.output_mode, Some(OutputMode::Query));
    assert!(app.should_quit);
}

// ========== Focus Switching Tests ==========

#[test]
fn test_shift_tab_switches_focus_to_results() {
    let mut app = app_with_query(".");
    app.focus = Focus::InputField;

    app.handle_key_event(key(KeyCode::BackTab));

    assert_eq!(app.focus, Focus::ResultsPane);
}

#[test]
fn test_shift_tab_switches_focus_to_input() {
    let mut app = app_with_query(".");
    app.focus = Focus::ResultsPane;

    app.handle_key_event(key(KeyCode::BackTab));

    assert_eq!(app.focus, Focus::InputField);
}

#[test]
fn test_global_keys_work_regardless_of_focus() {
    let mut app = app_with_query(".");
    app.focus = Focus::ResultsPane;

    app.handle_key_event(key_with_mods(KeyCode::Char('c'), KeyModifiers::CONTROL));

    // Ctrl+C should work even when results pane is focused
    assert!(app.should_quit);
}

// ========== Text Input and Query Execution Tests ==========

#[test]
fn test_insert_mode_text_input_updates_query() {
    let mut app = app_with_query("");
    app.input.editor_mode = EditorMode::Insert;

    // Simulate typing a character
    app.handle_key_event(key(KeyCode::Char('.')));

    assert_eq!(app.query(), ".");
}

#[test]
fn test_query_execution_resets_scroll() {
    let mut app = app_with_query("");
    app.input.editor_mode = EditorMode::Insert;
    app.results_scroll.offset = 50;

    // Insert text which should trigger query execution
    app.handle_key_event(key(KeyCode::Char('.')));

    // Scroll should be reset when query changes
    assert_eq!(app.results_scroll.offset, 0);
}

// ========== UTF-8 Edge Case Tests ==========

#[test]
fn test_history_with_emoji() {
    let mut app = app_with_query("");
    app.input.editor_mode = EditorMode::Insert;

    app.history.add_entry_in_memory(".emoji_field 🚀");

    app.handle_key_event(key_with_mods(KeyCode::Char('p'), KeyModifiers::CONTROL));
    assert_eq!(app.query(), ".emoji_field 🚀");
}

#[test]
fn test_history_with_multibyte_chars() {
    let mut app = app_with_query("");
    app.input.editor_mode = EditorMode::Insert;

    app.history.add_entry_in_memory(".café | .naïve");

    app.handle_key_event(key_with_mods(KeyCode::Char('p'), KeyModifiers::CONTROL));
    assert_eq!(app.query(), ".café | .naïve");
}

#[test]
fn test_history_search_with_unicode() {
    let mut app = app_with_query("");
    app.input.editor_mode = EditorMode::Insert;

    app.history.add_entry_in_memory(".café");
    app.history.add_entry_in_memory(".coffee");

    app.handle_key_event(key_with_mods(KeyCode::Char('r'), KeyModifiers::CONTROL));

    // Search for unicode
    app.handle_key_event(key(KeyCode::Char('c')));
    app.handle_key_event(key(KeyCode::Char('a')));
    app.handle_key_event(key(KeyCode::Char('f')));

    // Should filter to .café
    assert_eq!(app.history.filtered_count(), 1);
}

// ========== Boundary Condition Tests ==========

#[test]
fn test_cycling_stops_at_oldest() {
    let mut app = app_with_query("");
    app.input.editor_mode = EditorMode::Insert;

    app.history.add_entry_in_memory(".first");

    // Cycle to first entry
    app.handle_key_event(key_with_mods(KeyCode::Char('p'), KeyModifiers::CONTROL));
    assert_eq!(app.query(), ".first");

    // Spam Ctrl+P - should stay at .first
    app.handle_key_event(key_with_mods(KeyCode::Char('p'), KeyModifiers::CONTROL));
    app.handle_key_event(key_with_mods(KeyCode::Char('p'), KeyModifiers::CONTROL));
    assert_eq!(app.query(), ".first");
}

#[test]
fn test_history_popup_with_single_entry() {
    let mut app = app_with_query("");
    app.input.editor_mode = EditorMode::Insert;

    app.history.add_entry_in_memory(".single");

    app.handle_key_event(key_with_mods(KeyCode::Char('r'), KeyModifiers::CONTROL));
    assert!(app.history.is_visible());

    // Should wrap on navigation
    app.handle_key_event(key(KeyCode::Up));
    assert_eq!(app.history.selected_index(), 0);

    app.handle_key_event(key(KeyCode::Down));
    assert_eq!(app.history.selected_index(), 0);
}

#[test]
fn test_filter_with_no_matches() {
    let mut app = app_with_query("");
    app.input.editor_mode = EditorMode::Insert;

    app.history.add_entry_in_memory(".foo");
    app.history.add_entry_in_memory(".bar");

    app.handle_key_event(key_with_mods(KeyCode::Char('r'), KeyModifiers::CONTROL));

    // Search for something that doesn't match
    app.handle_key_event(key(KeyCode::Char('x')));
    app.handle_key_event(key(KeyCode::Char('y')));
    app.handle_key_event(key(KeyCode::Char('z')));

    // Should have zero matches
    assert_eq!(app.history.filtered_count(), 0);
}

#[test]
fn test_backspace_on_empty_search() {
    let mut app = app_with_query("");
    app.input.editor_mode = EditorMode::Insert;

    app.history.add_entry_in_memory(".test");

    app.handle_key_event(key_with_mods(KeyCode::Char('r'), KeyModifiers::CONTROL));

    // Search is empty initially
    assert_eq!(app.history.search_query(), "");

    // Press backspace - should not crash
    app.handle_key_event(key(KeyCode::Backspace));
    assert_eq!(app.history.search_query(), "");
}

// ========== 'q' key behavior tests ==========

#[test]
fn test_q_quits_in_results_pane_insert_mode() {
    let mut app = app_with_query("");
    app.focus = Focus::ResultsPane;
    app.input.editor_mode = EditorMode::Insert;

    // 'q' should quit even when editor is in Insert mode
    // because we're in ResultsPane (not editing text)
    app.handle_key_event(key(KeyCode::Char('q')));

    assert!(app.should_quit);
}

#[test]
fn test_q_does_not_quit_in_input_field_insert_mode() {
    let mut app = app_with_query("");
    app.focus = Focus::InputField;
    app.input.editor_mode = EditorMode::Insert;

    // 'q' should NOT quit when in InputField with Insert mode
    // (user is typing)
    app.handle_key_event(key(KeyCode::Char('q')));

    assert!(!app.should_quit);
    // The 'q' should be inserted into the query
    assert!(app.query().contains('q'));
}

#[test]
fn test_q_quits_in_input_field_normal_mode() {
    let mut app = app_with_query("");
    app.focus = Focus::InputField;
    app.input.editor_mode = EditorMode::Normal;

    // 'q' should quit when in Normal mode
    app.handle_key_event(key(KeyCode::Char('q')));

    assert!(app.should_quit);
}

#[test]
fn test_q_quits_in_results_pane_normal_mode() {
    let mut app = app_with_query("");
    app.focus = Focus::ResultsPane;
    app.input.editor_mode = EditorMode::Normal;

    // 'q' should quit when in ResultsPane Normal mode
    app.handle_key_event(key(KeyCode::Char('q')));

    assert!(app.should_quit);
}

#[test]
fn test_focus_switch_preserves_editor_mode() {
    let mut app = app_with_query("");
    app.focus = Focus::InputField;
    app.input.editor_mode = EditorMode::Insert;

    // Switch to ResultsPane
    app.handle_key_event(key(KeyCode::BackTab));

    // Editor mode should still be Insert
    assert_eq!(app.focus, Focus::ResultsPane);
    assert_eq!(app.input.editor_mode, EditorMode::Insert);

    // 'q' should quit in ResultsPane even with Insert mode
    app.handle_key_event(key(KeyCode::Char('q')));
    assert!(app.should_quit);
}

#[test]
fn test_backtab_hides_ai_popup_when_switching_to_results() {
    let mut app = app_with_query(".");
    app.focus = Focus::InputField;
    app.ai.visible = true;

    // Switch to ResultsPane
    app.handle_key_event(key(KeyCode::BackTab));

    assert_eq!(app.focus, Focus::ResultsPane);
    assert!(!app.ai.visible);
    assert!(app.saved_ai_visibility_for_results);
}

#[test]
fn test_backtab_hides_tooltip_when_switching_to_results() {
    let mut app = app_with_query(".");
    app.focus = Focus::InputField;
    app.tooltip.enabled = true;

    // Switch to ResultsPane
    app.handle_key_event(key(KeyCode::BackTab));

    assert_eq!(app.focus, Focus::ResultsPane);
    assert!(!app.tooltip.enabled);
    assert!(app.saved_tooltip_visibility_for_results);
}

#[test]
fn test_backtab_restores_ai_popup_when_switching_to_input() {
    let mut app = app_with_query(".");
    app.focus = Focus::ResultsPane;
    app.saved_ai_visibility_for_results = true;
    app.ai.visible = false;

    // Switch to InputField
    app.handle_key_event(key(KeyCode::BackTab));

    assert_eq!(app.focus, Focus::InputField);
    assert!(app.ai.visible);
}

#[test]
fn test_backtab_restores_tooltip_when_switching_to_input() {
    let mut app = app_with_query(".");
    app.focus = Focus::ResultsPane;
    app.saved_tooltip_visibility_for_results = true;
    app.tooltip.enabled = false;

    // Switch to InputField
    app.handle_key_event(key(KeyCode::BackTab));

    assert_eq!(app.focus, Focus::InputField);
    assert!(app.tooltip.enabled);
}

#[test]
fn test_backtab_round_trip_preserves_popup_state() {
    let mut app = app_with_query(".");
    app.focus = Focus::InputField;
    app.ai.visible = true;
    app.tooltip.enabled = true;

    // Switch to ResultsPane
    app.handle_key_event(key(KeyCode::BackTab));
    assert_eq!(app.focus, Focus::ResultsPane);
    assert!(!app.ai.visible);
    assert!(!app.tooltip.enabled);

    // Switch back to InputField
    app.handle_key_event(key(KeyCode::BackTab));
    assert_eq!(app.focus, Focus::InputField);
    assert!(app.ai.visible);
    assert!(app.tooltip.enabled);
}

// ========== Tooltip Toggle Tests (Ctrl+T) ==========

#[test]
fn test_tooltip_initializes_enabled() {
    let app = test_app(TEST_JSON);
    assert!(app.tooltip.enabled);
}

#[test]
fn test_ctrl_t_toggles_tooltip_from_enabled() {
    let mut app = app_with_query(".");
    assert!(app.tooltip.enabled);

    app.handle_key_event(key_with_mods(KeyCode::Char('t'), KeyModifiers::CONTROL));
    assert!(!app.tooltip.enabled);
}

#[test]
fn test_ctrl_t_toggles_tooltip_from_disabled() {
    let mut app = app_with_query(".");
    app.tooltip.toggle(); // disable first
    assert!(!app.tooltip.enabled);

    app.handle_key_event(key_with_mods(KeyCode::Char('t'), KeyModifiers::CONTROL));
    assert!(app.tooltip.enabled);
}

#[test]
fn test_ctrl_t_works_in_insert_mode() {
    let mut app = app_with_query(".");
    app.input.editor_mode = EditorMode::Insert;
    app.focus = Focus::InputField;
    assert!(app.tooltip.enabled);

    app.handle_key_event(key_with_mods(KeyCode::Char('t'), KeyModifiers::CONTROL));
    assert!(!app.tooltip.enabled);
}

#[test]
fn test_ctrl_t_works_in_normal_mode() {
    let mut app = app_with_query(".");
    app.input.editor_mode = EditorMode::Normal;
    app.focus = Focus::InputField;
    assert!(app.tooltip.enabled);

    app.handle_key_event(key_with_mods(KeyCode::Char('t'), KeyModifiers::CONTROL));
    assert!(!app.tooltip.enabled);
}

#[test]
fn test_ctrl_t_works_when_results_pane_focused() {
    let mut app = app_with_query(".");
    app.focus = Focus::ResultsPane;
    assert!(app.tooltip.enabled);

    app.handle_key_event(key_with_mods(KeyCode::Char('t'), KeyModifiers::CONTROL));
    assert!(!app.tooltip.enabled);
}

#[test]
fn test_ctrl_t_preserves_current_function() {
    let mut app = app_with_query("select(.x)");
    app.tooltip.set_current_function(Some("select".to_string()));
    assert!(app.tooltip.enabled);

    app.handle_key_event(key_with_mods(KeyCode::Char('t'), KeyModifiers::CONTROL));

    // Should toggle enabled but preserve current_function
    assert!(!app.tooltip.enabled);
    assert_eq!(app.tooltip.current_function, Some("select".to_string()));
}

#[test]
fn test_ctrl_t_round_trip() {
    let mut app = app_with_query(".");
    let initial_enabled = app.tooltip.enabled;

    // Toggle twice should return to original state
    app.handle_key_event(key_with_mods(KeyCode::Char('t'), KeyModifiers::CONTROL));
    app.handle_key_event(key_with_mods(KeyCode::Char('t'), KeyModifiers::CONTROL));

    assert_eq!(app.tooltip.enabled, initial_enabled);
}

// ========== Dispatch Order and Focus Tests ==========

#[test]
fn test_tab_does_not_accept_autocomplete_in_results_pane() {
    // Critical: Tab should only accept autocomplete when InputField is focused
    let mut app = app_with_query(".na");
    app.input.editor_mode = EditorMode::Insert;
    app.focus = Focus::ResultsPane; // Focus on RESULTS, not input

    let suggestions = vec![crate::autocomplete::Suggestion::new(
        ".name",
        crate::autocomplete::SuggestionType::Field,
    )];
    app.autocomplete.update_suggestions(suggestions);
    assert!(app.autocomplete.is_visible());

    // Press Tab while results pane is focused
    app.handle_key_event(key(KeyCode::Tab));

    // Should NOT accept autocomplete (focus check prevents it)
    assert_eq!(app.query(), ".na"); // Query unchanged
    assert!(app.autocomplete.is_visible()); // Still visible
}

#[test]
fn test_vim_navigation_blocked_when_help_visible() {
    // Critical: VIM navigation should be blocked when help popup is open
    let mut app = app_with_query(".test");
    app.input.editor_mode = EditorMode::Normal;
    app.focus = Focus::InputField;
    app.help.visible = true;

    // Try VIM navigation - should be blocked by help popup
    app.handle_key_event(key(KeyCode::Char('h'))); // Move left
    app.handle_key_event(key(KeyCode::Char('l'))); // Move right
    app.handle_key_event(key(KeyCode::Char('w'))); // Word forward
    app.handle_key_event(key(KeyCode::Char('x'))); // Delete char

    // Query should be unchanged (all keys blocked by help)
    assert_eq!(app.query(), ".test");
    assert!(app.help.visible);
}

#[test]
fn test_history_popup_enter_not_intercepted_by_global() {
    // Critical: Enter in history popup should select entry, not output mode
    let mut app = app_with_query("");
    app.input.editor_mode = EditorMode::Insert;

    app.history.add_entry_in_memory(".selected");

    // Open history popup
    app.handle_key_event(key_with_mods(KeyCode::Char('r'), KeyModifiers::CONTROL));
    assert!(app.history.is_visible());

    // Press Enter - should select from history, NOT set output mode
    app.handle_key_event(key(KeyCode::Enter));

    // History should be closed and query should be selected entry
    assert!(!app.history.is_visible());
    assert_eq!(app.query(), ".selected");
    assert!(app.output_mode.is_none()); // Should NOT set output mode
    assert!(!app.should_quit); // Should NOT quit
}

// ========== Debouncer Flush Tests ==========

#[test]
fn test_ctrl_q_executes_pending_query_before_exit() {
    let mut app = test_app(TEST_JSON);
    app.input.editor_mode = EditorMode::Insert;

    // Type a query without executing (debouncer should have it pending)
    app.handle_key_event(key(KeyCode::Char('.')));
    app.handle_key_event(key(KeyCode::Char('n')));
    app.handle_key_event(key(KeyCode::Char('a')));
    app.handle_key_event(key(KeyCode::Char('m')));
    app.handle_key_event(key(KeyCode::Char('e')));

    // Debouncer should have pending query
    assert!(app.debouncer.has_pending());

    // Press Ctrl+Q - should execute pending query before exit
    app.handle_key_event(key_with_mods(KeyCode::Char('q'), KeyModifiers::CONTROL));

    assert!(app.should_quit);
    assert_eq!(app.output_mode, Some(OutputMode::Query));
}

#[test]
fn test_shift_enter_executes_pending_query_before_exit() {
    let mut app = test_app(TEST_JSON);
    app.input.editor_mode = EditorMode::Insert;

    // Type a query without executing
    app.handle_key_event(key(KeyCode::Char('.')));
    app.handle_key_event(key(KeyCode::Char('t')));
    app.handle_key_event(key(KeyCode::Char('e')));
    app.handle_key_event(key(KeyCode::Char('s')));
    app.handle_key_event(key(KeyCode::Char('t')));

    // Debouncer should have pending query
    assert!(app.debouncer.has_pending());

    // Press Shift+Enter - should execute pending query before exit
    app.handle_key_event(key_with_mods(KeyCode::Enter, KeyModifiers::SHIFT));

    assert!(app.should_quit);
    assert_eq!(app.output_mode, Some(OutputMode::Query));
}

#[test]
fn test_alt_enter_executes_pending_query_before_exit() {
    let mut app = test_app(TEST_JSON);
    app.input.editor_mode = EditorMode::Insert;

    // Type a query without executing
    app.handle_key_event(key(KeyCode::Char('.')));
    app.handle_key_event(key(KeyCode::Char('i')));
    app.handle_key_event(key(KeyCode::Char('d')));

    // Debouncer should have pending query
    assert!(app.debouncer.has_pending());

    // Press Alt+Enter - should execute pending query before exit
    app.handle_key_event(key_with_mods(KeyCode::Enter, KeyModifiers::ALT));

    assert!(app.should_quit);
    assert_eq!(app.output_mode, Some(OutputMode::Query));
}

#[test]
fn test_enter_executes_pending_query_before_exit() {
    let mut app = test_app(TEST_JSON);
    app.input.editor_mode = EditorMode::Insert;

    // Type a query without executing - use a query that won't trigger autocomplete
    app.handle_key_event(key(KeyCode::Char('[')));
    app.handle_key_event(key(KeyCode::Char('0')));
    app.handle_key_event(key(KeyCode::Char(']')));

    // Debouncer should have pending query
    assert!(app.debouncer.has_pending());

    // Press Enter - should execute pending query before exit
    app.handle_key_event(key(KeyCode::Enter));

    assert!(app.should_quit);
    assert_eq!(app.output_mode, Some(OutputMode::Results));
}

// ========== Search Tests (Ctrl+F) ==========

#[test]
fn test_ctrl_f_opens_search() {
    let mut app = app_with_query(".test");

    // Initially search should not be active
    assert!(!app.search.is_visible());

    // Press Ctrl+F - should open search
    app.handle_key_event(key_with_mods(KeyCode::Char('f'), KeyModifiers::CONTROL));

    assert!(app.search.is_visible());
}

#[test]
fn test_ctrl_f_works_in_insert_mode() {
    let mut app = app_with_query(".test");
    app.input.editor_mode = EditorMode::Insert;
    app.focus = Focus::InputField;

    app.handle_key_event(key_with_mods(KeyCode::Char('f'), KeyModifiers::CONTROL));

    assert!(app.search.is_visible());
}

#[test]
fn test_ctrl_f_works_in_results_pane() {
    let mut app = app_with_query(".test");
    app.focus = Focus::ResultsPane;

    app.handle_key_event(key_with_mods(KeyCode::Char('f'), KeyModifiers::CONTROL));

    assert!(app.search.is_visible());
}
