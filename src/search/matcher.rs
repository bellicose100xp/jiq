use super::search_state::Match;

pub struct SearchMatcher;

impl SearchMatcher {
    /// Case-insensitive substring search over every line of `content`.
    /// Match columns and lengths are reported in characters of the
    /// original line, never in bytes of the lowercased copy.
    pub fn find_all(content: &str, query: &str) -> Vec<Match> {
        if query.is_empty() {
            return Vec::new();
        }

        let query_lower = query.to_lowercase();
        let mut matches = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            if line.is_ascii() {
                Self::find_in_ascii_line(line, &query_lower, line_num as u32, &mut matches);
            } else {
                Self::find_in_unicode_line(line, &query_lower, line_num as u32, &mut matches);
            }
        }

        matches
    }

    /// ASCII lines lowercase without changing length, so byte offsets are
    /// character offsets and no mapping is needed.
    fn find_in_ascii_line(line: &str, query_lower: &str, line_num: u32, out: &mut Vec<Match>) {
        let line_lower = line.to_ascii_lowercase();
        let mut search_start = 0;

        while let Some(pos) = line_lower[search_start..].find(query_lower) {
            let start = search_start + pos;
            out.push(Match {
                line: line_num,
                col: start as u16,
                len: query_lower.len() as u16,
            });
            search_start = start + query_lower.len();
        }
    }

    /// Lowercasing can change a character's byte length (`İ` grows,
    /// `ẞ` shrinks), so offsets found in the lowered line are mapped back
    /// to the original line through a per-character start table.
    fn find_in_unicode_line(line: &str, query_lower: &str, line_num: u32, out: &mut Vec<Match>) {
        let line_lower = line.to_lowercase();

        let mut lowered_starts: Vec<usize> = Vec::with_capacity(line.len());
        let mut offset = 0;
        for ch in line.chars() {
            lowered_starts.push(offset);
            offset += ch.to_lowercase().map(char::len_utf8).sum::<usize>();
        }
        debug_assert_eq!(offset, line_lower.len());

        let mut search_start = 0;
        while let Some(pos) = line_lower[search_start..].find(query_lower) {
            let start = search_start + pos;
            let end = start + query_lower.len();

            let col = lowered_starts
                .partition_point(|&s| s <= start)
                .saturating_sub(1);
            let end_col = lowered_starts.partition_point(|&s| s < end);
            let len = end_col.saturating_sub(col).max(1);

            out.push(Match {
                line: line_num,
                col: col as u16,
                len: len as u16,
            });
            search_start = end;
        }
    }
}

#[cfg(test)]
#[path = "matcher_tests.rs"]
mod matcher_tests;
