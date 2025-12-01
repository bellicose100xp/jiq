use ratatui::style::{Modifier, Style};
use tui_textarea::TextArea;

use super::matcher::HistoryMatcher;
use super::storage;

/// Maximum number of history items to display in the popup.
pub const MAX_VISIBLE_HISTORY: usize = 15;

/// Creates a TextArea configured for history search input.
fn create_search_textarea() -> TextArea<'static> {
    let mut textarea = TextArea::default();
    textarea.set_cursor_line_style(Style::default());
    textarea.set_cursor_style(Style::default().add_modifier(Modifier::REVERSED));
    textarea
}

/// Manages the state of the history popup.
///
/// Design note on `persist_to_disk`:
/// This flag allows tests to use in-memory-only history without writing to the real
/// history file. While trait-based dependency injection would be more "proper", it would
/// add significant complexity for a single-user CLI tool with minimal benefit.
/// This pragmatic approach keeps the codebase simple while ensuring test isolation.
pub struct HistoryState {
    entries: Vec<String>,
    filtered_indices: Vec<usize>,
    search_textarea: TextArea<'static>,
    selected_index: usize,
    visible: bool,
    matcher: HistoryMatcher,
    /// Controls whether history is persisted to disk (false in tests)
    persist_to_disk: bool,
    cycling_index: Option<usize>,
}

impl Default for HistoryState {
    fn default() -> Self {
        Self::new()
    }
}

impl HistoryState {
    /// Creates a new HistoryState and loads history from disk.
    pub fn new() -> Self {
        let entries = storage::load_history();
        let filtered_indices = (0..entries.len()).collect();

        Self {
            entries,
            filtered_indices,
            search_textarea: create_search_textarea(),
            selected_index: 0,
            visible: false,
            matcher: HistoryMatcher::new(),
            persist_to_disk: true,
            cycling_index: None,
        }
    }

    /// Creates an empty HistoryState without loading from disk.
    /// Useful for testing - does not persist to disk.
    #[cfg(test)]
    pub fn empty() -> Self {
        Self {
            entries: Vec::new(),
            filtered_indices: Vec::new(),
            search_textarea: create_search_textarea(),
            selected_index: 0,
            visible: false,
            matcher: HistoryMatcher::new(),
            persist_to_disk: false,
            cycling_index: None,
        }
    }

    /// Adds an entry to in-memory history only (does NOT persist to disk).
    /// Used for testing to avoid polluting real history file.
    #[cfg(test)]
    pub fn add_entry_in_memory(&mut self, query: &str) {
        if query.trim().is_empty() {
            return;
        }

        self.entries.retain(|e| e != query);
        self.entries.insert(0, query.to_string());
        self.filtered_indices = (0..self.entries.len()).collect();
    }

    /// Opens the history popup with an optional initial search query.
    pub fn open(&mut self, initial_query: Option<&str>) {
        self.visible = true;
        // Clear existing text and set initial query
        self.search_textarea.select_all();
        self.search_textarea.cut();
        if let Some(q) = initial_query {
            self.search_textarea.insert_str(q);
        }
        self.update_filter();
        self.selected_index = 0;
    }

    /// Closes the history popup and resets state.
    pub fn close(&mut self) {
        self.visible = false;
        self.search_textarea.select_all();
        self.search_textarea.cut();
        self.selected_index = 0;
        self.filtered_indices = (0..self.entries.len()).collect();
    }

    /// Returns whether the history popup is visible.
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Returns the current search query (used for testing).
    #[cfg(test)]
    pub fn search_query(&self) -> &str {
        self.search_textarea.lines().first().map(|s| s.as_str()).unwrap_or("")
    }

    /// Returns a mutable reference to the search TextArea for input handling.
    pub fn search_textarea_mut(&mut self) -> &mut TextArea<'static> {
        &mut self.search_textarea
    }

    /// Called after TextArea input to update the filter.
    pub fn on_search_input_changed(&mut self) {
        self.update_filter();
        self.selected_index = 0;
    }

    /// Selects the next item in the filtered list.
    pub fn select_next(&mut self) {
        if !self.filtered_indices.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.filtered_indices.len();
        }
    }

    /// Selects the previous item in the filtered list.
    pub fn select_previous(&mut self) {
        if !self.filtered_indices.is_empty() {
            self.selected_index = if self.selected_index == 0 {
                self.filtered_indices.len() - 1
            } else {
                self.selected_index - 1
            };
        }
    }

    /// Returns the currently selected entry, if any.
    pub fn selected_entry(&self) -> Option<&str> {
        self.filtered_indices
            .get(self.selected_index)
            .and_then(|&idx| self.entries.get(idx))
            .map(String::as_str)
    }

    /// Returns the index of the currently selected item.
    pub fn selected_index(&self) -> usize {
        self.selected_index
    }

    /// Returns the total number of entries (unfiltered).
    pub fn total_count(&self) -> usize {
        self.entries.len()
    }

    /// Returns the number of filtered entries.
    pub fn filtered_count(&self) -> usize {
        self.filtered_indices.len()
    }

    /// Returns an iterator over the visible (filtered) entries with their display indices.
    /// Limited to MAX_VISIBLE_HISTORY items.
    /// Returns entries in reverse order (most recent at bottom, closest to input).
    ///
    /// Note: This allocates a small Vec (~15 items) to enable reversing.
    /// For MAX_VISIBLE_HISTORY=15, this is ~240 bytes - acceptable for render frequency.
    pub fn visible_entries(&self) -> impl Iterator<Item = (usize, &str)> {
        let entries: Vec<(usize, &str)> = self.filtered_indices
            .iter()
            .take(MAX_VISIBLE_HISTORY)
            .enumerate()
            .filter_map(|(original_idx, &entry_idx)| {
                self.entries.get(entry_idx).map(|e| (original_idx, e.as_str()))
            })
            .collect();

        // Reverse the display order but keep original indices for selection highlighting
        entries.into_iter().rev()
    }

    /// Adds a query to the history (saves to disk if persist_to_disk is true).
    ///
    /// If disk save fails, continues with in-memory update and logs error to stderr.
    /// This allows the app to degrade gracefully - history works for current session
    /// even if persistence fails.
    pub fn add_entry(&mut self, query: &str) {
        if query.trim().is_empty() {
            return;
        }

        // Only persist to disk if enabled (disabled for tests)
        if self.persist_to_disk && let Err(e) = storage::add_entry(query) {
            eprintln!("Warning: Failed to save query history to disk: {}", e);
            eprintln!("History will work for this session only.");
            // Continue with in-memory update despite save failure
        }

        self.entries.retain(|e| e != query);
        self.entries.insert(0, query.to_string());

        self.filtered_indices = (0..self.entries.len()).collect();
    }

    /// Updates the filtered indices based on the current search query.
    fn update_filter(&mut self) {
        let query = self.search_textarea.lines().first().map(|s| s.as_str()).unwrap_or("");
        self.filtered_indices = self.matcher.filter(query, &self.entries);
    }

    /// Cycle to the previous history entry (older).
    /// Returns the entry if available.
    pub fn cycle_previous(&mut self) -> Option<String> {
        if self.entries.is_empty() {
            return None;
        }

        let next_idx = match self.cycling_index {
            None => 0,
            Some(idx) if idx + 1 < self.entries.len() => idx + 1,
            Some(idx) => idx, // At end, stay there
        };

        self.cycling_index = Some(next_idx);
        self.entries.get(next_idx).cloned()
    }

    /// Cycle to the next history entry (newer).
    /// Returns the entry if available, or None if at most recent.
    pub fn cycle_next(&mut self) -> Option<String> {
        match self.cycling_index {
            None => None,
            Some(0) => {
                // At most recent, reset cycling
                self.cycling_index = None;
                None
            }
            Some(idx) => {
                let next_idx = idx - 1;
                self.cycling_index = Some(next_idx);
                self.entries.get(next_idx).cloned()
            }
        }
    }

    /// Reset cycling state (called when user types).
    pub fn reset_cycling(&mut self) {
        self.cycling_index = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_state(entries: Vec<&str>) -> HistoryState {
        HistoryState {
            entries: entries.into_iter().map(String::from).collect(),
            filtered_indices: vec![0, 1, 2],
            search_textarea: create_search_textarea(),
            selected_index: 0,
            visible: false,
            matcher: HistoryMatcher::new(),
            persist_to_disk: false,
            cycling_index: None,
        }
    }

    #[test]
    fn test_open_sets_visible() {
        let mut state = create_test_state(vec![".foo", ".bar", ".baz"]);
        state.open(None);
        assert!(state.is_visible());
    }

    #[test]
    fn test_open_with_initial_query() {
        let mut state = create_test_state(vec![".foo", ".bar", ".baz"]);
        state.open(Some(".foo"));
        assert_eq!(state.search_query(), ".foo");
    }

    #[test]
    fn test_close_resets_state() {
        let mut state = create_test_state(vec![".foo", ".bar", ".baz"]);
        state.open(Some("test"));
        state.select_next();
        state.close();

        assert!(!state.is_visible());
        assert!(state.search_query().is_empty());
        assert_eq!(state.selected_index(), 0);
    }

    #[test]
    fn test_navigation_wraps() {
        let mut state = create_test_state(vec![".foo", ".bar", ".baz"]);
        state.filtered_indices = vec![0, 1, 2];

        state.select_previous();
        assert_eq!(state.selected_index(), 2);

        state.select_next();
        assert_eq!(state.selected_index(), 0);
    }

    #[test]
    fn test_selected_entry() {
        let mut state = create_test_state(vec![".foo", ".bar", ".baz"]);
        state.filtered_indices = vec![0, 1, 2];

        assert_eq!(state.selected_entry(), Some(".foo"));

        state.select_next();
        assert_eq!(state.selected_entry(), Some(".bar"));
    }

    #[test]
    fn test_textarea_search_input() {
        let mut state = create_test_state(vec![".foo", ".bar", ".baz"]);

        // Insert text via TextArea
        state.search_textarea_mut().insert_str("fo");
        assert_eq!(state.search_query(), "fo");

        // Clear via select_all + cut
        state.search_textarea_mut().select_all();
        state.search_textarea_mut().cut();
        assert_eq!(state.search_query(), "");
    }

    #[test]
    fn test_visible_entries_limited() {
        let entries: Vec<&str> = (0..20).map(|_| ".test").collect();
        let mut state = create_test_state(entries);
        state.filtered_indices = (0..20).collect();

        let visible: Vec<_> = state.visible_entries().collect();
        assert_eq!(visible.len(), MAX_VISIBLE_HISTORY);
    }

    #[test]
    fn test_empty_navigation() {
        let mut state = create_test_state(vec![]);
        state.filtered_indices = vec![];

        state.select_next();
        state.select_previous();
        assert_eq!(state.selected_index(), 0);
    }

    #[test]
    fn test_single_entry_navigation() {
        let mut state = create_test_state(vec![".only"]);
        state.filtered_indices = vec![0];

        // Should wrap to same entry
        state.select_next();
        assert_eq!(state.selected_index(), 0);
        assert_eq!(state.selected_entry(), Some(".only"));

        state.select_previous();
        assert_eq!(state.selected_index(), 0);
        assert_eq!(state.selected_entry(), Some(".only"));
    }

    #[test]
    fn test_filter_updates_reset_selection() {
        let mut state = create_test_state(vec![".apple", ".banana", ".apricot"]);
        state.filtered_indices = vec![0, 1, 2];
        state.selected_index = 2;

        // Input change resets selection to 0
        state.search_textarea_mut().insert_char('a');
        state.on_search_input_changed();
        assert_eq!(state.selected_index(), 0);
    }

    #[test]
    fn test_selected_entry_with_out_of_bounds_index() {
        let mut state = create_test_state(vec![".foo", ".bar"]);
        state.filtered_indices = vec![0, 1];
        state.selected_index = 5; // Out of bounds

        // Should return None gracefully
        assert_eq!(state.selected_entry(), None);
    }

    #[test]
    fn test_cycling_at_boundaries() {
        let mut state = create_test_state(vec![".first", ".second", ".third"]);

        // Cycle to end
        let e1 = state.cycle_previous();
        let e2 = state.cycle_previous();
        let e3 = state.cycle_previous();
        assert_eq!(e1, Some(".first".to_string()));
        assert_eq!(e2, Some(".second".to_string()));
        assert_eq!(e3, Some(".third".to_string()));

        // Spam Ctrl+P at end - should stay at .third
        let e4 = state.cycle_previous();
        let e5 = state.cycle_previous();
        assert_eq!(e4, Some(".third".to_string()));
        assert_eq!(e5, Some(".third".to_string()));
    }

    #[test]
    fn test_cycling_forward_to_none() {
        let mut state = create_test_state(vec![".first", ".second"]);

        // Cycle back
        state.cycle_previous();
        state.cycle_previous();

        // Cycle forward
        let e1 = state.cycle_next();
        assert_eq!(e1, Some(".first".to_string()));

        // Cycle forward to most recent
        let e2 = state.cycle_next();
        assert_eq!(e2, None); // At most recent, should reset
    }

    #[test]
    fn test_reset_cycling() {
        let mut state = create_test_state(vec![".first", ".second"]);

        state.cycle_previous();
        state.cycle_previous();
        assert_eq!(state.cycling_index, Some(1));

        state.reset_cycling();
        assert_eq!(state.cycling_index, None);

        // Next cycle should start fresh
        let entry = state.cycle_previous();
        assert_eq!(entry, Some(".first".to_string()));
    }
}
