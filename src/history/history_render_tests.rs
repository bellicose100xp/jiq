//! Tests for history popup rendering

use super::HISTORY_SEARCH_HEIGHT;

#[test]
fn test_history_search_height_constant() {
    assert_eq!(HISTORY_SEARCH_HEIGHT, 3);
}
