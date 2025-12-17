use super::scan_state::ScanState;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BraceType {
    Curly,
    Square,
    Paren,
}

#[derive(Debug, Clone)]
pub struct BraceTracker {
    open_braces: Vec<(usize, BraceType)>,
    query_snapshot: String,
}

impl Default for BraceTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl BraceTracker {
    pub fn new() -> Self {
        Self {
            open_braces: Vec::new(),
            query_snapshot: String::new(),
        }
    }

    pub fn rebuild(&mut self, query: &str) {
        self.open_braces.clear();
        self.query_snapshot = query.to_string();

        let mut state = ScanState::default();

        for (pos, ch) in query.char_indices() {
            if !state.is_in_string() {
                match ch {
                    '{' => self.open_braces.push((pos, BraceType::Curly)),
                    '[' => self.open_braces.push((pos, BraceType::Square)),
                    '(' => self.open_braces.push((pos, BraceType::Paren)),
                    '}' => {
                        if let Some((_, BraceType::Curly)) = self.open_braces.last() {
                            self.open_braces.pop();
                        }
                    }
                    ']' => {
                        if let Some((_, BraceType::Square)) = self.open_braces.last() {
                            self.open_braces.pop();
                        }
                    }
                    ')' => {
                        if let Some((_, BraceType::Paren)) = self.open_braces.last() {
                            self.open_braces.pop();
                        }
                    }
                    _ => {}
                }
            }
            state = state.advance(ch);
        }
    }

    pub fn context_at(&self, pos: usize) -> Option<BraceType> {
        for (brace_pos, brace_type) in self.open_braces.iter().rev() {
            if *brace_pos < pos {
                return Some(*brace_type);
            }
        }
        None
    }

    pub fn is_in_object(&self, pos: usize) -> bool {
        self.context_at(pos) == Some(BraceType::Curly)
    }

    #[allow(dead_code)]
    pub fn is_stale(&self, current_query: &str) -> bool {
        self.query_snapshot != current_query
    }
}

#[cfg(test)]
#[path = "brace_tracker_tests.rs"]
mod brace_tracker_tests;
