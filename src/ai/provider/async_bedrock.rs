//! Async AWS Bedrock client
//!
//! Implements async streaming for the AWS Bedrock Converse API with cancellation support.
//! Uses AWS SDK for Rust with tokio for async runtime.

use std::panic::AssertUnwindSafe;
use std::sync::mpsc::Sender;

use std::collections::HashMap;

use aws_config::BehaviorVersion;
use aws_sdk_bedrockruntime::Client as BedrockRuntimeClient;
use aws_sdk_bedrockruntime::types::{ContentBlock, ConversationRole, Message, SystemContentBlock};
use aws_smithy_types::Document;
use futures::FutureExt;
use tokio_util::sync::CancellationToken;

use super::AiError;
use crate::ai::ai_state::AiResponse;
use crate::ai::chat::{AiPrompt, ChatRole};
use crate::config::ai_types::AiEffort;

/// Anthropic beta flag that opts a request into the 1M-token context window.
const CONTEXT_1M_BETA: &str = "context-1m-2025-08-07";

/// Map a conversation role onto the Converse API's role enum.
fn conversation_role(role: ChatRole) -> ConversationRole {
    match role {
        ChatRole::User => ConversationRole::User,
        ChatRole::Assistant => ConversationRole::Assistant,
    }
}

/// Translate an [`AiPrompt`] into Converse API inputs: the optional system
/// block (None when the system prompt is empty) and one `Message` per turn.
fn build_conversation(
    prompt: &AiPrompt,
) -> Result<(Option<SystemContentBlock>, Vec<Message>), AiError> {
    let system =
        (!prompt.system.is_empty()).then(|| SystemContentBlock::Text(prompt.system.clone()));

    let messages = prompt
        .messages
        .iter()
        .map(|message| {
            Message::builder()
                .role(conversation_role(message.role))
                .content(ContentBlock::Text(message.content.clone()))
                .build()
                .map_err(|e| AiError::AwsSdk(format!("Failed to build message: {}", e)))
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok((system, messages))
}

/// Async AWS Bedrock client with streaming support
///
/// Uses AWS SDK for async requests with streaming support.
/// Supports cancellation via CancellationToken.
#[derive(Debug, Clone)]
pub struct AsyncBedrockClient {
    region: String,
    model: String,
    profile: Option<String>,
    effort: Option<AiEffort>,
    context_1m: bool,
    timeout: Option<std::time::Duration>,
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
            effort: None,
            context_1m: false,
            timeout: None,
        }
    }

    /// Apply a whole-operation timeout (from `[ai] request_timeout_secs`).
    /// None leaves the AWS SDK defaults.
    pub fn with_timeout(mut self, timeout: Option<std::time::Duration>) -> Self {
        self.timeout = timeout;
        self
    }

    /// Set the reasoning effort (Claude Sonnet/Opus 4.6+). None leaves the model default.
    pub fn with_effort(mut self, effort: Option<AiEffort>) -> Self {
        self.effort = effort;
        self
    }

    /// Enable the 1M-token context window beta (Claude Sonnet 4 / 4.5).
    pub fn with_context_1m(mut self, context_1m: bool) -> Self {
        self.context_1m = context_1m;
        self
    }

    /// Build the `additionalModelRequestFields` document for reasoning effort and
    /// the 1M-context beta, or None when neither is configured.
    ///
    /// The effort field shape depends on the model family, since Converse passes
    /// these through to the model verbatim and each family rejects the other's:
    /// - OpenAI models (`openai.` in the model ID): `{"reasoning_effort": "<level>"}`
    /// - Claude and everything else: adaptive thinking, i.e.
    ///   `{"thinking": {"type": "adaptive"}, "output_config": {"effort": "<level>"}}`
    ///
    /// The 1M-context beta is opted in via `{"anthropic_beta": ["context-1m-2025-08-07"]}`.
    fn build_additional_fields(&self) -> Option<Document> {
        let mut fields: HashMap<String, Document> = HashMap::new();

        if let Some(effort) = self.effort {
            if self.model.contains("openai.") {
                fields.insert(
                    "reasoning_effort".to_string(),
                    Document::String(effort.as_str().to_string()),
                );
            } else {
                fields.insert(
                    "thinking".to_string(),
                    Document::Object(HashMap::from([(
                        "type".to_string(),
                        Document::String("adaptive".to_string()),
                    )])),
                );
                fields.insert(
                    "output_config".to_string(),
                    Document::Object(HashMap::from([(
                        "effort".to_string(),
                        Document::String(effort.as_str().to_string()),
                    )])),
                );
            }
        }

        if self.context_1m {
            fields.insert(
                "anthropic_beta".to_string(),
                Document::Array(vec![Document::String(CONTEXT_1M_BETA.to_string())]),
            );
        }

        if fields.is_empty() {
            None
        } else {
            Some(Document::Object(fields))
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
        let timeout = self.timeout;

        // Wrap the AWS SDK config loading in catch_unwind to prevent panics
        // from corrupting the TUI. The AWS SDK can panic in certain credential
        // loading scenarios (e.g., web identity token issues).
        let config_result = AssertUnwindSafe(async {
            // Named profile credentials when set, otherwise the default chain
            let mut loader = aws_config::defaults(BehaviorVersion::latest()).region(region);
            if let Some(profile_name) = &profile {
                loader = loader.profile_name(profile_name);
            }
            if let Some(duration) = timeout {
                loader = loader.timeout_config(
                    aws_config::timeout::TimeoutConfig::builder()
                        .operation_timeout(duration)
                        .build(),
                );
            }
            loader.load().await
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
    /// * `prompt` - System prompt plus conversation turns to send to the API
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
        prompt: &AiPrompt,
        request_id: u64,
        cancel_token: CancellationToken,
        response_tx: Sender<AiResponse>,
    ) -> Result<(), AiError> {
        // Check if already cancelled before starting
        if cancel_token.is_cancelled() {
            return Err(AiError::Cancelled);
        }

        let (system, messages) = build_conversation(prompt)?;

        // Build the client
        let client = self.build_client().await?;

        // Start the streaming conversation
        // Note: For inference profile ARNs, the region in the ARN should match the client region
        let mut request = client
            .converse_stream()
            .model_id(&self.model)
            .set_messages(Some(messages));

        if let Some(system) = system {
            request = request.system(system);
        }

        // Attach reasoning effort and/or the 1M-context beta when configured
        if let Some(fields) = self.build_additional_fields() {
            request = request.additional_model_request_fields(fields);
        }

        let mut stream_output = request.send().await.map_err(|e| {
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
                    message: format!(
                        "Invalid configuration: {}. Check that model ID and region are correct.",
                        err_msg
                    ),
                }
            } else if err_msg.contains("ResourceNotFoundException") || err_msg.contains("not found")
            {
                AiError::NotConfigured {
                    provider: "Bedrock".to_string(),
                    message: format!(
                        "Model not found: {}. Verify model access is enabled in your AWS account.",
                        err_msg
                    ),
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
#[path = "async_bedrock_tests.rs"]
mod async_bedrock_tests;
