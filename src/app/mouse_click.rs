//! Mouse click handling
//!
//! Handles click events to change focus between UI components.

use ratatui::crossterm::event::MouseEvent;

use super::app_state::{App, Focus};
use crate::ai::ai_events;
use crate::editor::EditorMode;
use crate::layout::Region;
use crate::snippets::SnippetMode;

/// Handle left mouse button click for the given region
///
/// Routes click to change focus or activate components.
pub fn handle_click(app: &mut App, region: Option<Region>, mouse: MouseEvent) {
    // Dismiss help popup if clicking outside it
    if app.help.visible && region != Some(Region::HelpPopup) {
        app.help.visible = false;
        return;
    }

    // Dismiss error overlay if clicking outside it
    if app.error_overlay_visible && region != Some(Region::ErrorOverlay) {
        app.error_overlay_visible = false;
        return;
    }

    match region {
        Some(Region::ResultsPane) => click_results_pane(app, mouse),
        Some(Region::InputField) => click_input_field(app, mouse),
        Some(Region::SearchBar) => click_search_bar(app),
        Some(Region::AiWindow) => click_ai_window(app, mouse),
        Some(Region::SnippetList) => click_snippet_list(app, mouse),
        Some(Region::HelpPopup) => click_help_popup(app, mouse),
        _ => {}
    }
}

fn click_results_pane(app: &mut App, mouse: MouseEvent) {
    app.focus_results_pane();
    if app.search.is_visible() && !app.search.is_confirmed() {
        app.search.confirm();
    }

    let Some(results_rect) = app.layout_regions.results_pane else {
        return;
    };

    let inner_y = results_rect.y.saturating_add(1);
    let inner_height = results_rect.height.saturating_sub(2);

    if mouse.row < inner_y || mouse.row >= inner_y.saturating_add(inner_height) {
        return;
    }

    let relative_y = mouse.row.saturating_sub(inner_y) as u32;
    let clicked_line = app.results_scroll.offset as u32 + relative_y;

    if clicked_line < app.results_cursor.total_lines() {
        app.results_cursor.click_select(clicked_line);
    }
}

fn click_input_field(app: &mut App, mouse: MouseEvent) {
    // If unfocused, just focus and return (don't move cursor)
    if app.focus != Focus::InputField {
        app.focus_input_field();
        app.input.editor_mode = EditorMode::Insert;
        return;
    }

    // Already focused: position cursor at click location
    let Some(input_rect) = app.layout_regions.input_field else {
        return;
    };

    // Inner area is inside the border (1 char padding on each side)
    let inner_x = input_rect.x.saturating_add(1);
    let inner_width = input_rect.width.saturating_sub(2);

    // Check if click is within the inner horizontal bounds
    if mouse.column < inner_x || mouse.column >= inner_x.saturating_add(inner_width) {
        return;
    }

    // Calculate the character position relative to the visible area
    let relative_x = (mouse.column - inner_x) as usize;

    // Add scroll offset to get the actual character position
    let target_col = app.input.scroll_offset + relative_x;

    // Set cursor to the calculated position
    app.input.set_cursor_column(target_col);
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

fn click_snippet_list(app: &mut App, mouse: MouseEvent) {
    if !app.snippets.is_visible() || *app.snippets.mode() != SnippetMode::Browse {
        return;
    }

    let Some(list_rect) = app.layout_regions.snippet_list else {
        return;
    };

    let inner_x = list_rect.x.saturating_add(1);
    let inner_y = list_rect.y.saturating_add(1);
    let inner_width = list_rect.width.saturating_sub(2);
    let inner_height = list_rect.height.saturating_sub(2);

    if mouse.column < inner_x
        || mouse.column >= inner_x.saturating_add(inner_width)
        || mouse.row < inner_y
        || mouse.row >= inner_y.saturating_add(inner_height)
    {
        return;
    }

    let relative_y = mouse.row.saturating_sub(inner_y);
    if let Some(index) = app.snippets.snippet_at_y(relative_y) {
        app.snippets.select_at(index);
    }
}

fn click_help_popup(app: &mut App, mouse: MouseEvent) {
    if !app.help.visible {
        return;
    }

    let Some(help_rect) = app.layout_regions.help_popup else {
        return;
    };

    // Tab bar is inside the popup border, at the first row of inner area
    let tab_bar_y = help_rect.y.saturating_add(1);
    let inner_x = help_rect.x.saturating_add(1);
    let tab_bar_width = help_rect.width.saturating_sub(2);

    // Only handle clicks on the tab bar row
    if mouse.row != tab_bar_y {
        return;
    }

    // Check horizontal bounds
    if mouse.column < inner_x || mouse.column >= inner_x.saturating_add(tab_bar_width) {
        return;
    }

    let relative_x = mouse.column.saturating_sub(inner_x);
    if let Some(tab) = app.help.tab_at_x(relative_x, tab_bar_width) {
        app.help.active_tab = tab;
    }
}

#[cfg(test)]
#[path = "mouse_click_tests.rs"]
mod mouse_click_tests;
