//! Selection state module for AI suggestion navigation and application
//!
//! This module provides functionality for selecting and applying AI suggestions
//! via keyboard shortcuts (Alt+1-5 for direct selection, Alt+Up/Down/j/k for navigation).

pub mod apply;
pub mod keybindings;
pub mod state;

// Re-export main types
pub use state::SelectionState;
