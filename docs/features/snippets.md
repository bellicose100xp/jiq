---
title: Snippet library
parent: Features
nav_order: 4
description: Save and reuse jq queries across sessions, with fuzzy search.
---

# Snippet library

A persistent library of named jq queries. <kbd>Ctrl</kbd>+<kbd>S</kbd> opens it from anywhere.

Stored as TOML at `~/.config/jiq/snippets.toml`. Each snippet has a name, a query, and an optional description.

```toml
[[snippets]]
name = "active-users"
query = ".users[] | select(.active == true)"
description = "Users where .active is true"

[[snippets]]
name = "by-name-asc"
query = ".users | sort_by(.name)"
```

## Browse and apply

<div class="tui-mockup with-title" data-title="Ctrl+S — snippet library">
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

Type any characters to filter — fuzzy match against names, ranked by score. <kbd>Enter</kbd> replaces the current query with the selected snippet's query and runs it.

## Create, edit, replace

| Action | Steps |
|---|---|
| **Save current query** | <kbd>Ctrl</kbd>+<kbd>S</kbd> → <kbd>Ctrl</kbd>+<kbd>N</kbd> → fill name / description → <kbd>Enter</kbd> |
| **Edit a snippet** | <kbd>Ctrl</kbd>+<kbd>S</kbd> → highlight → <kbd>Ctrl</kbd>+<kbd>E</kbd> → step through fields with <kbd>Tab</kbd> → <kbd>Enter</kbd> |
| **Replace a snippet's query with current input** | <kbd>Ctrl</kbd>+<kbd>S</kbd> → highlight → <kbd>Ctrl</kbd>+<kbd>R</kbd> → confirm |
| **Delete** | <kbd>Ctrl</kbd>+<kbd>S</kbd> → highlight → <kbd>Ctrl</kbd>+<kbd>D</kbd> → confirm |

In create / edit mode, <kbd>Tab</kbd> and <kbd>Shift</kbd>+<kbd>Tab</kbd> walk forward and back through the name → query → description fields. <kbd>Enter</kbd> saves; <kbd>Esc</kbd> cancels.

## Shortcuts
{: .shortcuts }

### Browse

| Key | Action |
|---|---|
| <kbd>Ctrl</kbd>+<kbd>S</kbd> | Open / close |
| <kbd>↑</kbd> / <kbd>↓</kbd> | Navigate |
| Type chars | Fuzzy filter |
| <kbd>Enter</kbd> | Apply |
| <kbd>Ctrl</kbd>+<kbd>N</kbd> | New from current query |
| <kbd>Ctrl</kbd>+<kbd>E</kbd> | Edit selected |
| <kbd>Ctrl</kbd>+<kbd>R</kbd> | Replace selected query with current input |
| <kbd>Ctrl</kbd>+<kbd>D</kbd> | Delete selected |
| <kbd>Esc</kbd> | Close |

### Create / edit

| Key | Action |
|---|---|
| <kbd>Tab</kbd> / <kbd>Shift</kbd>+<kbd>Tab</kbd> | Walk fields |
| <kbd>Enter</kbd> | Save |
| <kbd>Esc</kbd> | Cancel |
