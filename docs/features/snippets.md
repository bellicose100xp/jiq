---
title: Snippet library
parent: Features
nav_order: 4
description: Save and reuse jq queries across sessions, with fuzzy search and inline editing.
---

# Snippet library

A persistent library of jq queries. Saved to `~/.config/jiq/snippets.toml`. Fuzzy-search and apply across sessions. The preview pane shows the saved query with jq syntax highlighting.

---

## What it looks like

<div class="tui-mockup with-title" data-title="Ctrl+S — Snippets popup with fuzzy filter">

```
╭─ Snippets ───────────────────────────────────────────────╮
│ Filter: act                                              │
│                                                          │
│ ▸ active-users          .users[] | select(.active)       │
│   active-with-emails    .users[] | select(.active) | ... │
│   inactive-count        [.users[] | select(.active==fa..)│
│                                                          │
│ ┌─ Preview ─────────────────────────────────────────────┐│
│ │ Title: active-users                                   ││
│ │ .users[] | select(.active == true)                    ││
│ └───────────────────────────────────────────────────────┘│
│ Enter Apply · Ctrl+N New · Ctrl+E Edit · Ctrl+D Delete   │
╰──────────────────────────────────────────────────────────╯
```

</div>

The cursor row (`▸`) drives the preview. Typing fuzzy-filters by title — `act` matches `active-users`, `inactive-count`, and any title containing those characters in order.

---

## Snippet shape on disk

```toml
[[snippets]]
title = "active-users"
query = ".users[] | select(.active == true)"
description = "Filter to users where .active is true"

[[snippets]]
title = "by-name-asc"
query = ".users | sort_by(.name)"
```

jiq writes back to this file on save / edit / delete. Open it directly only for bulk edits or syncing across machines.
{: .note }

---

## Workflows

### Save the current query as a snippet

1. Type the jq query in the input.
2. <kbd>Ctrl</kbd>+<kbd>S</kbd> — open the popup.
3. <kbd>Ctrl</kbd>+<kbd>N</kbd> — new snippet pre-filled with the current input.
4. Fill in `Title` and `Description`. <kbd>Tab</kbd> / <kbd>Shift</kbd>+<kbd>Tab</kbd> moves between fields.
5. <kbd>Enter</kbd> saves.

### Apply a saved snippet

1. <kbd>Ctrl</kbd>+<kbd>S</kbd>.
2. Fuzzy-filter or arrow-key to the entry.
3. <kbd>Enter</kbd> — the snippet's query replaces the current input and runs.

### Update an existing snippet's query

1. With the improved query in the input, <kbd>Ctrl</kbd>+<kbd>S</kbd>.
2. Select the snippet to overwrite.
3. <kbd>Ctrl</kbd>+<kbd>R</kbd> — replaces its query with the current input. Title and description are preserved.

For full edit (title + description + query), use <kbd>Ctrl</kbd>+<kbd>E</kbd>.

---

## Keybindings

### Browse mode

| Key | Action |
|---|---|
| <kbd>Ctrl</kbd>+<kbd>S</kbd> | Open snippets popup |
| <kbd>↑</kbd> / <kbd>↓</kbd> | Navigate snippets |
| Type characters | Fuzzy-filter by title |
| <kbd>Enter</kbd> | Apply selected snippet |
| <kbd>Ctrl</kbd>+<kbd>N</kbd> | Create new snippet from current query |
| <kbd>Ctrl</kbd>+<kbd>E</kbd> | Edit selected snippet (title, description, query) |
| <kbd>Ctrl</kbd>+<kbd>R</kbd> | Replace selected snippet's query with current input |
| <kbd>Ctrl</kbd>+<kbd>D</kbd> | Delete selected snippet |
| <kbd>Esc</kbd> | Close popup |

{: .shortcuts }

### Create / Edit mode

| Key | Action |
|---|---|
| <kbd>Tab</kbd> / <kbd>Shift</kbd>+<kbd>Tab</kbd> | Move between fields |
| <kbd>Enter</kbd> | Save |
| <kbd>Esc</kbd> | Cancel without saving |

{: .shortcuts }

---

## Mouse

- Click — select row, update preview.
- Double-click — apply (same as <kbd>Enter</kbd>).
- Mouse wheel — scroll the list.
