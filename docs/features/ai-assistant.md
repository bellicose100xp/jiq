---
title: AI assistant
parent: Features
nav_order: 3
description: Natural-language jq queries, error fixes, and contextual suggestions powered by Anthropic, OpenAI, Gemini, AWS Bedrock, or any OpenAI-compatible endpoint.
---

# AI assistant
{: .no_toc }

[Features](./) · [Quick reference](../quick-reference) · [Configuration](../configuration)
{: .fs-3 }

<details markdown="block">
  <summary>Table of contents</summary>

- TOC
{:toc}

</details>

## What it does

Press <kbd>Ctrl</kbd>+<kbd>A</kbd> and the assistant reads three things — your current query, a sampled schema of the loaded JSON, and any active error — then asks the configured model for up to five concrete jq queries you can apply with a single keystroke.

It runs in three modes, picked automatically based on what state your query is in:

- **Error fix.** Your query has a syntax error or runtime failure. Suggestions try to repair it while preserving intent. The assistant sees the exact error message, so the fixes are usually right on the first try (`select(.active = true)` → `select(.active == true)`).
- **Natural language.** Type plain English in the input — `show me users older than 30 with their emails` — and the assistant returns jq that does exactly that. The query box accepts free-form text; the AI rewrites it to executable jq.
- **Suggestion.** Your query already returns valid output. The assistant proposes useful next steps based on the result shape — `filter by active`, `sort by created_at desc`, `count by category`, `extract emails only`. Good for exploring an unfamiliar JSON document.

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

The popup appears anchored above the input. Hit <kbd>Alt</kbd>+<kbd>1</kbd> through <kbd>Alt</kbd>+<kbd>5</kbd> to apply that suggestion directly without navigating, or arrow through them and confirm with <kbd>Enter</kbd>.

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

The assistant is opt-in. Drop a `[ai]` section into your `~/.config/jiq/config.toml` (or the platform-specific config path — see [Configuration](../configuration)) with `enabled = true` and one provider block.

### Anthropic

[Get an API key →](https://console.anthropic.com/settings/keys)

```toml
[ai]
enabled = true
provider = "anthropic"

[ai.anthropic]
api_key = "sk-ant-..."
model = "claude-haiku-4-5-20251001"
```

### OpenAI

[Get an API key →](https://platform.openai.com/api-keys)

```toml
[ai]
enabled = true
provider = "openai"

[ai.openai]
api_key = "sk-proj-..."
model = "gpt-4o-mini"
```

### Gemini

[Get an API key →](https://aistudio.google.com/apikey)

```toml
[ai]
enabled = true
provider = "gemini"

[ai.gemini]
api_key = "AIza..."
model = "gemini-3-flash"
```

### AWS Bedrock

Uses your default AWS credential chain (or a named `profile`). No extra API key needed.

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

Any provider that speaks the OpenAI chat-completions wire format works — set `provider = "openai"` and point `base_url` at the endpoint.

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

Controls how much of the loaded JSON schema and sample output is sent to the model. Default is `100000` characters.

```toml
[ai]
max_context_length = 100000
```

{: .tip }
> Larger values send more schema and sample context, which usually produces better suggestions for complex documents — but burns more tokens per request. Smaller values are faster and cheaper. For deep nested schemas (think Kubernetes manifests, GitHub API payloads), bumping this to `200000+` noticeably improves suggestion quality. For flat data, `50000` is plenty.

## Non-ASCII keys

{: .note }
> The assistant ships with a client-side sanitizer that rewrites any `.X` suggestion where `X` is not a valid ASCII jq identifier into bracket notation `.["X"]`. So even if the model returns `.名前` or `.café`, jiq applies it as `.["名前"]` / `.["café"]` — which jq actually accepts. You don't need to coach the model around Unicode keys; suggestions stay syntactically valid regardless.

## Privacy

{: .warning }
> Whatever model you configure receives **your query text and a truncated view of the loaded JSON** (schema plus sample, capped at `max_context_length`). For confidential data — secrets, PII, customer payloads — point `provider` at a local model via Ollama or LM Studio so nothing leaves your machine. The OpenAI-compatible blocks above are the path for fully air-gapped use.

## Troubleshooting

- **Popup opens but never returns suggestions.** The provider request is failing silently. Re-launch with `jiq --debug data.json` and tail `/tmp/jiq-debug.log` — auth errors, network timeouts, and quota errors are logged there.
- **"Could not parse" message in the popup.** The model returned malformed output that jiq's parser couldn't recover. This is rare since v3.22.0 (which added a last-resort JSON-object extractor and stricter format prompts). If it persists, switch to a more capable model — `gpt-4o-mini` and `claude-haiku-4-5` are reliable; smaller local models occasionally drift.
- **Suggestions are stale or off-topic.** Increase `max_context_length` so the model sees more of your data structure.
