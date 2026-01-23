use ratatui::style::{Modifier, Style};
use tui_textarea::TextArea;

use super::matcher::HistoryMatcher;
use super::storage;
use crate::scroll::Scrollable;

pub const MAX_VISIBLE_HISTORY: usize = 15;

fn create_search_textarea() -> TextArea<'static> {
    let mut textarea = TextArea::default();
    textarea.set_cursor_line_style(Style::default());
    textarea.set_cursor_style(Style::default().add_modifier(Modifier::REVERSED));
    textarea
}

pub struct HistoryState {
    entries: Vec<String>,
    filtered_indices: Vec<usize>,
    search_textarea: TextArea<'static>,
    selected_index: usize,
    scroll_offset: usize,
    visible: bool,
    matcher: HistoryMatcher,
    persist_to_disk: bool,
    cycling_index: Option<usize>,
}

impl Default for HistoryState {
    fn default() -> Self {
        Self::new()
    }
}

impl HistoryState {
    pub fn new() -> Self {
        let entries = storage::load_history();
        let filtered_indices = (0..entries.len()).collect();

        Self {
            entries,
            filtered_indices,
            search_textarea: create_search_textarea(),
            selected_index: 0,
            scroll_offset: 0,
            visible: false,
            matcher: HistoryMatcher::new(),
            persist_to_disk: true,
            cycling_index: None,
        }
    }

    #[cfg(test)]
    pub fn empty() -> Self {
        Self {
            entries: Vec::new(),
            filtered_indices: Vec::new(),
            search_textarea: create_search_textarea(),
            selected_index: 0,
            scroll_offset: 0,
            visible: false,
            matcher: HistoryMatcher::new(),
            persist_to_disk: false,
            cycling_index: None,
        }
    }

    #[cfg(test)]
    pub fn add_entry_in_memory(&mut self, query: &str) {
        if query.trim().is_empty() {
            return;
        }

        self.entries.retain(|e| e != query);
        self.entries.insert(0, query.to_string());
        self.filtered_indices = (0..self.entries.len()).collect();
    }

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
        self.scroll_offset = 0;
    }

    pub fn close(&mut self) {
        self.visible = false;
        self.search_textarea.select_all();
        self.search_textarea.cut();
        self.selected_index = 0;
        self.scroll_offset = 0;
        self.filtered_indices = (0..self.entries.len()).collect();
    }

    pub fn is_visible(&self) -> bool {
        self.visible
    }

    #[cfg(test)]
    pub fn search_query(&self) -> &str {
        self.search_textarea
            .lines()
            .first()
            .map(|s| s.as_str())
            .unwrap_or("")
    }

    pub fn search_textarea_mut(&mut self) -> &mut TextArea<'static> {
        &mut self.search_textarea
    }

    pub fn on_search_input_changed(&mut self) {
        self.update_filter();
        self.selected_index = 0;
        self.scroll_offset = 0;
    }

    pub fn select_next(&mut self) {
        if !self.filtered_indices.is_empty() {
            if self.selected_index + 1 < self.filtered_indices.len() {
                self.selected_index += 1;
            }
            self.adjust_scroll_to_selection();
        }
    }

    pub fn select_previous(&mut self) {
        if !self.filtered_indices.is_empty() {
            if self.selected_index > 0 {
                self.selected_index -= 1;
            }
            self.adjust_scroll_to_selection();
        }
    }

    fn adjust_scroll_to_selection(&mut self) {
        let visible_count = self.filtered_indices.len().min(MAX_VISIBLE_HISTORY);

        if self.selected_index >= self.scroll_offset + visible_count {
            self.scroll_offset = self.selected_index - visible_count + 1;
        } else if self.selected_index < self.scroll_offset {
            self.scroll_offset = self.selected_index;
        }

        let max_offset = self.filtered_indices.len().saturating_sub(visible_count);
        self.scroll_offset = self.scroll_offset.min(max_offset);
    }

    pub fn selected_entry(&self) -> Option<&str> {
        self.filtered_indices
            .get(self.selected_index)
            .and_then(|&idx| self.entries.get(idx))
            .map(String::as_str)
    }

    pub fn selected_index(&self) -> usize {
        self.selected_index
    }

    pub fn total_count(&self) -> usize {
        self.entries.len()
    }

    pub fn filtered_count(&self) -> usize {
        self.filtered_indices.len()
    }

    pub fn visible_entries(&self) -> impl Iterator<Item = (usize, &str)> {
        let entries: Vec<(usize, &str)> = self
            .filtered_indices
            .iter()
            .skip(self.scroll_offset)
            .take(MAX_VISIBLE_HISTORY)
            .enumerate()
            .filter_map(|(display_idx, &entry_idx)| {
                self.entries
                    .get(entry_idx)
                    .map(|e| (self.scroll_offset + display_idx, e.as_str()))
            })
            .collect();

        entries.into_iter().rev()
    }

    pub fn add_entry(&mut self, query: &str) {
        if query.trim().is_empty() {
            return;
        }

        // Only persist to disk if enabled (disabled for tests)
        if self.persist_to_disk
            && let Err(e) = storage::add_entry(query)
        {
            eprintln!("Warning: Failed to save query history to disk: {}", e);
            eprintln!("History will work for this session only.");
            // Continue with in-memory update despite save failure
        }

        self.entries.retain(|e| e != query);
        self.entries.insert(0, query.to_string());

        self.filtered_indices = (0..self.entries.len()).collect();
    }

    fn update_filter(&mut self) {
        let query = self
            .search_textarea
            .lines()
            .first()
            .map(|s| s.as_str())
            .unwrap_or("");
        self.filtered_indices = self.matcher.filter(query, &self.entries);
    }

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

    pub fn reset_cycling(&mut self) {
        self.cycling_index = None;
    }

    /// Get the current scroll offset
    #[allow(dead_code)]
    pub fn scroll_offset(&self) -> usize {
        self.scroll_offset
    }
}

impl Scrollable for HistoryState {
    fn scroll_view_up(&mut self, lines: usize) {
        self.scroll_offset = self.scroll_offset.saturating_sub(lines);
    }

    fn scroll_view_down(&mut self, lines: usize) {
        let max = self.max_scroll();
        self.scroll_offset = (self.scroll_offset + lines).min(max);
    }

    fn scroll_offset(&self) -> usize {
        self.scroll_offset
    }

    fn max_scroll(&self) -> usize {
        self.filtered_indices
            .len()
            .saturating_sub(MAX_VISIBLE_HISTORY)
    }

    fn viewport_size(&self) -> usize {
        MAX_VISIBLE_HISTORY
    }
}

#[cfg(test)]
#[path = "history_state_tests.rs"]
mod history_state_tests;
