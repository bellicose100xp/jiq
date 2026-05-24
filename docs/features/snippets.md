---
title: Snippet library
parent: Features
nav_order: 4
description: Save and reuse jq queries across sessions, with fuzzy search and inline editing.
---

# Snippet library

[Features](./) · [Quick reference](../quick-reference)
{: .fs-3 .text-grey-dk-000 }

A persistent library of jq queries that you keep coming back to — `.users[] | select(.active)`, the timestamp-to-ISO converter you wrote once, the four-step pipeline that fits a particular API. Save it once, fuzzy-search and apply it on the next launch.

Snippets live on disk in TOML at `~/.config/jiq/snippets.toml` and persist across sessions. The preview pane shows the saved query with full jq syntax highlighting so you can scan a long pipeline at a glance.

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

The cursor row (`▸`) drives the preview. Typing characters narrows the list with fuzzy matching — `act` matches `active-users`, `inactive-count`, and anything else whose title contains those characters in order.

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

The file is auto-managed by jiq — saves, edits, and deletes from the popup write back here. Most users never open this file directly; reach for it only if you want to bulk-edit or sync snippets across machines.
{: .note }

---

## Workflows

### Save the current query as a snippet

1. Type the jq query you want to keep in the input field.
2. <kbd>Ctrl</kbd>+<kbd>S</kbd> — open the snippets popup.
3. <kbd>Ctrl</kbd>+<kbd>N</kbd> — start a new snippet pre-filled with your current input.
4. Fill in `Title` and `Description`. Use <kbd>Tab</kbd> / <kbd>Shift</kbd>+<kbd>Tab</kbd> to move between fields.
5. <kbd>Enter</kbd> — save and return to browse mode.

### Apply a saved snippet

1. <kbd>Ctrl</kbd>+<kbd>S</kbd> — open the snippets popup.
2. Type a few characters of the title to fuzzy-filter, or use <kbd>↑</kbd>/<kbd>↓</kbd> to navigate.
3. <kbd>Enter</kbd> — the snippet's query replaces the current input and runs immediately.

### Update an existing snippet's query

When you've evolved a snippet's query interactively and want to overwrite the saved version with what's currently in the input:

1. With your improved query in the input, press <kbd>Ctrl</kbd>+<kbd>S</kbd>.
2. Select the snippet you want to overwrite (filter or arrow-key to it).
3. <kbd>Ctrl</kbd>+<kbd>R</kbd> — replace that snippet's query with the current input. Title and description are preserved.

For full edit (title + description + query), use <kbd>Ctrl</kbd>+<kbd>E</kbd> instead.

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

- **Click** any row to select it and update the preview.
- **Double-click** a row to apply the snippet — same as selecting and pressing <kbd>Enter</kbd>.
- **Mouse wheel** scrolls the snippet list.
