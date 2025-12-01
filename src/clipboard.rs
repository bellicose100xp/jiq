//! Clipboard module for jiq
//!
//! Provides clipboard functionality with support for:
//! - System clipboard (via arboard)
//! - OSC 52 escape sequences (for remote terminals)
//! - Auto mode (system with OSC 52 fallback)

mod backend;
pub mod clipboard_events;
mod osc52;
mod system;
