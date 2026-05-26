//! Tests for the shared double-click tracker.

use std::thread::sleep;
use std::time::Duration;

use ratatui::crossterm::event::{KeyModifiers, MouseButton, MouseEvent, MouseEventKind};

use crate::layout::Region;

use super::{DoubleClickTracker, Granularity};

fn click(col: u16, row: u16) -> MouseEvent {
    MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: col,
        row,
        modifiers: KeyModifiers::NONE,
    }
}

#[test]
fn first_click_is_never_a_double() {
    let mut t = DoubleClickTracker::new();
    assert!(!t.check_and_record(click(5, 10), Region::ResultsPane, Granularity::SameRow));
}

#[test]
fn same_row_second_click_within_threshold_is_double() {
    let mut t = DoubleClickTracker::new();
    t.check_and_record(click(5, 10), Region::ResultsPane, Granularity::SameRow);
    assert!(t.check_and_record(click(20, 10), Region::ResultsPane, Granularity::SameRow));
}

#[test]
fn same_cell_requires_exact_column_match() {
    let mut t = DoubleClickTracker::new();
    t.check_and_record(click(5, 10), Region::Autocomplete, Granularity::SameCell);
    assert!(
        !t.check_and_record(click(6, 10), Region::Autocomplete, Granularity::SameCell),
        "different column under SameCell must not pair"
    );

    let mut t = DoubleClickTracker::new();
    t.check_and_record(click(5, 10), Region::Autocomplete, Granularity::SameCell);
    assert!(t.check_and_record(click(5, 10), Region::Autocomplete, Granularity::SameCell));
}

#[test]
fn different_row_under_same_row_granularity_does_not_pair() {
    let mut t = DoubleClickTracker::new();
    t.check_and_record(click(5, 10), Region::ResultsPane, Granularity::SameRow);
    assert!(!t.check_and_record(click(5, 11), Region::ResultsPane, Granularity::SameRow));
}

#[test]
fn region_change_breaks_the_pair() {
    let mut t = DoubleClickTracker::new();
    t.check_and_record(click(5, 10), Region::ResultsPane, Granularity::SameRow);
    assert!(!t.check_and_record(click(5, 10), Region::Autocomplete, Granularity::SameRow));
}

#[test]
fn second_click_past_threshold_does_not_pair() {
    let mut t = DoubleClickTracker::new();
    t.check_and_record(click(5, 10), Region::ResultsPane, Granularity::SameRow);
    sleep(Duration::from_millis(420));
    assert!(!t.check_and_record(click(5, 10), Region::ResultsPane, Granularity::SameRow));
}

#[test]
fn successful_double_does_not_chain_into_triple() {
    let mut t = DoubleClickTracker::new();
    t.check_and_record(click(5, 10), Region::ResultsPane, Granularity::SameRow);
    assert!(t.check_and_record(click(5, 10), Region::ResultsPane, Granularity::SameRow));
    assert!(
        !t.check_and_record(click(5, 10), Region::ResultsPane, Granularity::SameRow),
        "third rapid click must not pair as another double-click"
    );
}

#[test]
fn reset_clears_pending_click() {
    let mut t = DoubleClickTracker::new();
    t.check_and_record(click(5, 10), Region::ResultsPane, Granularity::SameRow);
    t.reset();
    assert!(!t.check_and_record(click(5, 10), Region::ResultsPane, Granularity::SameRow));
}
