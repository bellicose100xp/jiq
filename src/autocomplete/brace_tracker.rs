use super::jq_functions::{JQ_FUNCTION_METADATA, is_element_context_function};
use super::scan_state::ScanState;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BraceType {
    Curly,
    Square,
    Paren,
}

/// Context for parentheses that changes how autocomplete behaves
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FunctionContext {
    /// Function that iterates over elements (map, select, sort_by, etc.)
    /// Inner value is the function name for debugging/display.
    ElementIterator(&'static str),
}

/// Information about an open brace/bracket/paren
#[derive(Debug, Clone)]
pub struct BraceInfo {
    pub pos: usize,
    pub brace_type: BraceType,
    pub context: Option<FunctionContext>,
}

#[derive(Debug, Clone)]
pub struct BraceTracker {
    open_braces: Vec<BraceInfo>,
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
                    '{' => self.open_braces.push(BraceInfo {
                        pos,
                        brace_type: BraceType::Curly,
                        context: None,
                    }),
                    '[' => self.open_braces.push(BraceInfo {
                        pos,
                        brace_type: BraceType::Square,
                        context: None,
                    }),
                    '(' => {
                        let context = Self::detect_function_context(query, pos);
                        self.open_braces.push(BraceInfo {
                            pos,
                            brace_type: BraceType::Paren,
                            context,
                        });
                    }
                    '}' => {
                        if let Some(info) = self.open_braces.last()
                            && info.brace_type == BraceType::Curly
                        {
                            self.open_braces.pop();
                        }
                    }
                    ']' => {
                        if let Some(info) = self.open_braces.last()
                            && info.brace_type == BraceType::Square
                        {
                            self.open_braces.pop();
                        }
                    }
                    ')' => {
                        if let Some(info) = self.open_braces.last()
                            && info.brace_type == BraceType::Paren
                        {
                            self.open_braces.pop();
                        }
                    }
                    _ => {}
                }
            }
            state = state.advance(ch);
        }
    }

    /// Detect if the parenthesis at `paren_pos` is preceded by an element-context function.
    /// Scans backwards to find a function name, then checks if it's in ELEMENT_CONTEXT_FUNCTIONS.
    fn detect_function_context(query: &str, paren_pos: usize) -> Option<FunctionContext> {
        let bytes = query.as_bytes();
        if paren_pos == 0 {
            return None;
        }

        // Skip whitespace before the paren
        let mut end = paren_pos;
        while end > 0 && bytes[end - 1].is_ascii_whitespace() {
            end -= 1;
        }

        if end == 0 {
            return None;
        }

        // Check if there's a word character before the paren
        if !Self::is_word_char(bytes[end - 1]) {
            return None;
        }

        // Find the start of the word
        let mut start = end - 1;
        while start > 0 && Self::is_word_char(bytes[start - 1]) {
            start -= 1;
        }

        // Extract the function name
        let func_name = &query[start..end];

        // Look up in JQ_FUNCTION_METADATA to get static str reference
        let static_name = JQ_FUNCTION_METADATA
            .iter()
            .find(|f| f.name == func_name)
            .map(|f| f.name)?;

        if is_element_context_function(static_name) {
            Some(FunctionContext::ElementIterator(static_name))
        } else {
            None
        }
    }

    fn is_word_char(b: u8) -> bool {
        b.is_ascii_alphanumeric() || b == b'_'
    }

    pub fn context_at(&self, pos: usize) -> Option<BraceType> {
        for info in self.open_braces.iter().rev() {
            if info.pos < pos {
                return Some(info.brace_type);
            }
        }
        None
    }

    pub fn is_in_object(&self, pos: usize) -> bool {
        self.context_at(pos) == Some(BraceType::Curly)
    }

    /// Check if the cursor position is inside any element-context function.
    /// This checks ALL enclosing parens, not just the innermost one,
    /// so `map(limit(5; .` correctly returns true (map provides element context).
    pub fn is_in_element_context(&self, pos: usize) -> bool {
        self.open_braces.iter().any(|info| {
            info.pos < pos
                && info.brace_type == BraceType::Paren
                && matches!(info.context, Some(FunctionContext::ElementIterator(_)))
        })
    }

    /// Check if tracker is stale (test helper)
    #[cfg(test)]
    pub fn is_stale(&self, current_query: &str) -> bool {
        self.query_snapshot != current_query
    }

    /// Check if the cursor position is in a non-executing context where cache doesn't reflect
    /// the expression being typed.
    ///
    /// Non-executing contexts include:
    /// - Inside element-iterating functions (map, select, sort_by, etc.)
    /// - Inside array builders `[expr, expr]`
    /// - Inside object builder values `{key: expr}`
    ///
    /// In these contexts, the cache is stale and we need to extract path and navigate.
    pub fn is_in_non_executing_context(&self, pos: usize) -> bool {
        // Element-iterating functions (existing check)
        if self.is_in_element_context(pos) {
            return true;
        }

        // Check innermost brace context
        for info in self.open_braces.iter().rev() {
            if info.pos >= pos {
                continue;
            }

            match info.brace_type {
                // Array builder: non-executing (but not array iteration `.[]`)
                BraceType::Square => {
                    if self.is_array_builder(info.pos) {
                        return true;
                    }
                }
                // Object builder value position: non-executing
                BraceType::Curly => {
                    if self.is_after_colon_in_object(pos) {
                        return true;
                    }
                }
                // Parentheses: only non-executing if it's an element-context function
                // (already handled above by is_in_element_context)
                BraceType::Paren => {}
            }
        }

        false
    }

    /// Get the innermost open brace info at a position.
    pub fn innermost_brace_info(&self, pos: usize) -> Option<&BraceInfo> {
        self.open_braces.iter().rev().find(|info| info.pos < pos)
    }

    /// Check if the square bracket at `bracket_pos` is an array builder vs iteration.
    ///
    /// Array iteration: `.[]`, `.[0]`, `.foo[]`
    /// Array builder: `[.a, .b]`, `[1, 2, 3]`
    fn is_array_builder(&self, bracket_pos: usize) -> bool {
        if bracket_pos == 0 {
            return true; // `[...]` at start is always builder
        }

        let before = &self.query_snapshot[..bracket_pos];
        let trimmed = before.trim_end();

        if trimmed.is_empty() {
            return true;
        }

        let last_char = trimmed.chars().last().unwrap();

        // Array iteration follows: `.`, `]`, `?`, identifier chars
        // Array builder follows: `|`, `;`, `(`, `[`, `{`, `,`, `:`
        matches!(last_char, '|' | ';' | '(' | '[' | '{' | ',' | ':')
    }

    /// Check if position is after a colon in an object (value position).
    fn is_after_colon_in_object(&self, pos: usize) -> bool {
        // Find the innermost curly brace
        let curly_pos = self
            .open_braces
            .iter()
            .rev()
            .find(|info| info.pos < pos && info.brace_type == BraceType::Curly)
            .map(|info| info.pos);

        if let Some(curly_pos) = curly_pos {
            let inside = &self.query_snapshot[curly_pos + 1..pos.min(self.query_snapshot.len())];
            // Check if we're after a colon (value position) vs before (key position)
            if let Some(last_colon) = inside.rfind(':') {
                if let Some(last_comma) = inside.rfind(',') {
                    return last_colon > last_comma;
                }
                return true;
            }
        }
        false
    }
}

#[cfg(test)]
#[path = "brace_tracker_tests.rs"]
mod brace_tracker_tests;
