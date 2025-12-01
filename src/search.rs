//! Search module
//!
//! Provides text search functionality within the results pane.
//! Users can search for text, see matches highlighted, and navigate between matches.

pub mod events;
mod matcher;
pub mod search_render;
mod state;

pub use state::{Match, SearchState};
