//! Property-based tests for AI event handling

use super::*;

// **Feature: ai-request-cancellation, Property 4: Stale responses are discarded**
// *For any* response with a request_id less than the current request_id,
// the response should be ignored and not affect the AiState.
// **Validates: Requirements 2.1, 2.3**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop_stale_chunk_responses_discarded(
        stale_text in "[a-zA-Z0-9 ]{1,50}",
        current_text in "[a-zA-Z0-9 ]{1,50}",
        num_requests in 2u64..10
    ) {
        let mut ai_state = AiState::new(true);
        let (tx, rx) = mpsc::channel();
        ai_state.response_rx = Some(rx);

        // Simulate multiple requests to increment request_id
        for _ in 0..num_requests {
            ai_state.start_request();
        }
        let current_request_id = ai_state.current_request_id();
        let stale_request_id = current_request_id.saturating_sub(1);

        // Send a stale chunk (from an old request)
        tx.send(AiResponse::Chunk {
            text: stale_text.clone(),
            request_id: stale_request_id,
        })
        .unwrap();

        // Poll to process
        poll_response_channel(&mut ai_state);

        // Stale chunk should be ignored - response should be empty
        prop_assert!(
            ai_state.response.is_empty(),
            "Stale chunk should be discarded, but response was: '{}'",
            ai_state.response
        );

        // Now send a current chunk
        tx.send(AiResponse::Chunk {
            text: current_text.clone(),
            request_id: current_request_id,
        })
        .unwrap();

        poll_response_channel(&mut ai_state);

        // Current chunk should be processed
        prop_assert_eq!(
            ai_state.response, current_text,
            "Current chunk should be processed"
        );
    }

    #[test]
    fn prop_stale_complete_responses_discarded(
        chunk_text in "[a-zA-Z0-9 ]{1,50}",
        num_requests in 2u64..10
    ) {
        let mut ai_state = AiState::new(true);
        let (tx, rx) = mpsc::channel();
        ai_state.response_rx = Some(rx);

        // Simulate multiple requests to increment request_id
        for _ in 0..num_requests {
            ai_state.start_request();
        }
        let current_request_id = ai_state.current_request_id();
        let stale_request_id = current_request_id.saturating_sub(1);

        // Send a current chunk first
        tx.send(AiResponse::Chunk {
            text: chunk_text.clone(),
            request_id: current_request_id,
        })
        .unwrap();

        poll_response_channel(&mut ai_state);

        // Loading should still be true (no Complete yet)
        prop_assert!(ai_state.loading, "Loading should be true before Complete");

        // Send a stale Complete (from an old request)
        tx.send(AiResponse::Complete {
            request_id: stale_request_id,
        })
        .unwrap();

        poll_response_channel(&mut ai_state);

        // Stale Complete should be ignored - loading should still be true
        prop_assert!(
            ai_state.loading,
            "Stale Complete should be discarded, loading should still be true"
        );

        // Now send a current Complete
        tx.send(AiResponse::Complete {
            request_id: current_request_id,
        })
        .unwrap();

        poll_response_channel(&mut ai_state);

        // Current Complete should be processed
        prop_assert!(
            !ai_state.loading,
            "Current Complete should be processed, loading should be false"
        );
    }

    #[test]
    fn prop_current_responses_not_discarded(
        chunks in prop::collection::vec("[a-zA-Z0-9 ]{1,20}", 1..5)
    ) {
        let mut ai_state = AiState::new(true);
        let (tx, rx) = mpsc::channel();
        ai_state.response_rx = Some(rx);

        ai_state.start_request();
        let request_id = ai_state.current_request_id();

        // Send all chunks with current request_id
        let expected: String = chunks.iter().cloned().collect();
        for chunk in &chunks {
            tx.send(AiResponse::Chunk {
                text: chunk.clone(),
                request_id,
            })
            .unwrap();
        }

        poll_response_channel(&mut ai_state);

        // All chunks should be processed
        prop_assert_eq!(
            &ai_state.response, &expected,
            "All current chunks should be processed"
        );

        // Send Complete
        tx.send(AiResponse::Complete { request_id }).unwrap();
        poll_response_channel(&mut ai_state);

        prop_assert!(
            !ai_state.loading,
            "Current Complete should be processed"
        );
    }
}

// **Feature: ai-assistant, Property 10: Streaming concatenation**
// *For any* sequence of response chunks [c1, c2, ..., cn], the final displayed
// response should equal c1 + c2 + ... + cn.
// **Validates: Requirements 4.1**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop_streaming_concatenation(
        chunks in prop::collection::vec("[a-zA-Z0-9 .,!?]{0,50}", 0..10)
    ) {
        let mut ai_state = AiState::new(true);
        let (tx, rx) = mpsc::channel();
        ai_state.response_rx = Some(rx);
        ai_state.start_request();
        let request_id = ai_state.current_request_id();

        // Calculate expected concatenation
        let expected: String = chunks.iter().cloned().collect();

        // Send all chunks with matching request_id
        for chunk in &chunks {
            tx.send(AiResponse::Chunk {
                text: chunk.clone(),
                request_id,
            })
            .unwrap();
        }

        // Poll to process all chunks
        poll_response_channel(&mut ai_state);

        prop_assert_eq!(
            ai_state.response, expected,
            "Response should be concatenation of all chunks"
        );
    }
}

// **Feature: ai-assistant, Property 11: Loading state during request**
// *For any* AiState that has sent a request and not received Complete or Error,
// `loading` should be true.
// **Validates: Requirements 4.2**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop_loading_state_during_request(
        num_chunks in 1usize..10,
        chunk_content in "[a-zA-Z0-9 ]{1,20}"
    ) {
        let mut ai_state = AiState::new(true);
        let (tx, rx) = mpsc::channel();
        ai_state.response_rx = Some(rx);

        // Start a request (sets loading = true)
        ai_state.start_request();
        let request_id = ai_state.current_request_id();
        prop_assert!(ai_state.loading, "Loading should be true after start_request");

        // Send chunks but NOT Complete or Error
        for _ in 0..num_chunks {
            tx.send(AiResponse::Chunk {
                text: chunk_content.clone(),
                request_id,
            })
            .unwrap();
        }

        // Poll to process chunks
        poll_response_channel(&mut ai_state);

        // Loading should still be true (no Complete/Error received)
        prop_assert!(
            ai_state.loading,
            "Loading should remain true until Complete or Error is received"
        );

        // Now send Complete
        tx.send(AiResponse::Complete { request_id }).unwrap();
        poll_response_channel(&mut ai_state);

        // Loading should now be false
        prop_assert!(
            !ai_state.loading,
            "Loading should be false after Complete is received"
        );
    }

    #[test]
    fn prop_loading_state_cleared_on_error(
        error_msg in "[a-zA-Z0-9 ]{1,50}"
    ) {
        let mut ai_state = AiState::new(true);
        let (tx, rx) = mpsc::channel();
        ai_state.response_rx = Some(rx);

        // Start a request
        ai_state.start_request();
        prop_assert!(ai_state.loading, "Loading should be true after start_request");

        // Send Error
        tx.send(AiResponse::Error(error_msg.clone())).unwrap();
        poll_response_channel(&mut ai_state);

        // Loading should be false after error
        prop_assert!(
            !ai_state.loading,
            "Loading should be false after Error is received"
        );
        prop_assert_eq!(
            ai_state.error,
            Some(error_msg),
            "Error message should be set"
        );
    }
}
