# Syntax Highlighting

jiq now includes simple keyword-based syntax highlighting for jq queries in the input field.

## Color Scheme

The following elements are highlighted:

| Element | Color | Examples |
|---------|-------|----------|
| **Keywords** | Yellow | `if`, `then`, `else`, `end`, `and`, `or`, `not`, `as`, `def`, `reduce`, `foreach`, `try`, `catch`, `empty`, `null`, `true`, `false` |
| **Built-in Functions** | Blue | `map`, `select`, `sort`, `group_by`, `keys`, `values`, `has`, `contains`, `split`, `join`, `test`, `match`, `type`, `length`, `add`, `flatten`, `unique`, `reverse`, etc. |
| **Object Field Names** | Cyan | `name` in `{name: .value}`, `firstName` in `{firstName: .first}` |
| **Field Accessors** | Default (white) | `.name`, `.users`, `.address.city`, `.organization.headquarters` |
| **String Literals** | Green | `"hello"`, `"Alice"`, `"test string"` |
| **Numbers** | Cyan | `123`, `45.67`, `-10` |
| **Operators** | Magenta | `\|`, `==`, `!=`, `<=`, `>=`, `+`, `-`, `*`, `/`, `(`, `)`, `[`, `]`, `{`, `}`, etc. |

## Implementation

The syntax highlighting uses a simple character-by-character parser that:

1. Identifies string literals (preserving escape sequences)
2. Recognizes numbers (including negative and decimal)
3. Detects operators and special characters
4. Matches keywords against a predefined list
5. Identifies built-in jq functions
6. Highlights field accessors (starting with `.`)

This is **not** a full jq parser, so it may occasionally highlight incorrectly in complex edge cases (e.g., keywords inside strings). However, it provides good-enough highlighting for the vast majority of jq queries.

## Examples

### Simple field access
```jq
.name
```
- `.name` → Default white (field accessor)

### Pipeline with filter
```jq
.users[] | select(.active == true)
```
- `.users` → Default white (field accessor)
- `[]` → Magenta (operators)
- `|` → Magenta (pipe operator)
- `select` → Blue (built-in function)
- `(` `)` → Magenta (operators)
- `.active` → Default white (field accessor)
- `==` → Magenta (operator)
- `true` → Yellow (keyword)

### Conditional expression
```jq
if .count > 10 then .high else .low end
```
- `if`, `then`, `else`, `end` → Yellow (keywords)
- `.count`, `.high`, `.low` → Default white (field accessors)
- `>` → Magenta (operator)
- `10` → Cyan (number)

### String matching
```jq
.users[] | select(.name == "Alice") | .age
```
- `.users`, `.name`, `.age` → Default white (field accessors)
- `[]`, `|`, `(`, `)`, `==` → Magenta (operators)
- `select` → Blue (built-in function)
- `"Alice"` → Green (string literal)

### Object constructor
```jq
{firstName: .first, lastName: .last, age: .age}
```
- `firstName`, `lastName`, `age` → Cyan (object field names)
- `.first`, `.last`, `.age` → Default white (field accessors)
- `{`, `}`, `,`, `:` → Magenta (operators)

## Future Improvements

Possible enhancements for the future:

1. **Context-aware highlighting**: Use a proper jq parser to understand context (e.g., don't highlight keywords inside strings)
2. **Error highlighting**: Highlight syntax errors in red
3. **Configurable colors**: Allow users to customize the color scheme
4. **More language features**: Support for advanced jq features like custom function definitions
5. **Integration with tree-sitter**: Use tree-sitter-jq for more accurate parsing

## Technical Details

- **Location**: `src/syntax/mod.rs`
- **Rendering**: Overlay approach using `Paragraph` widget on top of `tui-textarea`
- **Performance**: Highlighting runs on every render, but is fast enough for single-line queries
- **Dependencies**: None - uses only built-in Rust string processing and ratatui's styling
- **String handling**: Strings are parsed as single units - keywords inside strings are NOT highlighted separately (e.g., `"if"` is entirely green, not yellow)
