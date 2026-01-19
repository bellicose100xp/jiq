use ratatui::style::{Modifier, Style};
use serde::{Deserialize, Serialize};
use tui_textarea::TextArea;

use super::snippet_matcher::SnippetMatcher;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Snippet {
    pub name: String,
    pub query: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

fn create_search_textarea() -> TextArea<'static> {
    let mut textarea = TextArea::default();
    textarea.set_cursor_line_style(Style::default());
    textarea.set_cursor_style(Style::default().add_modifier(Modifier::REVERSED));
    textarea
}

pub struct SnippetState {
    visible: bool,
    snippets: Vec<Snippet>,
    filtered_indices: Vec<usize>,
    search_textarea: TextArea<'static>,
    selected_index: usize,
    scroll_offset: usize,
    visible_count: usize,
    matcher: SnippetMatcher,
}

impl Default for SnippetState {
    fn default() -> Self {
        Self::new()
    }
}

impl SnippetState {
    pub fn new() -> Self {
        Self {
            visible: false,
            snippets: Vec::new(),
            filtered_indices: Vec::new(),
            search_textarea: create_search_textarea(),
            selected_index: 0,
            scroll_offset: 0,
            visible_count: 10,
            matcher: SnippetMatcher::new(),
        }
    }

    pub fn open(&mut self) {
        self.snippets = super::snippet_storage::load_snippets();
        self.search_textarea.select_all();
        self.search_textarea.cut();
        self.update_filter();
        self.selected_index = 0;
        self.scroll_offset = 0;
        self.visible = true;
    }

    pub fn close(&mut self) {
        self.visible = false;
        self.search_textarea.select_all();
        self.search_textarea.cut();
        self.selected_index = 0;
        self.scroll_offset = 0;
        self.filtered_indices = (0..self.snippets.len()).collect();
    }

    pub fn is_visible(&self) -> bool {
        self.visible
    }

    pub fn is_editing(&self) -> bool {
        false
    }

    pub fn snippets(&self) -> &[Snippet] {
        &self.snippets
    }

    pub fn filtered_count(&self) -> usize {
        self.filtered_indices.len()
    }

    pub fn selected_index(&self) -> usize {
        self.selected_index
    }

    pub fn selected_snippet(&self) -> Option<&Snippet> {
        self.filtered_indices
            .get(self.selected_index)
            .and_then(|&idx| self.snippets.get(idx))
    }

    pub fn select_next(&mut self) {
        if !self.filtered_indices.is_empty()
            && self.selected_index < self.filtered_indices.len() - 1
        {
            self.selected_index += 1;
            self.adjust_scroll_to_selection();
        }
    }

    pub fn select_prev(&mut self) {
        if self.selected_index > 0 {
            self.selected_index = self.selected_index.saturating_sub(1);
            self.adjust_scroll_to_selection();
        }
    }

    pub fn set_visible_count(&mut self, count: usize) {
        self.visible_count = count.max(1);
        self.adjust_scroll_to_selection();
    }

    pub fn visible_snippets(&self) -> impl Iterator<Item = (usize, &Snippet)> {
        self.filtered_indices
            .iter()
            .enumerate()
            .skip(self.scroll_offset)
            .take(self.visible_count)
            .filter_map(|(display_idx, &snippet_idx)| {
                self.snippets
                    .get(snippet_idx)
                    .map(|s| (self.scroll_offset + display_idx, s))
            })
    }

    pub fn search_textarea_mut(&mut self) -> &mut TextArea<'static> {
        &mut self.search_textarea
    }

    pub fn on_search_input_changed(&mut self) {
        self.update_filter();
        self.selected_index = 0;
        self.scroll_offset = 0;
    }

    fn update_filter(&mut self) {
        let query = self
            .search_textarea
            .lines()
            .first()
            .map(|s| s.as_str())
            .unwrap_or("");
        self.filtered_indices = self.matcher.filter(query, &self.snippets);
    }

    fn adjust_scroll_to_selection(&mut self) {
        if self.selected_index >= self.scroll_offset + self.visible_count {
            self.scroll_offset = self.selected_index - self.visible_count + 1;
        } else if self.selected_index < self.scroll_offset {
            self.scroll_offset = self.selected_index;
        }

        let max_offset = self
            .filtered_indices
            .len()
            .saturating_sub(self.visible_count);
        self.scroll_offset = self.scroll_offset.min(max_offset);
    }

    #[cfg(test)]
    pub fn search_query(&self) -> &str {
        self.search_textarea
            .lines()
            .first()
            .map(|s| s.as_str())
            .unwrap_or("")
    }

    #[cfg(test)]
    pub fn set_snippets(&mut self, snippets: Vec<Snippet>) {
        self.snippets = snippets;
        self.filtered_indices = (0..self.snippets.len()).collect();
        self.selected_index = 0;
        self.scroll_offset = 0;
    }

    #[cfg(test)]
    pub fn set_selected_index(&mut self, index: usize) {
        if index < self.filtered_indices.len() || self.filtered_indices.is_empty() {
            self.selected_index = index;
            self.adjust_scroll_to_selection();
        }
    }

    #[cfg(test)]
    pub fn set_search_query(&mut self, query: &str) {
        self.search_textarea.select_all();
        self.search_textarea.cut();
        self.search_textarea.insert_str(query);
        self.update_filter();
        self.selected_index = 0;
        self.scroll_offset = 0;
    }
}

#[cfg(test)]
#[path = "snippet_state_tests.rs"]
mod snippet_state_tests;
