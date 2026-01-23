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
        Some(Region::AiWindow) => hover_ai_window(app, mouse),
        _ => {
            clear_ai_hover(app);
        }
    }
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

#[cfg(test)]
#[path = "mouse_hover_tests.rs"]
mod mouse_hover_tests;
