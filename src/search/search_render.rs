//! Search bar rendering
//!
//! This module handles rendering of the search bar at the bottom of the results pane.

use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders},
    Frame,
};

use crate::app::App;

// Search bar display constants
pub const SEARCH_BAR_HEIGHT: u16 = 3;

/// Render the search bar at the bottom of the results pane
pub fn render_bar(app: &mut App, frame: &mut Frame, area: Rect) {
    // Build match count display for the right side
    let match_count = app.search.match_count_display();
    let match_count_style = if app.search.matches().is_empty() && !app.search.query().is_empty() {
        // No matches found - show in red
        Style::default().fg(Color::Red)
    } else {
        Style::default().fg(Color::Gray)
    };

    // Create the search bar block with cyan border (matching other popups)
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Search: ")
        .title_top(
            Line::from(Span::styled(format!(" {} ", match_count), match_count_style))
                .alignment(Alignment::Right),
        )
        .border_style(Style::default().fg(Color::Cyan))
        .style(Style::default().bg(Color::Black));

    // Calculate inner area for the TextArea
    let inner_area = block.inner(area);

    // Render the block first
    frame.render_widget(block, area);

    // Configure and render the search TextArea
    let search_textarea = app.search.search_textarea_mut();
    search_textarea.set_style(Style::default().fg(Color::White).bg(Color::Black));
    search_textarea.set_cursor_line_style(Style::default());
    frame.render_widget(&*search_textarea, inner_area);
}
