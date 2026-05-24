---
title: Query history
parent: Features
nav_order: 6
description: Go back to any query you've run before, without retyping it.
---

# Query history

Every query that produces output is saved automatically. jiq keeps up to 1,000 entries, deduplicated, most recent first. History persists across sessions — closing and reopening jiq doesn't clear it.

## Go back to the previous query

Press **Ctrl+P** to step backward through recent queries, one at a time. The query input updates immediately and jiq re-runs it.

Press **Ctrl+N** to step forward again.

This works without opening any popup — useful when you just want to get back one or two queries.

## Find a specific query from your history

When you want to search further back:

1. Press **Ctrl+R** to open the history popup.
2. Type any part of the query to filter the list.
3. Use **↑** / **↓** to highlight the one you want.
4. Press **Enter** or **Tab** to apply it.

<div class="tui-mockup with-title" data-title="History popup — Ctrl+R">
<pre>┌─ History ───────────────────────────────────────┐
│ Filter: select                                  │
│                                                 │
│ ▸ .users[] | select(.active)             ✕      │
│   .events[] | select(.type == "click")          │
│   .data | map(select(.tier == "gold"))          │
│   .items[] | select(.price &gt; 100)               │
│                                                 │
└── Enter Apply · Ctrl+D Delete · Esc Close ─────┘</pre>
</div>

## Delete a history entry

To remove an entry you no longer want:

- In the popup, highlight it and press **Ctrl+D**.
- Or hover over any row with the mouse — a `✕` button appears on the right. Click it to delete that entry.

The popup closes automatically if you delete the last remaining entry.

## Where history is stored

| OS | File |
|---|---|
| Linux | `~/.local/share/jiq/history` |
| macOS | `~/Library/Application Support/jiq/history` |
| Windows | `%APPDATA%\jiq\history` |

One query per line. You can edit or clear this file directly if needed.

## All keys

| Key | Action |
|---|---|
| `Ctrl+P` | Go to the previous (older) query |
| `Ctrl+N` | Go to the next (newer) query |
| `Ctrl+R` | Open the history popup |
| `↑` / `↓` | Move through the list |
| Type | Filter the list |
| `Enter` / `Tab` | Apply the selected query |
| `Ctrl+D` | Delete the selected entry |
| `Esc` | Close the popup |
