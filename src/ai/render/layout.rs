//! Layout calculations for AI popup
//!
//! Handles popup positioning and size calculations.

#![allow(dead_code)] // Phase 1: Reserved for future layout calculations

use ratatui::layout::Rect;

// AI popup display constants
/// Minimum width for the AI popup to ensure readability
pub const AI_POPUP_MIN_WIDTH: u16 = 40;
/// Reserved space for autocomplete area on the left (35 chars + 2 margin)
pub const AUTOCOMPLETE_RESERVED_WIDTH: u16 = 37;
/// Border height (top + bottom)
const BORDER_HEIGHT: u16 = 2;
/// Minimum height for the popup
const MIN_HEIGHT: u16 = 6;
/// Maximum height as percentage of available space (Phase 2: reduced from 50%)
const MAX_HEIGHT_PERCENT: u16 = 40;
/// Maximum width as percentage of available space (Phase 2)
const MAX_WIDTH_PERCENT: u16 = 70;

/// Calculate the AI popup area based on frame dimensions
///
/// The popup is positioned on the right side, above the input bar,
/// reserving space for the autocomplete area on the left.
/// The bottom of the AI popup aligns with the bottom of the autocomplete popup.
///
/// # Arguments
/// * `frame_area` - The full frame area
/// * `input_area` - The input bar area (popup renders above this)
///
/// # Returns
/// A `Rect` for the AI popup, or `None` if there's not enough space
pub fn calculate_popup_area(frame_area: Rect, input_area: Rect) -> Option<Rect> {
    // Calculate available width after reserving autocomplete space
    let available_width = frame_area.width.saturating_sub(AUTOCOMPLETE_RESERVED_WIDTH);

    // Check if we have minimum width
    if available_width < AI_POPUP_MIN_WIDTH {
        return None;
    }

    // Phase 2: Use up to 70% of available width (after autocomplete reservation)
    let max_width = (available_width * MAX_WIDTH_PERCENT) / 100;
    let popup_width = available_width.min(max_width).max(AI_POPUP_MIN_WIDTH);

    // Calculate available height above input bar
    let available_height = input_area.y;

    // Phase 2: Max 40% of available height (reduced from 50%)
    let max_height = (available_height * MAX_HEIGHT_PERCENT) / 100;
    let popup_height = max_height.max(MIN_HEIGHT).min(available_height);

    // Check if we have enough vertical space
    if popup_height < MIN_HEIGHT {
        return None;
    }

    // Position on right side
    let popup_x = frame_area.width.saturating_sub(popup_width + 1);

    // Position above input bar (bottom of popup aligns with top of input)
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
/// - width - 4: accounts for borders (2) and padding (2)
/// - height - 2: accounts for top and bottom borders
/// - / 5: approximate characters per word with spacing (Phase 2.1: more generous)
///
/// # Requirements
/// - 7.1: Formula-based calculation
/// - 7.2: Minimum 100 words
/// - 7.3: Maximum 800 words (Phase 2.1: increased from 500)
/// - 7.5: Pure and deterministic
pub fn calculate_word_limit(width: u16, height: u16) -> u16 {
    let content_width = width.saturating_sub(4); // borders + padding
    let content_height = height.saturating_sub(2); // borders
    let raw_limit = (content_width as u32 * content_height as u32) / 5;
    raw_limit.clamp(100, 800) as u16
}
