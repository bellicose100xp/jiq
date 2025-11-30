//! Stats module for computing and displaying result statistics
//!
//! This module provides fast, character-based parsing to compute statistics
//! about jq query results without full JSON parsing.

mod parser;
mod state;
mod types;

// Re-export public types
pub use state::StatsState;
