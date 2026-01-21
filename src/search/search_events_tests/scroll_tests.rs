use super::super::*;
use crate::test_utils::test_helpers::test_app;
use proptest::prelude::*;

#[test]
fn test_scroll_to_match_with_margin_when_below_viewport() {
    let mut app = test_app(r#"{"name": "test"}"#);
    app.results_scroll.viewport_height = 20;
    app.results_scroll.max_offset = 100;
    app.results_scroll.offset = 0;

    // Set up a match at line 50 (below viewport 0-20)
    app.search.open();
    app.search.search_textarea_mut().insert_str("test");
    // Manually set matches to control the test
    let content = (0..120)
        .map(|i| if i == 50 { "test\n" } else { "line\n" })
        .collect::<String>();
    app.search.update_matches(&content);

    // Navigate to the match at line 50
    while app.search.current_match().map(|m| m.line) != Some(50) {
        app.search.next_match();
    }

    scroll::scroll_to_match(&mut app);

    // Neovim-style: position match near bottom with margin
    // new_offset = target_line + margin + 1 - viewport_height = 50 + 5 + 1 - 20 = 36
    assert_eq!(app.results_scroll.offset, 36);
}

#[test]
fn test_scroll_to_match_with_margin_when_above_viewport() {
    let mut app = test_app(r#"{"name": "test"}"#);
    app.results_scroll.viewport_height = 20;
    app.results_scroll.max_offset = 100;
    app.results_scroll.offset = 50;

    // Set up a match at line 10 (above viewport 50-70)
    app.search.open();
    app.search.search_textarea_mut().insert_str("test");
    let content = (0..120)
        .map(|i| if i == 10 { "test\n" } else { "line\n" })
        .collect::<String>();
    app.search.update_matches(&content);

    scroll::scroll_to_match(&mut app);

    // Neovim-style: position match near top with margin
    // new_offset = target_line - margin = 10 - 5 = 5
    assert_eq!(app.results_scroll.offset, 5);
}

#[test]
fn test_scroll_to_match_no_scroll_if_visible_with_margin() {
    let mut app = test_app(r#"{"name": "test"}"#);
    app.results_scroll.viewport_height = 20;
    app.results_scroll.max_offset = 100;
    app.results_scroll.offset = 10;

    // Set up a match at line 20 (within viewport 10-30, and outside margin zones)
    // Margin zone at top: 10-15, margin zone at bottom: 25-30
    // Line 20 is safely in the middle
    app.search.open();
    app.search.search_textarea_mut().insert_str("test");
    let content = (0..120)
        .map(|i| if i == 20 { "test\n" } else { "line\n" })
        .collect::<String>();
    app.search.update_matches(&content);

    scroll::scroll_to_match(&mut app);

    // Should not change offset since line 20 is visible and outside margin zones
    assert_eq!(app.results_scroll.offset, 10);
}

#[test]
fn test_scroll_to_match_clamps_to_max() {
    let mut app = test_app(r#"{"name": "test"}"#);
    app.results_scroll.viewport_height = 20;
    app.results_scroll.max_offset = 50;
    app.results_scroll.offset = 0;

    // Set up a match at line 100
    app.search.open();
    app.search.search_textarea_mut().insert_str("test");
    let content = (0..120)
        .map(|i| if i == 100 { "test\n" } else { "line\n" })
        .collect::<String>();
    app.search.update_matches(&content);

    // Navigate to the match at line 100
    while app.search.current_match().map(|m| m.line) != Some(100) {
        app.search.next_match();
    }

    scroll::scroll_to_match(&mut app);

    // Should clamp to max_offset (50)
    assert_eq!(app.results_scroll.offset, 50);
}

#[test]
fn test_scroll_to_match_horizontal() {
    let mut app = test_app(r#"{"name": "test"}"#);
    app.results_scroll.viewport_height = 20;
    app.results_scroll.max_offset = 100;
    app.results_scroll.max_h_offset = 200;
    app.results_scroll.offset = 0;
    app.results_scroll.h_offset = 0;

    // Set up a match at column 150 (beyond typical viewport)
    app.search.open();
    app.search.search_textarea_mut().insert_str("test");
    // Create content with match far to the right
    let content = format!("{}test\n", " ".repeat(150));
    app.search.update_matches(&content);

    scroll::scroll_to_match(&mut app);

    // Should scroll horizontally to show the match (150 - 10 margin = 140)
    assert_eq!(app.results_scroll.h_offset, 140);
}

// Feature: search-in-results, Property 8: Auto-scroll positions match in viewport
// *For any* match navigation, the resulting scroll offset should position the
// match line within the visible viewport.
// **Validates: Requirements 3.3**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop_auto_scroll_positions_match_in_viewport(
        // Generate viewport and scroll parameters
        viewport_height in 5u16..50,
        max_offset in 10u16..200,
        initial_offset in 0u16..100,
        // Constrain target_line to be within the scrollable content
        // (max_offset + viewport_height represents the total content height)
        target_line_factor in 0.0f64..1.0,
    ) {
        let mut app = test_app(r#"{"name": "test"}"#);

        // Set up scroll state
        app.results_scroll.viewport_height = viewport_height;
        app.results_scroll.max_offset = max_offset;
        app.results_scroll.offset = initial_offset.min(max_offset);

        // Calculate target line within valid content range
        // Content height = max_offset + viewport_height (the last visible line when scrolled to max)
        let content_height = max_offset as u32 + viewport_height as u32;
        let target_line = ((target_line_factor * content_height as f64) as u32).min(content_height.saturating_sub(1));

        // Set up a match at the target line so scroll_to_match works
        app.search.open();
        app.search.search_textarea_mut().insert_str("test");
        // Create content with a match at the target line
        let content: String = (0..content_height)
            .map(|i| if i == target_line { "test\n" } else { "line\n" })
            .collect();
        app.search.update_matches(&content);

        // Navigate to the match at target_line (there should be exactly one match)
        prop_assert_eq!(app.search.matches().len(), 1, "Should have exactly one match");
        prop_assert_eq!(app.search.current_match().map(|m| m.line), Some(target_line), "Match should be at target line");

        // Call scroll_to_match
        scroll::scroll_to_match(&mut app);

        // Get the resulting offset
        let result_offset = app.results_scroll.offset;

        // Calculate visible range after scroll
        let visible_start = result_offset as u32;
        let visible_end = visible_start + viewport_height as u32;

        // The target line should be within the visible viewport
        prop_assert!(
            target_line >= visible_start && target_line < visible_end,
            "Target line {} should be within visible range [{}, {}), offset={}, viewport_height={}, max_offset={}",
            target_line, visible_start, visible_end, result_offset, viewport_height, max_offset
        );

        // Verify offset is within valid bounds
        prop_assert!(
            result_offset <= max_offset,
            "Scroll offset {} should not exceed max_offset {}",
            result_offset, max_offset
        );
    }
}
