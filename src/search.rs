//! Search module
//!
//! Provides text search functionality within the results pane.
//! Users can search for text, see matches highlighted, and navigate between matches.

mod matcher;
pub mod search_events;
pub mod search_render;
mod search_state;

pub use search_state::{Match, SearchState};
