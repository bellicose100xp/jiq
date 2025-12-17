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
#[path = "popup_tests.rs"]
mod popup_tests;
