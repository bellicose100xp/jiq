---
title: Query history
parent: Features
nav_order: 6
description: Searchable, fuzzy-filterable history of every successful query, persisted across sessions.
---

# Query history

Every query that produced output is saved across sessions. Up to 1000 entries, deduplicated, most recent first.

## Where it's stored

| OS | Path |
|---|---|
| Linux | `~/.local/share/jiq/history` |
| macOS | `~/Library/Application Support/jiq/history` |
| Windows | `%APPDATA%\jiq\history` |

One query per line. Saved automatically; no flag needed.

## Cycling without the popup

From the input field, walk the history in place — no popup, no selection step.

| Key | Action |
|---|---|
| <kbd>Ctrl</kbd>+<kbd>P</kbd> | Older query |
| <kbd>Ctrl</kbd>+<kbd>N</kbd> | Newer query |
{: .shortcuts }

## The history popup

<kbd>Ctrl</kbd>+<kbd>R</kbd> opens a fuzzy-filterable list of recent queries.

<div class="tui-mockup with-title" data-title="Ctrl+R — history popup">
<pre>┌─ History ──────────────────────────────────────────────────┐
│ Filter: select                                             │
│                                                            │
│ ▸ .users[] | select(.active == true)                  ✕    │
│   .events[] | select(.type == "click") | length            │
│   .data | map(select(.tier == "gold"))                     │
│   .items[] | select(.price &gt; 100) | .name                  │
│                                                            │
└── Enter Apply · Ctrl+D Delete · Esc Close ────────────────┘</pre>
</div>

Type to filter. <kbd>Enter</kbd> or <kbd>Tab</kbd> applies the highlighted entry to the input and runs it. <kbd>Ctrl</kbd>+<kbd>D</kbd> removes the highlighted entry from disk. The popup closes when the last entry is deleted.

Hovering a row reveals an `✕` on the right edge — click it to delete that row directly.

## Shortcuts

| Key | Action |
|---|---|
| <kbd>Ctrl</kbd>+<kbd>R</kbd> | Open the popup |
| <kbd>↑</kbd> / <kbd>↓</kbd> | Move selection |
| Type chars | Fuzzy filter |
| <kbd>Enter</kbd> / <kbd>Tab</kbd> | Apply selection |
| <kbd>Ctrl</kbd>+<kbd>D</kbd> | Delete selected |
| Click `✕` | Delete hovered row |
| <kbd>Esc</kbd> | Close |
{: .shortcuts }
