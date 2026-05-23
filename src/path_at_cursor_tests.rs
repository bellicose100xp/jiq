use super::*;
use serde_json::json;
use std::sync::Arc;

#[test]
fn cache_hits_on_same_row() {
    let v = Arc::new(json!({"a": 1, "b": 2}));
    let mut cache = PathAtCursorCache::new();
    let p1 = cache.resolve(&v, 1).unwrap();
    let p2 = cache.resolve(&v, 1).unwrap();
    assert_eq!(p1.to_jq(), ".a");
    assert_eq!(p2.to_jq(), ".a");
}

#[test]
fn cache_misses_on_different_row() {
    let v = Arc::new(json!({"a": 1, "b": 2}));
    let mut cache = PathAtCursorCache::new();
    let p1 = cache.resolve(&v, 1).unwrap();
    let p2 = cache.resolve(&v, 2).unwrap();
    assert_eq!(p1.to_jq(), ".a");
    assert_eq!(p2.to_jq(), ".b");
}

#[test]
fn invalidate_drops_cached_entry() {
    let v = Arc::new(json!({"a": 1, "b": 2}));
    let mut cache = PathAtCursorCache::new();
    let _ = cache.resolve(&v, 1);
    cache.invalidate();
    let v2 = Arc::new(json!({"x": 1, "y": 2}));
    let p = cache.resolve(&v2, 1).unwrap();
    assert_eq!(p.to_jq(), ".x");
}

#[test]
fn resolve_past_end_returns_none() {
    let v = Arc::new(json!({"a": 1}));
    let mut cache = PathAtCursorCache::new();
    assert!(cache.resolve(&v, 1000).is_none());
}

#[test]
fn resolve_root_scalar_returns_root_path() {
    let v = Arc::new(json!(42));
    let mut cache = PathAtCursorCache::new();
    let p = cache.resolve(&v, 0).unwrap();
    assert_eq!(p.to_jq(), ".");
}

#[test]
fn caches_none_on_past_end_too() {
    let v = Arc::new(json!({"a": 1}));
    let mut cache = PathAtCursorCache::new();
    let p1 = cache.resolve(&v, 100);
    let p2 = cache.resolve(&v, 100);
    assert!(p1.is_none() && p2.is_none());
}
