//! Stats module for computing and displaying result statistics
//!
//! This module provides fast, character-based parsing to compute statistics
//! about jq query results without full JSON parsing.

mod parser;
mod stats_state;
mod types;

// Re-export public types
pub use stats_state::StatsState;
pub use stats_state::update_stats_from_app;
