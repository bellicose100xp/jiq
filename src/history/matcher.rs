use std::fmt;

use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;

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

    pub fn filter(&self, query: &str, entries: &[String]) -> Vec<usize> {
        if query.is_empty() {
            return (0..entries.len()).collect();
        }

        // Split query into terms (space-separated, like fzf)
        let terms: Vec<&str> = query.split_whitespace().collect();
        if terms.is_empty() {
            return (0..entries.len()).collect();
        }

        let mut scored: Vec<(usize, i64)> = entries
            .iter()
            .enumerate()
            .filter_map(|(idx, entry)| {
                // All terms must match (AND logic)
                let mut total_score: i64 = 0;
                for term in &terms {
                    match self.matcher.fuzzy_match(entry, term) {
                        Some(score) => total_score += score,
                        None => return None, // Term didn't match, exclude entry
                    }
                }
                Some((idx, total_score))
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
        let entries = vec![".foo".to_string(), ".bar".to_string(), ".baz".to_string()];

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

    #[test]
    fn test_multi_word_search_ands_terms() {
        let matcher = HistoryMatcher::new();
        let entries = vec![
            ".organization.headquarters.facilities.buildings | .[].departments".to_string(),
            ".headquarters.offices".to_string(),
            ".buildings.floors".to_string(),
            ".unrelated.data".to_string(),
        ];

        // Both "headquarters" and "building" must match
        let result = matcher.filter("headquarters building", &entries);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], 0); // Only first entry has both terms

        // Single term should match more
        let result = matcher.filter("headquarters", &entries);
        assert_eq!(result.len(), 2); // First two entries
    }
}
