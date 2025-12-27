//! Tests for SuggestionCache

use super::*;

#[test]
fn test_cache_new() {
    let cache = SuggestionCache::new();
    // Cache should be created without panic
    assert!(format!("{:?}", cache).contains("SuggestionCache"));
}

#[test]
fn test_cache_default() {
    let cache = SuggestionCache::default();
    // Default should work identically to new
    assert!(format!("{:?}", cache).contains("SuggestionCache"));
}

#[test]
fn test_cache_debug() {
    let cache = SuggestionCache::new();
    let debug_str = format!("{:?}", cache);
    assert!(!debug_str.is_empty());
}
