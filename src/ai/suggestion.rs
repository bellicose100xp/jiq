//! Suggestion module for AI assistant
//!
//! This module provides types and parsing logic for AI suggestions.

pub mod parser;

// Re-export main types
pub use parser::{Suggestion, SuggestionType, parse_suggestions};
