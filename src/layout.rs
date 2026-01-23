//! Layout module for tracking UI component regions
//!
//! This module provides region tracking for position-aware mouse interactions.
//! The `LayoutRegions` struct tracks where UI components are rendered, and
//! `region_at()` determines which component is at a given screen position.

mod layout_hit_test;
mod layout_regions;

#[allow(unused_imports)]
pub use layout_hit_test::region_at;
pub use layout_regions::LayoutRegions;
#[allow(unused_imports)]
pub use layout_regions::Region;

#[cfg(test)]
#[path = "layout/layout_regions_tests.rs"]
mod layout_regions_tests;

#[cfg(test)]
#[path = "layout/layout_hit_test_tests.rs"]
mod layout_hit_test_tests;
