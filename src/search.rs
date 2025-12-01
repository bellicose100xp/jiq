//! Search module
//!
//! Provides text search functionality within the results pane.
//! Users can search for text, see matches highlighted, and navigate between matches.

pub mod search_events;
mod matcher;
pub mod search_render;
mod search_state;

pub use search_state::{Match, SearchState};
