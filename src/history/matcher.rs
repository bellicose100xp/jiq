use std::fmt;

use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;

/// A fuzzy matcher using the Skim algorithm (fzf-style matching).
pub struct HistoryMatcher {
    matcher: SkimMatcherV2,
}

impl fmt::Debug for HistoryMatcher {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HistoryMatcher").finish_non_exhaustive()
    }
}

impl Default for HistoryMatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl HistoryMatcher {
    pub fn new() -> Self {
        Self {
            matcher: SkimMatcherV2::default(),
        }
    }

    /// Filters entries by the given query using fuzzy matching.
    /// Returns indices of matching entries sorted by score (highest first).
    /// If query is empty, returns all indices in original order.
    pub fn filter(&self, query: &str, entries: &[String]) -> Vec<usize> {
        if query.is_empty() {
            return (0..entries.len()).collect();
        }

        let mut scored: Vec<(usize, i64)> = entries
            .iter()
            .enumerate()
            .filter_map(|(idx, entry)| {
                self.matcher
                    .fuzzy_match(entry, query)
                    .map(|score| (idx, score))
            })
            .collect();

        scored.sort_by(|a, b| b.1.cmp(&a.1));

        scored.into_iter().map(|(idx, _)| idx).collect()
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_query_returns_all_indices() {
        let matcher = HistoryMatcher::new();
        let entries = vec![
            ".foo".to_string(),
            ".bar".to_string(),
            ".baz".to_string(),
        ];

        let result = matcher.filter("", &entries);
        assert_eq!(result, vec![0, 1, 2]);
    }

    #[test]
    fn test_exact_match_scores_highest() {
        let matcher = HistoryMatcher::new();
        let entries = vec![
            ".items".to_string(),
            ".items[] | .name".to_string(),
            ".foo".to_string(),
        ];

        let result = matcher.filter(".items", &entries);
        assert!(!result.is_empty());
        assert_eq!(result[0], 0);
    }

    #[test]
    fn test_fuzzy_matching() {
        let matcher = HistoryMatcher::new();
        let entries = vec![
            ".items[] | .name".to_string(),
            ".foo | .bar".to_string(),
            ".data.results".to_string(),
        ];

        let result = matcher.filter("itm", &entries);
        assert!(result.contains(&0));
    }

    #[test]
    fn test_case_insensitive() {
        let matcher = HistoryMatcher::new();
        let entries = vec![".Items".to_string(), ".ITEMS".to_string()];

        let result = matcher.filter("items", &entries);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_no_matches_returns_empty() {
        let matcher = HistoryMatcher::new();
        let entries = vec![".foo".to_string(), ".bar".to_string()];

        let result = matcher.filter("xyz", &entries);
        assert!(result.is_empty());
    }

}
