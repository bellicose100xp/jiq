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

```json
{ "users": [
  { "name": "alice", "active": true,  "email": "alice@example.com" },
  { "name": "bob",   "active": false, "email": "bob@example.com" },
  { "name": "carol", "active": true,  "email": "carol@example.com" }
] }
```

```bash
jiq users.json
```

1. Type `.users[]` — all three users stream into the results pane.
2. Add ` | select(.active)` — Bob disappears.
3. Add ` | .email` — two emails left.
4. <kbd>Enter</kbd> prints the filtered JSON. <kbd>Ctrl</kbd>+<kbd>Q</kbd> prints the query string.
