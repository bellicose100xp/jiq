use super::*;

use std::sync::mpsc::Receiver;

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

    let ok = state.send_request("explain .foo".to_string());

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
            assert_eq!(prompt, "explain .foo");
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

    let ok = state.send_request("x".to_string());

    assert!(
        !ok,
        "send_request must return false when the channel is closed"
    );
    assert!(
        state.current_cancel_token.is_none(),
        "a failed send must clear current_cancel_token so a later cancel can't act on a dead request"
    );
}
