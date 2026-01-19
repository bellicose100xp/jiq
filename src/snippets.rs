pub mod snippet_events;
pub mod snippet_render;
mod snippet_state;
pub mod snippet_storage;

#[allow(unused_imports)] // Snippet will be used externally in later phases
pub use snippet_state::{Snippet, SnippetState};
