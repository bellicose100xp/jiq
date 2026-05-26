---
title: Snippet library
parent: Features
nav_order: 4
description: Save your most-used jq queries by name and apply them instantly.
---

# Snippet library

Save queries you run repeatedly and recall them by name in two keystrokes.

<div class="before-after">
  <input type="radio" name="ba-snippets" id="ba-snippets-before" checked>
  <input type="radio" name="ba-snippets" id="ba-snippets-after">
  <div class="ba-header">
    <label for="ba-snippets-before" class="ba-toggle">Without snippets</label>
    <label for="ba-snippets-after" class="ba-toggle">With snippets</label>
  </div>
  <div class="ba-state">
    <p class="ba-caption">You retype the same query from memory every session, fixing typos along the way.</p>
    <div class="ba-terminal">.users[] | select(.active == true) | {name, email}
<span class="term-dim">-- typed from scratch, again --</span>
<span class="term-error">error: unexpected ")" at line 1</span>
<span class="term-dim">-- fix typo, retry --</span>
.users[] | select(.active == true) | {name, email}</div>
  </div>
  <div class="ba-state">
    <p class="ba-caption">Open the library, type a few letters, press Enter. The query is applied instantly.</p>
    <div class="ba-terminal"><span class="term-dim">Press</span> <span class="term-highlight">Ctrl+S</span>
<span class="term-dim">Filter:</span> act
<span class="term-dim">  ▸ active-users    .users[] | select(.active == true) | {name, email}</span>
<span class="term-dim">Press</span> <span class="term-highlight">Enter</span>
<span class="term-success">Snippet applied.</span></div>
  </div>
</div>

---

## Apply a saved snippet

<div class="animated-terminal">
  <div class="terminal-chrome">
    <span class="dot red"></span>
    <span class="dot yellow"></span>
    <span class="dot green"></span>
    <span class="terminal-title">Snippet library</span>
  </div>
  <div class="terminal-body">
    <div class="term-line"><span class="term-dim">Press</span> <span class="term-highlight">Ctrl+S</span> <span class="term-dim">to open</span></div>
    <div class="term-line">&nbsp;</div>
    <div class="term-line"><span class="term-dim">Filter:</span> <span class="term-input">act</span><span class="term-cursor"></span></div>
    <div class="term-line">&nbsp;</div>
    <div class="term-line"><span class="term-highlight">▸ active-users</span>      <span class="term-output">.users[] | select(.active == true) | {name, email}</span></div>
    <div class="term-line"><span class="term-dim">  active-emails</span>    <span class="term-output">.users[] | select(.active) | .email</span></div>
    <div class="term-line">&nbsp;</div>
    <div class="term-line"><span class="term-dim">Press</span> <span class="term-highlight">Enter</span> <span class="term-dim">to apply</span></div>
  </div>
</div>

1. Press **Ctrl+S** to open the snippet library.
2. Type any part of the name to filter the list.
3. Use **Up** / **Down** to highlight the one you want.
4. Press **Enter** to apply it to the query input.

---

## Save the current query

<div class="step-flow">
  <div class="step-item done">
    <div class="step-circle">1</div>
    <div class="step-text">Ctrl+S</div>
    <div class="step-connector"></div>
  </div>
  <div class="step-item done">
    <div class="step-circle">2</div>
    <div class="step-text">Ctrl+N</div>
    <div class="step-connector"></div>
  </div>
  <div class="step-item done">
    <div class="step-circle">3</div>
    <div class="step-text">Name it</div>
    <div class="step-connector"></div>
  </div>
  <div class="step-item done">
    <div class="step-circle">4</div>
    <div class="step-text">Tab</div>
    <div class="step-connector"></div>
  </div>
  <div class="step-item active">
    <div class="step-circle">5</div>
    <div class="step-text">Enter</div>
  </div>
</div>

1. Press **Ctrl+S** to open the library.
2. Press **Ctrl+N** to create a new snippet.
3. Type a short name (e.g. `active-users`).
4. Press **Tab** — the query field is pre-filled with your current input. Edit it or leave it.
5. Optionally **Tab** again to add a description.
6. Press **Enter** to save.

---

## Edit or update a snippet

To **edit** a snippet's name, query, or description:

1. Press **Ctrl+S** to open the library.
2. Highlight the snippet with **Up** / **Down**.
3. Press **Ctrl+E** to open the edit form.
4. Use **Tab** / **Shift+Tab** to move between fields.
5. Press **Enter** to save, or **Esc** to cancel.

To **replace** a snippet's query with whatever is in the input right now:

1. Open the library with **Ctrl+S**.
2. Highlight the snippet.
3. Press **Ctrl+R** — the snippet's query is replaced immediately.

---

## Delete a snippet

Open the library (**Ctrl+S**), highlight the snippet, press **Ctrl+D**.

---

## Workflows at a glance

<div class="tab-container">
  <input type="radio" name="snippet-flow" id="snippet-tab-browse" checked>
  <input type="radio" name="snippet-flow" id="snippet-tab-create">
  <input type="radio" name="snippet-flow" id="snippet-tab-edit">
  <div class="tab-bar">
    <label for="snippet-tab-browse" class="tab-label">Browse</label>
    <label for="snippet-tab-create" class="tab-label">Create</label>
    <label for="snippet-tab-edit" class="tab-label">Edit</label>
  </div>
  <div class="tab-panel">
    <div class="animated-terminal">
      <div class="terminal-chrome">
        <span class="dot red"></span>
        <span class="dot yellow"></span>
        <span class="dot green"></span>
        <span class="terminal-title">Browse mode</span>
      </div>
      <div class="terminal-body">
        <div class="term-line"><span class="term-highlight">Ctrl+S</span> <span class="term-dim">Open library</span></div>
        <div class="term-line"><span class="term-dim">Type to filter, Up/Down to navigate</span></div>
        <div class="term-line">&nbsp;</div>
        <div class="term-line"><span class="term-highlight">▸ active-users</span>       <span class="term-output">.users[] | select(.active == true)</span></div>
        <div class="term-line"><span class="term-dim">  by-name-asc</span>        <span class="term-output">.users | sort_by(.name)</span></div>
        <div class="term-line"><span class="term-dim">  extract-emails</span>     <span class="term-output">[.users[].email]</span></div>
        <div class="term-line">&nbsp;</div>
        <div class="term-line"><span class="term-highlight">Enter</span> <span class="term-dim">Apply</span>  <span class="term-highlight">Esc</span> <span class="term-dim">Close</span></div>
      </div>
    </div>
  </div>
  <div class="tab-panel">
    <div class="animated-terminal">
      <div class="terminal-chrome">
        <span class="dot red"></span>
        <span class="dot yellow"></span>
        <span class="dot green"></span>
        <span class="terminal-title">Create mode</span>
      </div>
      <div class="terminal-body">
        <div class="term-line"><span class="term-highlight">Ctrl+N</span> <span class="term-dim">from browse mode</span></div>
        <div class="term-line">&nbsp;</div>
        <div class="term-line"><span class="term-dim">Name:</span>  <span class="term-input">active-users</span><span class="term-cursor"></span></div>
        <div class="term-line"><span class="term-dim">Query:</span> <span class="term-output">.users[] | select(.active == true)</span></div>
        <div class="term-line"><span class="term-dim">Desc:</span>  <span class="term-output">Users where .active is true</span></div>
        <div class="term-line">&nbsp;</div>
        <div class="term-line"><span class="term-highlight">Tab</span> <span class="term-dim">Next field</span>  <span class="term-highlight">Enter</span> <span class="term-dim">Save</span>  <span class="term-highlight">Esc</span> <span class="term-dim">Cancel</span></div>
      </div>
    </div>
  </div>
  <div class="tab-panel">
    <div class="animated-terminal">
      <div class="terminal-chrome">
        <span class="dot red"></span>
        <span class="dot yellow"></span>
        <span class="dot green"></span>
        <span class="terminal-title">Edit mode</span>
      </div>
      <div class="terminal-body">
        <div class="term-line"><span class="term-highlight">Ctrl+E</span> <span class="term-dim">on selected snippet</span></div>
        <div class="term-line">&nbsp;</div>
        <div class="term-line"><span class="term-dim">Name:</span>  <span class="term-input">active-users</span></div>
        <div class="term-line"><span class="term-dim">Query:</span> <span class="term-input">.users[] | select(.active == true) | {name, email}</span><span class="term-cursor"></span></div>
        <div class="term-line"><span class="term-dim">Desc:</span>  <span class="term-input">Active users with name and email</span></div>
        <div class="term-line">&nbsp;</div>
        <div class="term-line"><span class="term-highlight">Tab</span> <span class="term-dim">Next field</span>  <span class="term-highlight">Enter</span> <span class="term-dim">Save</span>  <span class="term-highlight">Esc</span> <span class="term-dim">Cancel</span></div>
      </div>
    </div>
  </div>
</div>

---

## File format

Snippets are stored as plain TOML at `~/.config/jiq/snippets.toml`. You can edit it directly:

```toml
[[snippets]]
name        = "active-users"
query       = ".users[] | select(.active == true)"
description = "Users where .active is true"

[[snippets]]
name  = "by-name-asc"
query = ".users | sort_by(.name)"
```

---

## All keys

### Browse mode

| Key | Action |
|---|---|
| `Ctrl+S` | Open or close |
| `Up` / `Down` | Navigate the list |
| Type | Fuzzy filter by name |
| `Enter` | Apply the selected snippet |
| `Ctrl+N` | Create a new snippet from current query |
| `Ctrl+E` | Edit the selected snippet |
| `Ctrl+R` | Replace selected snippet's query with current input |
| `Ctrl+D` | Delete the selected snippet |
| `Esc` | Close |

### Create / edit mode

| Key | Action |
|---|---|
| `Tab` / `Shift+Tab` | Move between fields |
| `Enter` | Save |
| `Esc` | Cancel |
