---
title: Getting started
nav_order: 2
description: Install jiq, run your first query, learn the loop in two minutes.
---

# Getting started

Install jiq, run a query, see the loop.

## Requirements

- **`jq`** 1.6+ (1.8.1+ recommended). [Install](https://jqlang.org/download/).

## Install

### Homebrew — macOS

```bash
brew install bellicose100xp/tap/jiq
```

### Install script — macOS, Linux

```bash
curl --proto '=https' --tlsv1.2 -LsSf \
  https://github.com/bellicose100xp/jiq/releases/latest/download/jiq-installer.sh | sh
```

### Cargo — any platform with Rust

```bash
cargo install jiq
```

### Pre-built binary — Windows, Linux, macOS

Download from the [releases page](https://github.com/bellicose100xp/jiq/releases/latest), unpack, and put the `jiq` (or `jiq.exe`) binary on your `PATH`.

### From source — any platform with Rust

```bash
git clone https://github.com/bellicose100xp/jiq
cd jiq && cargo build --release
# Linux / macOS:
sudo cp target/release/jiq /usr/local/bin/
# Windows: copy target\release\jiq.exe to a directory on PATH
```

## Input

```bash
jiq data.json                              # file
cat data.json | jiq                        # stdin
jiq                                        # clipboard, with paste-box fallback
```

With no file and no piped stdin, jiq reads from the clipboard; if it's empty or invalid, the in-app paste box opens.

## First query

Save the following as `users.json` and run `jiq users.json`:

```json
{ "users": [
  { "name": "alice", "active": true,  "email": "alice@example.com" },
  { "name": "bob",   "active": false, "email": "bob@example.com" },
  { "name": "carol", "active": true,  "email": "carol@example.com" }
] }
```

**Step 1 — type `.users[]`**

<div class="tui-mockup with-title" data-title="Query · .users[]">
<pre>╭─ Input [INSERT] ───────────────────────────────╮
│ .users[]                                        │
╰─────────────────────────────────────────────────╯
╭─ Array [3] · .[] ──────────────────────────────╮
│ {"name":"alice","active":true,"email":"alice…"} │
│ {"name":"bob","active":false,"email":"bob@…"}   │
│ {"name":"carol","active":true,"email":"carol…"} │
╰─────────────────────────────────────────────────╯</pre>
</div>

**Step 2 — add `| select(.active)`**

<div class="tui-mockup with-title" data-title="Query · .users[] | select(.active)">
<pre>╭─ Input [INSERT] ───────────────────────────────╮
│ .users[] | select(.active)                      │
╰─────────────────────────────────────────────────╯
╭─ Object · .[] ─────────────────────────────────╮
│ {"name":"alice","active":true,"email":"alice…"} │
│ {"name":"carol","active":true,"email":"carol…"} │
╰─────────────────────────────────────────────────╯</pre>
</div>

**Step 3 — add `| .email`**

<div class="tui-mockup with-title" data-title="Query · .users[] | select(.active) | .email">
<pre>╭─ Input [INSERT] ───────────────────────────────╮
│ .users[] | select(.active) | .email             │
╰─────────────────────────────────────────────────╯
╭─ String · .email ──────────────────────────────╮
│ "alice@example.com"                             │
│ "carol@example.com"                             │
╰─────────────────────────────────────────────────╯</pre>
</div>

Press <kbd>Enter</kbd> to exit and print the filtered JSON to stdout. Press <kbd>Ctrl</kbd>+<kbd>Q</kbd> to print just the query string instead.
