---
title: Autocomplete
parent: Features
nav_order: 2
description: Schema-aware field and function suggestions, with type hints and bracket-notation safety.
---

# Autocomplete

Suggestions appear as you type. <kbd>Tab</kbd> accepts the highlighted entry.

Five suggestion kinds, picked from cursor context:

- **Function** — jq builtins (`select`, `map`, `keys`, `to_entries`, `group_by`, `sort_by`, `unique_by`, `with_entries`, …). Inserts the open paren when the function takes arguments (`select(`).
- **Field** — keys discovered in the actual JSON, with a JSON type hint (`String`, `Number`, `Boolean`, `Null`, `Object`, `Array`, `Array[String]`, …).
- **Operator** — pipe and comparison tokens.
- **Variable** — `$name` bindings declared earlier in the query (via `as $x`, `[$a, $b]`, `{k: $v}`) plus jq's built-ins `$ENV` and `$__loc__`.
- **Iterator** — `[]` patterns in path-flow contexts.

<div class="tui-mockup with-title" data-title="Field suggestions after typing .users[0].">
<pre>╭─ Input ───────────────────────────────────────╮
│ .users[0].                                    │
╰───────────────────────────────────────────────╯
  ┌──────────────────────────────────┐
  │ ▸ name           String          │
  │   age            Number          │
  │   email          String          │
  │   tags           Array[String]   │
  │   profile        Object          │
  │   active         Boolean         │
  └──────────────────────────────────┘</pre>
</div>

Up to 10 entries visible at a time; longer lists scroll. Filter is case-insensitive substring match (variables are case-sensitive, like jq).

## Schema-aware fields

Field suggestions come from the value at the cursor's path in your loaded JSON. Walking deeper into the path narrows the list.

<div class="drill-chain">
  <div class="step">.</div>
  <div class="arrow">→</div>
  <div class="step">.users</div>
  <div class="arrow">→</div>
  <div class="step">.users[0]</div>
  <div class="arrow">→</div>
  <div class="step active">.users[0].profile.</div>
</div>

For arrays whose elements have differing shapes, jiq samples up to `array_sample_size` elements (default 10, configurable) and unions their keys.

Inside `to_entries` / `with_entries`, suggestions for entry access are `.key` and `.value`.

### Bracket notation

jq's `.field` shorthand only accepts ASCII identifiers matching `[A-Za-z_][A-Za-z_0-9]*`. Anything else — CJK, emoji, accented Latin, hyphens, spaces, digit-start — is suggested in bracket form.

<div class="io-pair">
  <div>
    <div class="io-label">Field name</div>
    <div class="io-block">名前
café
my-field
👋
2nd</div>
  </div>
  <div class="io-arrow">→</div>
  <div>
    <div class="io-label">Suggestion inserts</div>
    <div class="io-block">.["名前"]
.["café"]
.["my-field"]
.["👋"]
.["2nd"]</div>
  </div>
</div>

Plain ASCII names insert as `.name` as expected.

### Tuning

Bump `array_sample_size` in `~/.config/jiq/config.toml` for arrays that mix shapes:

```toml
[autocomplete]
array_sample_size = 25  # default 10, range 1–1000
```

See [Configuration](../configuration#autocomplete).

## Shortcuts
{: .shortcuts }

| Key | Action |
|---|---|
| <kbd>Tab</kbd> | Accept the highlighted suggestion |
| <kbd>↑</kbd> / <kbd>↓</kbd> | Navigate the list |
| <kbd>Esc</kbd> | Close the popup |
| Mouse click | Highlight a suggestion |
| Mouse double-click | Apply a suggestion |
