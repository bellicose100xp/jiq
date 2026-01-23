//! Mouse event dispatcher
//!
//! Routes mouse events to appropriate handlers based on position.

use ratatui::crossterm::event::{MouseButton, MouseEvent, MouseEventKind};

use super::app_state::App;
use super::mouse_click;
use super::mouse_hover;
use super::mouse_scroll;
use crate::layout::region_at;

/// Handle mouse events by routing to appropriate handlers
pub fn handle_mouse_event(app: &mut App, mouse: MouseEvent) {
    let region = region_at(&app.layout_regions, mouse.column, mouse.row);

    match mouse.kind {
        MouseEventKind::ScrollDown => {
            mouse_scroll::handle_scroll(app, region, mouse_scroll::ScrollDirection::Down);
        }
        MouseEventKind::ScrollUp => {
            mouse_scroll::handle_scroll(app, region, mouse_scroll::ScrollDirection::Up);
        }
        MouseEventKind::Down(MouseButton::Left) => {
            mouse_click::handle_click(app, region, mouse);
        }
        MouseEventKind::Moved | MouseEventKind::Drag(MouseButton::Left) => {
            mouse_hover::handle_hover(app, region, mouse);
        }
        _ => {}
    }
}
