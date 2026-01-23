//! Mouse click handling
//!
//! Handles click events to change focus between UI components.

use ratatui::crossterm::event::MouseEvent;

use super::app_state::{App, Focus};
use crate::ai::ai_events;
use crate::editor::EditorMode;
use crate::layout::Region;

/// Handle left mouse button click for the given region
///
/// Routes click to change focus or activate components.
pub fn handle_click(app: &mut App, region: Option<Region>, mouse: MouseEvent) {
    match region {
        Some(Region::ResultsPane) => click_results_pane(app),
        Some(Region::InputField) => click_input_field(app),
        Some(Region::SearchBar) => click_search_bar(app),
        Some(Region::AiWindow) => click_ai_window(app, mouse),
        _ => {}
    }
}

fn click_results_pane(app: &mut App) {
    if app.focus != Focus::ResultsPane {
        app.focus = Focus::ResultsPane;
    }
}

fn click_input_field(app: &mut App) {
    if app.focus != Focus::InputField {
        app.focus = Focus::InputField;
        app.input.editor_mode = EditorMode::Insert;
    }
}

fn click_search_bar(app: &mut App) {
    if app.search.is_visible() && app.search.is_confirmed() {
        app.search.unconfirm();
    }
}

fn click_ai_window(app: &mut App, mouse: MouseEvent) {
    if !app.ai.visible || app.ai.suggestions.is_empty() {
        return;
    }

    let Some(ai_rect) = app.layout_regions.ai_window else {
        return;
    };

    let inner_x = ai_rect.x.saturating_add(1);
    let inner_y = ai_rect.y.saturating_add(1);
    let inner_width = ai_rect.width.saturating_sub(2);
    let inner_height = ai_rect.height.saturating_sub(2);

    if mouse.column < inner_x
        || mouse.column >= inner_x.saturating_add(inner_width)
        || mouse.row < inner_y
        || mouse.row >= inner_y.saturating_add(inner_height)
    {
        return;
    }

    let relative_y = mouse.row.saturating_sub(inner_y);
    let suggestion_index = app.ai.selection.suggestion_at_y(relative_y);

    if let Some(index) = suggestion_index
        && let Some(suggestion) = app.ai.suggestions.get(index)
    {
        let query_state = match &mut app.query {
            Some(q) => q,
            None => return,
        };

        ai_events::apply_clicked_suggestion(
            suggestion,
            &mut app.input,
            query_state,
            &mut app.autocomplete,
        );
        app.ai.selection.clear_selection();
    }
}

#[cfg(test)]
#[path = "mouse_click_tests.rs"]
mod mouse_click_tests;
