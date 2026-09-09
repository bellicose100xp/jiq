use super::*;

use std::sync::mpsc::Receiver;

use crate::ai::chat::ChatRole;

/// Build an `AiState` wired to a live request channel, returning the receiver
/// the worker thread would normally hold so a test can inspect what was sent.
fn state_with_channel() -> (AiState, Receiver<AiRequest>) {
    let mut state = AiState::new(true);
    let (req_tx, req_rx) = std::sync::mpsc::channel::<AiRequest>();
    let (_resp_tx, resp_rx) = std::sync::mpsc::channel::<AiResponse>();
    state.set_channels(req_tx, resp_rx);
    (state, req_rx)
}

#[test]
fn test_send_request_with_channel_delivers_query_and_returns_true() {
    let (mut state, req_rx) = state_with_channel();

    let ok = state.send_request(AiPrompt::single("system text", "explain .foo"));

    assert!(ok, "send_request should succeed with a live channel");
    assert!(
        state.current_cancel_token.is_some(),
        "a cancel token must be retained so the request can be aborted later"
    );
    assert_eq!(
        state.request_id, 1,
        "send_request increments request_id via start_request"
    );

    let req = req_rx
        .recv()
        .expect("the request should be delivered on the channel");
    match req {
        AiRequest::Query {
            prompt, request_id, ..
        } => {
            assert_eq!(prompt.system, "system text");
            assert_eq!(prompt.messages.len(), 1);
            assert_eq!(prompt.messages[0].role, ChatRole::User);
            assert_eq!(prompt.messages[0].content, "explain .foo");
            assert_eq!(
                request_id, 1,
                "the delivered request_id must match the incremented state request_id"
            );
        }
    }
}

#[test]
fn test_send_request_send_failure_clears_cancel_token_and_returns_false() {
    let (mut state, req_rx) = state_with_channel();

    // Drop the receiver so the worker thread is "gone" and tx.send returns Err.
    drop(req_rx);

    let ok = state.send_request(AiPrompt::single("s", "x"));

    assert!(
        !ok,
        "send_request must return false when the channel is closed"
    );
    assert!(
        state.current_cancel_token.is_none(),
        "a failed send must clear current_cancel_token so a later cancel can't act on a dead request"
    );
}

#[test]
fn test_send_request_without_channel_returns_false_and_leaves_state_idle() {
    let mut state = AiState::new(true);

    let ok = state.send_request(AiPrompt::single("s", "x"));

    assert!(!ok);
    assert!(
        !state.loading,
        "no request started when there is no channel"
    );
    assert_eq!(state.request_id, 0);
}

#[test]
fn test_send_request_cancels_previous_in_flight_request() {
    let (mut state, req_rx) = state_with_channel();

    assert!(state.send_request(AiPrompt::single("s", "first")));
    let first_token = state
        .current_cancel_token
        .clone()
        .expect("first request keeps a token");

    assert!(state.send_request(AiPrompt::single("s", "second")));

    assert!(
        first_token.is_cancelled(),
        "sending a new request aborts the previous one"
    );
    assert_eq!(state.request_id, 2);
    let ids: Vec<u64> = req_rx
        .try_iter()
        .map(|r| match r {
            AiRequest::Query { request_id, .. } => request_id,
        })
        .collect();
    assert_eq!(ids, vec![1, 2]);
}
