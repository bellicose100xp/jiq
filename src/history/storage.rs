use std::fs::{self, File};
use std::io::{self, BufRead, BufReader, Write};
use std::path::PathBuf;

const MAX_HISTORY_ENTRIES: usize = 1000;
const HISTORY_DIR: &str = "jiq";
const HISTORY_FILE: &str = "history";

pub fn history_path() -> Option<PathBuf> {
    dirs::data_dir().map(|p| p.join(HISTORY_DIR).join(HISTORY_FILE))
}

pub fn load_history() -> Vec<String> {
    let Some(path) = history_path() else {
        return Vec::new();
    };

    let file = match File::open(&path) {
        Ok(f) => f,
        Err(_) => return Vec::new(),
    };

    let reader = BufReader::new(file);
    reader
        .lines()
        .map_while(Result::ok)
        .filter(|line| !line.trim().is_empty())
        .collect()
}

pub fn save_history(entries: &[String]) -> io::Result<()> {
    let Some(path) = history_path() else {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "Could not determine history file path",
        ));
    };

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut file = File::create(&path)?;

    let unique_entries = deduplicate(entries);
    let trimmed = trim_to_max(&unique_entries);

    for entry in trimmed {
        writeln!(file, "{}", entry)?;
    }

    Ok(())
}

/// No file locking - last writer wins if multiple instances run simultaneously.
pub fn add_entry(query: &str) -> io::Result<()> {
    let query = query.trim();
    if query.is_empty() {
        return Ok(());
    }

    let mut entries = load_history();

    entries.retain(|e| e != query);
    entries.insert(0, query.to_string());

    save_history(&entries)
}

/// Removes duplicate entries, keeping the first occurrence of each.
fn deduplicate(entries: &[String]) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    entries
        .iter()
        .filter(|e| seen.insert(e.as_str()))
        .cloned()
        .collect()
}

/// Trims the entries to the maximum allowed size.
fn trim_to_max(entries: &[String]) -> Vec<String> {
    entries.iter().take(MAX_HISTORY_ENTRIES).cloned().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplicate_keeps_first_occurrence() {
        let entries = vec![
            "a".to_string(),
            "b".to_string(),
            "a".to_string(),
            "c".to_string(),
            "b".to_string(),
        ];
        let result = deduplicate(&entries);
        assert_eq!(result, vec!["a", "b", "c"]);
    }

    #[test]
    fn test_trim_to_max() {
        let entries: Vec<String> = (0..1500).map(|i| format!("entry{}", i)).collect();
        let trimmed = trim_to_max(&entries);
        assert_eq!(trimmed.len(), MAX_HISTORY_ENTRIES);
        assert_eq!(trimmed[0], "entry0");
    }
}
