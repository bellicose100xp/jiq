use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

use serde::Deserialize;

use super::snippet_state::Snippet;

const CONFIG_DIR: &str = "jiq";
const SNIPPETS_FILE: &str = "snippets.toml";

#[derive(Deserialize)]
struct SnippetsFile {
    #[serde(default)]
    snippets: Vec<Snippet>,
}

pub fn snippets_path() -> Option<PathBuf> {
    dirs::home_dir().map(|p| p.join(".config").join(CONFIG_DIR).join(SNIPPETS_FILE))
}

pub fn load_snippets() -> Vec<Snippet> {
    let Some(path) = snippets_path() else {
        return Vec::new();
    };

    load_snippets_from_path(&path)
}

pub fn load_snippets_from_path(path: &PathBuf) -> Vec<Snippet> {
    let mut file = match File::open(path) {
        Ok(f) => f,
        Err(_) => return Vec::new(),
    };

    let mut contents = String::new();
    if file.read_to_string(&mut contents).is_err() {
        return Vec::new();
    }

    parse_snippets_toml(&contents)
}

pub fn parse_snippets_toml(content: &str) -> Vec<Snippet> {
    match toml::from_str::<SnippetsFile>(content) {
        Ok(snippets_file) => snippets_file.snippets,
        Err(_) => Vec::new(),
    }
}

#[cfg(test)]
#[path = "snippet_storage_tests.rs"]
mod snippet_storage_tests;
