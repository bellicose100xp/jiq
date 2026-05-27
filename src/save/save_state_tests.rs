use std::fs;

use tempfile::TempDir;

use super::*;

const TS: &str = "20260527-104522";

fn open_state(initial_pattern_override: Option<&str>) -> SaveState {
    let mut s = SaveState::new();
    s.open(TS.to_string());
    if let Some(text) = initial_pattern_override {
        let ta = s.filename_mut();
        for _ in 0..ta.lines().join("").chars().count() {
            ta.delete_char();
        }
        ta.insert_str(text);
    }
    s
}

#[test]
fn new_state_is_closed() {
    let s = SaveState::new();
    assert!(!s.is_visible());
    assert_eq!(s.mode(), &SaveMode::Closed);
}

#[test]
fn open_advances_to_enter_filename() {
    let s = open_state(None);
    assert!(s.is_visible());
    assert_eq!(s.mode(), &SaveMode::EnterFilename);
}

#[test]
fn open_locks_timestamp() {
    let s = open_state(None);
    assert_eq!(s.locked_timestamp(), TS);
}

#[test]
fn open_prefills_filename_with_default_pattern() {
    let s = open_state(None);
    let text = s.current_filename_text();
    assert!(text.contains(TS));
    assert!(text.ends_with(".json"));
}

#[test]
fn close_returns_to_closed_state() {
    let mut s = open_state(None);
    s.close();
    assert!(!s.is_visible());
    assert_eq!(s.mode(), &SaveMode::Closed);
}

#[test]
fn mark_filename_edited_sets_dirty_flag() {
    let mut s = open_state(None);
    assert!(!s.filename_dirty());
    s.mark_filename_edited();
    assert!(s.filename_dirty());
}

#[test]
fn prepare_write_returns_ready_for_new_path() {
    let dir = TempDir::new().unwrap();
    let target = dir.path().join("brand-new.json");
    let s = open_state(Some(target.to_string_lossy().as_ref()));
    match s.prepare_write() {
        WriteOutcome::ReadyToWrite(p) => assert_eq!(p, target),
        other => panic!("expected ReadyToWrite, got {:?}", other),
    }
}

#[test]
fn prepare_write_returns_ready_when_file_exists_too() {
    let dir = TempDir::new().unwrap();
    let target = dir.path().join("exists.json");
    fs::write(&target, "old").unwrap();
    let s = open_state(Some(target.to_string_lossy().as_ref()));
    match s.prepare_write() {
        WriteOutcome::ReadyToWrite(p) => assert_eq!(p, target),
        other => panic!(
            "expected ReadyToWrite (overwrite is up to caller), got {:?}",
            other
        ),
    }
}

#[test]
fn prepare_write_errors_on_empty_filename() {
    let s = open_state(Some(""));
    match s.prepare_write() {
        WriteOutcome::Error(_) => {}
        other => panic!("expected Error, got {:?}", other),
    }
}

#[test]
fn prepare_write_errors_on_missing_env_var() {
    unsafe {
        std::env::remove_var("JIQ_TEST_STATE_MISSING");
    }
    let s = open_state(Some("$JIQ_TEST_STATE_MISSING/foo.json"));
    match s.prepare_write() {
        WriteOutcome::Error(_) => {}
        other => panic!("expected Error, got {:?}", other),
    }
}

#[test]
fn compute_preview_for_new_path_is_ready_not_existing() {
    let dir = TempDir::new().unwrap();
    let target = dir.path().join("fresh.json");
    let s = open_state(Some(target.to_string_lossy().as_ref()));
    match s.compute_preview() {
        PathPreview::Ready { resolved, exists } => {
            assert_eq!(resolved, target);
            assert!(!exists);
        }
        other => panic!("expected Ready, got {:?}", other),
    }
}

#[test]
fn compute_preview_flags_existing_file() {
    let dir = TempDir::new().unwrap();
    let target = dir.path().join("hit.json");
    fs::write(&target, "x").unwrap();
    let s = open_state(Some(target.to_string_lossy().as_ref()));
    match s.compute_preview() {
        PathPreview::Ready { resolved, exists } => {
            assert_eq!(resolved, target);
            assert!(exists, "exists flag must be true for a real file");
        }
        other => panic!("expected Ready{{exists:true}}, got {:?}", other),
    }
}

#[test]
fn compute_preview_returns_error_on_empty_input() {
    let s = open_state(Some(""));
    assert!(matches!(s.compute_preview(), PathPreview::Error(_)));
}

#[test]
fn compute_preview_returns_error_on_unset_env_var() {
    unsafe {
        std::env::remove_var("JIQ_TEST_PREVIEW_MISSING");
    }
    let s = open_state(Some("$JIQ_TEST_PREVIEW_MISSING/x.json"));
    assert!(matches!(s.compute_preview(), PathPreview::Error(_)));
}

#[test]
fn would_overwrite_helper_reflects_preview() {
    let dir = TempDir::new().unwrap();
    let fresh = dir.path().join("fresh.json");
    let exists = dir.path().join("exists.json");
    fs::write(&exists, "x").unwrap();

    let s_fresh = open_state(Some(fresh.to_string_lossy().as_ref()));
    assert!(!s_fresh.compute_preview().would_overwrite());

    let s_exists = open_state(Some(exists.to_string_lossy().as_ref()));
    assert!(s_exists.compute_preview().would_overwrite());
}

#[test]
fn open_resets_dirty_flag() {
    let mut s = SaveState::new();
    s.open(TS.to_string());
    s.mark_filename_edited();
    assert!(s.filename_dirty());
    s.open(TS.to_string());
    assert!(!s.filename_dirty());
}

#[test]
fn open_replaces_locked_timestamp_each_time() {
    let mut s = SaveState::new();
    s.open("20260101-000000".to_string());
    assert_eq!(s.locked_timestamp(), "20260101-000000");
    s.open("20260202-111111".to_string());
    assert_eq!(s.locked_timestamp(), "20260202-111111");
}
