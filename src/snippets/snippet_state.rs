use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Snippet {
    pub name: String,
    pub query: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

pub struct SnippetState {
    visible: bool,
    snippets: Vec<Snippet>,
    selected_index: usize,
    scroll_offset: usize,
    visible_count: usize,
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
            selected_index: 0,
            scroll_offset: 0,
            visible_count: 10,
        }
    }

    pub fn open(&mut self) {
        self.snippets = super::snippet_storage::load_snippets();
        self.selected_index = 0;
        self.scroll_offset = 0;
        self.visible = true;
    }

    pub fn close(&mut self) {
        self.visible = false;
        self.scroll_offset = 0;
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

    pub fn selected_index(&self) -> usize {
        self.selected_index
    }

    pub fn selected_snippet(&self) -> Option<&Snippet> {
        self.snippets.get(self.selected_index)
    }

    pub fn select_next(&mut self) {
        if !self.snippets.is_empty() && self.selected_index < self.snippets.len() - 1 {
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
        self.snippets
            .iter()
            .enumerate()
            .skip(self.scroll_offset)
            .take(self.visible_count)
    }

    fn adjust_scroll_to_selection(&mut self) {
        if self.selected_index >= self.scroll_offset + self.visible_count {
            self.scroll_offset = self.selected_index - self.visible_count + 1;
        } else if self.selected_index < self.scroll_offset {
            self.scroll_offset = self.selected_index;
        }

        let max_offset = self.snippets.len().saturating_sub(self.visible_count);
        self.scroll_offset = self.scroll_offset.min(max_offset);
    }

    #[cfg(test)]
    pub fn set_snippets(&mut self, snippets: Vec<Snippet>) {
        self.snippets = snippets;
        self.selected_index = 0;
        self.scroll_offset = 0;
    }

    #[cfg(test)]
    pub fn set_selected_index(&mut self, index: usize) {
        if index < self.snippets.len() || self.snippets.is_empty() {
            self.selected_index = index;
            self.adjust_scroll_to_selection();
        }
    }
}

#[cfg(test)]
#[path = "snippet_state_tests.rs"]
mod snippet_state_tests;
