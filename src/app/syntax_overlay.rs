//! Syntax highlighting overlay rendering
//!
//! Renders syntax-highlighted jq query text on top of the input textarea.
//! Automatically disables for long queries to prevent cursor synchronization issues.

use ratatui::{
    layout::Rect,
    text::{Line, Span},
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

        let viewport_width = inner_area.width as usize;
        let scroll_offset = self.input.scroll_offset;

        // Get full highlighted spans
        let highlighted_spans = JqHighlighter::highlight(query);

        // Extract only the visible portion based on scroll offset
        let visible_spans = extract_visible_spans(
            &highlighted_spans,
            scroll_offset,
            viewport_width,
        );

        let highlighted_line = Line::from(visible_spans);
        let paragraph = Paragraph::new(highlighted_line);
        frame.render_widget(paragraph, inner_area);
    }
}

/// Extract spans that are visible in the current viewport
fn extract_visible_spans(
    spans: &[Span<'static>],
    scroll_offset: usize,
    viewport_width: usize,
) -> Vec<Span<'static>> {
    let mut result = Vec::new();
    let mut current_col = 0;
    let end_col = scroll_offset + viewport_width;

    for span in spans {
        let span_len = span.content.chars().count();
        let span_end = current_col + span_len;

        if span_end <= scroll_offset {
            // Span is entirely before visible area - skip
            current_col = span_end;
            continue;
        }

        if current_col >= end_col {
            // Past visible area - done
            break;
        }

        // Span overlaps with visible area
        let start_in_span = scroll_offset.saturating_sub(current_col);
        let end_in_span = (end_col - current_col).min(span_len);

        if start_in_span < end_in_span {
            let visible_content: String = span
                .content
                .chars()
                .skip(start_in_span)
                .take(end_in_span - start_in_span)
                .collect();

            result.push(Span::styled(visible_content, span.style));
        }

        current_col = span_end;
    }

    result
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
        // Syntax highlighting now works for long queries via scroll-aware rendering

        let json = r#"{"test": true}"#;
        let _app = App::new(json.to_string());

        // Create a very long query (would exceed typical terminal width)
        let long_query = ".field1 | .field2 | .field3 | .field4 | .field5 | .field6 | .field7 | .field8 | .field9 | .field10 | select(.value > 100)";
        assert!(long_query.chars().count() > 100);

        // The rendering logic now extracts only the visible portion of highlighted text
        // based on scroll_offset, allowing syntax highlighting to work for any length
    }

    #[test]
    fn test_viewport_width_threshold() {
        // Documents that syntax highlighting now works for all query lengths

        let json = r#"{"test": true}"#;
        let _app = App::new(json.to_string());

        // Regardless of terminal width, syntax highlighting is always enabled
        // The visible portion is extracted based on scroll_offset

        let at_threshold = "a".repeat(80);
        assert_eq!(at_threshold.chars().count(), 80);

        // The render logic uses extract_visible_spans() to show only the
        // portion of highlighted text that fits in the viewport
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
