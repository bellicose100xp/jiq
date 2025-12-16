use ratatui::{Frame, layout::Rect, widgets::Clear};

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

pub fn popup_above_anchor(anchor: Rect, width: u16, height: u16, x_offset: u16) -> Rect {
    let popup_x = anchor.x + x_offset;
    let popup_y = anchor.y.saturating_sub(height);

    Rect {
        x: popup_x,
        y: popup_y,
        width: width.min(anchor.width.saturating_sub(x_offset * 2)),
        height: height.min(anchor.y),
    }
}

pub fn inset_rect(area: Rect, horizontal_margin: u16, vertical_margin: u16) -> Rect {
    Rect {
        x: area.x + horizontal_margin,
        y: area.y + vertical_margin,
        width: area.width.saturating_sub(horizontal_margin * 2),
        height: area.height.saturating_sub(vertical_margin * 2),
    }
}

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

        assert_eq!(popup.x, 30);
        assert_eq!(popup.y, 15);
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

        assert_eq!(popup.x, 12);
        assert_eq!(popup.y, 20);
        assert_eq!(popup.width, 60);
        assert_eq!(popup.height, 10);
    }

    #[test]
    fn test_popup_above_anchor_no_overflow() {
        let anchor = Rect {
            x: 0,
            y: 5,
            width: 100,
            height: 3,
        };

        let popup = popup_above_anchor(anchor, 80, 10, 0);

        assert_eq!(popup.y, 0);
        assert_eq!(popup.height, 5);
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

        let inset = inset_rect(area, 20, 20);

        assert_eq!(inset.width, 0);
        assert_eq!(inset.height, 0);
    }
}
