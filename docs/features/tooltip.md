---
title: Tooltip & overlays
parent: Features
nav_order: 11
description: Quick-reference function tooltip, syntax error overlay, and the help popup with full keybind reference.
---

# Tooltip & overlays
{: .no_toc }

Three pop-up overlays surface contextual information without leaving the query input: an inline function reference, a full syntax-error detail, and the in-app keybind cheat sheet.

[Features](./){: .btn .btn-outline .mr-2 }
[Quick reference](../quick-reference){: .btn .btn-outline }

<details open markdown="block">
<summary>Table of contents</summary>

1. TOC
{:toc}

</details>

---

## Function tooltip

When the cursor sits on a jq function name (e.g., `select`, `map`, `to_entries`, `group_by`, `sort_by`), press <kbd>Ctrl</kbd>+<kbd>T</kbd> to pop up a brief reference card showing the function's signature and a usage example.

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

The tooltip toggles off when you press <kbd>Ctrl</kbd>+<kbd>T</kbd> again or move the cursor off the function name. It only appears when the cursor is positioned on a recognized jq builtin — typing or navigating through plain text is a no-op.

{: .tip }
> Use the tooltip mid-query as a memory jog instead of breaking flow to look up jq's manual. It's most useful for the higher-arity builtins like `reduce`, `foreach`, `walk`, and `paths`.

---

## Error overlay

A query with a syntax error gets two visual cues right away: the input border turns red, and the title bar shows the `⚠ Syntax Error` badge. That's usually enough to spot the problem (a missing paren, a stray `|`). For the **full jq error message** with the offending column and reason, press <kbd>Ctrl</kbd>+<kbd>E</kbd> to toggle the detail overlay.

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

The **previous successful result** stays visible behind the overlay (dimmed) so you can keep reading the JSON you were exploring while you fix the typo. The overlay closes on the next <kbd>Ctrl</kbd>+<kbd>E</kbd> or as soon as your query parses cleanly again.

{: .note }
> The compact `⚠ Syntax Error` badge is shown in the input title regardless of whether the overlay is open — the overlay just expands the diagnosis. If the input is still valid, <kbd>Ctrl</kbd>+<kbd>E</kbd> is a no-op.

---

## Help popup

Press <kbd>F1</kbd> or <kbd>?</kbd> anywhere in jiq to open the in-app keybind reference. It's a tabbed popup grouped by mode and pane (Global, Input · INSERT, Input · NORMAL, Results pane, Search, History, Snippets, AI). Click a tab — or use the arrow keys — to switch sections.

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

{: .tip }
> The in-app popup is the **fast lookup** — what you reach for mid-flow when you forget a chord. This page (and the [Quick reference](../quick-reference)) is the **deeper reference** with examples, edge cases, and links to the full per-feature pages. Use whichever fits the moment.

---

## Shortcuts

| Key | Action |
|-----|--------|
| <kbd>F1</kbd> / <kbd>?</kbd> | Toggle help popup |
| <kbd>Ctrl</kbd>+<kbd>T</kbd> | Toggle function tooltip (when cursor is on a function) |
| <kbd>Ctrl</kbd>+<kbd>E</kbd> | Toggle error overlay (when syntax error exists) |

{: .shortcuts }
