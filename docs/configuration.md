---
title: Configuration
nav_order: 5
description: config.toml reference — clipboard backend, autocomplete depth, AI providers.
---

# Configuration

`config.toml` reference covering clipboard, autocomplete, and AI.

## Location

Path: `~/.config/jiq/config.toml` (all platforms). Optional — defaults are reasonable; only AI requires config.

## Clipboard

```toml
[clipboard]
backend = "auto"
```

| Value | Use when |
|:---|:---|
| `auto` (default) | OS clipboard first, OSC 52 fallback. |
| `system` | Local desktop only. |
| `osc52` | SSH / tmux / mosh on a terminal that forwards OSC 52 (Ghostty, kitty, WezTerm, foot). |

## Autocomplete

```toml
[autocomplete]
array_sample_size = 10
```

For heterogeneous arrays, increase to union more keys (range: 1–1000).

## Tooltip

```toml
[tooltip]
auto_show = true
```

When `true` (default), the function tooltip appears automatically as the cursor lands on a known jq function. Set to `false` to require <kbd>Ctrl</kbd>+<kbd>T</kbd> to open it.

## AI

```toml
[ai]
enabled            = true
provider           = "anthropic"   # "openai" | "gemini" | "bedrock"
max_context_length = 100000        # characters of schema/sample context
```

| Provider | Recommended model |
|:---|:---|
| Anthropic | `claude-haiku-4-5-20251001` |
| OpenAI | `gpt-4o-mini` |
| Gemini | `gemini-3-flash` |
| Bedrock | `global.anthropic.claude-haiku-4-5-20251001-v1:0` |

See [AI assistant](./features/ai-assistant) for per-provider config.

## Full example

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

## Environment overrides

| Variable | Effect |
|:---|:---|
| `JIQ_DEBUG=1` | Same as `--debug`: write debug logs to `/tmp/jiq-debug.log`. |

See [Troubleshooting](./troubleshooting).
