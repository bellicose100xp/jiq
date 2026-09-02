// AI configuration type definitions

use serde::Deserialize;

/// Test constant for max context length (matches default)
#[cfg(test)]
pub const TEST_MAX_CONTEXT_LENGTH: u32 = 100_000;

// Model is now required - no default provided

/// Default max tokens for AI responses (kept short to fit in non-scrollable window)
fn default_max_tokens() -> u32 {
    512
}

/// Default max context length for JSON samples sent to AI (100KB of characters)
fn default_max_context_length() -> u32 {
    100_000
}

/// Default AI request timeout in seconds (0 = no timeout)
fn default_request_timeout_secs() -> u64 {
    120
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

/// Reasoning effort level for models that support it.
///
/// Maps to Claude's `output_config.effort` on Bedrock and to the OpenAI
/// Chat Completions `reasoning_effort` field on the OpenAI-compatible endpoint.
/// Not every level is valid for every model; unsupported values are rejected
/// by the provider API.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AiEffort {
    Minimal,
    Low,
    Medium,
    High,
    Xhigh,
    Max,
}

impl AiEffort {
    /// The wire value expected by the provider APIs.
    pub fn as_str(self) -> &'static str {
        match self {
            AiEffort::Minimal => "minimal",
            AiEffort::Low => "low",
            AiEffort::Medium => "medium",
            AiEffort::High => "high",
            AiEffort::Xhigh => "xhigh",
            AiEffort::Max => "max",
        }
    }
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
    /// Reasoning effort for Claude 4.6+ models (None = model default)
    pub effort: Option<AiEffort>,
    /// Enable the 1M-token context window beta (Claude Sonnet 4 / 4.5).
    /// Also raise `max_context_length` to send larger samples, or this has no effect.
    #[serde(default)]
    pub context_1m: bool,
}

impl Default for AnthropicConfig {
    fn default() -> Self {
        AnthropicConfig {
            api_key: None,
            model: None,
            max_tokens: default_max_tokens(),
            effort: None,
            context_1m: false,
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
    /// Reasoning effort for Claude Sonnet/Opus 4.6+ (None = model default)
    pub effort: Option<AiEffort>,
    /// Enable the 1M-token context window beta (Claude Sonnet 4 / 4.5).
    /// Also raise `max_context_length` to send larger samples, or this has no effect.
    #[serde(default)]
    pub context_1m: bool,
}

/// OpenAI-specific configuration
#[derive(Debug, Clone, Deserialize, Default)]
pub struct OpenAiConfig {
    /// API key for OpenAI (required when AI is enabled with OpenAI provider)
    pub api_key: Option<String>,
    /// Model to use (required, e.g., "gpt-4o-mini")
    pub model: Option<String>,
    /// Base URL for OpenAI-compatible API (optional, defaults to api.openai.com)
    pub base_url: Option<String>,
    /// Reasoning effort for models that support it, e.g. GPT-5.x (None = model default)
    pub effort: Option<AiEffort>,
}

/// Gemini-specific configuration
#[derive(Debug, Clone, Deserialize, Default)]
pub struct GeminiConfig {
    /// API key for Gemini (required when AI is enabled with Gemini provider)
    pub api_key: Option<String>,
    /// Model to use (required, e.g., "gemini-2.0-flash")
    pub model: Option<String>,
    /// Reasoning effort, mapped to the Gemini 3 thinking level (None = model default).
    /// `xhigh`/`max` clamp to Gemini's highest level.
    pub effort: Option<AiEffort>,
}

/// AI assistant configuration section
#[derive(Debug, Clone, Deserialize)]
pub struct AiConfig {
    /// Whether AI features are enabled
    #[serde(default)]
    pub enabled: bool,
    /// Which AI provider to use (None when not configured)
    #[serde(default)]
    pub provider: Option<AiProviderType>,
    /// Maximum character length for JSON context samples sent to AI
    #[serde(default = "default_max_context_length")]
    pub max_context_length: u32,
    /// Extra instructions appended to the built-in prompt (e.g. style
    /// preferences). Never replaces the built-in prompt: the output-format
    /// contract the suggestion parser depends on always takes precedence.
    #[serde(default)]
    pub extra_instructions: Option<String>,
    /// Whole-request timeout for AI API calls in seconds (0 disables it)
    #[serde(default = "default_request_timeout_secs")]
    pub request_timeout_secs: u64,
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

// Manual impl so `AiConfig::default()` matches the serde field defaults
// (a derived impl would zero max_context_length and request_timeout_secs).
impl Default for AiConfig {
    fn default() -> Self {
        AiConfig {
            enabled: false,
            provider: None,
            max_context_length: default_max_context_length(),
            extra_instructions: None,
            request_timeout_secs: default_request_timeout_secs(),
            anthropic: AnthropicConfig::default(),
            bedrock: BedrockConfig::default(),
            openai: OpenAiConfig::default(),
            gemini: GeminiConfig::default(),
        }
    }
}

#[cfg(test)]
#[path = "ai_types_tests.rs"]
mod ai_types_tests;
