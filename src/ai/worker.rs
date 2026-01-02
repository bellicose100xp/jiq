//! AI Worker Thread
//!
//! Handles AI requests in a background thread to avoid blocking the UI.
//! Receives requests via channel, makes HTTP calls to the AI provider,
//! and streams responses back to the main thread.
//!
//! Uses a tokio runtime for async HTTP streaming with cancellation support.
//! Includes panic handling to prevent TUI corruption from AWS SDK panics.

use std::panic::{self, AssertUnwindSafe};
use std::sync::mpsc::{Receiver, Sender};

use tokio_util::sync::CancellationToken;

use super::ai_state::{AiRequest, AiResponse};
use super::provider::{AiError, AsyncAiProvider};
use crate::config::ai_types::AiConfig;

/// Spawn the AI worker thread
///
/// Creates a background thread with a tokio runtime that:
/// 1. Listens for requests on the request channel
/// 2. Makes async HTTP calls to the AI provider with cancellation support
/// 3. Streams responses back via the response channel
///
/// The worker thread includes panic handling to prevent panics (e.g., from
/// AWS SDK credential loading) from corrupting the TUI.
///
/// # Arguments
/// * `config` - AI configuration (for creating the provider)
/// * `request_rx` - Channel to receive requests from the main thread
/// * `response_tx` - Channel to send responses to the main thread
///
/// # Requirements
/// - 4.1: WHEN the AI provider sends a streaming response THEN the AI_Popup
///   SHALL display text incrementally as chunks arrive
/// - 4.2: WHEN the worker thread is spawned THEN it SHALL create a tokio runtime
///   for async operations
pub fn spawn_worker(
    config: &AiConfig,
    request_rx: Receiver<AiRequest>,
    response_tx: Sender<AiResponse>,
) {
    // Try to create the async provider from config
    let provider_result = AsyncAiProvider::from_config(config);

    std::thread::spawn(move || {
        // Set a custom panic hook for this thread to suppress output
        // The default panic hook prints to stderr which corrupts the TUI
        let response_tx_clone = response_tx.clone();
        let prev_hook = panic::take_hook();
        panic::set_hook(Box::new(move |panic_info| {
            // Extract panic message
            let panic_msg = if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
                s.to_string()
            } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
                s.clone()
            } else {
                "Unknown panic in AI worker".to_string()
            };

            // Log the panic instead of printing to stderr
            log::error!(
                "AI worker panic: {} at {:?}",
                panic_msg,
                panic_info.location()
            );

            // Try to send error to main thread
            let _ = response_tx_clone.send(AiResponse::Error(format!(
                "AI worker crashed: {}",
                panic_msg
            )));
        }));

        // Wrap the entire worker in catch_unwind to handle panics gracefully
        let result = panic::catch_unwind(AssertUnwindSafe(|| {
            // Create a single-threaded tokio runtime for this worker thread
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("Failed to create tokio runtime");

            // Run the async worker loop on the runtime
            rt.block_on(worker_loop(provider_result, request_rx, response_tx));
        }));

        // Restore the previous panic hook
        panic::set_hook(prev_hook);

        if let Err(e) = result {
            // Extract panic message for logging
            let panic_msg = if let Some(s) = e.downcast_ref::<&str>() {
                s.to_string()
            } else if let Some(s) = e.downcast_ref::<String>() {
                s.clone()
            } else {
                "Unknown panic".to_string()
            };
            log::error!("AI worker thread panicked: {}", panic_msg);
        }
    });
}

/// Main async worker loop - processes requests until the channel is closed
///
/// Uses blocking `recv()` on the request channel (fine in dedicated thread)
/// and processes each query with the async handler.
///
/// # Requirements
/// - 4.2: WHEN the worker thread is spawned THEN it SHALL create a tokio runtime
///   for async operations
async fn worker_loop(
    provider_result: Result<AsyncAiProvider, AiError>,
    request_rx: Receiver<AiRequest>,
    response_tx: Sender<AiResponse>,
) {
    // Check if provider was created successfully
    let provider = match provider_result {
        Ok(p) => Some(p),
        Err(_e) => None,
    };

    // Process requests until the channel is closed
    // Using blocking recv() is fine here since we're in a dedicated thread
    while let Ok(request) = request_rx.recv() {
        match request {
            AiRequest::Query {
                prompt,
                request_id,
                cancel_token,
            } => {
                handle_query_async(&provider, &prompt, request_id, cancel_token, &response_tx)
                    .await;
            }
        }
    }
}

/// Handle a query request asynchronously
///
/// Uses `tokio::select!` with biased mode to check cancellation first,
/// then processes the async stream from the AI provider.
///
/// # Requirements
/// - 1.2: WHEN a cancel signal is received THEN the Worker_Thread SHALL abort
///   the HTTP request immediately
/// - 3.2: WHEN a request is cancelled THEN the system SHALL send AiResponse::Cancelled
async fn handle_query_async(
    provider: &Option<AsyncAiProvider>,
    prompt: &str,
    request_id: u64,
    cancel_token: CancellationToken,
    response_tx: &Sender<AiResponse>,
) {
    // Check if already cancelled before starting
    if cancel_token.is_cancelled() {
        let _ = response_tx.send(AiResponse::Cancelled { request_id });
        return;
    }

    // Check if provider is available
    let provider = match provider {
        Some(p) => p,
        None => {
            let _ = response_tx.send(AiResponse::Error(
                "AI not configured. Enable AI in your config file with 'enabled = true' and configure a provider. See https://github.com/bellicose100xp/jiq#configuration for setup instructions.".to_string(),
            ));
            return;
        }
    };

    // Stream the response with cancellation support
    // The async provider handles cancellation internally via tokio::select!
    match provider
        .stream_with_cancel(prompt, request_id, cancel_token, response_tx.clone())
        .await
    {
        Ok(()) => {
            // Stream completed successfully
            let _ = response_tx.send(AiResponse::Complete { request_id });
        }
        Err(AiError::Cancelled) => {
            // Request was cancelled - send Cancelled response
            let _ = response_tx.send(AiResponse::Cancelled { request_id });
        }
        Err(e) => {
            let _ = response_tx.send(AiResponse::Error(e.to_string()));
        }
    }
}

#[cfg(test)]
#[path = "worker_tests.rs"]
mod worker_tests;
