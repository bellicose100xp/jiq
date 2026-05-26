//! Double-click detection.
//!
//! Crossterm has no native double-click event on Unix terminals — apps must
//! buffer the previous click's position/time and decide on the next one.
//! This module owns that detection so every consumer (results pane drill,
//! autocomplete accept, future widgets) shares one well-tested primitive.
//!
//! Defaults match the prevailing TUI convention (zellij, alacritty): 400 ms
//! threshold, hard-coded. State is reset on the first hover or scroll
//! after a click, so an idle pointer can't accumulate a stale second click.

use std::time::{Duration, Instant};

use ratatui::crossterm::event::MouseEvent;

use crate::layout::Region;

const DOUBLE_CLICK_THRESHOLD: Duration = Duration::from_millis(400);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Granularity {
    SameCell,
    SameRow,
}

#[derive(Debug, Clone, Copy)]
struct LastClick {
    instant: Instant,
    col: u16,
    row: u16,
    region: Region,
}

#[derive(Debug, Default, Clone)]
pub struct DoubleClickTracker {
    last: Option<LastClick>,
}

impl DoubleClickTracker {
    pub fn new() -> Self {
        Self { last: None }
    }

    /// Record this click and report whether it completes a double-click pair
    /// against the previous click in the same `region` and within the
    /// threshold. On a successful match, internal state is cleared so a
    /// third click doesn't keep the streak going.
    pub fn check_and_record(
        &mut self,
        mouse: MouseEvent,
        region: Region,
        granularity: Granularity,
    ) -> bool {
        let now = Instant::now();
        let is_double = self
            .last
            .filter(|prev| prev.region == region)
            .filter(|prev| now.duration_since(prev.instant) <= DOUBLE_CLICK_THRESHOLD)
            .filter(|prev| match granularity {
                Granularity::SameCell => prev.col == mouse.column && prev.row == mouse.row,
                Granularity::SameRow => prev.row == mouse.row,
            })
            .is_some();

        if is_double {
            self.last = None;
        } else {
            self.last = Some(LastClick {
                instant: now,
                col: mouse.column,
                row: mouse.row,
                region,
            });
        }
        is_double
    }

    /// Drop any pending click. Called on hover/scroll so an unrelated
    /// pointer movement between two clicks invalidates the pair.
    pub fn reset(&mut self) {
        self.last = None;
    }
}

#[cfg(test)]
#[path = "double_click_tests.rs"]
mod double_click_tests;
