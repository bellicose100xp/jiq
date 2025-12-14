//! Global key handlers
//!
//! This module handles keys that work regardless of which pane has focus,
//! including help popup navigation, quit commands, output mode selection,
//! focus switching, and error overlay toggle.

use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::super::app_state::{App, Focus, OutputMode};

/// Accept autocomplete suggestion when visible in input field
/// Returns true if autocomplete was handled, false otherwise
fn accept_autocomplete_suggestion(app: &mut App) -> bool {
    if app.focus == Focus::InputField && app.autocomplete.is_visible() {
        if let Some(suggestion) = app.autocomplete.selected() {
            let suggestion_clone = suggestion.clone();
            app.insert_autocomplete_suggestion(&suggestion_clone);
            app.debouncer.mark_executed();
            app.update_tooltip();
        }
        return true;
    }
    false
}

/// Handle global keys that work regardless of focus
/// Returns true if key was handled, false otherwise
pub fn handle_global_keys(app: &mut App, key: KeyEvent) -> bool {
    // Don't intercept keys when history popup is visible (except BackTab for focus switch)
    // (Enter, Tab need to be handled by history handler)
    if app.history.is_visible() && key.code != KeyCode::BackTab {
        return false;
    }

    // Note: ESC does NOT close AI popup - only Ctrl+A toggles it
    // This allows ESC to be used for other purposes (closing autocomplete, switching modes)

    // Handle help popup when visible (must be first to block other keys)
    if app.help.visible {
        match key.code {
            // Close help
            KeyCode::Esc | KeyCode::F(1) => {
                app.help.visible = false;
                app.help.scroll.reset();
                return true;
            }
            KeyCode::Char('q') if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                app.help.visible = false;
                app.help.scroll.reset();
                return true;
            }
            KeyCode::Char('?') => {
                app.help.visible = false;
                app.help.scroll.reset();
                return true;
            }
            // Scroll down (j, J, Down, Ctrl+D)
            KeyCode::Char('j') | KeyCode::Down => {
                app.help.scroll.scroll_down(1);
                return true;
            }
            KeyCode::Char('J') => {
                app.help.scroll.scroll_down(10);
                return true;
            }
            KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                app.help.scroll.scroll_down(10);
                return true;
            }
            KeyCode::PageDown => {
                app.help.scroll.scroll_down(10);
                return true;
            }
            // Scroll up (k, K, Up, Ctrl+U, PageUp)
            KeyCode::Char('k') | KeyCode::Up => {
                app.help.scroll.scroll_up(1);
                return true;
            }
            KeyCode::Char('K') => {
                app.help.scroll.scroll_up(10);
                return true;
            }
            KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                app.help.scroll.scroll_up(10);
                return true;
            }
            KeyCode::PageUp => {
                app.help.scroll.scroll_up(10);
                return true;
            }
            // Jump to top/bottom
            KeyCode::Char('g') | KeyCode::Home => {
                app.help.scroll.jump_to_top();
                return true;
            }
            KeyCode::Char('G') | KeyCode::End => {
                app.help.scroll.jump_to_bottom();
                return true;
            }
            _ => {
                // Help popup blocks other keys
                return true;
            }
        }
    }

    // Global keys (work even when help is not visible)
    match key.code {
        // Quit (Ctrl+C always works)
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.should_quit = true;
            true
        }
        // Quit with 'q' in Normal mode (but not in Insert mode input field)
        KeyCode::Char('q') if !key.modifiers.contains(KeyModifiers::CONTROL) => {
            // Only quit in Normal mode or when results pane is focused
            match app.focus {
                Focus::ResultsPane => {
                    app.should_quit = true;
                    true
                }
                Focus::InputField => {
                    // Check editor mode - only quit in Normal mode
                    if app.input.editor_mode == crate::editor::EditorMode::Normal {
                        app.should_quit = true;
                        true
                    } else {
                        false // 'q' in Insert mode is just typing
                    }
                }
            }
        }

        // Output query string: Ctrl+Q (primary), Shift+Enter, or Alt+Enter
        KeyCode::Char('q') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            // Execute any pending debounced query immediately (bypass debounce)
            if app.debouncer.has_pending() {
                crate::editor::editor_events::execute_query(app);
                app.debouncer.mark_executed();
            }
            // Save successful queries to history
            if app.query.result.is_ok() && !app.query().is_empty() {
                let query = app.query().to_string();
                app.history.add_entry(&query);
            }
            app.output_mode = Some(OutputMode::Query);
            app.should_quit = true;
            true
        }
        KeyCode::Enter if key.modifiers.contains(KeyModifiers::SHIFT) => {
            // Execute any pending debounced query immediately (bypass debounce)
            if app.debouncer.has_pending() {
                crate::editor::editor_events::execute_query(app);
                app.debouncer.mark_executed();
            }
            // Save successful queries to history
            if app.query.result.is_ok() && !app.query().is_empty() {
                let query = app.query().to_string();
                app.history.add_entry(&query);
            }
            app.output_mode = Some(OutputMode::Query);
            app.should_quit = true;
            true
        }
        KeyCode::Enter if key.modifiers.contains(KeyModifiers::ALT) => {
            // Execute any pending debounced query immediately (bypass debounce)
            if app.debouncer.has_pending() {
                crate::editor::editor_events::execute_query(app);
                app.debouncer.mark_executed();
            }
            // Save successful queries to history
            if app.query.result.is_ok() && !app.query().is_empty() {
                let query = app.query().to_string();
                app.history.add_entry(&query);
            }
            app.output_mode = Some(OutputMode::Query);
            app.should_quit = true;
            true
        }
        KeyCode::Enter => {
            // Accept autocomplete suggestion if visible (same behavior as Tab)
            if accept_autocomplete_suggestion(app) {
                return true;
            }

            // Fall through to existing exit behavior when autocomplete not visible
            // Execute any pending debounced query immediately (bypass debounce)
            if app.debouncer.has_pending() {
                crate::editor::editor_events::execute_query(app);
                app.debouncer.mark_executed();
            }
            // Save successful queries to history
            if app.query.result.is_ok() && !app.query().is_empty() {
                let query = app.query().to_string();
                app.history.add_entry(&query);
            }
            app.output_mode = Some(OutputMode::Results);
            app.should_quit = true;
            true
        }

        // Accept autocomplete with Tab (only if visible in input field)
        KeyCode::Tab if !key.modifiers.contains(KeyModifiers::CONTROL) => {
            accept_autocomplete_suggestion(app)
        }

        // Switch focus with Shift+Tab
        KeyCode::BackTab => {
            // Close history popup if it's open
            if app.history.is_visible() {
                app.history.close();
            }

            app.focus = match app.focus {
                Focus::InputField => Focus::ResultsPane,
                Focus::ResultsPane => Focus::InputField,
            };
            true
        }

        // Toggle help popup (F1 or ?)
        KeyCode::F(1) => {
            app.help.visible = !app.help.visible;
            true
        }
        KeyCode::Char('?') => {
            // Only toggle help in Normal mode (Insert mode needs '?' for typing)
            if app.input.editor_mode == crate::editor::EditorMode::Normal
                || app.focus == Focus::ResultsPane
            {
                app.help.visible = !app.help.visible;
                true
            } else {
                false
            }
        }

        // Toggle error overlay with Ctrl+E
        KeyCode::Char('e') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            // Only toggle if there's an error to show
            if app.query.result.is_err() {
                app.error_overlay_visible = !app.error_overlay_visible;
            }
            true
        }

        // Toggle tooltip with Ctrl+T (T for Tooltip)
        // Requirements 2.1, 2.2, 2.3: Toggle tooltip state on/off
        KeyCode::Char('t') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            crate::tooltip::tooltip_events::handle_tooltip_toggle(&mut app.tooltip);
            true
        }

        // Open search with Ctrl+F (works from any pane)
        // Requirements 1.1: Ctrl+F opens search from any pane
        KeyCode::Char('f') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            crate::search::search_events::open_search(app);
            true
        }

        // Toggle AI assistant popup with Ctrl+A
        // Requirements 2.1: WHEN a user presses Ctrl+A THEN the AI_Popup SHALL toggle its visibility state
        KeyCode::Char('a') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.ai.toggle();
            true
        }

        _ => false, // Key not handled
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::editor::EditorMode;
    use crate::history::HistoryState;
    use crate::test_utils::test_helpers::{
        TEST_JSON, app_with_query, key, key_with_mods, test_app,
    };
    use tui_textarea::CursorMove;

    // Helper to execute any pending debounced query
    // In tests, we need to manually trigger execution since there's no event loop
    fn flush_debounced_query(app: &mut App) {
        if app.debouncer.has_pending() {
            crate::editor::editor_events::execute_query(app);
            app.debouncer.mark_executed();
        }
    }

    // ========== Error Overlay Tests ==========

    #[test]
    fn test_error_overlay_initializes_hidden() {
        let app = test_app(TEST_JSON);
        assert!(!app.error_overlay_visible);
    }

    #[test]
    fn test_ctrl_e_toggles_error_overlay_when_error_exists() {
        let mut app = test_app(TEST_JSON);
        app.input.editor_mode = EditorMode::Insert;

        // Type an invalid query (| is invalid jq syntax)
        app.handle_key_event(key(KeyCode::Char('|')));
        // Flush debounced query to execute immediately (simulates debounce period passing)
        flush_debounced_query(&mut app);

        // Should have an error now
        assert!(app.query.result.is_err());
        assert!(!app.error_overlay_visible); // Initially hidden

        // Press Ctrl+E to show overlay
        app.handle_key_event(key_with_mods(KeyCode::Char('e'), KeyModifiers::CONTROL));
        assert!(app.error_overlay_visible);

        // Press Ctrl+E again to hide overlay
        app.handle_key_event(key_with_mods(KeyCode::Char('e'), KeyModifiers::CONTROL));
        assert!(!app.error_overlay_visible);
    }

    #[test]
    fn test_ctrl_e_does_nothing_when_no_error() {
        let mut app = test_app(TEST_JSON);
        // Initial query "." should succeed
        assert!(app.query.result.is_ok());
        assert!(!app.error_overlay_visible);

        // Press Ctrl+E (should do nothing since no error)
        app.handle_key_event(key_with_mods(KeyCode::Char('e'), KeyModifiers::CONTROL));
        assert!(!app.error_overlay_visible); // Should remain hidden
    }

    #[test]
    fn test_error_overlay_hides_on_query_change() {
        let mut app = test_app(TEST_JSON);
        app.input.editor_mode = EditorMode::Insert;

        // Type invalid query
        app.handle_key_event(key(KeyCode::Char('|')));
        // Flush debounced query to execute immediately
        flush_debounced_query(&mut app);
        assert!(app.query.result.is_err());

        // Show error overlay
        app.handle_key_event(key_with_mods(KeyCode::Char('e'), KeyModifiers::CONTROL));
        assert!(app.error_overlay_visible);

        // Change query by pressing backspace to delete the invalid character
        app.handle_key_event(key(KeyCode::Backspace));

        // Overlay should auto-hide after query change
        assert!(!app.error_overlay_visible);
    }

    #[test]
    fn test_error_overlay_hides_on_query_change_in_normal_mode() {
        let mut app = test_app(TEST_JSON);
        app.input.editor_mode = EditorMode::Insert;

        // Type invalid query
        app.handle_key_event(key(KeyCode::Char('|')));
        // Flush debounced query to execute immediately
        flush_debounced_query(&mut app);
        assert!(app.query.result.is_err());

        // Show error overlay
        app.handle_key_event(key_with_mods(KeyCode::Char('e'), KeyModifiers::CONTROL));
        assert!(app.error_overlay_visible);

        // Switch to Normal mode and delete the character
        app.handle_key_event(key(KeyCode::Esc));
        app.input.textarea.move_cursor(CursorMove::Head);
        app.handle_key_event(key(KeyCode::Char('x')));

        // Overlay should auto-hide after query change
        assert!(!app.error_overlay_visible);
    }

    #[test]
    fn test_ctrl_e_works_in_normal_mode() {
        let mut app = test_app(TEST_JSON);
        app.input.editor_mode = EditorMode::Insert;

        // Type invalid query
        app.handle_key_event(key(KeyCode::Char('|')));
        // Flush debounced query to execute immediately
        flush_debounced_query(&mut app);
        assert!(app.query.result.is_err());

        // Switch to Normal mode
        app.handle_key_event(key(KeyCode::Esc));
        assert_eq!(app.input.editor_mode, EditorMode::Normal);

        // Press Ctrl+E in Normal mode
        app.handle_key_event(key_with_mods(KeyCode::Char('e'), KeyModifiers::CONTROL));
        assert!(app.error_overlay_visible);
    }

    #[test]
    fn test_ctrl_e_works_when_results_pane_focused() {
        let mut app = test_app(TEST_JSON);
        app.input.editor_mode = EditorMode::Insert;

        // Type invalid query
        app.handle_key_event(key(KeyCode::Char('|')));
        // Flush debounced query to execute immediately
        flush_debounced_query(&mut app);
        assert!(app.query.result.is_err());

        // Switch focus to results pane
        app.handle_key_event(key(KeyCode::BackTab));
        assert_eq!(app.focus, Focus::ResultsPane);

        // Press Ctrl+E while results pane is focused
        app.handle_key_event(key_with_mods(KeyCode::Char('e'), KeyModifiers::CONTROL));
        assert!(app.error_overlay_visible);
    }

    // ========== Global Key Handler Tests ==========

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
        assert!(app.query.result.is_ok());

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
        assert!(app.query.result.is_err());

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
        assert!(app.query.result.is_err());

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
        assert!(app.query.result.is_err());

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
        assert!(app.query.result.is_err());

        app.handle_key_event(key_with_mods(KeyCode::Enter, KeyModifiers::ALT));

        // Should NOT save to history
        assert_eq!(app.history.total_count(), initial_count);
        // But should still exit with query output mode
        assert_eq!(app.output_mode, Some(OutputMode::Query));
        assert!(app.should_quit);
    }

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

    // ========== Help Popup Tests ==========

    #[test]
    fn test_help_popup_initializes_hidden() {
        let app = test_app(TEST_JSON);
        assert!(!app.help.visible);
    }

    #[test]
    fn test_f1_toggles_help_popup() {
        let mut app = app_with_query(".");
        assert!(!app.help.visible);

        app.handle_key_event(key(KeyCode::F(1)));
        assert!(app.help.visible);

        app.handle_key_event(key(KeyCode::F(1)));
        assert!(!app.help.visible);
    }

    #[test]
    fn test_question_mark_toggles_help_in_normal_mode() {
        let mut app = app_with_query(".");
        app.input.editor_mode = EditorMode::Normal;
        app.focus = Focus::InputField;

        app.handle_key_event(key(KeyCode::Char('?')));
        assert!(app.help.visible);

        app.handle_key_event(key(KeyCode::Char('?')));
        assert!(!app.help.visible);
    }

    #[test]
    fn test_question_mark_does_not_toggle_help_in_insert_mode() {
        let mut app = app_with_query("");
        app.input.editor_mode = EditorMode::Insert;
        app.focus = Focus::InputField;

        app.handle_key_event(key(KeyCode::Char('?')));
        // Should type '?' not toggle help
        assert!(!app.help.visible);
        assert!(app.query().contains('?'));
    }

    #[test]
    fn test_esc_closes_help_popup() {
        let mut app = app_with_query(".");
        app.help.visible = true;

        app.handle_key_event(key(KeyCode::Esc));
        assert!(!app.help.visible);
    }

    #[test]
    fn test_q_closes_help_popup() {
        let mut app = app_with_query(".");
        app.help.visible = true;

        app.handle_key_event(key(KeyCode::Char('q')));
        assert!(!app.help.visible);
    }

    #[test]
    fn test_help_popup_blocks_other_keys() {
        let mut app = app_with_query(".");
        app.help.visible = true;
        app.input.editor_mode = EditorMode::Insert;

        // Try to type - should be blocked
        app.handle_key_event(key(KeyCode::Char('x')));
        assert!(!app.query().contains('x'));
        assert!(app.help.visible);
    }

    #[test]
    fn test_f1_works_in_insert_mode() {
        let mut app = app_with_query(".");
        app.input.editor_mode = EditorMode::Insert;
        app.focus = Focus::InputField;

        app.handle_key_event(key(KeyCode::F(1)));
        assert!(app.help.visible);
    }

    #[test]
    fn test_help_popup_scroll_j_scrolls_down() {
        let mut app = app_with_query(".");
        app.help.visible = true;

        // Set up bounds for help content (48 lines + padding, viewport 20)
        app.help.scroll.update_bounds(60, 20);
        app.help.scroll.offset = 0;

        app.handle_key_event(key(KeyCode::Char('j')));
        assert_eq!(app.help.scroll.offset, 1);
    }

    #[test]
    fn test_help_popup_scroll_k_scrolls_up() {
        let mut app = app_with_query(".");
        app.help.visible = true;
        app.help.scroll.offset = 5;

        app.handle_key_event(key(KeyCode::Char('k')));
        assert_eq!(app.help.scroll.offset, 4);
    }

    #[test]
    fn test_help_popup_scroll_down_arrow() {
        let mut app = app_with_query(".");
        app.help.visible = true;

        // Set up bounds for help content
        app.help.scroll.update_bounds(60, 20);
        app.help.scroll.offset = 0;

        app.handle_key_event(key(KeyCode::Down));
        assert_eq!(app.help.scroll.offset, 1);
    }

    #[test]
    fn test_help_popup_scroll_up_arrow() {
        let mut app = app_with_query(".");
        app.help.visible = true;
        app.help.scroll.offset = 5;

        app.handle_key_event(key(KeyCode::Up));
        assert_eq!(app.help.scroll.offset, 4);
    }

    #[test]
    fn test_help_popup_scroll_capital_j_scrolls_10() {
        let mut app = app_with_query(".");
        app.help.visible = true;

        // Set up bounds for help content
        app.help.scroll.update_bounds(60, 20);
        app.help.scroll.offset = 0;

        app.handle_key_event(key(KeyCode::Char('J')));
        assert_eq!(app.help.scroll.offset, 10);
    }

    #[test]
    fn test_help_popup_scroll_capital_k_scrolls_10() {
        let mut app = app_with_query(".");
        app.help.visible = true;
        app.help.scroll.offset = 15;

        app.handle_key_event(key(KeyCode::Char('K')));
        assert_eq!(app.help.scroll.offset, 5);
    }

    #[test]
    fn test_help_popup_scroll_ctrl_d() {
        let mut app = app_with_query(".");
        app.help.visible = true;

        // Set up bounds for help content
        app.help.scroll.update_bounds(60, 20);
        app.help.scroll.offset = 0;

        app.handle_key_event(key_with_mods(KeyCode::Char('d'), KeyModifiers::CONTROL));
        assert_eq!(app.help.scroll.offset, 10);
    }

    #[test]
    fn test_help_popup_scroll_ctrl_u() {
        let mut app = app_with_query(".");
        app.help.visible = true;
        app.help.scroll.offset = 15;

        app.handle_key_event(key_with_mods(KeyCode::Char('u'), KeyModifiers::CONTROL));
        assert_eq!(app.help.scroll.offset, 5);
    }

    #[test]
    fn test_help_popup_scroll_g_jumps_to_top() {
        let mut app = app_with_query(".");
        app.help.visible = true;
        app.help.scroll.offset = 20;

        app.handle_key_event(key(KeyCode::Char('g')));
        assert_eq!(app.help.scroll.offset, 0);
    }

    #[test]
    fn test_help_popup_scroll_capital_g_jumps_to_bottom() {
        let mut app = app_with_query(".");
        app.help.visible = true;

        // Set up bounds for help content
        app.help.scroll.update_bounds(60, 20);
        app.help.scroll.offset = 0;

        app.handle_key_event(key(KeyCode::Char('G')));
        assert_eq!(app.help.scroll.offset, app.help.scroll.max_offset);
    }

    #[test]
    fn test_help_popup_scroll_k_saturates_at_zero() {
        let mut app = app_with_query(".");
        app.help.visible = true;
        app.help.scroll.offset = 0;

        app.handle_key_event(key(KeyCode::Char('k')));
        assert_eq!(app.help.scroll.offset, 0);
    }

    #[test]
    fn test_help_popup_close_resets_scroll() {
        let mut app = app_with_query(".");
        app.help.visible = true;
        app.help.scroll.offset = 10;

        app.handle_key_event(key(KeyCode::Esc));
        assert!(!app.help.visible);
        assert_eq!(app.help.scroll.offset, 0);
    }

    #[test]
    fn test_help_popup_scroll_page_down() {
        let mut app = app_with_query(".");
        app.help.visible = true;

        // Set up bounds for help content
        app.help.scroll.update_bounds(60, 20);
        app.help.scroll.offset = 0;

        app.handle_key_event(key(KeyCode::PageDown));
        assert_eq!(app.help.scroll.offset, 10);
    }

    #[test]
    fn test_help_popup_scroll_page_up() {
        let mut app = app_with_query(".");
        app.help.visible = true;
        app.help.scroll.offset = 15;

        app.handle_key_event(key(KeyCode::PageUp));
        assert_eq!(app.help.scroll.offset, 5);
    }

    #[test]
    fn test_help_popup_scroll_home_jumps_to_top() {
        let mut app = app_with_query(".");
        app.help.visible = true;
        app.help.scroll.offset = 20;

        app.handle_key_event(key(KeyCode::Home));
        assert_eq!(app.help.scroll.offset, 0);
    }

    #[test]
    fn test_help_popup_scroll_end_jumps_to_bottom() {
        let mut app = app_with_query(".");
        app.help.visible = true;

        // Set up bounds for help content
        app.help.scroll.update_bounds(60, 20);
        app.help.scroll.offset = 0;

        app.handle_key_event(key(KeyCode::End));
        assert_eq!(app.help.scroll.offset, app.help.scroll.max_offset);
    }

    // ========== Tab Autocomplete Acceptance Tests ==========

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

    use proptest::prelude::*;

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
}
