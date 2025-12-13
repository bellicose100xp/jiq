# UX Improvement Recommendations for jiq
**Research Date:** December 2025
**Purpose:** High-impact user experience improvements based on competitive analysis and user feedback

## Executive Summary
Based on research of similar tools (jnv, jless, fx, jqp) and analysis of jq user complaints, the following improvements would have immediate and large impact on jiq's user experience.

---

## 🔥 CRITICAL PRIORITY - Highest Impact

### 1. Better Error Messages & Query Hints
**Impact:** VERY HIGH | **Effort:** Medium

**Why This Matters for Query Building:**
- Users spend most time **fixing broken queries**
- Faster error resolution = faster query building
- Learning jq syntax through helpful hints

**What Users Want:**
- Understand WHY their query failed
- Hints on how to fix common mistakes
- Examples of correct syntax

**Current State:** jiq shows "Syntax Error" in overlay (good!) but could be more helpful

**Recommendations:**
1. Parse jq error messages and add context:
   - "Cannot index string with number" → "Hint: Use .[] for arrays, not strings"
   - Show relevant jq function documentation inline
2. Add "Did you mean...?" suggestions for typos (`.legnth` → `.length`)
3. Show example of correct syntax for the attempted operation
4. Inline query validation hints as you type

**Impact:** Dramatically speeds up query building workflow

### 2. Query Templates & Custom Functions
**Impact:** HIGH | **Effort:** Medium

**Why This Matters for Query Building:**
- Reusing common patterns saves time
- Complex queries built from proven components
- Project-specific jq libraries

**What Users Want:**
- Define reusable jq functions
- Quick-insert common query patterns
- Save frequently used queries

**Evidence:**
- jnv issue #64: "Allow specifying a jq file to expose custom functions"
- Users have complex, repeated queries across sessions

**Recommendations:**
1. Support `~/.config/jiq/functions.jq` for custom functions
2. Add keyboard shortcuts for saved queries (Ctrl+1-9)
3. Show available custom functions in autocomplete
4. Built-in templates: "Extract all X", "Filter by date", "Flatten nested"
5. In-app query saving: type query → Ctrl+S → name it → reuse later

**Example Templates:**
```
Ctrl+1: .[] | select(.FIELD == "VALUE")
Ctrl+2: [.[] | {key: .FIELD}]
Ctrl+3: .. | select(type == "string")
```

### 3. Enhanced Result Type Information
**Impact:** HIGH | **Effort:** Low

**Why This Matters for Query Building:**
- Know what your query produces **without looking at output**
- Type info guides next step in query chain
- Catch mistakes early (expected array, got object)

**Current State:** Stats bar shows "Array [5 objects]" (good!)

**Enhancement Recommendations:**
```
Current: Array [5 items]
Better:  Array [5 objects] | Keys: name, age, city | 12.3 KB
```

**Show:**
- Detailed type breakdown (Objects: 5, Strings: 12, Numbers: 8)
- Available keys in objects (for autocomplete context)
- Data size (helps understand query impact)
- Nesting depth for complex structures

**Impact:** Helps users understand their query results at a glance

---

## 🚀 HIGH PRIORITY - High Impact

### 5. Result Set Information Enhancement
**Impact:** HIGH | **Effort:** Low

**What Users Want:**
- Detailed stats about current results
- Array/object length immediately visible
- Data type distribution

**Current State:** Stats bar shows "Array [5 objects]" (good!)

**Enhancement Recommendations:**
```
Array [5 items, 245 KB] | Objects: 5 | Strings: 12 | Numbers: 8 | Nulls: 2
```
- Show memory size of results
- Show type breakdown
- Show depth of nesting for objects

### 6. Custom jq Functions/Presets
**Impact:** HIGH | **Effort:** Medium

**What Users Want:**
- Define reusable jq functions
- Load commonly used queries as shortcuts
- Project-specific jq libraries

**Evidence:**
- jnv issue #64: "Allow specifying a jq file to expose custom functions"
- Users have complex, repeated queries

**Recommendations:**
1. Support `~/.config/jiq/functions.jq` for custom functions
2. Add keyboard shortcuts for saved queries (Ctrl+1, Ctrl+2, etc.)
3. Show available custom functions in autocomplete
4. "Save current query" feature (Ctrl+S)

### 7. Streaming JSON Lines (JSONL) Support
**Impact:** HIGH | **Effort:** Medium

**What Users Want:**
- Process newline-delimited JSON (logs, data streams)
- Auto-detect JSONL format
- Slurp mode to array conversion

**Evidence:**
- jnv request for slurp mode
- fx supports streaming
- Common log format for cloud services (CloudWatch, etc.)

**Recommendation:**
- Auto-detect JSONL (multiple root objects)
- Show "JSONL detected. Press S to slurp into array" notification
- Add `--slurp` flag equivalent

### 8. Improved Query History
**Impact:** MEDIUM-HIGH | **Effort:** Low-Medium

**Current State:** jiq has history with Ctrl+R (excellent!)

**Enhancements:**
1. Show query frequency/usage count
2. Star/favorite important queries
3. Export/import history for sharing with team
4. Add tags to queries: `#users #production #debug`
5. Filter history by tag in search

---

## 📊 MEDIUM PRIORITY - Good Impact

### 9. Diff Mode
**Impact:** MEDIUM | **Effort:** High

**What Users Want:**
- Compare two JSON objects side-by-side
- Highlight differences
- Show before/after of transformations

**Evidence:**
- jless issue #130: "Diffing support"
- Common use case: API response changes, config drift

**Recommendation:**
```bash
jiq --diff file1.json file2.json
# Or: jiq file.json --compare-query '.users[0]' '.users[1]'
```

### 10. Data Validation Features
**Impact:** MEDIUM | **Effort:** High

**What Users Want:**
- JSON Schema validation
- Show schema violations
- Follow $ref references

**Evidence:**
- jless issue #131: "Follow Json Schema References"
- API development workflow needs validation

**Recommendation:**
- `jiq --schema schema.json data.json`
- Highlight invalid fields in red
- Show schema violations in error overlay

### 11. Table View for Array of Objects
**Impact:** MEDIUM | **Effort:** Medium

**What Users Want:**
- Tabular display for uniform data
- Sort columns
- Filter rows

**Evidence:**
- json-tui feature: "turn arrays of objects into tables"
- Easier to scan structured data

**Recommendation:**
When results are array of objects with same keys:
```
┌─────────┬─────────┬─────────┐
│ name    │ age     │ city    │
├─────────┼─────────┼─────────┤
│ Alice   │ 30      │ NYC     │
│ Bob     │ 25      │ LA      │
└─────────┴─────────┴─────────┘
Press T to toggle table/JSON view
```

### 12. Editor Integration
**Impact:** MEDIUM | **Effort:** Low

**What Users Want:**
- Open current JSON in editor (vim, vscode, etc.)
- Jump to specific line/path

**Evidence:**
- jless issue #133: "Open file at selected node in vim"
- Common workflow: explore → edit

**Recommendation:**
- `Ctrl+O` to open in $EDITOR
- Respect $EDITOR or VISUAL environment variables
- Pass line number for nested selection

---

## 🔧 NICE TO HAVE - Lower Priority

### 13. Alternative Format Support
- YAML input/output
- TOML input/output (fx supports this)
- XML to JSON conversion

### 14. Relative/Absolute Line Numbers Toggle
- jless issue #134
- Useful for vim users

### 15. Better Bracket/Special Character Input
- jnv users report difficulty entering []{}
- May be terminal-specific, but worth investigating

### 16. Recursive Expand/Collapse
- jless issue #127: "Open/close children recursively"
- Ctrl+Click to expand all nested children
- Shift+Click to collapse all

---

## 🎯 Quick Wins (Low Effort, Medium-High Impact)

### Immediate Improvements:

1. **Better First-Time Experience**
   - Show welcome message on first run with key shortcuts
   - "Press F1 for help, Tab for autocomplete, Shift+Tab to switch panes"

2. **Clipboard Path Copy**
   - Add `Ctrl+Shift+P` to copy JSONPath of current cursor position
   - Example: `.users[0].addresses[2].street`

3. **File Info in Title Bar**
   - Show filename, size, root type
   - `data.json (2.3 MB) - Object with 15 keys`

4. **Success Feedback**
   - Flash green border on successful copy
   - Show "Copied!" notification briefly

5. **Query Validation Hints**
   - Show autocomplete even for incomplete queries
   - jnv issue: "Fallback to longest valid prefix"

6. **Improved Stats for Streams**
   - Current: "Stream [3 values]"
   - Better: "Stream [3 values] - Use -s flag to collect into array"

---

## 📈 Competitive Analysis Summary

### What Makes Other Tools Popular:

**fx:**
- JavaScript/jq syntax options (flexibility)
- Streaming support
- Multi-format (JSON, YAML, TOML)
- Fast autocomplete even with huge files

**jless:**
- Excellent navigation (vim-style)
- Clean, readable output
- Good for exploration, not editing

**jnv:**
- Real-time filtering (like jiq!)
- Good keyboard shortcuts
- Active development, responsive to issues

**jiq's Unique Strengths:**
- ✅ Best-in-class autocomplete with context awareness
- ✅ Function tooltips with examples
- ✅ Floating error overlay (non-disruptive)
- ✅ Query history with fuzzy search
- ✅ VIM keybindings
- ✅ Real-time execution

**Where jiq Can Improve:**
- ❌ Export formats (others have CSV, YAML)
- ❌ Large file performance (users complain about this everywhere)
- ❌ Table view for arrays
- ❌ JSON path display
- ❌ Streaming/JSONL support

---

## 🎬 Implementation Roadmap

### Phase 1: Quick Wins (1-2 weeks)
1. File info in title bar
2. Clipboard path copy (JSONPath)
3. Enhanced stats bar
4. Welcome message for new users
5. Better error hints

### Phase 2: High-Impact Features (1-2 months)
1. Export to CSV/YAML
2. JSON path breadcrumb display
3. Large file optimizations (pagination, streaming)
4. Custom jq functions support
5. JSONL support

### Phase 3: Advanced Features (2-3 months)
1. Table view for arrays
2. Diff mode
3. JSON Schema validation
4. Editor integration
5. Advanced history features (tags, favorites)

---

## 📚 Sources & References

### Research Sources:

**jq Core Issues:**
- [High memory usage #620](https://github.com/jqlang/jq/issues/620)
- [jq 1.6 extremely slow #1826](https://github.com/jqlang/jq/issues/1826)
- [Debugging & performance tuning #808](https://github.com/jqlang/jq/issues/808)

**Competitive Tools:**
- [jnv - Interactive JSON filter](https://github.com/ynqa/jnv)
- [jless - Command-line JSON viewer](https://github.com/PaulJuliusMartinez/jless)
- [fx - Terminal JSON viewer](https://github.com/antonmedv/fx)
- [jqp - TUI playground for jq](https://github.com/noahgorstein/jqp)

**User Discussions:**
- [fx on Hacker News](https://news.ycombinator.com/item?id=29861043)
- [jnv on Hacker News](https://news.ycombinator.com/item?id=39759325)
- [jless on Hacker News](https://news.ycombinator.com/item?id=30273940)

**Performance Research:**
- [HugeJsonViewer](https://medevel.com/hugejsonviewer/)
- [JSON Performance Optimization Guide](https://superjson.ai/blog/2025-08-30-json-performance-optimization-large-files-guide/)

---

## 💡 Key Takeaways

1. **Users value speed and efficiency** - Real-time feedback is jiq's strength, don't lose it
2. **Export is critical** - Almost every competing tool supports multiple output formats
3. **Large files are painful** - This is THE #1 complaint across all JSON tools
4. **Context matters** - Users want to know WHERE they are in complex structures
5. **Reusability saves time** - History, saved queries, and custom functions are highly valued
6. **Error messages should teach** - Don't just say "error", explain how to fix it

---

## 🎯 Recommended Focus

**If you can only do 3 things, do these:**

1. **Export to CSV/YAML** - Immediate utility, users ask for this constantly
2. **Large file performance** - Addresses #1 pain point across all tools
3. **JSON path breadcrumbs** - Huge UX win for navigation and understanding

These three features would differentiate jiq from competitors while addressing the most common user frustrations.
