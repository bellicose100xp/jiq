// AI configuration type definitions

use serde::Deserialize;

// Model is now required - no default provided

/// Default max tokens for AI responses (kept short to fit in non-scrollable window)
fn default_max_tokens() -> u32 {
    512
}

/// AI provider selection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AiProviderType {
    Anthropic,
    Bedrock,
    Openai,
    Gemini,
}

/// Anthropic-specific configuration
#[derive(Debug, Clone, Deserialize)]
pub struct AnthropicConfig {
    /// API key for Anthropic (required when AI is enabled)
    pub api_key: Option<String>,
    /// Model to use (required - user must specify)
    pub model: Option<String>,
    /// Maximum tokens in response
    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,
}

impl Default for AnthropicConfig {
    fn default() -> Self {
        AnthropicConfig {
            api_key: None,
            model: None,
            max_tokens: default_max_tokens(),
        }
    }
}

/// Bedrock provider configuration
#[derive(Debug, Clone, Deserialize, Default)]
pub struct BedrockConfig {
    /// AWS region for Bedrock API calls (required)
    pub region: Option<String>,
    /// Bedrock model ID (required, e.g., "anthropic.claude-3-haiku-20240307-v1:0")
    pub model: Option<String>,
    /// AWS profile name (optional - if not specified, uses default credential chain)
    pub profile: Option<String>,
}

/// OpenAI-specific configuration
#[derive(Debug, Clone, Deserialize, Default)]
pub struct OpenAiConfig {
    /// API key for OpenAI (required when AI is enabled with OpenAI provider)
    pub api_key: Option<String>,
    /// Model to use (required, e.g., "gpt-4o-mini")
    pub model: Option<String>,
}

/// Gemini-specific configuration
#[derive(Debug, Clone, Deserialize, Default)]
pub struct GeminiConfig {
    /// API key for Gemini (required when AI is enabled with Gemini provider)
    pub api_key: Option<String>,
    /// Model to use (required, e.g., "gemini-2.0-flash")
    pub model: Option<String>,
}

/// AI assistant configuration section
#[derive(Debug, Clone, Default, Deserialize)]
pub struct AiConfig {
    /// Whether AI features are enabled
    #[serde(default)]
    pub enabled: bool,
    /// Which AI provider to use (None when not configured)
    #[serde(default)]
    pub provider: Option<AiProviderType>,
    /// Anthropic-specific configuration
    #[serde(default)]
    pub anthropic: AnthropicConfig,
    /// Bedrock-specific configuration
    #[serde(default)]
    pub bedrock: BedrockConfig,
    /// OpenAI-specific configuration
    #[serde(default)]
    pub openai: OpenAiConfig,
    /// Gemini-specific configuration
    #[serde(default)]
    pub gemini: GeminiConfig,
}

#[cfg(test)]
#[path = "ai_types_tests.rs"]
mod ai_types_tests;
