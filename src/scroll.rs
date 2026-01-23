mod scroll_state;
mod scroll_trait;

pub use scroll_state::ScrollState;
pub use scroll_trait::Scrollable;

#[cfg(test)]
#[path = "scroll/scroll_state_tests.rs"]
mod scroll_state_tests;

#[cfg(test)]
#[path = "scroll/scroll_trait_tests.rs"]
mod scroll_trait_tests;
