/// Common interface for scrollable components
///
/// Provides a unified API for mouse scroll handling in Phase 2.
/// Components implement this trait to support scroll wheel events.
#[allow(dead_code)]
pub trait Scrollable {
    /// Scroll the view up by the given number of lines
    fn scroll_view_up(&mut self, lines: usize);

    /// Scroll the view down by the given number of lines
    fn scroll_view_down(&mut self, lines: usize);

    /// Get the current scroll offset
    fn scroll_offset(&self) -> usize;

    /// Get the maximum scroll offset (content_size - viewport_size)
    fn max_scroll(&self) -> usize;

    /// Get the viewport size (number of visible items/lines)
    fn viewport_size(&self) -> usize;
}
