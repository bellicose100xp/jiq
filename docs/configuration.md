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

## Theme

```toml
[theme]
mode = "auto"
```

| Value | Effect |
|:---|:---|
| `auto` (default) | Detect the terminal background at startup and pick the light or dark palette. Falls back to dark if detection is unavailable (e.g. inside tmux/screen without passthrough, or a non-responding terminal). |
| `light` | Force the light palette. |
| `dark` | Force the dark palette (the classic Galaxy theme). |

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

When `true` (default), the function tooltip appears automatically as the cursor lands on a known jq function. Set to `false` to require <kbd>Ctrl</kbd>+<kbd>I</kbd> to open it.

## AI

```toml
[ai]
enabled              = true
provider             = "anthropic"   # "openai" | "gemini" | "bedrock"
max_context_length   = 100000        # characters of schema/sample context
extra_instructions   = ""            # optional guidance appended to every AI prompt
request_timeout_secs = 120           # whole-request timeout; 0 disables it
```

| Provider | Recommended model |
|:---|:---|
| Anthropic | `claude-haiku-4-5-20251001` |
| OpenAI | `gpt-4o-mini` |
| Gemini | `gemini-3-flash` |
| Bedrock | `global.anthropic.claude-haiku-4-5-20251001-v1:0` |

Every provider takes an optional `effort` level for reasoning-capable models; Anthropic and Bedrock Claude also take a `context_1m` toggle:

```toml
[ai.anthropic]
api_key    = "sk-ant-..."
model      = "claude-sonnet-4-6"
effort     = "high"    # low | medium | high | xhigh | max (Claude 4.6+)
context_1m = false     # 1M-token context window beta (Claude Sonnet 4/4.5)

[ai.bedrock]
region     = "us-east-1"
model      = "global.anthropic.claude-sonnet-4-6-v1:0"   # or "us.openai.gpt-5.6-sol"
effort     = "high"    # shaped per model family (Claude vs OpenAI)
context_1m = false     # Claude Sonnet 4/4.5 only

[ai.openai]
api_key = "sk-proj-..."
model   = "gpt-5.1-mini"
effort  = "medium"     # minimal | low | medium | high | xhigh | max

[ai.gemini]
api_key = "AIza..."
model   = "gemini-3-flash"
effort  = "medium"     # minimal | low | medium | high (Gemini 3+)
```

OpenAI models on Bedrock work through `provider = "bedrock"` (Converse + AWS profile) or the OpenAI-compatible endpoint with a Bedrock API key. See [AI assistant](./features/ai-assistant) for details per provider.

See [AI assistant](./features/ai-assistant) for per-provider config.

## Full example

```toml
[clipboard]
backend = "auto"

[theme]
mode = "auto"

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

## Query

```toml
[query]
debounce_ms = 150   # delay between the last keystroke and the jq re-run
```

Lower values feel snappier on small files; raise it if large inputs make retyping laggy.

## History

```toml
[history]
max_entries = 1000   # cap on persisted query-history entries
```

## Save

```toml
[save]
default_pattern = "jiq-{timestamp}.json"   # initial filename in the save dialog
```

Supports `{timestamp}`, `{cwd}`, and `~` expansion, e.g. `~/exports/jiq-{timestamp}.json`.

## Environment overrides

| Variable | Effect |
|:---|:---|
| `JIQ_DEBUG=1` | Same as `--debug`: write debug logs to `/tmp/jiq-debug.log`. |

See [Troubleshooting](./troubleshooting).
