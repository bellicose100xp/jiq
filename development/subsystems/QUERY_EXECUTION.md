# Query Execution

jq subprocess management and result handling.

**Location:** `src/query/executor.rs` (146 lines, 8 tests)

## Core Design

```rust
pub struct JqExecutor {
    json_input: String, // Cached for multiple executions
}

pub fn execute(&self, query: &str) -> Result<String, String>
```

**Key decisions:**
- External jq binary (not native Rust parser)
- Synchronous execution (blocks event loop)
- ANSI color codes preserved
- Empty query defaults to `.` (identity filter)

## Subprocess Flow

```
1. Spawn jq process
   ├─ stdin: piped
   ├─ stdout: piped
   ├─ stderr: piped
   └─ arg: --color-output

2. Write JSON to stdin
   └─ Entire json_input sent at once

3. Wait for completion
   └─ Blocking call

4. Capture output
   ├─ success → stdout (with ANSI codes)
   └─ failure → stderr (jq error message)
```

**Command:**
```bash
echo "$json_input" | jq --color-output "$query"
```

## Key Implementation Details

### Empty Query Handling

```rust
let query = if query.trim().is_empty() { "." } else { query };
```

Shows full JSON when field is empty - better UX than error.

### Color Output

```rust
.arg("--color-output")
```

ANSI escape codes preserved, then parsed by `ansi-to-tui` in renderer.

### Error Handling

```rust
if output.status.success() {
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
} else {
    Err(String::from_utf8_lossy(&output.stderr).to_string())
}
```

jq errors returned as-is - rendered in red by UI.

## Performance Characteristics

**Execution time:**
- Process spawn: ~20-50ms
- jq query: ~10-50ms (depends on query complexity)
- Total: ~50-100ms per keystroke

**Acceptable because:**
- Interactive latency tolerance ~100-200ms
- Real-time feedback is core feature
- Could add debouncing if needed (not needed yet)

**Memory:**
- JSON input cached (no re-reading file)
- Results not cached (fresh execution each time)

## Why External jq?

**Pros:**
- Correctness guaranteed (15+ years of development)
- All jq features supported
- No maintenance burden for query logic
- Users already have jq installed (prerequisite)

**Cons:**
- Subprocess overhead (~50ms)
- Requires jq in PATH

**Alternative considered:** Native Rust jq implementation
- Rejected: Massive scope, poor correctness guarantees

## Error Types

```rust
Result<String, String>
```

Simple error type - just message strings:
- Spawn failure: "Failed to spawn jq: {error}"
- Stdin write failure: "Failed to write to jq stdin: {error}"
- Output read failure: "Failed to read jq output: {error}"
- jq parse error: {jq's stderr output}

## Testing Strategy

```rust
#[test]
fn test_identity_filter() { ... }           // Basic functionality
#[test]
fn test_empty_query_defaults_to_identity() // Edge case
#[test]
fn test_field_selection() { ... }           // Common query
#[test]
fn test_array_iteration() { ... }           // Complex query
#[test]
fn test_invalid_query_returns_error() { ... } // Error path
#[test]
fn test_nested_field_access() { ... }       // Nested data
#[test]
fn test_color_output_flag_present() { ... } // Verify ANSI codes
```

**Coverage:** Success path, error path, edge cases, ANSI output

## Future Optimizations

### 1. Debouncing

```rust
// Only execute after 100ms of no typing
if last_execution.elapsed() < Duration::from_millis(100) {
    return cached_result;
}
```

Not needed currently - latency acceptable.

### 2. Async Execution

```rust
tokio::spawn(async move {
    executor.execute(query).await
});
```

Would prevent blocking event loop. Trade-off: complexity vs minimal gain.

### 3. Query Caching

```rust
cache: HashMap<String, String>  // query → result
```

Limited value: queries change constantly during typing.

### 4. jq Process Pool

```rust
// Keep jq process alive, pipe multiple queries
let mut jq_process = Command::new("jq")
    .stdin(Stdio::piped())
    .spawn()?;
```

Complex - would need stateful protocol. Marginal performance gain.

---

**Maintainer notes:**
- Simplicity over optimization (for now)
- External jq is non-negotiable (correctness)
- If latency becomes issue: add debouncing first, async second
