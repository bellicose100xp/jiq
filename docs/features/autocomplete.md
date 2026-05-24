---
title: Autocomplete
parent: Features
nav_order: 2
description: Schema-aware field and function suggestions with type hints, nested path navigation, and bracket-notation safety.
---

# Autocomplete
{: .no_toc }

[Back to Features](./){: .btn .btn-outline .fs-3 .mr-2 }
[Quick reference](../quick-reference){: .btn .btn-outline .fs-3 }

<details open markdown="block">
  <summary>Contents</summary>
  {: .text-delta }
- TOC
{:toc}
</details>

---

## What it is

Autocomplete is the dropdown that appears as you type, offering two kinds of suggestions:

- **Fields** — pulled from the JSON you actually loaded, not a generic schema. Each entry is annotated with its JSON type (`string`, `number`, `bool`, `array`, `object`, `null`), so you can tell what you're about to drill into before you hit <kbd>Tab</kbd>.
- **Functions** — bare-word jq builtins (`select`, `map`, `length`, `keys`, `to_entries`, `group_by`, `unique`, etc.).

The popup opens automatically whenever your cursor is at the end of a path or partial identifier. There is no manual trigger key.

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

The highlighted row is what <kbd>Tab</kbd> will commit. <kbd>↑</kbd> / <kbd>↓</kbd> move the highlight; <kbd>Esc</kbd> dismisses the popup without inserting anything.

---

## Field discovery & array sampling

For object values, jiq simply lists the keys. For arrays where elements have *different* shapes (a heterogeneous list of records, common in real-world API responses), jiq samples the first **N** elements and unions their keys to build a single suggestion list.

`N` is `array_sample_size` in your config — default **10**, range 1–1000. Larger samples catch rare fields at the cost of a small upfront walk through your data.

```toml
[autocomplete]
array_sample_size = 25
```

See the [Configuration page](../configuration#autocomplete) for the full reference.

{: .note }
Sampling only matters when array elements diverge. A homogeneous array (every element has the same keys) needs `array_sample_size = 1` worth of work either way.

---

## Nested-path navigation

Each dot you add narrows the suggestion list to the keys *at that path*. The popup walks down the tree alongside you:

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

At every step the popup re-renders with only the keys reachable from the current path, so you can compose deep selectors entirely from <kbd>Tab</kbd> presses.

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

When the cursor sits on a bare identifier (not after a dot), jiq suggests jq builtins instead of fields:

<div class="tui-mockup">
<pre>
.users | map(sel
            ┌──────────────────┐
            │ ▸ select         │
            │   sort_by        │
            └──────────────────┘
</pre>
</div>

Common functions in the suggestion set: `select`, `map`, `length`, `keys`, `values`, `to_entries`, `with_entries`, `from_entries`, `group_by`, `unique`, `unique_by`, `sort_by`, `min_by`, `max_by`, `add`, `any`, `all`, `paths`, `leaf_paths`, `recurse`, `flatten`, `range`, `tostring`, `tonumber`, `type`.

### Entry context inside `to_entries` / `with_entries`

Inside the body of `to_entries(...)` or `with_entries(...)`, the iteration value is a `{key, value}` object — so jiq's suggestions switch to `.key` and `.value` instead of the parent record's fields. This works identically in both functions.

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

jq's `.field` shorthand only accepts ASCII identifiers. Anything else — CJK characters, emoji, accented Latin, hyphens, spaces, keys that start with a digit — must use **bracket notation** or **quoted-dot notation**, otherwise jq returns a syntax error.

jiq's autocomplete handles this for you: it auto-emits the bracket form so the suggestion you accept is always valid jq.

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

The same rule applies to anything outside `[A-Za-z_][A-Za-z0-9_]*`: emoji, CJK (`中文`, `日本語`), accented Latin (`café`), hyphens (`first-name`), spaces (`full name`), digit-leading keys (`3rd`).

---

## Editing in the middle of a query

{: .note }
> When you move the cursor into the middle of an existing query and start editing, autocomplete falls back to root-level suggestions instead of resolving the path at the cursor position. This is a known limitation — for context-aware suggestions, edit at the end of the path or rebuild from the start.

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
