//! Query Worker Thread
//!
//! Handles jq query execution in a background thread to avoid blocking the UI.
//! Receives requests via channel, executes jq with cancellation support,
//! and sends responses back to the main thread.

use std::panic::{self, AssertUnwindSafe};
use std::sync::mpsc::{Receiver, Sender};

use super::preprocess::preprocess_result;
use super::types::{QueryError, QueryRequest, QueryResponse};
use crate::query::executor::JqExecutor;

/// Spawn the query worker thread
///
/// Creates a background thread that:
/// 1. Listens for query requests on the request channel
/// 2. Executes jq queries with cancellation support
/// 3. Sends responses back via the response channel
///
/// Includes panic handling to prevent TUI corruption.
///
/// # Arguments
/// * `json_input` - JSON input for query execution
/// * `request_rx` - Channel to receive requests
/// * `response_tx` - Channel to send responses
pub fn spawn_worker(
    json_input: String,
    request_rx: Receiver<QueryRequest>,
    response_tx: Sender<QueryResponse>,
) {
    std::thread::spawn(move || {
        // Set panic hook to prevent TUI corruption
        let response_tx_clone = response_tx.clone();
        let prev_hook = panic::take_hook();
        panic::set_hook(Box::new(move |panic_info| {
            let panic_msg = if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
                s.to_string()
            } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
                s.clone()
            } else {
                "Unknown panic in query worker".to_string()
            };

            log::error!(
                "Query worker panic: {} at {:?}",
                panic_msg,
                panic_info.location()
            );

            // Try to send error to main thread
            // Use request_id = 0 to indicate worker-level error
            let _ = response_tx_clone.send(QueryResponse::Error {
                message: format!("Query worker crashed: {}", panic_msg),
                query: String::new(), // No specific query for worker-level errors
                request_id: 0,
            });
        }));

        // Wrap worker in catch_unwind
        let result = panic::catch_unwind(AssertUnwindSafe(|| {
            worker_loop(&json_input, request_rx, response_tx);
        }));

        // Restore panic hook
        panic::set_hook(prev_hook);

        if let Err(e) = result {
            let panic_msg = if let Some(s) = e.downcast_ref::<&str>() {
                s.to_string()
            } else if let Some(s) = e.downcast_ref::<String>() {
                s.clone()
            } else {
                "Unknown panic".to_string()
            };
            log::error!("Query worker thread panicked: {}", panic_msg);
        }
    });
}

/// Main worker loop - processes requests until channel closes
///
/// Uses blocking recv() which is fine in dedicated thread.
fn worker_loop(
    json_input: &str,
    request_rx: Receiver<QueryRequest>,
    response_tx: Sender<QueryResponse>,
) {
    log::debug!("Query worker thread started");
    let executor = JqExecutor::new(json_input.to_string());

    // Process requests until channel closes
    while let Ok(request) = request_rx.recv() {
        log::debug!(
            "Worker received request {}: {}",
            request.request_id,
            request.query
        );
        handle_request(&executor, request, &response_tx);
    }

    log::debug!("Query worker thread shutting down");
}

/// Handle a single query request
fn handle_request(
    executor: &JqExecutor,
    request: QueryRequest,
    response_tx: &Sender<QueryResponse>,
) {
    // Check if already cancelled
    if request.cancel_token.is_cancelled() {
        let _ = response_tx.send(QueryResponse::Cancelled {
            request_id: request.request_id,
        });
        return;
    }

    // Execute query with cancellation support
    log::debug!("Executing query {} with jq", request.request_id);
    let query = request.query.clone();
    match executor.execute_with_cancel(&request.query, &request.cancel_token) {
        Ok(output) => {
            log::debug!(
                "Query {} succeeded, preprocessing result",
                request.request_id
            );

            // Preprocess result (expensive operations done in worker thread)
            match preprocess_result(output, &query, &request.cancel_token) {
                Ok(processed) => {
                    log::debug!(
                        "Query {} preprocessing complete, sending response",
                        request.request_id
                    );
                    let _ = response_tx.send(QueryResponse::ProcessedSuccess {
                        processed,
                        request_id: request.request_id,
                    });
                }
                Err(QueryError::Cancelled) => {
                    log::debug!("Query {} preprocessing was cancelled", request.request_id);
                    let _ = response_tx.send(QueryResponse::Cancelled {
                        request_id: request.request_id,
                    });
                }
                Err(e) => {
                    log::debug!("Query {} preprocessing failed: {}", request.request_id, e);
                    let _ = response_tx.send(QueryResponse::Error {
                        message: e.to_string(),
                        query: query.clone(),
                        request_id: request.request_id,
                    });
                }
            }
        }
        Err(QueryError::Cancelled) => {
            log::debug!("Query {} was cancelled", request.request_id);
            let _ = response_tx.send(QueryResponse::Cancelled {
                request_id: request.request_id,
            });
        }
        Err(e) => {
            log::debug!("Query {} failed: {}", request.request_id, e);
            let _ = response_tx.send(QueryResponse::Error {
                message: e.to_string(),
                query: request.query,
                request_id: request.request_id,
            });
        }
    }
}

#[cfg(test)]
#[path = "thread_tests.rs"]
mod thread_tests;
