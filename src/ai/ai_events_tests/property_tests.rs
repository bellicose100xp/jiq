//! Property-based tests for AI event handling

use super::*;

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
        let mut ai_state = AiState::new(true, 1000);
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
        let mut ai_state = AiState::new(true, 1000);
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
        let mut ai_state = AiState::new(true, 1000);
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
