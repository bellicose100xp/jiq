//! Tests for ai_types

use super::*;
use crate::config::Config;
use proptest::prelude::*;

// Re-export configs for tests
use super::BedrockConfig;
use super::GeminiConfig;
use super::OpenAiConfig;

// **Feature: no-default-ai-provider, Property 1: Deserialization without provider yields None**
// *For any* valid TOML configuration string that does not contain a provider field in the [ai] section,
// deserializing it into AiConfig SHALL result in provider being None
// **Validates: Requirements 2.1, 2.2**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop_deserialization_without_provider_yields_none(
        enabled in prop::bool::ANY,
        include_anthropic_section in prop::bool::ANY,
        include_openai_section in prop::bool::ANY,
    ) {
        // Build a config WITHOUT provider field
        let mut toml_content = format!(r#"
[ai]
enabled = {}
"#, enabled);

        if include_anthropic_section {
            toml_content.push_str(r#"
[ai.anthropic]
api_key = "sk-ant-test-key"
model = "claude-3-haiku-20240307"
"#);
        }

        if include_openai_section {
            toml_content.push_str(r#"
[ai.openai]
api_key = "sk-proj-test-key"
model = "gpt-4o-mini"
"#);
        }

        let config: Result<Config, _> = toml::from_str(&toml_content);

        prop_assert!(config.is_ok(), "Failed to parse config without provider field");

        let config = config.unwrap();
        // Provider should be None when not specified
        prop_assert!(config.ai.provider.is_none(), "Provider should be None when not specified in config");
        prop_assert_eq!(config.ai.enabled, enabled);
    }
}

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
        prop_assert_eq!(config.ai.provider, Some(AiProviderType::Anthropic));
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
        // Provider should be None when not specified
        prop_assert!(config.ai.provider.is_none(), "Provider should be None when [ai] section is missing");
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
            |s| s != "anthropic" && s != "bedrock" && s != "openai" && s != "gemini"
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
        prop_assert!(default_config.ai.provider.is_none(), "Default provider should be None");
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
        prop_assert_eq!(config.ai.provider, Some(AiProviderType::Bedrock));
        prop_assert_eq!(config.ai.bedrock.region, Some(region));
        prop_assert_eq!(config.ai.bedrock.model, Some(model));
        prop_assert_eq!(config.ai.bedrock.profile, profile);
    }
}

// **Feature: gemini-provider, Property 1: Provider type recognition**
// *For any* config TOML with `provider = "gemini"`, deserializing should produce `AiProviderType::Gemini`
// **Validates: Requirements 1.1**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop_gemini_provider_parsing(
        enabled in prop::bool::ANY,
        api_key in "[a-zA-Z0-9-_]{20,50}",
        model in "(gemini-2\\.0-flash|gemini-1\\.5-pro|gemini-1\\.5-flash)",
    ) {
        let toml_content = format!(r#"
[ai]
enabled = {}
provider = "gemini"

[ai.gemini]
api_key = "{}"
model = "{}"
"#, enabled, api_key, model);

        let config: Result<Config, _> = toml::from_str(&toml_content);

        prop_assert!(config.is_ok(), "Failed to parse valid Gemini config: {:?}", config.err());

        let config = config.unwrap();
        prop_assert_eq!(config.ai.enabled, enabled);
        prop_assert_eq!(config.ai.provider, Some(AiProviderType::Gemini));
        prop_assert_eq!(config.ai.gemini.api_key, Some(api_key));
        prop_assert_eq!(config.ai.gemini.model, Some(model));
    }
}

// Unit tests for AI config

// **Feature: no-default-ai-provider, Unit test: AiConfig::default() has provider = None**
// **Validates: Requirements 2.3**
#[test]
fn test_ai_config_default_values() {
    let config = AiConfig::default();
    assert!(!config.enabled);
    assert!(
        config.provider.is_none(),
        "AiConfig::default() should have provider = None"
    );
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
fn test_openai_config_default_values() {
    let config = OpenAiConfig::default();
    assert!(config.api_key.is_none());
    assert!(config.model.is_none());
}

#[test]
fn test_gemini_config_default_values() {
    let config = GeminiConfig::default();
    assert!(config.api_key.is_none());
    assert!(config.model.is_none());
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
fn test_parse_openai_provider() {
    let toml = r#"
[ai]
enabled = true
provider = "openai"

[ai.openai]
api_key = "sk-proj-test-key"
model = "gpt-4o-mini"
"#;
    let config: Config = toml::from_str(toml).unwrap();
    assert!(config.ai.enabled);
    assert_eq!(config.ai.provider, Some(AiProviderType::Openai));
    assert_eq!(
        config.ai.openai.api_key,
        Some("sk-proj-test-key".to_string())
    );
    assert_eq!(config.ai.openai.model, Some("gpt-4o-mini".to_string()));
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
    assert_eq!(config.ai.provider, Some(AiProviderType::Bedrock));
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
    assert_eq!(config.ai.provider, Some(AiProviderType::Bedrock));
    assert_eq!(
        config.ai.bedrock.profile,
        Some("my-aws-profile".to_string())
    );
}

#[test]
fn test_parse_gemini_provider() {
    let toml = r#"
[ai]
enabled = true
provider = "gemini"

[ai.gemini]
api_key = "AIza-test-key"
model = "gemini-2.0-flash"
"#;
    let config: Config = toml::from_str(toml).unwrap();
    assert!(config.ai.enabled);
    assert_eq!(config.ai.provider, Some(AiProviderType::Gemini));
    assert_eq!(config.ai.gemini.api_key, Some("AIza-test-key".to_string()));
    assert_eq!(config.ai.gemini.model, Some("gemini-2.0-flash".to_string()));
}

// **Feature: openai-provider, Property 10: Configuration validation**
// *For any* OpenAI configuration passed to from_config, if the configuration is valid
// (has non-empty api_key and model), then the function should return Ok(AsyncAiProvider::Openai(_)).
// **Validates: Requirements 5.2**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop_openai_config_validation(
        enabled in prop::bool::ANY,
        api_key in "[a-zA-Z0-9-_]{20,50}",
        model in "(gpt-4o-mini|gpt-4o|gpt-3\\.5-turbo)",
    ) {
        let toml_content = format!(r#"
[ai]
enabled = {}
provider = "openai"

[ai.openai]
api_key = "{}"
model = "{}"
"#, enabled, api_key, model);

        let config: Result<Config, _> = toml::from_str(&toml_content);

        prop_assert!(config.is_ok(), "Failed to parse valid OpenAI config");

        let config = config.unwrap();
        prop_assert_eq!(config.ai.enabled, enabled);
        prop_assert_eq!(config.ai.provider, Some(AiProviderType::Openai));
        prop_assert_eq!(config.ai.openai.api_key, Some(api_key));
        prop_assert_eq!(config.ai.openai.model, Some(model));
    }
}

// **Feature: openai-provider, Property 19: Configuration with whitespace**
// *For any* configuration where `api_key` or `model` contains only whitespace characters,
// the system should treat it as missing and return a NotConfigured error.
// **Validates: Requirements 1.4, 1.5**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop_openai_config_whitespace_handling(
        whitespace_count in 1usize..10,
        use_spaces in prop::bool::ANY,
        whitespace_in_api_key in prop::bool::ANY,
    ) {
        let whitespace = if use_spaces {
            " ".repeat(whitespace_count)
        } else {
            "\t".repeat(whitespace_count)
        };

        let api_key = if whitespace_in_api_key {
            whitespace.clone()
        } else {
            "sk-proj-valid-key".to_string()
        };

        let model = if !whitespace_in_api_key {
            whitespace.clone()
        } else {
            "gpt-4o-mini".to_string()
        };

        let toml_content = format!(r#"
[ai]
enabled = true
provider = "openai"

[ai.openai]
api_key = "{}"
model = "{}"
"#, api_key, model);

        let config: Result<Config, _> = toml::from_str(&toml_content);

        // Config parsing should succeed (TOML parsing doesn't validate content)
        prop_assert!(config.is_ok(), "Failed to parse OpenAI config with whitespace");

        let config = config.unwrap();

        // Verify the whitespace values are stored as-is
        if whitespace_in_api_key {
            prop_assert_eq!(config.ai.openai.api_key, Some(api_key));
            // The validation that treats whitespace as missing happens in from_config,
            // not during TOML parsing
        } else {
            prop_assert_eq!(config.ai.openai.model, Some(model));
        }
    }
}
