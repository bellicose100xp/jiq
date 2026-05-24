---
title: Snippet library
parent: Features
nav_order: 4
description: Save your most-used jq queries by name and apply them instantly.
---

# Snippet library

A snippet is a saved jq query with a name and an optional description. Use snippets for queries you run repeatedly — instead of retyping them or hunting through history.

Snippets are stored as plain text at `~/.config/jiq/snippets.toml` and persist across sessions.

## Apply a saved snippet

1. Press **Ctrl+S** to open the snippet library.
2. Type any part of the name to filter the list.
3. Use **↑** / **↓** to highlight the snippet you want.
4. Press **Enter** to apply it.

<div class="tui-mockup with-title" data-title="Snippet library — Ctrl+S">
<pre>╭─ Snippets ─────────────────────────────────────╮
│ Filter: act                                    │
│                                                │
│ ▸ active-users      .users[] | select(.act...  │
│   active-emails     .users[] | select(.act...  │
│   inactive-count    [.users[] | select(...)... │
│                                                │
│ ┌─ Preview ────────────────────────────────┐   │
│ │ active-users                              │  │
│ │ .users[] | select(.active == true)        │  │
│ └───────────────────────────────────────────┘  │
│ Enter Apply · Ctrl+N New · Ctrl+D Delete       │
╰────────────────────────────────────────────────╯</pre>
</div>

## Save the current query as a snippet

1. Press **Ctrl+S** to open the library.
2. Press **Ctrl+N** to open the create form.
3. Type a name for the snippet, then press **Tab** to move to the next field.
4. The query field is pre-filled with your current input — edit if needed.
5. Optionally add a description, then press **Enter** to save.

## Edit a snippet

1. Press **Ctrl+S** to open the library.
2. Highlight the snippet you want to change.
3. Press **Ctrl+E** to open the edit form.
4. Use **Tab** / **Shift+Tab** to move between the name, query, and description fields.
5. Press **Enter** to save your changes, or **Esc** to cancel.

## Update a snippet's query to match your current input

If you've refined a query and want to update the saved version:

1. Press **Ctrl+S** to open the library.
2. Highlight the snippet to update.
3. Press **Ctrl+R** to replace its query with your current input.

## Delete a snippet

1. Press **Ctrl+S** to open the library.
2. Highlight the snippet.
3. Press **Ctrl+D** to delete it.

## File format

Snippets are stored as TOML. You can edit the file directly if you prefer:

```toml
[[snippets]]
name        = "active-users"
query       = ".users[] | select(.active == true)"
description = "Users where .active is true"

[[snippets]]
name  = "by-name-asc"
query = ".users | sort_by(.name)"
```

## All keys

### In the library

| Key | Action |
|---|---|
| `Ctrl+S` | Open or close |
| `↑` / `↓` | Move through the list |
| Type | Filter by name |
| `Enter` | Apply the selected snippet |
| `Ctrl+N` | Create a new snippet from the current query |
| `Ctrl+E` | Edit the selected snippet |
| `Ctrl+R` | Replace the selected snippet's query with current input |
| `Ctrl+D` | Delete the selected snippet |
| `Esc` | Close |

### In the create / edit form

| Key | Action |
|---|---|
| `Tab` / `Shift+Tab` | Move between fields |
| `Enter` | Save |
| `Esc` | Cancel |
