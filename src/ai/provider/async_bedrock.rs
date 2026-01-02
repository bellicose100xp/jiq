//! Async AWS Bedrock client
//!
//! Implements async streaming for the AWS Bedrock Converse API with cancellation support.
//! Uses AWS SDK for Rust with tokio for async runtime.

use std::panic::AssertUnwindSafe;
use std::sync::mpsc::Sender;

use aws_config::BehaviorVersion;
use aws_sdk_bedrockruntime::Client as BedrockRuntimeClient;
use aws_sdk_bedrockruntime::types::{ContentBlock, ConversationRole, Message};
use futures::FutureExt;
use tokio_util::sync::CancellationToken;

use super::AiError;
use crate::ai::ai_state::AiResponse;

/// Async AWS Bedrock client with streaming support
///
/// Uses AWS SDK for async requests with streaming support.
/// Supports cancellation via CancellationToken.
#[derive(Debug, Clone)]
pub struct AsyncBedrockClient {
    region: String,
    model: String,
    profile: Option<String>,
}

impl AsyncBedrockClient {
    /// Create a new async Bedrock client
    ///
    /// # Arguments
    /// * `region` - AWS region for Bedrock API calls
    /// * `model` - Bedrock model ID (e.g., "anthropic.claude-3-haiku-20240307-v1:0")
    /// * `profile` - Optional AWS profile name (None = use default credential chain)
    pub fn new(region: String, model: String, profile: Option<String>) -> Self {
        Self {
            region,
            model,
            profile,
        }
    }

    /// Build the AWS Bedrock client based on configuration
    ///
    /// Uses named profile credentials if profile is Some,
    /// otherwise uses the default credential chain.
    /// Catches panics from the AWS SDK to prevent TUI corruption.
    async fn build_client(&self) -> Result<BedrockRuntimeClient, AiError> {
        let region = aws_config::Region::new(self.region.clone());
        let profile = self.profile.clone();

        // Wrap the AWS SDK config loading in catch_unwind to prevent panics
        // from corrupting the TUI. The AWS SDK can panic in certain credential
        // loading scenarios (e.g., web identity token issues).
        let config_result = AssertUnwindSafe(async {
            match &profile {
                Some(profile_name) => {
                    // Use named profile credentials
                    aws_config::defaults(BehaviorVersion::latest())
                        .profile_name(profile_name)
                        .region(region)
                        .load()
                        .await
                }
                None => {
                    // Use default credential chain
                    aws_config::defaults(BehaviorVersion::latest())
                        .region(region)
                        .load()
                        .await
                }
            }
        })
        .catch_unwind()
        .await;

        match config_result {
            Ok(config) => Ok(BedrockRuntimeClient::new(&config)),
            Err(panic_info) => {
                // Extract panic message if possible
                let panic_msg = if let Some(s) = panic_info.downcast_ref::<&str>() {
                    s.to_string()
                } else if let Some(s) = panic_info.downcast_ref::<String>() {
                    s.clone()
                } else {
                    "Unknown panic during AWS SDK initialization".to_string()
                };
                Err(AiError::AwsSdk(format!(
                    "AWS SDK initialization failed: {}",
                    panic_msg
                )))
            }
        }
    }

    /// Stream a response from the Bedrock API with cancellation support
    ///
    /// Uses `tokio::select!` to race the stream against the cancellation token.
    /// Sends chunks via the response channel as they arrive.
    ///
    /// # Arguments
    /// * `prompt` - The prompt to send to the API
    /// * `request_id` - Unique ID for this request
    /// * `cancel_token` - Token to cancel the request
    /// * `response_tx` - Channel to send response chunks
    ///
    /// # Returns
    /// * `Ok(())` - Stream completed successfully
    /// * `Err(AiError::Cancelled)` - Request was cancelled
    /// * `Err(AiError::*)` - Other errors
    pub async fn stream_with_cancel(
        &self,
        prompt: &str,
        request_id: u64,
        cancel_token: CancellationToken,
        response_tx: Sender<AiResponse>,
    ) -> Result<(), AiError> {
        // Check if already cancelled before starting
        if cancel_token.is_cancelled() {
            return Err(AiError::Cancelled);
        }

        // Build the client
        let client = self.build_client().await?;

        // Create the message for the conversation
        let message = Message::builder()
            .role(ConversationRole::User)
            .content(ContentBlock::Text(prompt.to_string()))
            .build()
            .map_err(|e| AiError::AwsSdk(format!("Failed to build message: {}", e)))?;

        // Start the streaming conversation
        // Note: For inference profile ARNs, the region in the ARN should match the client region
        let mut stream_output = client
            .converse_stream()
            .model_id(&self.model)
            .messages(message)
            .send()
            .await
            .map_err(|e| {
                let err_msg = e.to_string();

                // Provide more detailed error messages
                if err_msg.contains("credentials")
                    || err_msg.contains("Credentials")
                    || err_msg.contains("authentication")
                {
                    AiError::NotConfigured {
                        provider: "Bedrock".to_string(),
                        message: format!("AWS credentials error: {}", err_msg),
                    }
                } else if err_msg.contains("network")
                    || err_msg.contains("connection")
                    || err_msg.contains("timeout")
                {
                    AiError::Network {
                        provider: "Bedrock".to_string(),
                        message: err_msg,
                    }
                } else if err_msg.contains("ValidationException") || err_msg.contains("validation") {
                    AiError::NotConfigured {
                        provider: "Bedrock".to_string(),
                        message: format!("Invalid configuration: {}. Check that model ID and region are correct.", err_msg),
                    }
                } else if err_msg.contains("ResourceNotFoundException") || err_msg.contains("not found") {
                    AiError::NotConfigured {
                        provider: "Bedrock".to_string(),
                        message: format!("Model not found: {}. Verify model access is enabled in your AWS account.", err_msg),
                    }
                } else {
                    // Include full error for debugging
                    AiError::AwsSdk(format!("Bedrock API error: {}", err_msg))
                }
            })?;

        // Process stream with cancellation support
        loop {
            tokio::select! {
                biased;

                // Check cancellation first (biased mode)
                _ = cancel_token.cancelled() => {
                    return Err(AiError::Cancelled);
                }

                // Process next event from stream
                event_result = stream_output.stream.recv() => {
                    match event_result {
                        Ok(Some(event)) => {
                            // Extract text from ContentBlockDelta events
                            if let Some(text) = Self::extract_text_from_event(&event)
                                && !text.is_empty()
                                && response_tx
                                    .send(AiResponse::Chunk {
                                        text,
                                        request_id,
                                    })
                                    .is_err()
                            {
                                // Main thread disconnected
                                return Ok(());
                            }
                        }
                        Ok(None) => {
                            // Stream ended
                            break;
                        }
                        Err(e) => {
                            let err_msg = e.to_string();
                            // Map to appropriate error type
                            if err_msg.contains("throttl") || err_msg.contains("rate") {
                                return Err(AiError::Api {
                                    provider: "Bedrock".to_string(),
                                    code: 429,
                                    message: err_msg,
                                });
                            } else if err_msg.contains("access")
                                || err_msg.contains("permission")
                                || err_msg.contains("denied")
                            {
                                return Err(AiError::Api {
                                    provider: "Bedrock".to_string(),
                                    code: 403,
                                    message: err_msg,
                                });
                            } else {
                                return Err(AiError::AwsSdk(err_msg));
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Extract text content from a Bedrock stream event
    fn extract_text_from_event(
        event: &aws_sdk_bedrockruntime::types::ConverseStreamOutput,
    ) -> Option<String> {
        use aws_sdk_bedrockruntime::types::ConverseStreamOutput;

        match event {
            ConverseStreamOutput::ContentBlockDelta(delta) => {
                if let Some(content_delta) = delta.delta() {
                    use aws_sdk_bedrockruntime::types::ContentBlockDelta;
                    match content_delta {
                        ContentBlockDelta::Text(text) => Some(text.clone()),
                        _ => None,
                    }
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod async_bedrock_tests {
    use super::*;

    #[test]
    fn test_new_creates_client_with_fields() {
        let client = AsyncBedrockClient::new(
            "us-east-1".to_string(),
            "anthropic.claude-3-haiku-20240307-v1:0".to_string(),
            Some("my-profile".to_string()),
        );

        assert_eq!(client.region, "us-east-1");
        assert_eq!(client.model, "anthropic.claude-3-haiku-20240307-v1:0");
        assert_eq!(client.profile, Some("my-profile".to_string()));
    }

    #[test]
    fn test_new_without_profile() {
        let client = AsyncBedrockClient::new(
            "us-west-2".to_string(),
            "amazon.titan-text-express-v1".to_string(),
            None,
        );

        assert_eq!(client.region, "us-west-2");
        assert_eq!(client.model, "amazon.titan-text-express-v1");
        assert_eq!(client.profile, None);
    }
}
