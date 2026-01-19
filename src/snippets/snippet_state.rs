pub struct SnippetState {
    visible: bool,
}

impl Default for SnippetState {
    fn default() -> Self {
        Self::new()
    }
}

impl SnippetState {
    pub fn new() -> Self {
        Self { visible: false }
    }

    pub fn open(&mut self) {
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
}

#[cfg(test)]
#[path = "snippet_state_tests.rs"]
mod snippet_state_tests;
