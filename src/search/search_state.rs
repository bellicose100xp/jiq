use ratatui::style::{Modifier, Style};
use std::collections::HashMap;
use tui_textarea::TextArea;

/// Represents a single match position in the results
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Match {
    /// Line number (0-indexed)
    pub line: u32,
    /// Column position (0-indexed, in characters not bytes)
    pub col: u16,
    /// Length of match in characters
    pub len: u16,
}

/// Creates a TextArea configured for search input.
fn create_search_textarea() -> TextArea<'static> {
    let mut textarea = TextArea::default();
    textarea.set_cursor_line_style(Style::default());
    textarea.set_cursor_style(Style::default().add_modifier(Modifier::REVERSED));
    textarea
}

/// Manages the state of the search feature
pub struct SearchState {
    /// Whether search bar is visible
    visible: bool,
    /// Whether search has been confirmed (Enter pressed)
    /// When confirmed, n/N navigate matches instead of typing
    confirmed: bool,
    /// Search query text input
    search_textarea: TextArea<'static>,
    /// All matches found in results
    matches: Vec<Match>,
    /// Index of current match (for navigation)
    current_index: usize,
    /// Cached query to detect changes
    last_query: String,
    /// Indexed matches by line for O(1) lookup during render
    matches_by_line: HashMap<u32, Vec<usize>>,
}

impl Default for SearchState {
    fn default() -> Self {
        Self::new()
    }
}

impl SearchState {
    /// Creates a new SearchState
    pub fn new() -> Self {
        Self {
            visible: false,
            confirmed: false,
            search_textarea: create_search_textarea(),
            matches: Vec::new(),
            current_index: 0,
            last_query: String::new(),
            matches_by_line: HashMap::new(),
        }
    }

    /// Opens the search bar
    pub fn open(&mut self) {
        self.visible = true;
    }

    /// Closes the search bar and clears all state
    pub fn close(&mut self) {
        self.visible = false;
        self.confirmed = false;
        self.search_textarea.select_all();
        self.search_textarea.cut();
        self.matches.clear();
        self.current_index = 0;
        self.last_query.clear();
        self.matches_by_line.clear();
    }

    /// Returns whether the search has been confirmed (Enter pressed)
    pub fn is_confirmed(&self) -> bool {
        self.confirmed
    }

    /// Confirms the search, enabling n/N navigation
    pub fn confirm(&mut self) {
        self.confirmed = true;
    }

    /// Unconfirms the search (when query changes)
    pub fn unconfirm(&mut self) {
        self.confirmed = false;
    }

    /// Returns whether the search bar is visible
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Returns the current search query
    pub fn query(&self) -> &str {
        self.search_textarea
            .lines()
            .first()
            .map(|s| s.as_str())
            .unwrap_or("")
    }

    /// Returns a mutable reference to the search TextArea for input handling
    pub fn search_textarea_mut(&mut self) -> &mut TextArea<'static> {
        &mut self.search_textarea
    }

    /// Get current match for highlighting
    pub fn current_match(&self) -> Option<&Match> {
        self.matches.get(self.current_index)
    }

    /// Get all matches for highlighting
    pub fn matches(&self) -> &[Match] {
        &self.matches
    }

    /// Get match count display string "(current/total)"
    pub fn match_count_display(&self) -> String {
        if self.matches.is_empty() {
            "(0/0)".to_string()
        } else {
            format!("({}/{})", self.current_index + 1, self.matches.len())
        }
    }

    /// Navigate to next match, returns line to scroll to
    pub fn next_match(&mut self) -> Option<u32> {
        if self.matches.is_empty() {
            return None;
        }
        self.current_index = (self.current_index + 1) % self.matches.len();
        self.matches.get(self.current_index).map(|m| m.line)
    }

    /// Navigate to previous match, returns line to scroll to
    pub fn prev_match(&mut self) -> Option<u32> {
        if self.matches.is_empty() {
            return None;
        }
        self.current_index = if self.current_index == 0 {
            self.matches.len() - 1
        } else {
            self.current_index - 1
        };
        self.matches.get(self.current_index).map(|m| m.line)
    }

    /// Update matches based on query and content
    pub fn update_matches(&mut self, content: &str) {
        use super::matcher::SearchMatcher;

        let query = self.query().to_string();

        // Only update if query changed
        if query == self.last_query {
            return;
        }

        self.last_query = query.clone();
        self.matches = SearchMatcher::find_all(content, &query);
        self.current_index = 0;

        // Build line index for O(1) lookup during render
        self.matches_by_line.clear();
        for (idx, m) in self.matches.iter().enumerate() {
            self.matches_by_line.entry(m.line).or_default().push(idx);
        }
    }

    /// Get the current match index (0-indexed)
    pub fn current_index(&self) -> usize {
        self.current_index
    }

    /// Get all matches on a specific line with their global indices (O(1) lookup)
    /// Returns (global_index, &Match) tuples for current match highlighting
    pub fn matches_on_line(&self, line: u32) -> impl Iterator<Item = (usize, &Match)> + '_ {
        self.matches_by_line
            .get(&line)
            .into_iter()
            .flat_map(|indices| indices.iter().map(|&i| (i, &self.matches[i])))
    }
}

#[cfg(test)]
#[path = "search_state_tests.rs"]
mod search_state_tests;
