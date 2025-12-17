use super::search_state::Match;

pub struct SearchMatcher;

impl SearchMatcher {
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
#[path = "matcher_tests.rs"]
mod matcher_tests;
