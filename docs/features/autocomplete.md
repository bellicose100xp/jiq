---
title: Autocomplete
parent: Features
nav_order: 2
description: Get field name and function suggestions as you type, drawn from your actual JSON.
---

# Autocomplete

As you type a query, jiq shows a suggestion list below the input. Suggestions come from two sources: field names pulled directly from your loaded JSON, and jq's built-in functions.

## Accept a suggestion

When the suggestion list appears:

1. Use **↑** / **↓** to highlight the suggestion you want.
2. Press **Tab** to insert it into the query.

To dismiss the list without accepting anything, press **Esc**.

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

## Understand what you're looking at

Each suggestion shows a type label on the right. These come from the actual values in your data — if a field holds a string, it says `String`; if it holds an array of strings, it says `Array[String]`.

Suggestion kinds:

- **Field** — a key from your JSON, with its value type
- **Function** — a jq built-in like `select`, `map`, `keys`, `group_by`. Functions that take arguments insert the opening parenthesis automatically (`select(`)
- **Operator** — pipe and comparison tokens
- **Variable** — `$name` bindings declared earlier in the query, plus jq's built-ins `$ENV` and `$__loc__`
- **Iterator** — `[]` in path-flow contexts

## Navigate deeper paths

Field suggestions narrow as you type deeper into a path. Type `.users[0].profile.` and jiq shows only the fields inside `profile`.

<div class="drill-chain">
  <div class="step">.</div>
  <div class="arrow">→</div>
  <div class="step">.users</div>
  <div class="arrow">→</div>
  <div class="step">.users[0]</div>
  <div class="arrow">→</div>
  <div class="step active">.users[0].profile.</div>
</div>

Inside `to_entries` and `with_entries`, the suggestions automatically switch to `.key` and `.value`.

## Handle unusual field names

jq's `.field` shorthand only works for simple ASCII names. If a field name contains hyphens, spaces, starts with a digit, or uses non-ASCII characters, jiq inserts it in bracket notation automatically.

<div class="io-pair">
  <div>
    <div class="io-label">Field in your JSON</div>
    <div class="io-block">名前
café
my-field
2nd</div>
  </div>
  <div class="io-arrow">→</div>
  <div>
    <div class="io-label">What jiq inserts</div>
    <div class="io-block">.["名前"]
.["café"]
.["my-field"]
.["2nd"]</div>
  </div>
</div>

You don't need to think about this — jiq handles it for you.

## Tune suggestions for arrays with mixed shapes

When your JSON has an array whose elements don't all have the same fields, jiq samples up to 10 elements to build the suggestion list. If that's not enough, increase the sample size in `~/.config/jiq/config.toml`:

```toml
[autocomplete]
array_sample_size = 50   # default 10, range 1–1000
```

## All keys

| Key | Action |
|---|---|
| `↑` / `↓` | Move through the list |
| `Tab` | Accept the highlighted suggestion |
| `Esc` | Close the list |
| Mouse click | Highlight a suggestion |
| Mouse double-click | Accept a suggestion |
