//! Popup rendering utilities
//!
//! Provides reusable functions for positioning and rendering popup windows
//! like autocomplete suggestions, history, help, and error overlays.

use ratatui::{Frame, layout::Rect, widgets::Clear};

/// Calculate a centered popup rectangle within the given frame area
///
/// # Arguments
/// * `frame_area` - The full frame area to center within
/// * `width` - Desired popup width
/// * `height` - Desired popup height
///
/// # Returns
/// A `Rect` centered within the frame area, clamped to fit
pub fn centered_popup(frame_area: Rect, width: u16, height: u16) -> Rect {
    let popup_width = width.min(frame_area.width);
    let popup_height = height.min(frame_area.height);

    let popup_x = (frame_area.width.saturating_sub(popup_width)) / 2;
    let popup_y = (frame_area.height.saturating_sub(popup_height)) / 2;

    Rect {
        x: popup_x,
        y: popup_y,
        width: popup_width,
        height: popup_height,
    }
}

/// Calculate a popup rectangle positioned above an anchor area
///
/// # Arguments
/// * `anchor` - The area to position above (typically an input field)
/// * `width` - Desired popup width
/// * `height` - Desired popup height
/// * `x_offset` - Horizontal offset from anchor's x position (default: 0)
///
/// # Returns
/// A `Rect` positioned above the anchor, clamped to not overflow
pub fn popup_above_anchor(anchor: Rect, width: u16, height: u16, x_offset: u16) -> Rect {
    let popup_x = anchor.x + x_offset;
    let popup_y = anchor.y.saturating_sub(height);

    Rect {
        x: popup_x,
        y: popup_y,
        width: width.min(anchor.width.saturating_sub(x_offset * 2)),
        height: height.min(anchor.y), // Don't overflow above anchor
    }
}

/// Calculate an inset rectangle with margins applied
///
/// # Arguments
/// * `area` - The parent area to inset from
/// * `horizontal_margin` - Margin on left and right (total: 2x this value)
/// * `vertical_margin` - Margin on top and bottom (total: 2x this value)
///
/// # Returns
/// A `Rect` inset from the parent area by the specified margins
pub fn inset_rect(area: Rect, horizontal_margin: u16, vertical_margin: u16) -> Rect {
    Rect {
        x: area.x + horizontal_margin,
        y: area.y + vertical_margin,
        width: area.width.saturating_sub(horizontal_margin * 2),
        height: area.height.saturating_sub(vertical_margin * 2),
    }
}

/// Clear the background of a popup area to create a floating effect
///
/// This should be called before rendering popup content to ensure
/// the popup appears to float over the background content.
///
/// # Arguments
/// * `frame` - The frame to render to
/// * `area` - The area to clear
pub fn clear_area(frame: &mut Frame, area: Rect) {
    frame.render_widget(Clear, area);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_centered_popup_basic() {
        let frame = Rect {
            x: 0,
            y: 0,
            width: 100,
            height: 50,
        };

        let popup = centered_popup(frame, 40, 20);

        // Should be centered
        assert_eq!(popup.x, 30); // (100 - 40) / 2
        assert_eq!(popup.y, 15); // (50 - 20) / 2
        assert_eq!(popup.width, 40);
        assert_eq!(popup.height, 20);
    }

    #[test]
    fn test_centered_popup_too_large_is_clamped() {
        let frame = Rect {
            x: 0,
            y: 0,
            width: 100,
            height: 50,
        };

        let popup = centered_popup(frame, 200, 100);

        // Should be clamped to frame size
        assert_eq!(popup.width, 100);
        assert_eq!(popup.height, 50);
        assert_eq!(popup.x, 0);
        assert_eq!(popup.y, 0);
    }

    #[test]
    fn test_popup_above_anchor_basic() {
        let anchor = Rect {
            x: 10,
            y: 30,
            width: 80,
            height: 3,
        };

        let popup = popup_above_anchor(anchor, 60, 10, 2);

        // Should be above anchor with offset
        assert_eq!(popup.x, 12); // 10 + 2
        assert_eq!(popup.y, 20); // 30 - 10
        assert_eq!(popup.width, 60);
        assert_eq!(popup.height, 10);
    }

    #[test]
    fn test_popup_above_anchor_no_overflow() {
        let anchor = Rect {
            x: 0,
            y: 5, // Only 5 rows above
            width: 100,
            height: 3,
        };

        let popup = popup_above_anchor(anchor, 80, 10, 0);

        // Height should be clamped to available space above
        assert_eq!(popup.y, 0);
        assert_eq!(popup.height, 5); // Clamped to anchor.y
    }

    #[test]
    fn test_inset_rect_basic() {
        let area = Rect {
            x: 10,
            y: 20,
            width: 100,
            height: 50,
        };

        let inset = inset_rect(area, 5, 3);

        assert_eq!(inset.x, 15); // 10 + 5
        assert_eq!(inset.y, 23); // 20 + 3
        assert_eq!(inset.width, 90); // 100 - 10
        assert_eq!(inset.height, 44); // 50 - 6
    }

    #[test]
    fn test_inset_rect_saturates() {
        let area = Rect {
            x: 0,
            y: 0,
            width: 10,
            height: 10,
        };

        // Request huge margins
        let inset = inset_rect(area, 20, 20);

        // Should saturate at 0
        assert_eq!(inset.width, 0);
        assert_eq!(inset.height, 0);
    }
}
