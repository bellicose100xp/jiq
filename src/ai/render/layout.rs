//! Layout calculations for AI popup
//!
//! Handles popup positioning and size calculations.

#![allow(dead_code)]

use ratatui::layout::Rect;

pub const AI_POPUP_MIN_WIDTH: u16 = 40;
pub const AUTOCOMPLETE_RESERVED_WIDTH: u16 = 37;
const BORDER_HEIGHT: u16 = 2;
const MIN_HEIGHT: u16 = 6;
const MAX_HEIGHT_PERCENT: u16 = 40;
const MAX_WIDTH_PERCENT: u16 = 70;

/// Calculate the AI popup area based on frame dimensions
///
/// The popup is positioned on the right side, above the input bar,
/// reserving space for the autocomplete area on the left.
///
/// # Arguments
/// * `frame_area` - The full frame area
/// * `input_area` - The input bar area (popup renders above this)
///
/// # Returns
/// A `Rect` for the AI popup, or `None` if there's not enough space
pub fn calculate_popup_area(frame_area: Rect, input_area: Rect) -> Option<Rect> {
    let available_width = frame_area.width.saturating_sub(AUTOCOMPLETE_RESERVED_WIDTH);

    if available_width < AI_POPUP_MIN_WIDTH {
        return None;
    }

    let max_width = (available_width * MAX_WIDTH_PERCENT) / 100;
    let popup_width = available_width.min(max_width).max(AI_POPUP_MIN_WIDTH);

    let available_height = input_area.y;

    let max_height = (available_height * MAX_HEIGHT_PERCENT) / 100;
    let popup_height = max_height.max(MIN_HEIGHT).min(available_height);

    if popup_height < MIN_HEIGHT {
        return None;
    }

    let popup_x = frame_area.width.saturating_sub(popup_width + 1);

    let popup_y = input_area.y.saturating_sub(popup_height);

    Some(Rect {
        x: popup_x,
        y: popup_y,
        width: popup_width,
        height: popup_height,
    })
}

/// Calculate dynamic word limit based on popup dimensions
///
/// Formula: (width - 4) * (height - 2) / 5, clamped to 100-800
pub fn calculate_word_limit(width: u16, height: u16) -> u16 {
    let content_width = width.saturating_sub(4);
    let content_height = height.saturating_sub(2);
    let raw_limit = (content_width as u32 * content_height as u32) / 5;
    raw_limit.clamp(100, 800) as u16
}
