use super::search_state::Match;

/// Finds all occurrences of a query in text (case-insensitive)
pub struct SearchMatcher;

impl SearchMatcher {
    /// Find all matches of query in content
    /// Returns Vec of Match structs with (line, col, len)
    pub fn find_all(content: &str, query: &str) -> Vec<Match> {
        if query.is_empty() {
            return Vec::new();
        }

        let query_lower = query.to_lowercase();
        let mut matches = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            let line_lower = line.to_lowercase();
            let mut search_start = 0;

            while let Some(byte_pos) = line_lower[search_start..].find(&query_lower) {
                let absolute_byte_pos = search_start + byte_pos;
                // Convert byte position to character position
                let col = line[..absolute_byte_pos].chars().count() as u16;
                let len = query.chars().count() as u16;

                matches.push(Match {
                    line: line_num as u32,
                    col,
                    len,
                });

                // Move past this match to find overlapping matches
                search_start = absolute_byte_pos + query_lower.len();
            }
        }

        matches
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn test_empty_query() {
        let matches = SearchMatcher::find_all("hello world", "");
        assert!(matches.is_empty());
    }

    #[test]
    fn test_empty_content() {
        let matches = SearchMatcher::find_all("", "hello");
        assert!(matches.is_empty());
    }

    #[test]
    fn test_single_match() {
        let matches = SearchMatcher::find_all("hello world", "world");
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].line, 0);
        assert_eq!(matches[0].col, 6);
        assert_eq!(matches[0].len, 5);
    }

    #[test]
    fn test_case_insensitive() {
        let matches = SearchMatcher::find_all("Hello WORLD", "world");
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].col, 6);
    }

    #[test]
    fn test_multiple_matches_same_line() {
        let matches = SearchMatcher::find_all("foo bar foo baz foo", "foo");
        assert_eq!(matches.len(), 3);
        assert_eq!(matches[0].col, 0);
        assert_eq!(matches[1].col, 8);
        assert_eq!(matches[2].col, 16);
    }

    #[test]
    fn test_multiple_lines() {
        let content = "line one\nline two\nline three";
        let matches = SearchMatcher::find_all(content, "line");
        assert_eq!(matches.len(), 3);
        assert_eq!(matches[0].line, 0);
        assert_eq!(matches[1].line, 1);
        assert_eq!(matches[2].line, 2);
    }

    #[test]
    fn test_unicode_content() {
        let content = "héllo wörld";
        let matches = SearchMatcher::find_all(content, "wörld");
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].col, 6); // Character position, not byte
    }

    #[test]
    fn test_no_match() {
        let matches = SearchMatcher::find_all("hello world", "xyz");
        assert!(matches.is_empty());
    }

    // =========================================================================
    // Property-Based Tests
    // =========================================================================

    // Feature: search-in-results, Property 5: Case-insensitive matching
    // *For any* search query and results content, the matcher should find the
    // same matches regardless of case differences between query and content.
    // **Validates: Requirements 2.1**
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn prop_case_insensitive_matching(
            // Generate content with mixed case
            content in "[a-zA-Z0-9 \n]{1,200}",
            // Generate a query that's a substring we'll inject
            query in "[a-zA-Z]{1,10}",
        ) {
            // Skip empty queries (already tested separately)
            prop_assume!(!query.is_empty());

            // Create content with the query in different cases
            let content_with_lower = format!("{} {}", content, query.to_lowercase());
            let content_with_upper = format!("{} {}", content, query.to_uppercase());
            let content_with_mixed = format!("{} {}", content, query);

            // Search with lowercase query
            let matches_lower_query = SearchMatcher::find_all(&content_with_lower, &query.to_lowercase());
            let matches_upper_query = SearchMatcher::find_all(&content_with_lower, &query.to_uppercase());

            // Both should find at least the injected match
            prop_assert!(
                !matches_lower_query.is_empty(),
                "Lowercase query should find matches in content with lowercase text"
            );
            prop_assert!(
                !matches_upper_query.is_empty(),
                "Uppercase query should find matches in content with lowercase text"
            );

            // The number of matches should be the same regardless of query case
            prop_assert_eq!(
                matches_lower_query.len(),
                matches_upper_query.len(),
                "Match count should be same regardless of query case"
            );

            // Also verify with uppercase content
            let matches_in_upper = SearchMatcher::find_all(&content_with_upper, &query.to_lowercase());
            prop_assert!(
                !matches_in_upper.is_empty(),
                "Lowercase query should find matches in uppercase content"
            );

            // And mixed case content
            let matches_in_mixed = SearchMatcher::find_all(&content_with_mixed, &query.to_lowercase());
            prop_assert!(
                !matches_in_mixed.is_empty(),
                "Lowercase query should find matches in mixed case content"
            );
        }
    }
}
