use ratatui::style::{Modifier, Style};
use serde::{Deserialize, Serialize};
use tui_textarea::TextArea;

use super::snippet_matcher::SnippetMatcher;
use crate::scroll::Scrollable;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Snippet {
    pub name: String,
    pub query: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum SnippetMode {
    #[default]
    Browse,
    CreateName,
    CreateQuery,
    CreateDescription,
    EditName {
        original_name: String,
    },
    EditQuery {
        original_query: String,
    },
    EditDescription {
        original_description: Option<String>,
    },
    ConfirmDelete {
        snippet_name: String,
    },
    ConfirmUpdate {
        snippet_name: String,
        old_query: String,
        new_query: String,
    },
}

fn create_search_textarea() -> TextArea<'static> {
    let mut textarea = TextArea::default();
    textarea.set_cursor_line_style(Style::default());
    textarea.set_cursor_style(Style::default().add_modifier(Modifier::REVERSED));
    textarea
}

fn create_name_textarea() -> TextArea<'static> {
    let mut textarea = TextArea::default();
    textarea.set_cursor_line_style(Style::default());
    textarea.set_cursor_style(Style::default().add_modifier(Modifier::REVERSED));
    textarea
}

fn create_description_textarea() -> TextArea<'static> {
    let mut textarea = TextArea::default();
    textarea.set_cursor_line_style(Style::default());
    textarea.set_cursor_style(Style::default().add_modifier(Modifier::REVERSED));
    textarea
}

fn create_query_textarea() -> TextArea<'static> {
    let mut textarea = TextArea::default();
    textarea.set_cursor_line_style(Style::default());
    textarea.set_cursor_style(Style::default().add_modifier(Modifier::REVERSED));
    textarea
}

pub struct SnippetState {
    visible: bool,
    mode: SnippetMode,
    snippets: Vec<Snippet>,
    filtered_indices: Vec<usize>,
    search_textarea: TextArea<'static>,
    name_textarea: TextArea<'static>,
    description_textarea: TextArea<'static>,
    query_textarea: TextArea<'static>,
    pending_query: String,
    selected_index: usize,
    scroll_offset: usize,
    visible_count: usize,
    matcher: SnippetMatcher,
    persist_to_disk: bool,
    hovered_index: Option<usize>,
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
            mode: SnippetMode::Browse,
            snippets: Vec::new(),
            filtered_indices: Vec::new(),
            search_textarea: create_search_textarea(),
            name_textarea: create_name_textarea(),
            description_textarea: create_description_textarea(),
            query_textarea: create_query_textarea(),
            pending_query: String::new(),
            selected_index: 0,
            scroll_offset: 0,
            visible_count: 10,
            matcher: SnippetMatcher::new(),
            persist_to_disk: true,
            hovered_index: None,
        }
    }

    #[cfg(test)]
    pub fn new_without_persistence() -> Self {
        Self {
            visible: false,
            mode: SnippetMode::Browse,
            snippets: Vec::new(),
            filtered_indices: Vec::new(),
            search_textarea: create_search_textarea(),
            name_textarea: create_name_textarea(),
            description_textarea: create_description_textarea(),
            query_textarea: create_query_textarea(),
            pending_query: String::new(),
            selected_index: 0,
            scroll_offset: 0,
            visible_count: 10,
            matcher: SnippetMatcher::new(),
            persist_to_disk: false,
            hovered_index: None,
        }
    }

    pub fn open(&mut self) {
        if self.persist_to_disk {
            self.snippets = super::snippet_storage::load_snippets();
        }
        self.search_textarea.select_all();
        self.search_textarea.cut();
        self.update_filter();
        self.selected_index = 0;
        self.scroll_offset = 0;
        self.visible = true;
    }

    pub fn close(&mut self) {
        self.visible = false;
        self.mode = SnippetMode::Browse;
        self.search_textarea.select_all();
        self.search_textarea.cut();
        self.name_textarea.select_all();
        self.name_textarea.cut();
        self.description_textarea.select_all();
        self.description_textarea.cut();
        self.query_textarea.select_all();
        self.query_textarea.cut();
        self.pending_query.clear();
        self.selected_index = 0;
        self.scroll_offset = 0;
        self.filtered_indices = (0..self.snippets.len()).collect();
        self.hovered_index = None;
    }

    pub fn is_visible(&self) -> bool {
        self.visible
    }

    pub fn is_editing(&self) -> bool {
        matches!(
            self.mode,
            SnippetMode::CreateName
                | SnippetMode::CreateQuery
                | SnippetMode::CreateDescription
                | SnippetMode::EditName { .. }
                | SnippetMode::EditQuery { .. }
                | SnippetMode::EditDescription { .. }
        )
    }

    pub fn mode(&self) -> &SnippetMode {
        &self.mode
    }

    #[cfg(test)]
    pub fn pending_query(&self) -> &str {
        &self.pending_query
    }

    pub fn enter_create_mode(&mut self, current_query: &str) {
        self.mode = SnippetMode::CreateName;
        self.pending_query = current_query.to_string();
        self.name_textarea.select_all();
        self.name_textarea.cut();
        self.query_textarea.select_all();
        self.query_textarea.cut();
        self.query_textarea.insert_str(current_query);
        self.description_textarea.select_all();
        self.description_textarea.cut();
    }

    pub fn cancel_create(&mut self) {
        self.mode = SnippetMode::Browse;
        self.pending_query.clear();
        self.name_textarea.select_all();
        self.name_textarea.cut();
        self.query_textarea.select_all();
        self.query_textarea.cut();
        self.description_textarea.select_all();
        self.description_textarea.cut();
    }

    pub fn next_field(&mut self) {
        let snippet_info = self
            .selected_snippet()
            .map(|s| (s.name.clone(), s.query.clone(), s.description.clone()));
        let pending_query = self.pending_query.clone();
        let current_query = self
            .query_textarea
            .lines()
            .first()
            .map(|s| s.to_string())
            .unwrap_or_default();

        match self.mode.clone() {
            SnippetMode::CreateName => {
                self.mode = SnippetMode::CreateQuery;
                self.query_textarea.select_all();
                self.query_textarea.cut();
                self.query_textarea.insert_str(&pending_query);
            }
            SnippetMode::CreateQuery => {
                self.pending_query = current_query;
                self.mode = SnippetMode::CreateDescription;
            }
            SnippetMode::CreateDescription => {
                self.mode = SnippetMode::CreateName;
            }
            SnippetMode::EditName { .. } => {
                if let Some((_, query, _)) = snippet_info {
                    self.query_textarea.select_all();
                    self.query_textarea.cut();
                    self.query_textarea.insert_str(&query);
                    self.mode = SnippetMode::EditQuery {
                        original_query: query,
                    };
                }
            }
            SnippetMode::EditQuery { .. } => {
                if let Some((_, _, description)) = snippet_info {
                    self.description_textarea.select_all();
                    self.description_textarea.cut();
                    if let Some(ref desc) = description {
                        self.description_textarea.insert_str(desc);
                    }
                    self.mode = SnippetMode::EditDescription {
                        original_description: description,
                    };
                }
            }
            SnippetMode::EditDescription { .. } => {
                if let Some((name, _, _)) = snippet_info {
                    self.name_textarea.select_all();
                    self.name_textarea.cut();
                    self.name_textarea.insert_str(&name);
                    self.mode = SnippetMode::EditName {
                        original_name: name,
                    };
                }
            }
            SnippetMode::Browse
            | SnippetMode::ConfirmDelete { .. }
            | SnippetMode::ConfirmUpdate { .. } => {}
        }
    }

    pub fn prev_field(&mut self) {
        let snippet_info = self
            .selected_snippet()
            .map(|s| (s.name.clone(), s.query.clone(), s.description.clone()));
        let pending_query = self.pending_query.clone();
        let current_query = self
            .query_textarea
            .lines()
            .first()
            .map(|s| s.to_string())
            .unwrap_or_default();

        match self.mode.clone() {
            SnippetMode::CreateName => {
                self.mode = SnippetMode::CreateDescription;
            }
            SnippetMode::CreateQuery => {
                self.pending_query = current_query;
                self.mode = SnippetMode::CreateName;
            }
            SnippetMode::CreateDescription => {
                self.mode = SnippetMode::CreateQuery;
                self.query_textarea.select_all();
                self.query_textarea.cut();
                self.query_textarea.insert_str(&pending_query);
            }
            SnippetMode::EditName { .. } => {
                if let Some((_, _, description)) = snippet_info {
                    self.description_textarea.select_all();
                    self.description_textarea.cut();
                    if let Some(ref desc) = description {
                        self.description_textarea.insert_str(desc);
                    }
                    self.mode = SnippetMode::EditDescription {
                        original_description: description,
                    };
                }
            }
            SnippetMode::EditQuery { .. } => {
                if let Some((name, _, _)) = snippet_info {
                    self.name_textarea.select_all();
                    self.name_textarea.cut();
                    self.name_textarea.insert_str(&name);
                    self.mode = SnippetMode::EditName {
                        original_name: name,
                    };
                }
            }
            SnippetMode::EditDescription { .. } => {
                if let Some((_, query, _)) = snippet_info {
                    self.query_textarea.select_all();
                    self.query_textarea.cut();
                    self.query_textarea.insert_str(&query);
                    self.mode = SnippetMode::EditQuery {
                        original_query: query,
                    };
                }
            }
            SnippetMode::Browse
            | SnippetMode::ConfirmDelete { .. }
            | SnippetMode::ConfirmUpdate { .. } => {}
        }
    }

    pub fn save_new_snippet(&mut self) -> Result<(), String> {
        let name = self
            .name_textarea
            .lines()
            .first()
            .map(|s| s.trim().to_string())
            .unwrap_or_default();

        if name.is_empty() {
            return Err("Name cannot be empty".to_string());
        }

        let query = self.pending_query.trim();
        if query.is_empty() {
            return Err("Query cannot be empty".to_string());
        }

        let name_lower = name.to_lowercase();
        if self
            .snippets
            .iter()
            .any(|s| s.name.to_lowercase() == name_lower)
        {
            return Err(format!("Snippet '{}' already exists", name));
        }

        let description = self
            .description_textarea
            .lines()
            .first()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());

        let snippet = Snippet {
            name,
            query: query.to_string(),
            description,
        };

        self.snippets.insert(0, snippet);

        if self.persist_to_disk
            && let Err(e) = super::snippet_storage::save_snippets(&self.snippets)
        {
            self.snippets.remove(0);
            return Err(format!("Failed to save: {}", e));
        }

        self.filtered_indices = (0..self.snippets.len()).collect();
        self.cancel_create();
        Ok(())
    }

    pub fn update_snippet_name(&mut self) -> Result<(), String> {
        let SnippetMode::EditName { ref original_name } = self.mode else {
            return Err("Not in edit name mode".to_string());
        };
        let original_name = original_name.clone();

        let new_name = self
            .name_textarea
            .lines()
            .first()
            .map(|s| s.trim().to_string())
            .unwrap_or_default();

        if new_name.is_empty() {
            return Err("Name cannot be empty".to_string());
        }

        let new_name_lower = new_name.to_lowercase();
        let original_name_lower = original_name.to_lowercase();

        if self.snippets.iter().any(|s| {
            let s_lower = s.name.to_lowercase();
            s_lower == new_name_lower && s_lower != original_name_lower
        }) {
            return Err(format!("Snippet '{}' already exists", new_name));
        }

        let snippet_idx = self
            .filtered_indices
            .get(self.selected_index)
            .copied()
            .ok_or_else(|| "No snippet selected".to_string())?;

        self.snippets[snippet_idx].name = new_name;

        if self.persist_to_disk
            && let Err(e) = super::snippet_storage::save_snippets(&self.snippets)
        {
            self.snippets[snippet_idx].name = original_name;
            return Err(format!("Failed to save: {}", e));
        }

        Ok(())
    }

    pub fn name_textarea_mut(&mut self) -> &mut TextArea<'static> {
        &mut self.name_textarea
    }

    pub fn description_textarea_mut(&mut self) -> &mut TextArea<'static> {
        &mut self.description_textarea
    }

    pub fn query_textarea_mut(&mut self) -> &mut TextArea<'static> {
        &mut self.query_textarea
    }

    pub fn enter_edit_mode(&mut self) {
        if let Some(snippet) = self.selected_snippet() {
            let original_name = snippet.name.clone();
            let query = snippet.query.clone();
            let description = snippet.description.clone();

            self.name_textarea.select_all();
            self.name_textarea.cut();
            self.name_textarea.insert_str(&original_name);

            self.query_textarea.select_all();
            self.query_textarea.cut();
            self.query_textarea.insert_str(&query);

            self.description_textarea.select_all();
            self.description_textarea.cut();
            if let Some(ref desc) = description {
                self.description_textarea.insert_str(desc);
            }

            self.mode = SnippetMode::EditName { original_name };
        }
    }

    pub fn cancel_edit(&mut self) {
        self.mode = SnippetMode::Browse;
        self.name_textarea.select_all();
        self.name_textarea.cut();
        self.query_textarea.select_all();
        self.query_textarea.cut();
        self.description_textarea.select_all();
        self.description_textarea.cut();
    }

    pub fn update_snippet_query(&mut self) -> Result<(), String> {
        let SnippetMode::EditQuery { .. } = self.mode else {
            return Err("Not in edit query mode".to_string());
        };

        let new_query = self
            .query_textarea
            .lines()
            .first()
            .map(|s| s.trim().to_string())
            .unwrap_or_default();

        if new_query.is_empty() {
            return Err("Query cannot be empty".to_string());
        }

        let snippet_idx = self
            .filtered_indices
            .get(self.selected_index)
            .copied()
            .ok_or_else(|| "No snippet selected".to_string())?;

        let original_query = self.snippets[snippet_idx].query.clone();
        self.snippets[snippet_idx].query = new_query;

        if self.persist_to_disk
            && let Err(e) = super::snippet_storage::save_snippets(&self.snippets)
        {
            self.snippets[snippet_idx].query = original_query;
            return Err(format!("Failed to save: {}", e));
        }

        Ok(())
    }

    pub fn update_snippet_description(&mut self) -> Result<(), String> {
        let SnippetMode::EditDescription { .. } = self.mode else {
            return Err("Not in edit description mode".to_string());
        };

        let new_description = self
            .description_textarea
            .lines()
            .first()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());

        let snippet_idx = self
            .filtered_indices
            .get(self.selected_index)
            .copied()
            .ok_or_else(|| "No snippet selected".to_string())?;

        let original_description = self.snippets[snippet_idx].description.clone();
        self.snippets[snippet_idx].description = new_description;

        if self.persist_to_disk
            && let Err(e) = super::snippet_storage::save_snippets(&self.snippets)
        {
            self.snippets[snippet_idx].description = original_description;
            return Err(format!("Failed to save: {}", e));
        }

        Ok(())
    }

    pub fn enter_delete_mode(&mut self) {
        if let Some(snippet) = self.selected_snippet() {
            let snippet_name = snippet.name.clone();
            self.mode = SnippetMode::ConfirmDelete { snippet_name };
        }
    }

    pub fn cancel_delete(&mut self) {
        self.mode = SnippetMode::Browse;
    }

    pub fn confirm_delete(&mut self) -> Result<(), String> {
        let SnippetMode::ConfirmDelete { ref snippet_name } = self.mode else {
            return Err("Not in delete confirmation mode".to_string());
        };
        let snippet_name = snippet_name.clone();

        let snippet_idx = self
            .filtered_indices
            .get(self.selected_index)
            .copied()
            .ok_or_else(|| "No snippet selected".to_string())?;

        if self.snippets[snippet_idx].name != snippet_name {
            return Err("Selected snippet does not match".to_string());
        }

        let removed_snippet = self.snippets.remove(snippet_idx);

        if self.persist_to_disk
            && let Err(e) = super::snippet_storage::save_snippets(&self.snippets)
        {
            self.snippets.insert(snippet_idx, removed_snippet);
            return Err(format!("Failed to save: {}", e));
        }

        self.filtered_indices = (0..self.snippets.len()).collect();
        if self.selected_index >= self.filtered_indices.len() && self.selected_index > 0 {
            self.selected_index -= 1;
        }
        self.adjust_scroll_to_selection();
        self.mode = SnippetMode::Browse;
        Ok(())
    }

    pub fn enter_update_confirmation(&mut self, new_query: String) -> Result<(), String> {
        let snippet = self
            .selected_snippet()
            .ok_or_else(|| "No snippet selected".to_string())?;

        let old_query = snippet.query.clone();
        let snippet_name = snippet.name.clone();

        if old_query.trim() == new_query.trim() {
            return Err("No changes to update".to_string());
        }

        self.mode = SnippetMode::ConfirmUpdate {
            snippet_name,
            old_query,
            new_query,
        };
        Ok(())
    }

    pub fn cancel_update(&mut self) {
        self.mode = SnippetMode::Browse;
    }

    pub fn confirm_update(&mut self) -> Result<(), String> {
        let SnippetMode::ConfirmUpdate {
            ref snippet_name,
            new_query: ref new_query_ref,
            ..
        } = self.mode
        else {
            return Err("Not in update confirmation mode".to_string());
        };
        let snippet_name = snippet_name.clone();
        let new_query = new_query_ref.clone();

        let snippet_idx = self
            .filtered_indices
            .get(self.selected_index)
            .copied()
            .ok_or_else(|| "No snippet selected".to_string())?;

        if self.snippets[snippet_idx].name != snippet_name {
            return Err("Selected snippet does not match".to_string());
        }

        let original_query = self.snippets[snippet_idx].query.clone();
        self.snippets[snippet_idx].query = new_query;

        if self.persist_to_disk
            && let Err(e) = super::snippet_storage::save_snippets(&self.snippets)
        {
            self.snippets[snippet_idx].query = original_query;
            return Err(format!("Failed to save: {}", e));
        }

        self.mode = SnippetMode::Browse;
        Ok(())
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
        let old_count = self.visible_count;
        self.visible_count = count.max(1);

        // Only adjust scroll if viewport shrunk and selection might be off-screen
        if self.visible_count < old_count {
            self.adjust_scroll_to_selection();
        }
    }

    pub fn visible_snippets(&self) -> impl Iterator<Item = (usize, &Snippet)> {
        self.filtered_indices
            .iter()
            .enumerate()
            .skip(self.scroll_offset)
            .take(self.visible_count)
            .filter_map(|(filtered_idx, &snippet_idx)| {
                self.snippets.get(snippet_idx).map(|s| (filtered_idx, s))
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

    #[cfg(test)]
    pub fn name_input(&self) -> &str {
        self.name_textarea
            .lines()
            .first()
            .map(|s| s.as_str())
            .unwrap_or("")
    }

    #[cfg(test)]
    pub fn description_input(&self) -> &str {
        self.description_textarea
            .lines()
            .first()
            .map(|s| s.as_str())
            .unwrap_or("")
    }

    #[cfg(test)]
    pub fn query_input(&self) -> &str {
        self.query_textarea
            .lines()
            .first()
            .map(|s| s.as_str())
            .unwrap_or("")
    }

    #[cfg(test)]
    pub fn disable_persistence(&mut self) {
        self.persist_to_disk = false;
    }

    /// Get the current scroll offset
    #[allow(dead_code)]
    pub fn scroll_offset(&self) -> usize {
        self.scroll_offset
    }

    /// Get the visible count (viewport size)
    #[allow(dead_code)]
    pub fn visible_count(&self) -> usize {
        self.visible_count
    }

    /// Get the currently hovered snippet index
    pub fn get_hovered(&self) -> Option<usize> {
        self.hovered_index
    }

    /// Set the hovered snippet index
    pub fn set_hovered(&mut self, index: Option<usize>) {
        self.hovered_index = index;
    }

    /// Clear the hover state
    pub fn clear_hover(&mut self) {
        self.hovered_index = None;
    }

    /// Find the snippet at a given Y coordinate within the list
    ///
    /// `inner_y` is the row relative to the inner content area (excluding border).
    /// Returns the filtered index if a snippet exists at that position.
    pub fn snippet_at_y(&self, inner_y: u16) -> Option<usize> {
        let index = self.scroll_offset + inner_y as usize;
        if index < self.filtered_indices.len() {
            Some(index)
        } else {
            None
        }
    }

    /// Select the snippet at the given filtered index
    pub fn select_at(&mut self, index: usize) {
        if index < self.filtered_indices.len() {
            self.selected_index = index;
            self.adjust_scroll_to_selection();
        }
    }
}

impl Scrollable for SnippetState {
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
            .saturating_sub(self.visible_count)
    }

    fn viewport_size(&self) -> usize {
        self.visible_count
    }
}

#[cfg(test)]
#[path = "snippet_state_tests.rs"]
mod snippet_state_tests;
