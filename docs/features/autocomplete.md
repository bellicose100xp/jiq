---
title: Autocomplete
parent: Features
nav_order: 2
description: Schema-aware field and function suggestions with type hints, nested path navigation, and bracket-notation safety.
---

# Autocomplete
{: .no_toc }

<details open markdown="block">
  <summary>Contents</summary>
  {: .text-delta }
- TOC
{:toc}
</details>

---

## What it is

The dropdown that opens as you type. Two kinds of suggestions:

- **Fields** — pulled from the loaded JSON, annotated with their JSON type (`string`, `number`, `bool`, `array`, `object`, `null`).
- **Functions** — bare-word jq builtins (`select`, `map`, `length`, `keys`, `to_entries`, `group_by`, `unique`, etc.).

Opens automatically when the cursor is at the end of a path or partial identifier. No manual trigger.

<div class="tui-mockup with-title" data-title="autocomplete preview">
<pre>
╭─ Input · INSERT ──────────────────────────────────╮
│ .users[0].                                         │
╰────────────────────────────────────────────────────╯
  ┌──────────────────────────────────────┐
  │ <span style="background:#5d8fdb;color:#0e0e12;">▸ name           string         </span>│
  │   age            number              │
  │   email          string              │
  │   tags           array               │
  │   profile        object              │
  │   active         bool                │
  └──────────────────────────────────────┘
</pre>
</div>

The highlighted row is what <kbd>Tab</kbd> commits. <kbd>↑</kbd> / <kbd>↓</kbd> move the highlight; <kbd>Esc</kbd> dismisses without inserting.

---

## Field discovery & array sampling

For objects, jiq lists the keys. For arrays with heterogeneous elements, jiq samples the first **N** and unions their keys.

`N` is `array_sample_size` — default **10**, range 1–1000.

```toml
[autocomplete]
array_sample_size = 25
```

See the [Configuration page](../configuration#autocomplete) for the full reference.

{: .note }
Sampling only matters for heterogeneous arrays. Homogeneous arrays need `array_sample_size = 1` worth of work either way.

---

## Nested-path navigation

Each dot narrows the list to the keys at that path:

<div class="drill-chain">
  <span class="step">.</span>
  <span class="arrow">→</span>
  <span class="step">.users</span>
  <span class="arrow">→</span>
  <span class="step">.users[0]</span>
  <span class="arrow">→</span>
  <span class="step">.users[0].profile</span>
  <span class="arrow">→</span>
  <span class="step active">.users[0].profile.email</span>
</div>

The popup re-renders with only the keys reachable from the current path. Deep selectors can be composed entirely from <kbd>Tab</kbd> presses.

<div class="io-pair">
  <div>
    <div class="io-label">type</div>
    <div class="io-block">.users[0].pro</div>
  </div>
  <div class="io-arrow">→</div>
  <div>
    <div class="io-label">popup shows</div>
    <div class="io-block">profile          object
profile_url      string</div>
  </div>
</div>

---

## Function suggestions

When the cursor sits on a bare identifier (not after a dot), suggestions switch to jq builtins:

<div class="tui-mockup">
<pre>
.users | map(sel
            ┌──────────────────┐
            │ ▸ select         │
            │   sort_by        │
            └──────────────────┘
</pre>
</div>

Common entries: `select`, `map`, `length`, `keys`, `values`, `to_entries`, `with_entries`, `from_entries`, `group_by`, `unique`, `unique_by`, `sort_by`, `min_by`, `max_by`, `add`, `any`, `all`, `paths`, `leaf_paths`, `recurse`, `flatten`, `range`, `tostring`, `tonumber`, `type`.

### Entry context inside `to_entries` / `with_entries`

Inside the body of either function, the iteration value is a `{key, value}` object, so suggestions switch to `.key` and `.value`.

<div class="io-pair">
  <div>
    <div class="io-label">type</div>
    <div class="io-block">to_entries | map(.</div>
  </div>
  <div class="io-arrow">→</div>
  <div>
    <div class="io-label">popup shows</div>
    <div class="io-block">key      string
value    any</div>
  </div>
</div>

---

## Non-ASCII and special-character keys

jq's `.field` shorthand accepts only ASCII identifiers. CJK, emoji, accented Latin, hyphens, spaces, and digit-leading keys require bracket or quoted-dot notation.

jiq auto-emits bracket form for these keys, so accepted suggestions are always valid jq.

<div class="io-pair">
  <div>
    <div class="io-label">key in your data</div>
    <div class="io-block">"名前": "Alice"
"first-name": "Bob"
"3rd": 3
"👋": "wave"</div>
  </div>
  <div class="io-arrow">→</div>
  <div>
    <div class="io-label">jiq inserts</div>
    <div class="io-block">.["名前"]
.["first-name"]
.["3rd"]
.["👋"]</div>
  </div>
</div>

| Form | Example | Status |
|---|---|---|
| `.["key"]` | `.["名前"]` | <span class="badge badge-green">Recommended</span> bracket notation, what jiq emits |
| `."key"` | `."名前"` | <span class="badge badge-cyan">Valid</span> quoted-dot, valid jq alternative |
| `.key` | `.名前` | <span class="badge badge-red">Invalid</span> jq syntax error for non-ASCII |
{: .shortcuts }

Applies to anything outside `[A-Za-z_][A-Za-z0-9_]*`: emoji, CJK (`中文`, `日本語`), accented Latin (`café`), hyphens (`first-name`), spaces (`full name`), digit-leading keys (`3rd`).

---

## Editing in the middle of a query

{: .note }
> Editing in the middle of a query falls back to root-level suggestions. For context-aware completion, edit at the end of the path.

---

## Shortcuts

| Key | Action |
|-----|--------|
| <kbd>Tab</kbd> | Accept the highlighted suggestion |
| <kbd>↑</kbd> / <kbd>↓</kbd> | Move the highlight up / down |
| <kbd>Esc</kbd> | Close the popup without inserting |
| Mouse click | Highlight a suggestion |
| Mouse double-click | Accept the suggestion under the pointer |
{: .shortcuts }
