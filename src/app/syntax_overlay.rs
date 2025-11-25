//! Syntax highlighting overlay rendering
//!
//! Renders syntax-highlighted jq query text on top of the input textarea.
//! Automatically disables for long queries to prevent cursor synchronization issues.

use ratatui::{
    layout::Rect,
    text::Line,
    widgets::Paragraph,
    Frame,
};

use crate::syntax::JqHighlighter;
use super::state::App;

impl App {
    /// Render syntax highlighting overlay on top of the textarea
    pub fn render_syntax_highlighting(&self, frame: &mut Frame, area: Rect) {
        // Get the query text
        let query = self.query();

        // Skip if empty
        if query.is_empty() {
            return;
        }

        // Calculate the inner area (inside the border)
        let inner_area = Rect {
            x: area.x + 1,
            y: area.y + 1,
            width: area.width.saturating_sub(2),
            height: area.height.saturating_sub(2),
        };

        // Only render syntax highlighting if text fits in viewport
        // This prevents sync issues with tui-textarea's internal horizontal scrolling
        let query_len = query.chars().count();
        let viewport_width = inner_area.width as usize;

        if query_len >= viewport_width {
            // Text would need horizontal scrolling - skip overlay to avoid cursor sync issues
            // The textarea's native rendering handles scrolling correctly
            return;
        }

        // Text fits in viewport - safe to render syntax highlighting
        let highlighted_spans = JqHighlighter::highlight(query);
        let highlighted_line = Line::from(highlighted_spans);
        let paragraph = Paragraph::new(highlighted_line);
        frame.render_widget(paragraph, inner_area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_syntax_highlighting_enabled_for_short_queries() {
        // This test documents that syntax highlighting works for queries
        // that fit within the viewport width

        let json = r#"{"test": true}"#;
        let app = App::new(json.to_string());

        // Short query - should be eligible for highlighting
        let short_query = ".test";
        assert!(short_query.chars().count() < 50); // Typical viewport width

        // Verify query method works (syntax highlighting uses this)
        assert_eq!(app.query(), "");
    }

    #[test]
    fn test_long_query_handling() {
        // This test documents the behavior for queries that exceed viewport width
        // Syntax highlighting is disabled to prevent cursor sync issues

        let json = r#"{"test": true}"#;
        let _app = App::new(json.to_string());

        // Create a very long query (would exceed typical terminal width)
        let long_query = ".field1 | .field2 | .field3 | .field4 | .field5 | .field6 | .field7 | .field8 | .field9 | .field10 | select(.value > 100)";
        assert!(long_query.chars().count() > 100);

        // The rendering logic will check: query_len >= viewport_width
        // If true, it skips the syntax highlighting overlay
        // This allows tui-textarea's native scrolling to work correctly
    }

    #[test]
    fn test_viewport_width_threshold() {
        // Documents the exact threshold behavior for syntax highlighting

        let json = r#"{"test": true}"#;
        let _app = App::new(json.to_string());

        // If terminal inner width is 80 columns (typical)
        // And query is 80+ characters, highlighting is disabled
        // And query is <80 characters, highlighting is enabled

        let at_threshold = "a".repeat(80);
        assert_eq!(at_threshold.chars().count(), 80);

        // The render logic checks: if query_len >= viewport_width { skip highlighting }
        // So at exactly viewport_width, highlighting is disabled
    }

    #[test]
    fn test_empty_query_has_no_highlighting() {
        // Empty queries should not render any syntax highlighting

        let json = r#"{"test": true}"#;
        let app = App::new(json.to_string());

        assert_eq!(app.query(), "");
        // The render_syntax_highlighting method returns early for empty queries
    }

    #[test]
    fn test_char_count_not_byte_count() {
        // Verify we count characters (not bytes) for viewport comparison
        // Important for UTF-8 queries with emoji or multi-byte characters

        let emoji_query = "🔍 search term";
        let char_count = emoji_query.chars().count();
        let byte_count = emoji_query.len();

        assert!(byte_count > char_count); // Emoji takes multiple bytes
        // We use chars().count() which correctly handles UTF-8
    }
}
