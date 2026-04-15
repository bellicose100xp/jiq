//! Suggestion module for AI assistant
//!
//! This module provides types and parsing logic for AI suggestions.

pub mod parser;
pub mod sanitizer;

// Re-export main types
pub use parser::{Suggestion, SuggestionType, parse_suggestions};
