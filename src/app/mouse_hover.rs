//! Mouse hover handling
//!
//! Handles hover events to update visual state based on cursor position.

use ratatui::crossterm::event::MouseEvent;

use super::app_state::App;
use crate::layout::Region;

/// Handle mouse hover for the given region
///
/// Updates hover state based on cursor position within components.
pub fn handle_hover(app: &mut App, region: Option<Region>, mouse: MouseEvent) {
    match region {
        Some(Region::ResultsPane) => hover_results_pane(app, mouse),
        Some(Region::AiWindow) => hover_ai_window(app, mouse),
        Some(Region::SnippetList) => hover_snippet_list(app, mouse),
        Some(Region::HelpPopup) => hover_help_popup(app, mouse),
        _ => {
            clear_results_hover(app);
            clear_ai_hover(app);
            clear_snippet_hover(app);
            clear_help_hover(app);
        }
    }
}

/// Handle hover within the results pane
fn hover_results_pane(app: &mut App, mouse: MouseEvent) {
    use ratatui::crossterm::event::MouseEventKind;

    clear_ai_hover(app);
    clear_snippet_hover(app);
    clear_help_hover(app);

    let Some(results_rect) = app.layout_regions.results_pane else {
        app.results_cursor.clear_hover();
        return;
    };

    let inner_y = results_rect.y.saturating_add(1);
    let inner_height = results_rect.height.saturating_sub(2);

    if mouse.row < inner_y || mouse.row >= inner_y.saturating_add(inner_height) {
        app.results_cursor.clear_hover();
        return;
    }

    let relative_y = mouse.row.saturating_sub(inner_y) as u32;
    let hovered_line = app.results_scroll.offset as u32 + relative_y;

    if hovered_line < app.results_cursor.total_lines() {
        if matches!(mouse.kind, MouseEventKind::Drag(_)) && app.results_cursor.is_visual_mode() {
            app.results_cursor.drag_extend(hovered_line);
        }
        app.results_cursor.set_hovered(Some(hovered_line));
    } else {
        app.results_cursor.clear_hover();
    }
}

/// Clear results pane hover state
fn clear_results_hover(app: &mut App) {
    app.results_cursor.clear_hover();
}

/// Handle hover within the AI window
///
/// Calculates which suggestion is under the cursor and updates hover state.
fn hover_ai_window(app: &mut App, mouse: MouseEvent) {
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
        app.ai.selection.clear_hover();
        return;
    }

    let relative_y = mouse.row.saturating_sub(inner_y);
    let suggestion_index = app.ai.selection.suggestion_at_y(relative_y);

    app.ai.selection.set_hovered(suggestion_index);

    if suggestion_index.is_some() && !app.ai.selection.is_navigation_active() {
        app.ai.selection.set_hovered(suggestion_index);
    }
}

/// Clear AI hover state when cursor leaves AI window
fn clear_ai_hover(app: &mut App) {
    if app.ai.selection.get_hovered().is_some() {
        app.ai.selection.clear_hover();
    }
}

/// Handle hover within the snippet list
fn hover_snippet_list(app: &mut App, mouse: MouseEvent) {
    if !app.snippets.is_visible() {
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
        app.snippets.clear_hover();
        return;
    }

    let relative_y = mouse.row.saturating_sub(inner_y);
    let snippet_index = app.snippets.snippet_at_y(relative_y);
    app.snippets.set_hovered(snippet_index);
}

/// Clear snippet hover state when cursor leaves snippet list
fn clear_snippet_hover(app: &mut App) {
    if app.snippets.get_hovered().is_some() {
        app.snippets.clear_hover();
    }
}

/// Handle hover within the help popup tab bar
fn hover_help_popup(app: &mut App, mouse: MouseEvent) {
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

    // Only highlight when hovering the tab bar row
    if mouse.row != tab_bar_y {
        app.help.clear_hovered_tab();
        return;
    }

    // Check horizontal bounds
    if mouse.column < inner_x || mouse.column >= inner_x.saturating_add(tab_bar_width) {
        app.help.clear_hovered_tab();
        return;
    }

    let relative_x = mouse.column.saturating_sub(inner_x);
    let hovered_tab = app.help.tab_at_x(relative_x, tab_bar_width);
    app.help.set_hovered_tab(hovered_tab);
}

/// Clear help popup hover state when cursor leaves
fn clear_help_hover(app: &mut App) {
    if app.help.get_hovered_tab().is_some() {
        app.help.clear_hovered_tab();
    }
}

#[cfg(test)]
#[path = "mouse_hover_tests.rs"]
mod mouse_hover_tests;
