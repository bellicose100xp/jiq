---
title: Tooltip & overlays
parent: Features
nav_order: 11
description: Quick-reference function tooltip, syntax error overlay, and the help popup with full keybind reference.
---

# Tooltip & overlays
{: .no_toc }

Three overlays without leaving the query input: an inline function reference, full syntax-error detail, and the in-app keybind cheat sheet.

<details open markdown="block">
<summary>Table of contents</summary>

1. TOC
{:toc}

</details>

---

## Function tooltip

With the cursor on a jq function name (`select`, `map`, `to_entries`, `group_by`, `sort_by`, etc.), <kbd>Ctrl</kbd>+<kbd>T</kbd> opens a card with the function's signature and a usage example.

<div class="tui-mockup" markdown="0">
<pre>
╭─ Input · INSERT ─────────────────────────────────────────╮
│ .users[] | select(.active)                               │
╰─────────────────────────────────────────────────────────╯
┌─ select ─────────────────────────────────────────────────┐
│ select(filter)                                           │
│   Pass through values where filter is truthy.            │
│                                                          │
│ Example:                                                 │
│   .[] | select(.age > 18)                                │
│                                                          │
│ Ctrl+T to close                                          │
└──────────────────────────────────────────────────────────┘
</pre>
</div>

Toggles off on the next <kbd>Ctrl</kbd>+<kbd>T</kbd> or when the cursor moves off the function name. Only triggers on recognized jq builtins.

---

## Error overlay

A syntax error turns the input border red and shows `⚠ Syntax Error` in the title bar. For the full jq error message with column and reason, press <kbd>Ctrl</kbd>+<kbd>E</kbd>.

<div class="tui-mockup" markdown="0">
<pre>
╭─ Input · INSERT · ⚠ Syntax Error ────────────────────────╮ (red border)
│ .users[] | select(.active                                │
╰─────────────────────────────────────────────────────────╯
┌─ Error ─────────────────────────────────────────────────┐
│ jq: error: syntax error, unexpected $end (Unix shell    │
│ quoting issues?) at &lt;top-level&gt;, line 1:                │
│ .users[] | select(.active                               │
│ jq: 1 compile error                                     │
│                                                          │
│ Ctrl+E to close                                         │
└──────────────────────────────────────────────────────────┘
</pre>
</div>

The previous successful result stays dimmed behind the overlay. Closes on the next <kbd>Ctrl</kbd>+<kbd>E</kbd> or as soon as the query parses cleanly. <kbd>Ctrl</kbd>+<kbd>E</kbd> is a no-op when the input is valid.

---

## Help popup

<kbd>F1</kbd> or <kbd>?</kbd> opens a tabbed keybind reference grouped by mode and pane (Global, Input · INSERT, Input · NORMAL, Results, Search, History, Snippets, AI). Click a tab or use arrow keys to switch sections.

<div class="tui-mockup" markdown="0">
<pre>
┌─ Help · Global  Input  Results  Search  History  Snippets  AI ────────┐
│                                                                       │
│  Global                                                               │
│  ─────────────────────────────────────────────────────────────────    │
│   F1 / ?           Toggle this help popup                             │
│   Shift+Tab        Switch focus between Input and Results             │
│   Ctrl+Y           Copy current query or results (focus-aware)        │
│   Ctrl+O           Copy results regardless of focus                   │
│   Ctrl+T           Toggle function tooltip                            │
│   Ctrl+E           Toggle error overlay                               │
│   Ctrl+A           Toggle AI assistant popup                          │
│   Enter            Exit and output filtered JSON                      │
│   Ctrl+Q           Exit and output query string only                  │
│   q / Ctrl+C       Quit without output                                │
│                                                                       │
│  F1/?/Esc to close                                                    │
└───────────────────────────────────────────────────────────────────────┘
</pre>
</div>

Close with <kbd>F1</kbd>, <kbd>?</kbd>, or <kbd>Esc</kbd>.

The in-app popup is the fast mid-flow lookup. This page and the [Quick reference](../quick-reference) cover edge cases and link to per-feature pages.

---

## Shortcuts

| Key | Action |
|-----|--------|
| <kbd>F1</kbd> / <kbd>?</kbd> | Toggle help popup |
| <kbd>Ctrl</kbd>+<kbd>T</kbd> | Toggle function tooltip (when cursor is on a function) |
| <kbd>Ctrl</kbd>+<kbd>E</kbd> | Toggle error overlay (when syntax error exists) |

{: .shortcuts }
