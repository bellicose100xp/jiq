use std::fmt;

use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;

use super::snippet_state::Snippet;

pub struct SnippetMatcher {
    matcher: SkimMatcherV2,
}

impl fmt::Debug for SnippetMatcher {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SnippetMatcher").finish_non_exhaustive()
    }
}

impl Default for SnippetMatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl SnippetMatcher {
    pub fn new() -> Self {
        Self {
            matcher: SkimMatcherV2::default(),
        }
    }

    pub fn filter(&self, query: &str, snippets: &[Snippet]) -> Vec<usize> {
        if query.is_empty() {
            return (0..snippets.len()).collect();
        }

        let terms: Vec<&str> = query.split_whitespace().collect();
        if terms.is_empty() {
            return (0..snippets.len()).collect();
        }

        let mut scored: Vec<(usize, i64)> = snippets
            .iter()
            .enumerate()
            .filter_map(|(idx, snippet)| {
                let mut total_score: i64 = 0;
                for term in &terms {
                    match self.matcher.fuzzy_match(&snippet.name, term) {
                        Some(score) => total_score += score,
                        None => return None,
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
#[path = "snippet_matcher_tests.rs"]
mod snippet_matcher_tests;
