---
title: Search in results
parent: Features
nav_order: 5
description: Find a specific value in the rendered output and step through every match.
---

# Search in results

Use search when you're looking for a specific value in a large result. jiq highlights every match as you type and lets you jump between them with a single key.

## Open search

Press **Ctrl+F** from anywhere, or press **/** when the results pane is focused.

A search bar appears at the bottom of the results pane.

<div class="tui-mockup with-title" data-title="Search active — 2 of 7 matches">
<pre>╭─ Results · 2/7 ─────────────────────────────────────╮
│ [                                                    │
│   {                                                  │
│     "email": "alice@example.com",  ← match 1         │
│     "email": "bob@example.com"     ← match 2 (cur)   │
│     ...                                              │
│ ]                                                    │
╰──────────────────────────────────────────────────────╯
╭─ Search ─────────────────────────────────────────────╮
│ /example                                             │
╰── Enter Next · N Prev · Esc Close ──────────────────╯</pre>
</div>

## Find and step through matches

1. Type your search term. jiq highlights every match in real time and scrolls to the first one. The counter in the title (`2/7`) shows your position.
2. Press **Enter** to confirm the search and jump to the next match.
3. Press **n** to move forward, **N** to move backward. Wraps around at the end.

Search is case-insensitive.

## Refine your search

To edit your search term after confirming it, press **Tab**, **Ctrl+F**, or **/** to return to editing mode.

## Drill into a matching row

With search open, you can drill into the row where a match lives — the same as drilling from the results pane normally.

1. Step to the match you want with **n** / **N**.
2. Press **`>`** to filter down to that value.

jiq applies the drill to the matched row and closes the search overlay.

## Close search

Press **Esc** to close the search bar and clear highlights.

## All keys

| Key | Action |
|---|---|
| `Ctrl+F` | Open search from any pane |
| `/` | Open search from results pane or NORMAL input |
| Type | Filter matches in real time |
| `Enter` | Confirm and jump to next match |
| `n` / `Enter` | Next match |
| `N` / `Shift+Enter` | Previous match |
| `Tab` | Return to editing the search term |
| `>` `*` `}` | Drill into the current match's row |
| `]` `[` | Jump cursor to next / prev sibling of the match's row |
| `Esc` | Close search |
