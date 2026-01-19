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
        }
    }

    pub fn open(&mut self) {
        self.snippets = super::snippet_storage::load_snippets();
        self.visible = true;
    }

    pub fn close(&mut self) {
        self.visible = false;
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

    #[cfg(test)]
    pub fn set_snippets(&mut self, snippets: Vec<Snippet>) {
        self.snippets = snippets;
    }
}

#[cfg(test)]
#[path = "snippet_state_tests.rs"]
mod snippet_state_tests;
