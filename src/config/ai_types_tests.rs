//! Tests for ai_types

use super::*;
use crate::config::Config;
use proptest::prelude::*;

// Re-export BedrockConfig for tests
use super::BedrockConfig;

// **Feature: ai-assistant, Property 1: Valid config parsing**
// *For any* valid TOML config with `[ai]` section containing valid fields,
// parsing should succeed and produce an AiConfig with the specified values.
// **Validates: Requirements 1.1**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop_valid_ai_config_parsing(
        enabled in prop::bool::ANY,
        max_tokens in 256u32..4096u32,
    ) {
        let toml_content = format!(r#"
[ai]
enabled = {}
provider = "anthropic"

[ai.anthropic]
api_key = "sk-ant-test-key"
model = "claude-3-haiku-20240307"
max_tokens = {}
"#, enabled, max_tokens);

        let config: Result<Config, _> = toml::from_str(&toml_content);

        prop_assert!(config.is_ok(), "Failed to parse valid AI config");

        let config = config.unwrap();
        prop_assert_eq!(config.ai.enabled, enabled);
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
            |s| s != "anthropic" && s != "bedrock"
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

// **Feature: bedrock-provider, Property 1: Config parsing recognizes Bedrock provider**
// *For any* valid TOML configuration with `provider = "bedrock"`, parsing SHALL
// produce an `AiProviderType::Bedrock` variant.
// **Validates: Requirements 1.1**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop_bedrock_provider_parsing(
        enabled in prop::bool::ANY,
        region in "[a-z]{2}-[a-z]{4,9}-[1-9]",
        model in "[a-z]+\\.[a-z0-9-]+",
        profile in proptest::option::of("[a-z][a-z0-9-]{2,15}"),
    ) {
        let profile_line = match &profile {
            Some(p) => format!("profile = \"{}\"", p),
            None => String::new(),
        };

        let toml_content = format!(r#"
[ai]
enabled = {}
provider = "bedrock"

[ai.bedrock]
region = "{}"
model = "{}"
{}
"#, enabled, region, model, profile_line);

        let config: Result<Config, _> = toml::from_str(&toml_content);

        prop_assert!(config.is_ok(), "Failed to parse valid Bedrock config: {:?}", config.err());

        let config = config.unwrap();
        prop_assert_eq!(config.ai.enabled, enabled);
        prop_assert_eq!(config.ai.provider, AiProviderType::Bedrock);
        prop_assert_eq!(config.ai.bedrock.region, Some(region));
        prop_assert_eq!(config.ai.bedrock.model, Some(model));
        prop_assert_eq!(config.ai.bedrock.profile, profile);
    }
}

// Unit tests for AI config

#[test]
fn test_ai_config_default_values() {
    let config = AiConfig::default();
    assert!(!config.enabled);
    assert_eq!(config.provider, AiProviderType::Anthropic);
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
fn test_bedrock_config_default_values() {
    let config = BedrockConfig::default();
    assert!(config.region.is_none());
    assert!(config.model.is_none());
    assert!(config.profile.is_none());
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
}

#[test]
fn test_partial_ai_section_uses_defaults() {
    let toml = r#"
[ai]
enabled = true
"#;
    let config: Config = toml::from_str(toml).unwrap();
    assert!(config.ai.enabled);
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

#[test]
fn test_parse_bedrock_provider() {
    let toml = r#"
[ai]
enabled = true
provider = "bedrock"

[ai.bedrock]
region = "us-east-1"
model = "anthropic.claude-3-haiku-20240307-v1:0"
"#;
    let config: Config = toml::from_str(toml).unwrap();
    assert!(config.ai.enabled);
    assert_eq!(config.ai.provider, AiProviderType::Bedrock);
    assert_eq!(config.ai.bedrock.region, Some("us-east-1".to_string()));
    assert_eq!(
        config.ai.bedrock.model,
        Some("anthropic.claude-3-haiku-20240307-v1:0".to_string())
    );
    assert!(config.ai.bedrock.profile.is_none());
}

#[test]
fn test_parse_bedrock_provider_with_profile() {
    let toml = r#"
[ai]
enabled = true
provider = "bedrock"

[ai.bedrock]
region = "us-west-2"
model = "anthropic.claude-3-sonnet-20240229-v1:0"
profile = "my-aws-profile"
"#;
    let config: Config = toml::from_str(toml).unwrap();
    assert_eq!(config.ai.provider, AiProviderType::Bedrock);
    assert_eq!(
        config.ai.bedrock.profile,
        Some("my-aws-profile".to_string())
    );
}
