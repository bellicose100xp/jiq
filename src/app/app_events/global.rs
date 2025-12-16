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

    // Handle AI suggestion selection (Alt+1-5, Alt+Up/Down/j/k, Enter)
    // This must be checked before other handlers to allow suggestion selection
    // Requirements 6.1-6.4: Selection works in all editor modes
    if crate::ai::ai_events::handle_suggestion_selection(
        key,
        &mut app.ai,
        &mut app.input,
        &mut app.query,
        &mut app.autocomplete,
    ) {
        return true;
    }

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
        // Requirements 9.1, 9.2, 9.3: Manage tooltip visibility when AI popup toggles
        KeyCode::Char('a') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            let was_visible = app.ai.visible;
            app.ai.toggle();

            // If AI popup is now visible (was hidden, now shown)
            if !was_visible && app.ai.visible {
                // Save current tooltip state and hide it
                app.saved_tooltip_visibility = app.tooltip.enabled;
                app.tooltip.enabled = false;
            }
            // If AI popup is now hidden (was visible, now hidden)
            else if was_visible && !app.ai.visible {
                // Restore saved tooltip state
                app.tooltip.enabled = app.saved_tooltip_visibility;
            }

            true
        }

        _ => false, // Key not handled
    }
}

#[cfg(test)]
#[path = "global_tests.rs"]
mod global_tests;
