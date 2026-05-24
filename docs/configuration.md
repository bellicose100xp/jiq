---
title: Configuration
nav_order: 5
description: config.toml reference — clipboard backend, autocomplete depth, AI providers (Anthropic, OpenAI, Gemini, AWS Bedrock, OpenAI-compatible).
---

# Configuration
{: .no_toc }

<details open markdown="block">
  <summary>On this page</summary>
  {: .text-delta }
- TOC
{: toc }
</details>

---

## Where the config lives

jiq looks for `config.toml` at the platform default location:

| OS | Path |
|:---|:---|
| Linux | `~/.config/jiq/config.toml` |
| macOS | `~/Library/Application Support/jiq/config.toml` |
| Windows | `%APPDATA%\jiq\config.toml` |

The file is optional. Out of the box, jiq runs with sensible defaults — only the AI section requires explicit configuration to enable AI features.

---

## Clipboard

```toml
[clipboard]
# Clipboard backend: "auto" (default), "system", or "osc52"
# - auto: tries system clipboard first, falls back to OSC 52
# - system: use only OS clipboard (may not work in SSH/tmux)
# - osc52: use terminal escape sequences (works in most modern terminals over SSH)
backend = "auto"
```

| Value | Use when |
|:---|:---|
| `auto` (default) | You don't know which to pick — tries OS clipboard, falls back to OSC 52. |
| `system` | Local desktop only — fastest, no terminal cooperation needed. |
| `osc52` | Persistent SSH / tmux / mosh — your terminal forwards OSC 52 (Ghostty, kitty, WezTerm, foot). |

See [Clipboard & paste recovery](./features/clipboard) for the full SSH/OSC 52 story.

---

## Autocomplete

```toml
[autocomplete]
# Number of array elements sampled to discover field suggestions for arrays
# where fields differ across elements. Increasing this may improve suggestions
# but adds a performance cost.
# Range: 1 - 1000 (default: 10)
array_sample_size = 10
```

When the cursor is positioned to suggest fields *inside* an array's elements, jiq samples up to `array_sample_size` elements and unions their keys. With heterogeneous arrays (e.g., a feed of mixed event types), bumping this surfaces more fields at the cost of a small startup-time hit per autocomplete trigger.

---

## AI

```toml
[ai]
# Enable AI assistant
enabled = true
# Provider: "anthropic", "openai", "gemini", or "bedrock"
provider = "anthropic"
# Character limit at which JSON schema and output samples are truncated (default: 100000)
# Larger values send more context to AI but increase token usage/costs
max_context_length = 100000
```

For faster responses, prefer lightweight models:

| Provider | Recommended model |
|:---|:---|
| Anthropic | `claude-haiku-4-5-20251001` |
| OpenAI | `gpt-4o-mini` |
| Gemini | `gemini-3-flash` |

See [AI assistant](./features/ai-assistant) for the full feature walkthrough.

### Anthropic

```toml
[ai.anthropic]
# https://console.anthropic.com/settings/keys
api_key = "your-api-key-here"
model = "claude-haiku-4-5-20251001"
```

### OpenAI

```toml
[ai.openai]
# https://platform.openai.com/api-keys
api_key = "sk-proj-..."
model = "gpt-4o-mini"
```

### OpenAI-compatible (Ollama, LM Studio, x.ai Grok, others)

Any API that follows the OpenAI request/response format works by setting `provider = "openai"` and overriding `base_url`.

**Pattern:**

```toml
[ai.openai]
base_url = "https://your-api-endpoint/v1"
api_key  = "your-api-key"      # optional if the provider doesn't require one
model    = "model-name"
```

**Ollama (local, no API key):**

```toml
[ai.openai]
base_url = "http://localhost:11434/v1"
model    = "llama3"
```

**LM Studio (local, no API key):**

```toml
[ai.openai]
base_url = "http://localhost:1234/v1"
model    = "local-model"
```

**x.ai Grok:**

```toml
[ai.openai]
api_key  = "your-xai-api-key"
base_url = "https://api.x.ai/v1"
model    = "grok-4-fast-non-reasoning"
```

### Gemini

```toml
[ai.gemini]
# https://aistudio.google.com/apikey
api_key = "AIza..."
model   = "gemini-3-flash-preview"
```

### AWS Bedrock

```toml
[ai.bedrock]
region  = "us-east-1"
model   = "global.anthropic.claude-haiku-4-5-20251001-v1:0"
profile = "default"   # optional — uses default credential chain if omitted
```

{: .warning }
> **Sensitive data?** AI providers receive your query text and a portion of the JSON schema/sample. For sensitive payloads, prefer a local model via Ollama or LM Studio.

---

## Full example

A complete `config.toml` with everything dialed in:

```toml
[clipboard]
backend = "auto"

[autocomplete]
array_sample_size = 25

[ai]
enabled            = true
provider           = "anthropic"
max_context_length = 80000

[ai.anthropic]
api_key = "sk-ant-..."
model   = "claude-haiku-4-5-20251001"
```

---

## Environment overrides

| Variable | Effect |
|:---|:---|
| `JIQ_DEBUG=1` | Same as `--debug`: write debug logs to `/tmp/jiq-debug.log`. |

See [Troubleshooting](./troubleshooting) for what those logs contain and how to attach them to a bug report.
