use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use super::snippet_state::Snippet;

const CONFIG_DIR: &str = "jiq";
const SNIPPETS_FILE: &str = "snippets.toml";

#[derive(Deserialize, Serialize)]
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

pub fn save_snippets(snippets: &[Snippet]) -> io::Result<()> {
    let Some(path) = snippets_path() else {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "Could not determine snippets file path",
        ));
    };

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let content = serialize_snippets_toml(snippets);
    let mut file = File::create(&path)?;
    file.write_all(content.as_bytes())?;

    Ok(())
}

pub fn serialize_snippets_toml(snippets: &[Snippet]) -> String {
    let file = SnippetsFile {
        snippets: snippets.to_vec(),
    };
    toml::to_string_pretty(&file).unwrap_or_default()
}

#[cfg(test)]
#[path = "snippet_storage_tests.rs"]
mod snippet_storage_tests;
