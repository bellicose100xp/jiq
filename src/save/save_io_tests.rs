use std::env;
use std::fs;

use chrono::TimeZone;
use tempfile::TempDir;

use super::*;

const TS: &str = "20260527-104522";

fn ts() -> &'static str {
    TS
}

#[test]
fn timestamp_format_is_sortable_and_fs_safe() {
    let dt = chrono::Local
        .with_ymd_and_hms(2026, 5, 27, 10, 45, 22)
        .unwrap();
    let s = format_timestamp(dt);
    assert_eq!(s, "20260527-104522");
    assert!(!s.contains(':'));
    assert!(!s.contains(' '));
}

#[test]
fn current_timestamp_matches_format() {
    let s = current_timestamp();
    assert_eq!(s.len(), 15);
    assert_eq!(s.as_bytes()[8], b'-');
}

#[test]
fn ext_for_result_is_json() {
    assert_eq!(ext_for_result(), "json");
}

#[test]
fn expand_replaces_timestamp_and_ext() {
    let p = expand_path("/tmp/jiq-{timestamp}.{ext}", "json", ts()).unwrap();
    assert_eq!(p.to_string_lossy(), "/tmp/jiq-20260527-104522.json");
}

#[test]
fn expand_replaces_cwd() {
    let cwd = env::current_dir().unwrap();
    let p = expand_path("{cwd}/out.json", "json", ts()).unwrap();
    assert_eq!(p, cwd.join("out.json"));
}

#[test]
fn expand_handles_tilde_only() {
    let home = dirs::home_dir().unwrap();
    let p = expand_path("~", "json", ts()).unwrap();
    assert_eq!(p, home);
}

#[test]
fn expand_handles_tilde_with_subpath() {
    let home = dirs::home_dir().unwrap();
    let p = expand_path("~/jiq.json", "json", ts()).unwrap();
    assert_eq!(p, home.join("jiq.json"));
}

#[test]
fn expand_does_not_mistake_internal_tilde() {
    let p = expand_path("/tmp/foo~bar.json", "json", ts()).unwrap();
    assert_eq!(p.to_string_lossy(), "/tmp/foo~bar.json");
}

#[test]
fn expand_dollar_braced_env_var() {
    // SAFETY: setting/removing env vars is safe in test threads as long as no
    // other test in the same binary reads the same name concurrently.
    unsafe {
        env::set_var("JIQ_TEST_BRACED", "/braced");
    }
    let p = expand_path("${JIQ_TEST_BRACED}/x.json", "json", ts()).unwrap();
    assert_eq!(p.to_string_lossy(), "/braced/x.json");
    unsafe {
        env::remove_var("JIQ_TEST_BRACED");
    }
}

#[test]
fn expand_dollar_unbraced_env_var() {
    unsafe {
        env::set_var("JIQ_TEST_UNBRACED", "/un");
    }
    let p = expand_path("$JIQ_TEST_UNBRACED/x.json", "json", ts()).unwrap();
    assert_eq!(p.to_string_lossy(), "/un/x.json");
    unsafe {
        env::remove_var("JIQ_TEST_UNBRACED");
    }
}

#[test]
fn expand_missing_env_var_errors() {
    unsafe {
        env::remove_var("JIQ_TEST_MISSING");
    }
    let err = expand_path("$JIQ_TEST_MISSING/x.json", "json", ts()).unwrap_err();
    match err {
        SaveError::EnvVarMissing(name) => assert_eq!(name, "JIQ_TEST_MISSING"),
        other => panic!("expected EnvVarMissing, got {:?}", other),
    }
}

#[test]
fn expand_unclosed_brace_errors() {
    let err = expand_path("${UNCLOSED/x.json", "json", ts()).unwrap_err();
    assert!(matches!(err, SaveError::BadPath(_)));
}

#[test]
fn expand_lone_dollar_at_eof_is_kept() {
    let p = expand_path("/tmp/foo$", "json", ts()).unwrap();
    assert_eq!(p.to_string_lossy(), "/tmp/foo$");
}

#[test]
fn expand_dollar_followed_by_invalid_is_kept() {
    let p = expand_path("/tmp/$1foo.json", "json", ts()).unwrap();
    assert_eq!(p.to_string_lossy(), "/tmp/$1foo.json");
}

#[test]
fn expand_empty_pattern_errors() {
    let err = expand_path("   ", "json", ts()).unwrap_err();
    assert!(matches!(err, SaveError::BadPath(_)));
}

#[test]
fn expand_combines_multiple_placeholders() {
    unsafe {
        env::set_var("JIQ_TEST_BASE", "/base");
    }
    let p = expand_path("$JIQ_TEST_BASE/{timestamp}.{ext}", "json", ts()).unwrap();
    assert_eq!(p.to_string_lossy(), "/base/20260527-104522.json");
    unsafe {
        env::remove_var("JIQ_TEST_BASE");
    }
}

#[test]
fn save_error_display_io() {
    let e = SaveError::Io(std::io::Error::other("oops"));
    assert!(format!("{}", e).contains("oops"));
}

#[test]
fn save_error_display_bad_path() {
    let e = SaveError::BadPath("nope".into());
    assert_eq!(format!("{}", e), "nope");
}

#[test]
fn save_error_display_env_var_missing() {
    let e = SaveError::EnvVarMissing("FOO".into());
    assert_eq!(format!("{}", e), "env var FOO not set");
}

#[test]
fn save_error_from_io_error() {
    let io_err = std::io::Error::other("x");
    let save_err: SaveError = io_err.into();
    assert!(matches!(save_err, SaveError::Io(_)));
}

#[test]
fn write_atomic_creates_file_with_contents() {
    let dir = TempDir::new().unwrap();
    let target = dir.path().join("out.json");
    let canonical = write_atomic(&target, "hello").unwrap();
    assert_eq!(fs::read_to_string(&canonical).unwrap(), "hello");
}

#[test]
fn write_atomic_overwrites_existing_file() {
    let dir = TempDir::new().unwrap();
    let target = dir.path().join("out.json");
    fs::write(&target, "old").unwrap();
    write_atomic(&target, "new").unwrap();
    assert_eq!(fs::read_to_string(&target).unwrap(), "new");
}

#[test]
fn write_atomic_returns_canonical_absolute_path() {
    let dir = TempDir::new().unwrap();
    let target = dir.path().join("out.json");
    let returned = write_atomic(&target, "x").unwrap();
    assert!(returned.is_absolute());
}

#[test]
fn write_atomic_leaves_no_tmp_file_on_success() {
    let dir = TempDir::new().unwrap();
    let target = dir.path().join("out.json");
    write_atomic(&target, "x").unwrap();
    let entries: Vec<_> = fs::read_dir(dir.path())
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().to_string())
        .collect();
    assert_eq!(entries, vec!["out.json".to_string()]);
}

#[test]
fn write_atomic_errors_when_parent_dir_missing() {
    let dir = TempDir::new().unwrap();
    let target = dir.path().join("missing/out.json");
    let err = write_atomic(&target, "x").unwrap_err();
    assert!(matches!(err, SaveError::Io(_)));
}

#[test]
fn write_atomic_errors_when_path_has_no_filename() {
    let dir = TempDir::new().unwrap();
    let err = write_atomic(dir.path(), "x").unwrap_err();
    assert!(matches!(err, SaveError::Io(_) | SaveError::BadPath(_)));
}

#[test]
#[cfg(unix)]
fn write_atomic_errors_on_readonly_dir() {
    use std::os::unix::fs::PermissionsExt;
    let dir = TempDir::new().unwrap();
    let mut perms = fs::metadata(dir.path()).unwrap().permissions();
    perms.set_mode(0o500);
    fs::set_permissions(dir.path(), perms).unwrap();

    let target = dir.path().join("out.json");
    let result = write_atomic(&target, "x");

    let mut perms = fs::metadata(dir.path()).unwrap().permissions();
    perms.set_mode(0o700);
    fs::set_permissions(dir.path(), perms).unwrap();

    assert!(result.is_err(), "expected write to fail on read-only dir");
}
