---
title: AI assistant
parent: Features
nav_order: 3
description: Natural-language jq queries, error fixes, and contextual suggestions powered by Anthropic, OpenAI, Gemini, AWS Bedrock, or any OpenAI-compatible endpoint.
---

# AI assistant
{: .no_toc }

<details markdown="block">
  <summary>Table of contents</summary>

- TOC
{:toc}

</details>

## What it does

<kbd>Ctrl</kbd>+<kbd>A</kbd> sends three things to the configured model — current query, sampled schema, active error — and returns up to five jq suggestions, applied with a single keystroke.

Three modes, picked automatically:

- **Error fix.** Query has a syntax or runtime error. Suggestions repair it (`select(.active = true)` → `select(.active == true)`).
- **Natural language.** Type English in the input (`show me users older than 30 with their emails`); the assistant returns executable jq.
- **Suggestion.** Query returns valid output. The assistant proposes next steps based on the result shape (`filter by active`, `sort by created_at desc`, `extract emails only`).

## Visual

<div class="tui-mockup with-title" data-title="Ctrl+A — AI assistant">
<pre>
╭─ AI Assistant ───────────────────────────────────╮
│ Query: show me active users with their emails    │
│                                                  │
│ ▸ 1. .users[] | select(.active) | .email         │
│   2. [.users[] | select(.active) | .email]       │
│   3. .users | map(select(.active) | .email)      │
│   4. .users[] | select(.active == true).email    │
│   5. .users[] | select(.active) | {name, email}  │
│                                                  │
│ Alt+1-5 apply · Alt+↑↓ navigate · Enter apply    │
╰──────────────────────────────────────────────────╯
</pre>
</div>

Anchored above the input. <kbd>Alt</kbd>+<kbd>1</kbd>..<kbd>5</kbd> applies a suggestion directly; arrow keys + <kbd>Enter</kbd> confirms.

## Shortcuts
{: .shortcuts }

| Key | Action |
|---|---|
| <kbd>Ctrl</kbd>+<kbd>A</kbd> | Toggle the AI popup (open / close) |
| <kbd>Alt</kbd>+<kbd>1</kbd> … <kbd>Alt</kbd>+<kbd>5</kbd> | Apply suggestion 1–5 directly |
| <kbd>Alt</kbd>+<kbd>↑</kbd> / <kbd>Alt</kbd>+<kbd>↓</kbd> | Navigate suggestions |
| <kbd>Alt</kbd>+<kbd>j</kbd> / <kbd>Alt</kbd>+<kbd>k</kbd> | Navigate suggestions (vim style) |
| <kbd>Enter</kbd> | Apply the highlighted suggestion |

## Configuration

Opt-in. Add an `[ai]` section to `~/.config/jiq/config.toml` (or the platform config path — see [Configuration](../configuration)) with `enabled = true` and one provider block.

### Anthropic

[API key](https://console.anthropic.com/settings/keys)

```toml
[ai]
enabled = true
provider = "anthropic"

[ai.anthropic]
api_key = "sk-ant-..."
model = "claude-haiku-4-5-20251001"
```

### OpenAI

[API key](https://platform.openai.com/api-keys)

```toml
[ai]
enabled = true
provider = "openai"

[ai.openai]
api_key = "sk-proj-..."
model = "gpt-4o-mini"
```

### Gemini

[API key](https://aistudio.google.com/apikey)

```toml
[ai]
enabled = true
provider = "gemini"

[ai.gemini]
api_key = "AIza..."
model = "gemini-3-flash"
```

### AWS Bedrock

Uses the default AWS credential chain (or a named `profile`). No API key.

```toml
[ai]
enabled = true
provider = "bedrock"

[ai.bedrock]
region = "us-east-1"
model = "global.anthropic.claude-haiku-4-5-20251001-v1:0"
profile = "default"  # optional
```

### OpenAI-compatible endpoints

Any provider that speaks the OpenAI chat-completions wire format works. Set `provider = "openai"` and point `base_url` at the endpoint.

#### Ollama (local)

```toml
[ai]
enabled = true
provider = "openai"

[ai.openai]
base_url = "http://localhost:11434/v1"
model = "llama3"
```

#### LM Studio (local)

```toml
[ai]
enabled = true
provider = "openai"

[ai.openai]
base_url = "http://localhost:1234/v1"
model = "local-model"
```

#### x.ai Grok

```toml
[ai]
enabled = true
provider = "openai"

[ai.openai]
api_key = "xai-..."
base_url = "https://api.x.ai/v1"
model = "grok-4-fast-non-reasoning"
```

## `max_context_length`

Caps the schema + sample sent to the model. Default `100000` characters.

```toml
[ai]
max_context_length = 100000
```

{: .tip }
> Larger values cost more tokens but improve suggestions on deep nested schemas (Kubernetes manifests, GitHub API payloads — bump to `200000+`). For flat data, `50000` is plenty.

## Non-ASCII keys

{: .note }
> A client-side sanitizer rewrites any `.X` suggestion where `X` is not a valid ASCII jq identifier into `.["X"]`. Suggestions stay syntactically valid regardless of model output.

## Privacy

{: .warning }
> The configured model receives the query text and a truncated view of the loaded JSON (schema + sample, capped at `max_context_length`). For confidential data, point `provider` at a local model via Ollama or LM Studio.

## Troubleshooting

- **Popup opens, no suggestions.** Re-launch with `jiq --debug data.json` and tail `/tmp/jiq-debug.log` for auth, network, or quota errors.
- **"Could not parse".** Switch to a more capable model. `gpt-4o-mini` and `claude-haiku-4-5` are reliable.
- **Suggestions stale or off-topic.** Increase `max_context_length`.
