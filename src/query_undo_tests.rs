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
fn empty_ring_pops_as_none() {
    let mut ring = QueryUndoRing::new();
    assert!(ring.is_empty());
    assert!(ring.pop().is_none());
}

#[test]
fn push_then_pop_round_trips() {
    let mut ring = QueryUndoRing::new();
    ring.push(".", vp_at(7, 4, 2));
    assert!(!ring.is_empty());
    let (prev, viewport) = ring.pop().unwrap();
    assert_eq!(prev, ".");
    assert_eq!(viewport, vp_at(7, 4, 2));
    assert!(ring.is_empty());
}

#[test]
fn pop_works_regardless_of_intervening_edits() {
    // The ring no longer tracks `expected_after`; `<` always pops the
    // most recent snapshot, even if the user manually edited the
    // textarea between drill-ins. The simpler mental model is the
    // explicit trade-off.
    let mut ring = QueryUndoRing::new();
    ring.push(".", vp());
    let (prev, _) = ring.pop().unwrap();
    assert_eq!(prev, ".");
    assert!(ring.is_empty());
}

#[test]
fn deep_chain_pops_in_reverse_order() {
    let mut ring = QueryUndoRing::new();
    ring.push(".", vp_at(1, 0, 0));
    ring.push(".a", vp_at(2, 0, 0));
    ring.push(".a | .b", vp_at(3, 0, 0));
    assert_eq!(ring.pop().unwrap(), (".a | .b".into(), vp_at(3, 0, 0)));
    assert_eq!(ring.pop().unwrap(), (".a".into(), vp_at(2, 0, 0)));
    assert_eq!(ring.pop().unwrap(), (".".into(), vp_at(1, 0, 0)));
    assert!(ring.pop().is_none());
}

#[test]
fn ring_caps_at_max_depth() {
    let mut ring = QueryUndoRing::new();
    for i in 0..100 {
        ring.push(format!("p{}", i), vp());
    }
    assert_eq!(ring.depth(), 20);
    let (prev, _) = ring.pop().unwrap();
    assert_eq!(prev, "p99");
}
