// AI configuration type definitions

use serde::Deserialize;

/// Default debounce delay in milliseconds
fn default_debounce_ms() -> u64 {
    1000
}

/// Default auto-show on error setting
fn default_auto_show_on_error() -> bool {
    true
}

// Model is now required - no default provided

/// Default max tokens for AI responses (kept short to fit in non-scrollable window)
fn default_max_tokens() -> u32 {
    512
}

/// AI provider selection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum AiProviderType {
    #[default]
    Anthropic,
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

/// AI assistant configuration section
#[derive(Debug, Clone, Deserialize)]
pub struct AiConfig {
    /// Whether AI features are enabled
    #[serde(default)]
    pub enabled: bool,
    /// Which AI provider to use
    #[serde(default)]
    pub provider: AiProviderType,
    /// Debounce delay in milliseconds before making API requests
    #[serde(default = "default_debounce_ms")]
    pub debounce_ms: u64,
    /// Whether to automatically show AI popup on query errors
    #[serde(default = "default_auto_show_on_error")]
    pub auto_show_on_error: bool,
    /// Anthropic-specific configuration
    #[serde(default)]
    pub anthropic: AnthropicConfig,
}

impl Default for AiConfig {
    fn default() -> Self {
        AiConfig {
            enabled: false,
            provider: AiProviderType::default(),
            debounce_ms: default_debounce_ms(),
            auto_show_on_error: default_auto_show_on_error(),
            anthropic: AnthropicConfig::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use proptest::prelude::*;

    // **Feature: ai-assistant, Property 1: Valid config parsing**
    // *For any* valid TOML config with `[ai]` section containing valid fields,
    // parsing should succeed and produce an AiConfig with the specified values.
    // **Validates: Requirements 1.1**
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn prop_valid_ai_config_parsing(
            enabled in prop::bool::ANY,
            debounce_ms in 100u64..5000u64,
            auto_show_on_error in prop::bool::ANY,
            max_tokens in 256u32..4096u32,
        ) {
            let toml_content = format!(r#"
[ai]
enabled = {}
provider = "anthropic"
debounce_ms = {}
auto_show_on_error = {}

[ai.anthropic]
api_key = "sk-ant-test-key"
model = "claude-3-haiku-20240307"
max_tokens = {}
"#, enabled, debounce_ms, auto_show_on_error, max_tokens);

            let config: Result<Config, _> = toml::from_str(&toml_content);

            prop_assert!(config.is_ok(), "Failed to parse valid AI config");

            let config = config.unwrap();
            prop_assert_eq!(config.ai.enabled, enabled);
            prop_assert_eq!(config.ai.debounce_ms, debounce_ms);
            prop_assert_eq!(config.ai.auto_show_on_error, auto_show_on_error);
            prop_assert_eq!(config.ai.provider, AiProviderType::Anthropic);
            prop_assert_eq!(config.ai.anthropic.max_tokens, max_tokens);
            prop_assert_eq!(config.ai.anthropic.api_key, Some("sk-ant-test-key".to_string()));
        }
    }

    // **Feature: ai-assistant, Property 2: Missing AI section defaults to disabled**
    // *For any* TOML config without an `[ai]` section, parsing should succeed
    // and produce an AiConfig with `enabled = false`.
    // **Validates: Requirements 1.2**
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn prop_missing_ai_section_defaults_to_disabled(
            include_clipboard in prop::bool::ANY,
            include_tooltip in prop::bool::ANY,
        ) {
            // Build a config without [ai] section
            let mut toml_content = String::new();

            if include_clipboard {
                toml_content.push_str(r#"
[clipboard]
backend = "auto"
"#);
            }

            if include_tooltip {
                toml_content.push_str(r#"
[tooltip]
auto_show = true
"#);
            }

            let config: Result<Config, _> = toml::from_str(&toml_content);

            prop_assert!(config.is_ok(), "Failed to parse config without AI section");

            let config = config.unwrap();
            // AI should be disabled by default when section is missing
            prop_assert!(!config.ai.enabled, "AI should be disabled when [ai] section is missing");
            // Other defaults should also be set
            prop_assert_eq!(config.ai.debounce_ms, 1000);
            prop_assert!(config.ai.auto_show_on_error);
            prop_assert_eq!(config.ai.provider, AiProviderType::Anthropic);
        }
    }

    // **Feature: ai-assistant, Property 4: Invalid provider fallback**
    // *For any* string that is not a valid provider name ("anthropic"),
    // parsing should fail and the system should use default config.
    // **Validates: Requirements 1.4**
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn prop_invalid_provider_fallback(
            invalid_provider in "[a-z]{3,10}".prop_filter(
                "not valid provider",
                |s| s != "anthropic"
            )
        ) {
            let toml_content = format!(r#"
[ai]
enabled = true
provider = "{}"
"#, invalid_provider);

            let config: Result<Config, _> = toml::from_str(&toml_content);

            // Should fail to parse (serde will reject invalid enum value)
            prop_assert!(config.is_err(), "Invalid provider '{}' should fail to parse", invalid_provider);

            // When parsing fails, the system should fall back to defaults
            let default_config = Config::default();
            prop_assert!(!default_config.ai.enabled, "Default AI config should be disabled");
            prop_assert_eq!(default_config.ai.provider, AiProviderType::Anthropic);
        }
    }

    // Unit tests for AI config

    #[test]
    fn test_ai_config_default_values() {
        let config = AiConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.provider, AiProviderType::Anthropic);
        assert_eq!(config.debounce_ms, 1000);
        assert!(config.auto_show_on_error);
        assert!(config.anthropic.api_key.is_none());
        assert!(config.anthropic.model.is_none());
        assert_eq!(config.anthropic.max_tokens, 512);
    }

    #[test]
    fn test_anthropic_config_default_values() {
        let config = AnthropicConfig::default();
        assert!(config.api_key.is_none());
        assert!(config.model.is_none());
        assert_eq!(config.max_tokens, 512);
    }

    #[test]
    fn test_parse_ai_enabled() {
        let toml = r#"
[ai]
enabled = true

[ai.anthropic]
api_key = "sk-ant-test"
"#;
        let config: Config = toml::from_str(toml).unwrap();
        assert!(config.ai.enabled);
        assert_eq!(config.ai.anthropic.api_key, Some("sk-ant-test".to_string()));
    }

    #[test]
    fn test_parse_ai_disabled() {
        let toml = r#"
[ai]
enabled = false
"#;
        let config: Config = toml::from_str(toml).unwrap();
        assert!(!config.ai.enabled);
    }

    #[test]
    fn test_empty_ai_section_uses_defaults() {
        let toml = r#"
[ai]
"#;
        let config: Config = toml::from_str(toml).unwrap();
        assert!(!config.ai.enabled);
        assert_eq!(config.ai.debounce_ms, 1000);
        assert!(config.ai.auto_show_on_error);
    }

    #[test]
    fn test_partial_ai_section_uses_defaults() {
        let toml = r#"
[ai]
enabled = true
"#;
        let config: Config = toml::from_str(toml).unwrap();
        assert!(config.ai.enabled);
        assert_eq!(config.ai.debounce_ms, 1000);
        assert!(config.ai.auto_show_on_error);
        assert!(config.ai.anthropic.api_key.is_none());
    }

    #[test]
    fn test_invalid_provider_fails_to_parse() {
        let toml = r#"
[ai]
enabled = true
provider = "openai"
"#;
        let result: Result<Config, _> = toml::from_str(toml);
        assert!(result.is_err());
    }
}
