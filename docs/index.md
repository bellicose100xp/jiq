---
title: Home
layout: home
nav_order: 1
description: "jiq — Interactive JSON query tool with real-time output. Type a jq query, see the results live, drill into nested values with a single keystroke."
permalink: /
---

# jiq
{: .fs-9 }

Interactive JSON query tool with real-time output. Type a jq query, see results live, drill into nested values with a single keystroke.
{: .fs-5 .fw-300 }

[Get started now](./getting-started){: .btn .btn-primary .fs-5 .mr-2 }
[Quick reference](./quick-reference){: .btn .fs-5 .mr-2 }
[View on GitHub](https://github.com/bellicose100xp/jiq){: .btn .fs-5 }

---

## Why jiq?

`jq` is the right tool for transforming JSON, but **building** the query is iterative — guess, edit, run, repeat. jiq turns that loop into a TUI: every keystroke re-runs `jq` against your data, so the result pane updates as fast as you can type. Then it adds the things `jq` itself doesn't do — autocomplete on your real schema, AI suggestions, drill-in navigation, snippets, history, and a clipboard-aware launch path so you can pipe almost anything into it.

<div class="feature-grid" markdown="1">

<div class="feature-card" markdown="1">
### [Path-at-cursor](./features/path-at-cursor)
Drill in / step back through nested values with `>` `<` `*` `^` `}`.
</div>

<div class="feature-card" markdown="1">
### [Autocomplete](./features/autocomplete)
Schema-aware field and function suggestions with type hints.
</div>

<div class="feature-card" markdown="1">
### [AI assistant](./features/ai-assistant)
Natural-language queries, error fixes, and suggestions powered by your model of choice.
</div>

<div class="feature-card" markdown="1">
### [Snippets](./features/snippets)
Save and reuse jq queries across sessions.
</div>

<div class="feature-card" markdown="1">
### [Search in results](./features/search)
Find and jump through matches in the output pane.
</div>

<div class="feature-card" markdown="1">
### [Query history](./features/history)
Searchable history of every successful query.
</div>

<div class="feature-card" markdown="1">
### [Mouse support](./features/mouse)
Click-to-focus, scroll, drag-select, click on suggestions.
</div>

<div class="feature-card" markdown="1">
### [Clipboard & paste recovery](./features/clipboard)
Auto-load from clipboard; in-app paste box when that fails.
</div>

<div class="feature-card" markdown="1">
### [VIM editing](./features/vim-editing)
Full motions, operators, text objects, undo/redo in the query input.
</div>

<div class="feature-card" markdown="1">
### [Results navigation](./features/results-pane)
Cursor + scroll + visual selection on the output pane.
</div>

<div class="feature-card" markdown="1">
### [Function tooltip](./features/tooltip)
Inline reference for any jq function under the cursor.
</div>

<div class="feature-card" markdown="1">
### [Configuration](./configuration)
`config.toml` reference: clipboard backend, AI providers, autocomplete depth.
</div>

</div>

---

## At a glance

```bash
# From a file
jiq data.json

# From stdin
curl https://api.example.com/data | jiq

# From the clipboard (no args, no pipe)
jiq
```

A single TUI window appears. Start typing — every keystroke is a jq query. Press <kbd>Enter</kbd> to print the filtered JSON to stdout, <kbd>Ctrl</kbd>+<kbd>Q</kbd> to print just the query string, <kbd>Ctrl</kbd>+<kbd>C</kbd> or <kbd>q</kbd> to quit silently.

[Install jiq →](./getting-started#install)
{: .fs-4 }
