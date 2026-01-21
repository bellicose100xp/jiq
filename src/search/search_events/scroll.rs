use crate::app::App;

const SCROLL_MARGIN: u16 = 5;

/// Scroll results pane to make the current match visible (both vertically and horizontally)
/// Uses Neovim-style scrolling with margin instead of centering
pub(super) fn scroll_to_match(app: &mut App) {
    let Some(current_match) = app.search.current_match() else {
        return;
    };

    let target_line = current_match.line.min(u16::MAX as u32) as u16;
    let target_col = current_match.col;
    let match_len = current_match.len;

    // Vertical scrolling - Neovim-style with scroll margin
    let viewport_height = app.results_scroll.viewport_height;
    let current_offset = app.results_scroll.offset;
    let max_offset = app.results_scroll.max_offset;

    if viewport_height > 0 && max_offset > 0 {
        let effective_margin = SCROLL_MARGIN.min(viewport_height / 2);
        let visible_start = current_offset;
        let visible_end = current_offset.saturating_add(viewport_height);

        if target_line < visible_start.saturating_add(effective_margin) {
            // Match is above viewport (or too close to top) - scroll up with margin at top
            let new_offset = target_line.saturating_sub(effective_margin);
            app.results_scroll.offset = new_offset.min(max_offset);
        } else if target_line >= visible_end.saturating_sub(effective_margin) {
            // Match is below viewport (or too close to bottom) - scroll down with margin at bottom
            let new_offset = target_line
                .saturating_add(effective_margin)
                .saturating_add(1)
                .saturating_sub(viewport_height);
            app.results_scroll.offset = new_offset.min(max_offset);
        }
    } else if viewport_height == 0 {
        // Haven't rendered yet, just set offset directly
        app.results_scroll.offset = target_line;
    }

    // Horizontal scrolling - ensure the match is visible horizontally
    let h_offset = app.results_scroll.h_offset;
    let max_h_offset = app.results_scroll.max_h_offset;
    let viewport_width = app.results_scroll.viewport_width;

    // If max_h_offset is 0, content fits horizontally - no scrolling needed
    // If viewport_width is 0, we haven't rendered yet
    if max_h_offset > 0 && viewport_width > 0 {
        let match_end = target_col.saturating_add(match_len);
        let visible_h_start = h_offset;
        let visible_h_end = h_offset.saturating_add(viewport_width);

        // Check if match is fully visible horizontally
        if target_col < visible_h_start || match_end > visible_h_end {
            // Match not fully visible, scroll to show it with some left context
            let left_margin: u16 = 10; // Show some context to the left of the match
            let new_h_offset = target_col.saturating_sub(left_margin);
            let clamped_h_offset = new_h_offset.min(max_h_offset);

            app.results_scroll.h_offset = clamped_h_offset;
        }
    } else if max_h_offset > 0 {
        // viewport_width is 0 (not rendered yet), just position the match with left margin
        let left_margin: u16 = 10;
        let new_h_offset = target_col.saturating_sub(left_margin);
        app.results_scroll.h_offset = new_h_offset.min(max_h_offset);
    }
}

/// Scroll results pane to make the given line visible (legacy function for compatibility)
pub(super) fn scroll_to_line(app: &mut App, _line: u32) {
    // Now delegates to scroll_to_match which handles both vertical and horizontal scrolling
    scroll_to_match(app);
}
