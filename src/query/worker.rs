//! Query Worker Module
//!
//! Handles jq query execution in a background thread to avoid blocking the UI.
//! Receives requests via channel, executes jq with cancellation support,
//! and sends responses back to the main thread.
//!
//! ## Architecture
//!
//! The worker follows the same pattern as the AI worker:
//! - Single background thread with std::sync::mpsc channels
//! - Blocking recv() in dedicated thread (not async)
//! - Panic hook to prevent TUI corruption
//! - Request/Response pattern with cancellation tokens
//!
//! ## Usage
//!
//! ```ignore
//! use std::sync::mpsc::channel;
//! use query::worker::{spawn_worker, QueryRequest, QueryResponse};
//! use tokio_util::sync::CancellationToken;
//!
//! // Create channels
//! let (request_tx, request_rx) = channel();
//! let (response_tx, response_rx) = channel();
//!
//! // Spawn worker
//! spawn_worker(json_input, request_rx, response_tx);
//!
//! // Send request
//! let cancel_token = CancellationToken::new();
//! request_tx.send(QueryRequest {
//!     query: ".foo".to_string(),
//!     request_id: 1,
//!     cancel_token,
//! }).unwrap();
//!
//! // Receive response
//! match response_rx.recv().unwrap() {
//!     QueryResponse::Success { output, .. } => println!("{}", output),
//!     QueryResponse::Error { message, .. } => eprintln!("{}", message),
//!     QueryResponse::Cancelled { .. } => println!("Cancelled"),
//! }
//! ```

pub mod thread;
pub mod types;

// Re-exports for convenience
pub use thread::spawn_worker;
pub use types::{QueryRequest, QueryResponse};
