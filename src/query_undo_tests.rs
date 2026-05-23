use super::*;

fn vp() -> ViewportState {
    ViewportState::default()
}

fn vp_at(cursor_row: u32, scroll_offset: u16, h_offset: u16) -> ViewportState {
    ViewportState {
        cursor_row,
        scroll_offset,
        h_offset,
    }
}

#[test]
fn empty_ring_pops_as_empty() {
    let mut ring = QueryUndoRing::new();
    assert!(ring.is_empty());
    assert_eq!(ring.pop_if_matches("."), PopOutcome::Empty);
}

#[test]
fn push_then_pop_round_trips_when_unchanged() {
    let mut ring = QueryUndoRing::new();
    ring.push(".", ".users", vp_at(7, 4, 2));
    assert!(!ring.is_empty());
    assert_eq!(
        ring.pop_if_matches(".users"),
        PopOutcome::Restored {
            query: ".".into(),
            viewport: vp_at(7, 4, 2),
        }
    );
    assert!(ring.is_empty());
}

#[test]
fn manual_edit_invalidates_ring() {
    let mut ring = QueryUndoRing::new();
    ring.push(".", ".users", vp());
    // User typed extra characters in the textarea after the drill-in.
    assert_eq!(ring.pop_if_matches(".users[0]"), PopOutcome::Invalidated);
    assert!(ring.is_empty(), "invalidation must clear the whole ring");
}

#[test]
fn manual_edit_clears_chain_not_just_top() {
    let mut ring = QueryUndoRing::new();
    ring.push(".", ".a", vp());
    ring.push(".a", ".a | .b", vp());
    ring.push(".a | .b", ".a | .b | .c", vp());
    // User edits between two drill-ins; only the top entry's expected
    // string matters for the invalidation check.
    assert_eq!(
        ring.pop_if_matches(".a | .b | tweaked"),
        PopOutcome::Invalidated
    );
    assert_eq!(ring.depth(), 0);
}

#[test]
fn deep_chain_pops_in_reverse_order() {
    let mut ring = QueryUndoRing::new();
    ring.push(".", ".a", vp_at(1, 0, 0));
    ring.push(".a", ".a | .b", vp_at(2, 0, 0));
    ring.push(".a | .b", ".a | .b | .c", vp_at(3, 0, 0));
    assert_eq!(
        ring.pop_if_matches(".a | .b | .c"),
        PopOutcome::Restored {
            query: ".a | .b".into(),
            viewport: vp_at(3, 0, 0),
        }
    );
    assert_eq!(
        ring.pop_if_matches(".a | .b"),
        PopOutcome::Restored {
            query: ".a".into(),
            viewport: vp_at(2, 0, 0),
        }
    );
    assert_eq!(
        ring.pop_if_matches(".a"),
        PopOutcome::Restored {
            query: ".".into(),
            viewport: vp_at(1, 0, 0),
        }
    );
    assert_eq!(ring.pop_if_matches("."), PopOutcome::Empty);
}

#[test]
fn ring_caps_at_max_depth() {
    let mut ring = QueryUndoRing::new();
    for i in 0..100 {
        ring.push(format!("p{}", i), format!("e{}", i), vp());
    }
    assert_eq!(ring.depth(), 20);
    // Top should still be the most-recent entry.
    assert_eq!(
        ring.pop_if_matches("e99"),
        PopOutcome::Restored {
            query: "p99".into(),
            viewport: vp(),
        }
    );
}
