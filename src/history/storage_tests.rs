//! Tests for history/storage

use super::*;
use std::env;
use tempfile::TempDir;

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

/// Exercises the entire filesystem-backed persistence layer (save_history,
/// load_history, add_entry, delete_entry) end-to-end by redirecting
/// `dirs::data_dir()` at a TempDir via `XDG_DATA_HOME`. Consolidated into a
/// single env-mutating test so the process-wide `XDG_DATA_HOME` change happens
/// at exactly one site, avoiding races (no other test reads data_dir()).
#[test]
fn test_storage_full_lifecycle_via_xdg_data_home() {
    let dir = TempDir::new().unwrap();
    // SAFETY: setting/removing env vars is safe in test threads as long as no
    // other test in the same binary reads the same name concurrently. No other
    // test touches XDG_DATA_HOME or dirs::data_dir(), and this is the only
    // env-mutating site for the storage layer.
    unsafe {
        env::set_var("XDG_DATA_HOME", dir.path());
    }

    // Missing-file path: load before any save returns empty (File::open Err arm).
    assert_eq!(
        load_history(),
        Vec::<String>::new(),
        "no history file yet -> empty"
    );

    // save_history writes entries; load_history reads them back in order,
    // skipping blank lines that may exist in the file.
    save_history(&[
        "first".to_string(),
        "second".to_string(),
        "third".to_string(),
    ])
    .unwrap();
    assert_eq!(load_history(), vec!["first", "second", "third"]);

    // Blank lines in the on-disk file are filtered out on read.
    let path = history_path().unwrap();
    fs::write(&path, "alpha\n\n   \nbeta\n").unwrap();
    assert_eq!(load_history(), vec!["alpha", "beta"]);

    // save_history deduplicates (keeping first occurrence) and trims to the cap.
    let mut many: Vec<String> = Vec::new();
    many.push("dup".to_string());
    many.push("dup".to_string()); // duplicate of the first -> dropped
    for i in 0..(MAX_HISTORY_ENTRIES + 50) {
        many.push(format!("e{}", i));
    }
    save_history(&many).unwrap();
    let reloaded = load_history();
    assert_eq!(
        reloaded.len(),
        MAX_HISTORY_ENTRIES,
        "save_history trims to MAX_HISTORY_ENTRIES"
    );
    assert_eq!(reloaded[0], "dup", "first occurrence of dup is kept");
    assert_eq!(
        reloaded.iter().filter(|e| *e == "dup").count(),
        1,
        "duplicate dropped on save"
    );

    // add_entry on a non-empty query moves it to the front, removing any prior
    // duplicate (retain + insert(0)).
    save_history(&["a".to_string(), "b".to_string(), "c".to_string()]).unwrap();
    add_entry("b").unwrap();
    assert_eq!(
        load_history(),
        vec!["b", "a", "c"],
        "add_entry moves existing query to front"
    );
    add_entry("new").unwrap();
    assert_eq!(
        load_history(),
        vec!["new", "b", "a", "c"],
        "add_entry inserts brand-new query at front"
    );

    // add_entry ignores blank/whitespace-only queries (empty guard).
    add_entry("   ").unwrap();
    assert_eq!(
        load_history(),
        vec!["new", "b", "a", "c"],
        "blank query is a no-op"
    );

    // delete_entry removes all matching entries and persists the change.
    // Write duplicates of the deletion target directly to disk (bypassing
    // save_history's dedup) so we can prove delete_entry's retain() drops
    // *every* matching occurrence, while distinct survivors stay in order.
    fs::write(&path, "k1\ndrop\nk2\ndrop\nk3\n").unwrap();
    delete_entry("drop").unwrap();
    assert_eq!(
        load_history(),
        vec!["k1", "k2", "k3"],
        "delete_entry removes all matching occurrences, survivors kept in order"
    );

    // delete_entry of an absent query short-circuits without rewriting the file.
    let before = fs::read_to_string(&path).unwrap();
    delete_entry("absent").unwrap();
    let after = fs::read_to_string(&path).unwrap();
    assert_eq!(
        before, after,
        "delete_entry of absent query leaves file unchanged"
    );

    unsafe {
        env::remove_var("XDG_DATA_HOME");
    }
}
