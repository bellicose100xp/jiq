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
#[path = "matcher_tests.rs"]
mod matcher_tests;
