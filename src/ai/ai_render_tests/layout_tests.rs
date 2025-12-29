//! Layout calculation tests for AI render module

use super::*;
use crate::ai::render::layout::{AI_POPUP_MIN_WIDTH, AUTOCOMPLETE_RESERVED_WIDTH};
use proptest::prelude::*;
use ratatui::layout::Rect;

// =========================================================================
// Property-Based Tests
// =========================================================================

// **Feature: ai-assistant, Property 17: Autocomplete area reservation**
// *For any* frame width and AI popup visibility, the popup x-position should be ≥ 37
// (35 chars autocomplete + 2 char margin).
// **Validates: Requirements 8.2**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop_autocomplete_area_reservation(
        frame_width in 80u16..300u16,
        frame_height in 20u16..100u16,
        input_y in 10u16..50u16
    ) {
        let input_y = input_y.min(frame_height.saturating_sub(4));
        let frame = Rect { x: 0, y: 0, width: frame_width, height: frame_height };
        // Input area at bottom of screen (3 lines high)
        let input = Rect { x: 0, y: input_y, width: frame_width, height: 3 };

        if let Some(area) = calculate_popup_area(frame, input) {
            // The popup x-position should leave room for autocomplete (37 chars)
            prop_assert!(
                area.x >= AUTOCOMPLETE_RESERVED_WIDTH,
                "Popup x ({}) should be >= {} to reserve autocomplete area",
                area.x,
                AUTOCOMPLETE_RESERVED_WIDTH
            );
        }
        // If None is returned, that's acceptable - not enough space
    }
}

// **Feature: ai-assistant, Property 18: Minimum popup width**
// *For any* frame width ≥ 80, the AI popup width should be ≥ 40 characters.
// **Validates: Requirements 8.3**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop_minimum_popup_width(
        frame_width in 80u16..300u16,
        frame_height in 20u16..100u16,
        input_y in 10u16..50u16
    ) {
        let input_y = input_y.min(frame_height.saturating_sub(4));
        let frame = Rect { x: 0, y: 0, width: frame_width, height: frame_height };
        // Input area at bottom of screen (3 lines high)
        let input = Rect { x: 0, y: input_y, width: frame_width, height: 3 };

        if let Some(area) = calculate_popup_area(frame, input) {
            // For frame width >= 80, popup should have minimum width
            prop_assert!(
                area.width >= AI_POPUP_MIN_WIDTH,
                "Popup width ({}) should be >= {} for frame width {}",
                area.width,
                AI_POPUP_MIN_WIDTH,
                frame_width
            );
        }
        // If None is returned, that's acceptable - not enough space
    }
}

// =========================================================================
// Phase 2 Property-Based Tests
// =========================================================================

// **Feature: ai-assistant-phase2, Property 1: Popup width respects maximum**
// *For any* terminal width, the AI popup width SHALL be at most 70% of available width.
// **Validates: Requirements 1.5, 6.2**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop_popup_width_respects_maximum(
        frame_width in 80u16..300u16,
        frame_height in 20u16..100u16,
        input_y in 10u16..50u16
    ) {
        let input_y = input_y.min(frame_height.saturating_sub(4));
        let frame = Rect { x: 0, y: 0, width: frame_width, height: frame_height };
        let input = Rect { x: 0, y: input_y, width: frame_width, height: 3 };

        if let Some(area) = calculate_popup_area(frame, input) {
            let available_width = frame_width.saturating_sub(AUTOCOMPLETE_RESERVED_WIDTH);
            let max_allowed = (available_width * 70) / 100;
            prop_assert!(
                area.width <= max_allowed || area.width == AI_POPUP_MIN_WIDTH,
                "Popup width ({}) should be <= 70% of available ({}) or minimum ({})",
                area.width, max_allowed, AI_POPUP_MIN_WIDTH
            );
        }
    }
}

// **Feature: ai-assistant-phase2, Property 2: Popup height respects maximum**
// *For any* terminal height, the AI popup height SHALL be at most 40% of available vertical space.
// **Validates: Requirements 1.2, 6.4**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop_popup_height_respects_maximum(
        frame_width in 80u16..300u16,
        frame_height in 20u16..100u16,
        input_y in 10u16..50u16
    ) {
        let input_y = input_y.min(frame_height.saturating_sub(4));
        let frame = Rect { x: 0, y: 0, width: frame_width, height: frame_height };
        let input = Rect { x: 0, y: input_y, width: frame_width, height: 3 };

        if let Some(area) = calculate_popup_area(frame, input) {
            let available_height = input_y;
            let max_allowed = (available_height * 40) / 100;
            let min_height = 6u16;
            prop_assert!(
                area.height <= available_height && (area.height <= max_allowed || area.height == min_height),
                "Popup height ({}) should be <= 40% of available ({}) or minimum ({})",
                area.height, max_allowed, min_height
            );
        }
    }
}

// **Feature: ai-assistant-phase2, Property 3: Minimum dimensions enforced**
// *For any* terminal size, the AI popup width SHALL be >= 40 and height >= 6, or not displayed.
// **Validates: Requirements 6.1, 6.3, 6.5**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop_minimum_dimensions_enforced(
        frame_width in 40u16..300u16,
        frame_height in 10u16..100u16,
        input_y in 5u16..50u16
    ) {
        let input_y = input_y.min(frame_height.saturating_sub(4));
        let frame = Rect { x: 0, y: 0, width: frame_width, height: frame_height };
        let input = Rect { x: 0, y: input_y, width: frame_width, height: 3 };

        match calculate_popup_area(frame, input) {
            Some(area) => {
                prop_assert!(
                    area.width >= AI_POPUP_MIN_WIDTH,
                    "Popup width ({}) must be >= minimum ({})",
                    area.width, AI_POPUP_MIN_WIDTH
                );
                let min_height = 6u16;
                prop_assert!(
                    area.height >= min_height,
                    "Popup height ({}) must be >= minimum ({})",
                    area.height, min_height
                );
            }
            None => {
                // If None, it means there wasn't enough space - that's valid
            }
        }
    }
}

// =========================================================================
// Unit Tests
// =========================================================================

#[test]
fn test_calculate_popup_area_basic() {
    let frame = Rect {
        x: 0,
        y: 0,
        width: 120,
        height: 40,
    };
    // Input area at bottom (y=37, height=3)
    let input = Rect {
        x: 0,
        y: 37,
        width: 120,
        height: 3,
    };

    let area = calculate_popup_area(frame, input);
    assert!(area.is_some());

    let area = area.unwrap();
    // Should be on right side, after autocomplete reserved space
    assert!(area.x >= AUTOCOMPLETE_RESERVED_WIDTH);
    assert!(area.width >= AI_POPUP_MIN_WIDTH);
    // Should be positioned above input bar
    assert!(area.y + area.height <= input.y);
}

#[test]
fn test_calculate_popup_area_too_narrow() {
    let frame = Rect {
        x: 0,
        y: 0,
        width: 50,
        height: 40,
    };
    let input = Rect {
        x: 0,
        y: 37,
        width: 50,
        height: 3,
    };

    let area = calculate_popup_area(frame, input);
    // Should return None if not enough space after autocomplete reservation
    // 50 - 37 = 13, which is less than MIN_WIDTH (40)
    assert!(area.is_none());
}

#[test]
fn test_calculate_popup_area_minimum_viable() {
    // Minimum viable: 37 (autocomplete) + 40 (min popup) = 77
    let frame = Rect {
        x: 0,
        y: 0,
        width: 80,
        height: 40,
    };
    let input = Rect {
        x: 0,
        y: 37,
        width: 80,
        height: 3,
    };

    let area = calculate_popup_area(frame, input);
    assert!(area.is_some());

    let area = area.unwrap();
    assert!(area.width >= AI_POPUP_MIN_WIDTH);
}
