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
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn test_empty_query() {
        let mut tracker = BraceTracker::new();
        tracker.rebuild("");
        assert_eq!(tracker.context_at(0), None);
        assert!(!tracker.is_in_object(0));
    }

    #[test]
    fn test_simple_object() {
        let mut tracker = BraceTracker::new();
        tracker.rebuild("{name");
        assert_eq!(tracker.context_at(1), Some(BraceType::Curly));
        assert!(tracker.is_in_object(1));
        assert!(tracker.is_in_object(5));
    }

    #[test]
    fn test_simple_array() {
        let mut tracker = BraceTracker::new();
        tracker.rebuild("[1, 2");
        assert_eq!(tracker.context_at(1), Some(BraceType::Square));
        assert!(!tracker.is_in_object(1));
    }

    #[test]
    fn test_simple_paren() {
        let mut tracker = BraceTracker::new();
        tracker.rebuild("map(");
        assert_eq!(tracker.context_at(4), Some(BraceType::Paren));
        assert!(!tracker.is_in_object(4));
    }

    #[test]
    fn test_closed_braces() {
        let mut tracker = BraceTracker::new();
        tracker.rebuild("{name: .name}");
        // After the closing brace, we're no longer in object context
        assert_eq!(tracker.context_at(13), None);
    }

    #[test]
    fn test_object_in_array() {
        let mut tracker = BraceTracker::new();
        tracker.rebuild("[{na");
        // Position 2 is inside the object (after '{')
        assert_eq!(tracker.context_at(2), Some(BraceType::Curly));
        assert!(tracker.is_in_object(2));
    }

    #[test]
    fn test_array_in_object() {
        let mut tracker = BraceTracker::new();
        tracker.rebuild("{items: [na");
        // Position 9 is inside the array (after '[')
        assert_eq!(tracker.context_at(9), Some(BraceType::Square));
        assert!(!tracker.is_in_object(9));
    }

    #[test]
    fn test_deep_nesting() {
        let mut tracker = BraceTracker::new();
        tracker.rebuild("{a: [{b: (c");
        // Position 10 is inside the paren
        assert_eq!(tracker.context_at(10), Some(BraceType::Paren));
        assert!(!tracker.is_in_object(10));
    }

    #[test]
    fn test_braces_in_string() {
        let mut tracker = BraceTracker::new();
        tracker.rebuild("\"{braces}\"");
        // Braces inside string should be ignored
        assert_eq!(tracker.context_at(5), None);
        assert!(!tracker.is_in_object(5));
    }

    #[test]
    fn test_escaped_quote_in_string() {
        let mut tracker = BraceTracker::new();
        tracker.rebuild("\"say \\\"hi\\\" {here\"");
        // The { is inside the string, should be ignored
        assert_eq!(tracker.context_at(12), None);
    }

    #[test]
    fn test_escaped_backslash_in_string() {
        let mut tracker = BraceTracker::new();
        tracker.rebuild("\"path\\\\{dir\"");
        // The { is inside the string after \\, should be ignored
        assert_eq!(tracker.context_at(8), None);
    }

    #[test]
    fn test_string_then_real_braces() {
        let mut tracker = BraceTracker::new();
        tracker.rebuild("\"{fake}\" | {real");
        // Position 12 is inside the real object
        assert_eq!(tracker.context_at(12), Some(BraceType::Curly));
        assert!(tracker.is_in_object(12));
    }

    #[test]
    fn test_object_key_after_comma() {
        let mut tracker = BraceTracker::new();
        tracker.rebuild("{name: .name, ag");
        // Position 14 is still inside the object
        assert!(tracker.is_in_object(14));
    }

    #[test]
    fn test_real_jq_pattern_select() {
        let mut tracker = BraceTracker::new();
        tracker.rebuild("select(.active)");
        // After the closing paren, no context
        assert_eq!(tracker.context_at(15), None);
    }

    #[test]
    fn test_real_jq_pattern_map() {
        let mut tracker = BraceTracker::new();
        tracker.rebuild("map({na");
        assert!(tracker.is_in_object(5));

        tracker.rebuild("map({name: .name})");
        assert_eq!(tracker.context_at(18), None);
        assert_eq!(tracker.context_at(5), None);
    }

    #[test]
    fn test_mismatched_braces() {
        let mut tracker = BraceTracker::new();
        tracker.rebuild("{test]");
        assert!(tracker.is_in_object(5));
    }

    #[test]
    fn test_unclosed_string() {
        let mut tracker = BraceTracker::new();
        tracker.rebuild("\"unclosed {");
        assert_eq!(tracker.context_at(10), None);
    }

    #[test]
    fn test_is_stale() {
        let mut tracker = BraceTracker::new();
        tracker.rebuild("{test");
        assert!(!tracker.is_stale("{test"));
        assert!(tracker.is_stale("{test2"));
        assert!(tracker.is_stale(""));
    }

    #[test]
    fn test_context_at_position_zero() {
        let mut tracker = BraceTracker::new();
        tracker.rebuild("{test");
        // Position 0 is before the opening brace
        assert_eq!(tracker.context_at(0), None);
    }

    proptest! {
        /// **Feature: object-key-autocomplete, Property 4: BraceTracker never panics**
        /// **Validates: Requirements 5.2, 5.3**
        ///
        /// For any arbitrary string input, calling rebuild() and context_at()
        /// shall not panic.
        #[test]
        fn prop_rebuild_never_panics(query in ".*") {
            let mut tracker = BraceTracker::new();
            tracker.rebuild(&query);
            // If we get here without panicking, the test passes
        }

        /// **Feature: object-key-autocomplete, Property 4: BraceTracker never panics**
        /// **Validates: Requirements 5.2, 5.3**
        ///
        /// For any position query on any string, context_at() shall not panic.
        #[test]
        fn prop_context_at_never_panics(query in ".*", pos in 0usize..1000) {
            let mut tracker = BraceTracker::new();
            tracker.rebuild(&query);
            let _ = tracker.context_at(pos);
            let _ = tracker.is_in_object(pos);
            // If we get here without panicking, the test passes
        }

        /// **Feature: object-key-autocomplete, Property 3: Braces inside strings are ignored**
        /// **Validates: Requirements 3.1, 3.2**
        ///
        /// For any query containing a string literal with braces inside,
        /// the BraceTracker shall report the same context as if those braces
        /// were not present.
        #[test]
        fn prop_string_braces_ignored(
            prefix in "[a-z .|]*",
            string_content in "[a-z{}\\[\\]()]*",
            suffix in "[a-z .|]*"
        ) {
            // Build query with braces inside a string
            let query_with_string_braces = format!("{}\"{}\"{}",  prefix, string_content, suffix);
            // Build query with empty string (no braces in string)
            let query_with_empty_string = format!("{}\"\"{}",  prefix, suffix);

            let mut tracker1 = BraceTracker::new();
            let mut tracker2 = BraceTracker::new();

            tracker1.rebuild(&query_with_string_braces);
            tracker2.rebuild(&query_with_empty_string);

            // The context after the string should be the same
            let pos_after_string1 = prefix.len() + string_content.len() + 2; // +2 for quotes
            let pos_after_string2 = prefix.len() + 2; // +2 for quotes

            prop_assert_eq!(
                tracker1.context_at(pos_after_string1),
                tracker2.context_at(pos_after_string2),
                "Context after string should be same regardless of braces inside string"
            );
        }

        /// **Feature: object-key-autocomplete, Property 4: BraceTracker never panics**
        /// **Validates: Requirements 5.2, 5.3**
        ///
        /// is_in_object should be consistent with context_at result.
        #[test]
        fn prop_is_in_object_consistent(query in ".*", pos in 0usize..500) {
            let mut tracker = BraceTracker::new();
            tracker.rebuild(&query);

            let context = tracker.context_at(pos);
            let is_object = tracker.is_in_object(pos);

            prop_assert_eq!(
                is_object,
                context == Some(BraceType::Curly),
                "is_in_object should match context_at == Curly"
            );
        }
    }
}
