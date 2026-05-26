---
title: Autocomplete
parent: Features
nav_order: 2
description: Get field name and function suggestions as you type, drawn from your actual JSON.
---

# Autocomplete

Stop guessing field names. jiq shows every available field and function as you type, pulled directly from your loaded JSON data.

<div class="before-after">
  <input type="radio" name="ba-autocomplete" id="ba-autocomplete-before" checked>
  <input type="radio" name="ba-autocomplete" id="ba-autocomplete-after">
  <div class="ba-header">
    <label for="ba-autocomplete-before" class="ba-toggle">Without autocomplete</label>
    <label for="ba-autocomplete-after" class="ba-toggle">With jiq autocomplete</label>
  </div>
  <div class="ba-state">
    <p class="ba-caption">You guess at field names, hit errors, scroll through raw JSON to find what you need:</p>
    <div class="ba-terminal">$ cat data.json | jq '.users[0].emial'
null

$ cat data.json | jq '.users[0].mail'
null

$ cat data.json | jq '.users[0]' | less
# scroll... scroll... found it: "email"

$ cat data.json | jq '.users[0].email'
"alice@example.com"</div>
  </div>
  <div class="ba-state">
    <p class="ba-caption">Type a dot and see every field with its type. Tab to insert. Navigate deeper instantly:</p>
    <div class="ba-terminal">Query: .users[0].

  name           String
  age            Number
  email          String      &lt;-- Tab
  tags           Array[String]
  profile        Object
  active         Boolean

Query: .users[0].email
"alice@example.com"</div>
  </div>
</div>

## Accept a suggestion

When the suggestion list appears below the input:

1. Use <kbd>Up</kbd> / <kbd>Down</kbd> to highlight the entry you want.
2. Press <kbd>Tab</kbd> to insert it into the query.
3. Press <kbd>Esc</kbd> to dismiss without accepting.

<div class="animated-terminal">
  <div class="terminal-chrome">
    <span class="dot red"></span>
    <span class="dot yellow"></span>
    <span class="dot green"></span>
    <span class="terminal-title">Field suggestions after typing .users[0].</span>
  </div>
  <div class="terminal-body">
    <div class="term-line"><span class="term-dim">Query:</span> <span class="term-highlight">.users[0].</span><span class="term-cursor"></span></div>
    <div class="term-line">&nbsp;</div>
    <div class="term-line"><span class="term-output">  name           </span><span class="term-dim">String</span></div>
    <div class="term-line"><span class="term-output">  age            </span><span class="term-dim">Number</span></div>
    <div class="term-line"><span class="term-highlight">  email          </span><span class="term-dim">String</span></div>
    <div class="term-line"><span class="term-output">  tags           </span><span class="term-dim">Array[String]</span></div>
    <div class="term-line"><span class="term-output">  profile        </span><span class="term-dim">Object</span></div>
    <div class="term-line"><span class="term-output">  active         </span><span class="term-dim">Boolean</span></div>
  </div>
</div>

## Understand suggestion types

Each suggestion shows a label on the right indicating what it is. These come from the actual values in your data:

| Kind | What it means | Example |
|---|---|---|
| **Field** | A key from your JSON, with its value type | `name` String |
| **Function** | A jq built-in; auto-inserts `(` for functions that take arguments | `select(`, `map(`, `keys` |
| **Operator** | Pipe and comparison tokens | `\|`, `==`, `!=` |
| **Variable** | `$name` bindings from your query, plus `$ENV` and `$__loc__` | `$item`, `$ENV` |
| **Iterator** | Array iterator in path-flow contexts | `[]` |

## Navigate deeper paths

Suggestions narrow as you type deeper into a path. Type `.users[0].profile.` and jiq shows only the fields inside `profile`.

<div class="drill-chain">
  <div class="step">.</div>
  <div class="arrow">.</div>
  <div class="step">.users</div>
  <div class="arrow">.</div>
  <div class="step">.users[0]</div>
  <div class="arrow">.</div>
  <div class="step active">.users[0].profile.</div>
</div>

Inside `to_entries` and `with_entries`, the suggestions automatically switch to `.key` and `.value` — matching the shape jq produces in those contexts.

## Handle unusual field names

jq's `.field` shorthand only works for simple ASCII identifiers. If a field name contains hyphens, spaces, starts with a digit, or uses non-ASCII characters, jiq inserts bracket notation automatically.

<div class="io-pair">
  <div>
    <div class="io-label">Field in your JSON</div>
    <div class="io-block">my-field
2nd-attempt
cafe
user name</div>
  </div>
  <div class="io-arrow">-></div>
  <div>
    <div class="io-label">What jiq inserts</div>
    <div class="io-block">.["my-field"]
.["2nd-attempt"]
.["cafe"]
.["user name"]</div>
  </div>
</div>

You don't need to think about this — jiq picks the right notation for you.

## Use function suggestions

When your cursor is after a pipe `|` or at the start of an expression, jiq suggests jq built-in functions. Functions that take arguments auto-insert the opening parenthesis:

<div class="animated-terminal">
  <div class="terminal-chrome">
    <span class="dot red"></span>
    <span class="dot yellow"></span>
    <span class="dot green"></span>
    <span class="terminal-title">Function suggestions after typing .users | sel</span>
  </div>
  <div class="terminal-body">
    <div class="term-line"><span class="term-dim">Query:</span> <span class="term-highlight">.users | sel</span><span class="term-cursor"></span></div>
    <div class="term-line">&nbsp;</div>
    <div class="term-line"><span class="term-highlight">  select(       </span><span class="term-dim">Function</span></div>
    <div class="term-line"><span class="term-output">  setpath(       </span><span class="term-dim">Function</span></div>
    <div class="term-line">&nbsp;</div>
    <div class="term-line"><span class="term-dim">Tab inserts:</span> <span class="term-output">.users | select(</span></div>
  </div>
</div>

## Tune suggestions for mixed-shape arrays

When your JSON has an array whose elements don't all share the same fields, jiq samples up to 10 elements to build the suggestion list. If that's not enough to see all fields, increase the sample size in `~/.config/jiq/config.toml`:

```toml
[autocomplete]
array_sample_size = 50   # default 10, range 1-1000
```

Higher values scan more elements for field discovery but add a small performance cost.

## All keys

| Key | Action |
|---|---|
| <kbd>Up</kbd> / <kbd>Down</kbd> | Move through the suggestion list |
| <kbd>Tab</kbd> | Accept the highlighted suggestion |
| <kbd>Esc</kbd> | Dismiss the list |
| Mouse click | Highlight a suggestion |
| Mouse double-click | Accept a suggestion |
