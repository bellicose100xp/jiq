use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::super::app_state::{App, Focus, OutputMode};

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

pub fn handle_global_keys(app: &mut App, key: KeyEvent) -> bool {
    if app.history.is_visible() && key.code != KeyCode::BackTab {
        return false;
    }

    if let Some(query) = &mut app.query
        && crate::ai::ai_events::handle_suggestion_selection(
            key,
            &mut app.ai,
            &mut app.input,
            query,
            &mut app.autocomplete,
        )
    {
        return true;
    }

    if app.help.visible {
        match key.code {
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
            KeyCode::Char('g') | KeyCode::Home => {
                app.help.scroll.jump_to_top();
                return true;
            }
            KeyCode::Char('G') | KeyCode::End => {
                app.help.scroll.jump_to_bottom();
                return true;
            }
            _ => {
                return true;
            }
        }
    }

    match key.code {
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.should_quit = true;
            true
        }
        KeyCode::Char('q') if !key.modifiers.contains(KeyModifiers::CONTROL) => match app.focus {
            Focus::ResultsPane => {
                app.should_quit = true;
                true
            }
            Focus::InputField => {
                if app.input.editor_mode == crate::editor::EditorMode::Normal {
                    app.should_quit = true;
                    true
                } else {
                    false
                }
            }
        },

        KeyCode::Char('q') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            if app.debouncer.has_pending() {
                crate::editor::editor_events::execute_query(app);
                app.debouncer.mark_executed();
            }
            if let Some(query) = &app.query
                && query.result.is_ok()
                && !app.query().is_empty()
            {
                let query_str = app.query().to_string();
                app.history.add_entry(&query_str);
            }
            app.output_mode = Some(OutputMode::Query);
            app.should_quit = true;
            true
        }
        KeyCode::Enter if key.modifiers.contains(KeyModifiers::SHIFT) => {
            if app.debouncer.has_pending() {
                crate::editor::editor_events::execute_query(app);
                app.debouncer.mark_executed();
            }
            if let Some(query) = &app.query
                && query.result.is_ok()
                && !app.query().is_empty()
            {
                let query_str = app.query().to_string();
                app.history.add_entry(&query_str);
            }
            app.output_mode = Some(OutputMode::Query);
            app.should_quit = true;
            true
        }
        KeyCode::Enter if key.modifiers.contains(KeyModifiers::ALT) => {
            if app.debouncer.has_pending() {
                crate::editor::editor_events::execute_query(app);
                app.debouncer.mark_executed();
            }
            if let Some(query) = &app.query
                && query.result.is_ok()
                && !app.query().is_empty()
            {
                let query_str = app.query().to_string();
                app.history.add_entry(&query_str);
            }
            app.output_mode = Some(OutputMode::Query);
            app.should_quit = true;
            true
        }
        KeyCode::Enter => {
            if accept_autocomplete_suggestion(app) {
                return true;
            }

            if app.debouncer.has_pending() {
                crate::editor::editor_events::execute_query(app);
                app.debouncer.mark_executed();
            }
            if let Some(query) = &app.query
                && query.result.is_ok()
                && !app.query().is_empty()
            {
                let query_str = app.query().to_string();
                app.history.add_entry(&query_str);
            }
            app.output_mode = Some(OutputMode::Results);
            app.should_quit = true;
            true
        }

        KeyCode::Tab if !key.modifiers.contains(KeyModifiers::CONTROL) => {
            accept_autocomplete_suggestion(app)
        }

        KeyCode::BackTab => {
            if app.history.is_visible() {
                app.history.close();
            }

            app.focus = match app.focus {
                Focus::InputField => Focus::ResultsPane,
                Focus::ResultsPane => Focus::InputField,
            };
            true
        }

        KeyCode::F(1) => {
            app.help.visible = !app.help.visible;
            true
        }
        KeyCode::Char('?') => {
            if app.input.editor_mode == crate::editor::EditorMode::Normal
                || app.focus == Focus::ResultsPane
            {
                app.help.visible = !app.help.visible;
                true
            } else {
                false
            }
        }

        KeyCode::Char('e') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            if let Some(query) = &app.query
                && query.result.is_err()
            {
                app.error_overlay_visible = !app.error_overlay_visible;
            }
            true
        }

        KeyCode::Char('t') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            crate::tooltip::tooltip_events::handle_tooltip_toggle(&mut app.tooltip);
            true
        }

        KeyCode::Char('f') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            crate::search::search_events::open_search(app);
            true
        }

        KeyCode::Char('a') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            let was_visible = app.ai.visible;
            app.ai.toggle();

            if !was_visible && app.ai.visible {
                // AI popup just became visible - hide tooltip
                app.saved_tooltip_visibility = app.tooltip.enabled;
                app.tooltip.enabled = false;

                // Trigger AI request for current context
                app.trigger_ai_request();
            } else if was_visible && !app.ai.visible {
                // AI popup just became hidden - restore tooltip
                app.tooltip.enabled = app.saved_tooltip_visibility;
            }

            true
        }

        _ => false,
    }
}

#[cfg(test)]
#[path = "global_tests.rs"]
mod global_tests;
