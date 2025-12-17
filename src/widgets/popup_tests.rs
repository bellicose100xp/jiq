//! Tests for widgets/popup

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
