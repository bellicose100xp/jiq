# jq Function Tooltip Research

Comprehensive research on jq function usage patterns, common examples, and tips gathered from Stack Overflow, official documentation, community cookbooks, and developer forums.

---

## Research Summary

### Key Findings

1. **Most Common Pain Points**:
   - Case-insensitive string matching (users often use `contains()` when `test()` is better)
   - Handling null values and missing keys
   - Understanding difference between `map()` and `.[] |`
   - Regex escaping in patterns
   - Memory issues with large files (streaming vs slurping)
   - Converting between types (especially strings to numbers)

2. **Most Searched Operations**:
   - Filtering arrays by condition
   - Extracting specific fields from nested objects
   - Grouping and counting
   - Removing duplicates
   - String manipulation (split, join, replace)
   - Converting JSON to CSV

3. **Common Gotchas**:
   - `0` is truthy in jq (unlike JavaScript)
   - Array indices start at 0
   - `unique` sorts the output
   - `add` returns null for empty arrays
   - Regex uses PCRE syntax, not grep

---

## Function Documentation

### Array/Filter Functions

#### `map`
**Description**: Apply expression to each element of an array

**Examples**:
```jq
map(.name)                    # Extract field from each object
map(. + 1)                    # Increment each number
map(select(.active))          # Filter to keep only active items
map({id, name})               # Reshape each object to specific fields
```

**Tip**: Use `[.[] | expr]` for same result - sometimes clearer for complex transforms and uses less memory for very large arrays.

---

#### `select`
**Description**: Filter elements that match a condition

**Examples**:
```jq
select(.age > 18)             # Filter by numeric comparison
select(.status == "active")   # Filter by exact string match
select(.tags | contains(["important"]))  # Filter by array contents
select(.name | test("^test"; "i"))       # Case-insensitive regex filter
```

**Tip**: For case-insensitive or regex matching, use `test()` instead of string comparison. For null-safe checks, use `select(.field? // false)`.

---

#### `sort_by`
**Description**: Sort array elements by a computed value

**Examples**:
```jq
sort_by(.name)                # Sort objects by field alphabetically
sort_by(.date) | reverse      # Sort by date descending
sort_by(.price | tonumber)    # Sort by numeric value (ensure number type)
sort_by(.id | scan("[0-9]+$") | tonumber)  # Sort by numeric suffix
```

**Tip**: For descending numeric sort, use `sort_by(-.field)` instead of `sort_by(.field) | reverse` - it's cleaner and faster.

---

#### `group_by`
**Description**: Group array elements by a computed key

**Examples**:
```jq
group_by(.category)           # Group objects by field value
group_by(.status) | map({key: .[0].status, items: .})  # Named groups
group_by(.type) | map({type: .[0].type, count: length})  # Count per group
group_by(.date[:10])          # Group by date (ignoring time)
```

**Tip**: Output is automatically sorted by the grouping key. Use `map({key: .[0].field, values: .})` pattern to create a more usable structure. For counting occurrences: `group_by(.) | map({value: .[0], count: length})`.

---

#### `unique_by`
**Description**: Remove duplicates based on a computed key

**Examples**:
```jq
unique_by(.id)                # Deduplicate by ID field
unique_by(.email | ascii_downcase)  # Case-insensitive dedupe
unique_by([.first, .last])    # Dedupe by multiple fields combined
unique_by(.timestamp | .[:10])  # Dedupe by date part only
```

**Tip**: Keeps the **first** occurrence of each duplicate AND **sorts** output by the expression value. To preserve original order while deduping, use `group_by(.field) | map(first)` instead.

---

#### `min_by` / `max_by`
**Description**: Find element with minimum/maximum value by expression

**Examples**:
```jq
min_by(.price)                # Object with lowest price
max_by(.score)                # Object with highest score
max_by(.date | fromdateiso8601)  # Most recent by ISO date
min_by(.name | length)        # Shortest name
```

**Tip**: Returns the **entire object**, not just the value. Use `.field` after to extract just the value: `max_by(.score).score`.

---

#### `limit`
**Description**: Take only first N results from a generator

**Examples**:
```jq
limit(5; .[])                 # First 5 elements from array
limit(10; recurse)            # Limit recursive traversal
limit(3; .[] | select(.active))  # First 3 matching items
[limit(100; inputs)]          # First 100 lines from JSONL
```

**Tip**: More efficient than `[:5]` for large arrays or infinite generators because it stops early. Essential when using `recurse` to prevent infinite loops.

---

#### `nth`
**Description**: Get the nth element from a generator

**Examples**:
```jq
nth(0)                        # First element (same as first)
nth(2; .[])                   # Third element from array
nth(0; .[] | select(.valid))  # First valid element
nth(4; inputs)                # 5th line from JSONL stream
```

**Tip**: Use `first` and `last` for clearer code when getting first/last elements. `nth` is best for specific positions in generators.

---

#### `range`
**Description**: Generate a sequence of numbers

**Examples**:
```jq
range(5)                      # 0, 1, 2, 3, 4
range(1; 11)                  # 1 through 10 (end exclusive)
range(0; 100; 10)             # 0, 10, 20, ..., 90 (step by 10)
[range(5)] | map(. * 2)       # Generate then transform: [0,2,4,6,8]
```

**Tip**: End value is **exclusive** (like Python). Wrap in `[range(n)]` to create an array. Useful for generating test data or indices.

---

#### `until` / `while`
**Description**: Loop until/while condition is true

**Examples**:
```jq
until(. >= 100; . * 2)        # Final value when >= 100: returns 128
1 | until(. > 10; . + 1)      # Final value: 11
[while(. < 100; . * 2)]       # ALL intermediate values: [1,2,4,8,16,32,64]
[1 | while(. <= 10; . + 1)]   # Count up: [1,2,3,4,5,6,7,8,9,10]
```

**Tip**: **Critical difference**: `until` returns only the **final value**; `while` emits **ALL intermediate values** as a stream (wrap in `[]` to collect). For iteration with state, consider `reduce` instead.

---

#### `recurse`
**Description**: Recursively apply expression (depth-first traversal)

**Examples**:
```jq
recurse(.children[]?)         # Traverse tree via children field
recurse | .name?              # Get all "name" fields in tree
recurse(.next?) | .value      # Follow linked list
recurse(if type == "object" then .[] else empty end)  # All nested values
```

**Tip**: Use `..` as shorthand for `recurse`. Add `?` to handle missing fields gracefully. Always use `limit()` when depth is unknown to prevent infinite loops.

---

#### `walk`
**Description**: Transform all values recursively (bottom-up)

**Examples**:
```jq
walk(if type == "string" then ascii_downcase else . end)  # Lowercase all strings
walk(if type == "object" then del(.internal) else . end)  # Remove field everywhere
walk(if type == "number" then . * 100 | round / 100 else . end)  # Round all numbers
walk(if . == null then "N/A" else . end)  # Replace all nulls
```

**Tip**: Processes **bottom-up** (children before parents). For top-down, use `recurse`. Great for cleaning/normalizing entire JSON structures.

---

#### `with_entries`
**Description**: Transform object's key-value pairs

**Examples**:
```jq
with_entries(.value |= . + 1)  # Increment all values
with_entries(select(.value != null))  # Remove null values
with_entries(.key |= "prefix_" + .)  # Prefix all keys
with_entries(select(.key | startswith("public_")))  # Keep only public_ keys
```

**Tip**: Shorthand for `to_entries | map(...) | from_entries`. Use for filtering by key names or transforming both keys and values together.

---

### Object Functions

#### `has`
**Description**: Check if object has a specific key

**Examples**:
```jq
has("email")                  # Check if key exists
select(has("config"))         # Filter objects that have config
if has("error") then .error else "ok" end  # Conditional on key presence
[.[] | select(has("id") and has("name"))]  # Objects with both keys
```

**Tip**: Only checks key existence, not value. For null-safe access with default, use `.key? // default`. For checking if value is non-null: `has("key") and .key != null`.

---

#### `del`
**Description**: Delete keys or paths from object/array

**Examples**:
```jq
del(.password, .secret)       # Remove sensitive fields
del(.users[0])                # Remove first array element
del(.config.debug)            # Remove nested field
map(del(.internal))           # Remove field from all objects
```

**Tip**: For removing keys matching a pattern, use `with_entries(select(.key | test("pattern") | not))`. Returns modified copy; original unchanged.

---

#### `getpath` / `setpath`
**Description**: Get/set values using dynamic path arrays

**Examples**:
```jq
getpath(["user", "address", "city"])  # Dynamic nested access
setpath(["config", "enabled"]; true)  # Set nested value
getpath($path)                # Use variable path
setpath(["items", 0]; "new")  # Set array element by index
```

**Tip**: Use for dynamic/computed paths. For static paths, `.a.b.c` is cleaner. `setpath` creates intermediate objects/arrays as needed.

---

#### `delpaths`
**Description**: Delete multiple paths at once

**Examples**:
```jq
delpaths([["a"], ["b", "c"]])  # Delete multiple specific paths
delpaths([paths(type == "null")])  # Delete all null values
delpaths([paths(. == "")])    # Delete all empty strings
delpaths([path(..|numbers)])  # Delete all numbers from tree
```

**Tip**: Combine with `[paths(...)]` to delete values matching conditions. More efficient than chaining multiple `del()` calls.

---

### String Functions

#### `split` / `join`
**Description**: Split string to array / join array to string

**Examples**:
```jq
split(",")                    # "a,b,c" -> ["a","b","c"]
split("\n") | map(select(. != ""))  # Split lines, remove empty
join(", ")                    # ["a","b"] -> "a, b"
[.items[].name] | join(" | ")  # Join field values
```

**Tip**: Use `splits("\\s+")` for regex splitting (e.g., by whitespace). For joining, ensure all elements are strings first with `map(tostring)` if needed.

---

#### `ltrimstr` / `rtrimstr`
**Description**: Remove prefix/suffix from string

**Examples**:
```jq
ltrimstr("https://")          # Remove URL scheme
rtrimstr(".json")             # Remove file extension
ltrimstr("v") | tonumber      # Parse version: "v1.2" -> 1.2 (after more processing)
.filename | rtrimstr(".bak")  # Remove backup suffix
```

**Tip**: Safe to use even if prefix/suffix doesn't exist (returns unchanged string). For regex-based trimming, use `sub()` instead.

---

#### `startswith` / `endswith`
**Description**: Check string prefix/suffix

**Examples**:
```jq
select(startswith("http"))    # Filter URLs
select(.email | endswith("@company.com"))  # Filter by email domain
select(.path | startswith("/api/"))  # Filter API paths
if endswith(".json") then "JSON" else "other" end
```

**Tip**: For case-insensitive checks, use `test("^pattern"; "i")` or `test("pattern$"; "i")` instead.

---

#### `test`
**Description**: Test if string matches regex pattern

**Examples**:
```jq
test("^[0-9]+$")              # All digits
test("[a-z]"; "i")            # Contains letter (case-insensitive)
select(test("error|warning"; "i"))  # Filter log lines
test("\\d{4}-\\d{2}-\\d{2}")  # Date pattern YYYY-MM-DD
```

**Tip**: More powerful than `contains()` for pattern matching. Flags: `"i"` case-insensitive, `"x"` extended (ignore whitespace), `"m"` multiline.

---

#### `match` / `capture`
**Description**: Extract regex match information / named capture groups

**Examples**:
```jq
match("v([0-9]+)\\.([0-9]+)") | .captures[].string  # Extract version parts
capture("(?<user>[^@]+)@(?<domain>.+)")  # Parse email to {user, domain}
capture("(?<y>\\d{4})-(?<m>\\d{2})-(?<d>\\d{2})")  # Parse date
match("\\d+"; "g") | .string  # All number matches (global)
```

**Tip**: Use `capture()` with named groups for cleaner code - returns object with field names. Use `match()` when you need position info or unnamed groups.

---

#### `scan`
**Description**: Find all regex matches (returns stream)

**Examples**:
```jq
[scan("[0-9]+")]              # Extract all numbers as array
[scan("\\w+")]                # Extract all words
scan("https?://[^\\s]+")      # Extract all URLs
[scan("[A-Z]{2,}")] | unique  # Find all acronyms
```

**Tip**: Returns a **stream**, not array - wrap in `[]` to collect. For single match, use `match()`. Great for extracting multiple values from text.

---

#### `sub` / `gsub`
**Description**: Replace first/all regex matches

**Examples**:
```jq
sub("old"; "new")             # Replace first occurrence
gsub("\\s+"; " ")             # Normalize whitespace
gsub("[^a-zA-Z0-9]"; "_")     # Sanitize to alphanumeric
sub("^v"; "")                 # Remove leading 'v'
gsub("(?<n>\\d+)"; "[\(.n)]") # Wrap numbers in brackets
```

**Tip**: Use capture groups in replacement with `\(.name)` syntax. Third argument for flags: `gsub("a"; "b"; "i")` for case-insensitive.

---

#### `splits`
**Description**: Split by regex pattern (returns stream)

**Examples**:
```jq
[splits("\\s+")]              # Split by whitespace
[splits("[,;]\\s*")]          # Split by comma or semicolon
[splits("\\n+")] | map(select(. != ""))  # Split lines, skip empty
[splits("::")] | .[1]         # Get second segment
```

**Tip**: Returns **stream**, wrap in `[]` for array. Use `split()` for literal string delimiters; `splits()` for regex patterns.

---

### Comparison/Search Functions

#### `contains` / `inside`
**Description**: Check if value contains/is contained by another

**Examples**:
```jq
contains("error")             # String contains substring
.tags | contains(["urgent"])  # Array contains element
{a:1} | contains({a:1})       # Object contains keys/values
"sub" | inside("substring")   # Inverse of contains
```

**Tip**: For regex or case-insensitive matching, use `test()` instead. `contains` does deep comparison for objects/arrays - all elements must match.

---

#### `index` / `rindex` / `indices`
**Description**: Find position(s) of value in array/string

**Examples**:
```jq
index(",")                    # First comma position
rindex("/")                   # Last slash position (for path parsing)
indices(",")                  # All comma positions
.path | rindex("/") as $i | .[$i+1:]  # Get filename from path
```

**Tip**: Returns `null` if not found (not -1 like other languages). Use `indices()` to find all occurrences.

---

### Date/Time Functions

#### `now`
**Description**: Current Unix timestamp

**Examples**:
```jq
now                           # Current timestamp (float)
now | floor                   # Current timestamp (integer)
now | strftime("%Y-%m-%d")    # Today's date
now | todate                  # Current time as ISO 8601
```

**Tip**: Returns seconds since epoch as **float** with microsecond precision. Use `floor` for integer timestamp.

---

#### `strftime` / `strptime`
**Description**: Format/parse timestamps with custom format

**Examples**:
```jq
now | strftime("%Y-%m-%d %H:%M:%S")  # Custom format
now | strftime("%B %d, %Y")   # "January 15, 2024"
strptime("%Y-%m-%d") | .[0]   # Parse date string to timestamp
"2024-01-15" | strptime("%Y-%m-%d") | mktime  # Full timestamp
```

**Tip**: For ISO 8601 dates, use `todate`/`fromdate` instead - simpler and handles timezones. Common format codes: `%Y`=year, `%m`=month, `%d`=day, `%H`=hour, `%M`=minute, `%S`=second.

---

#### `fromdate` / `todate`
**Description**: Parse/format ISO 8601 dates

**Examples**:
```jq
"2024-01-15T10:30:00Z" | fromdate  # ISO 8601 to timestamp
now | todate                  # Timestamp to ISO 8601
.created_at | fromdate | . + 86400 | todate  # Add 1 day
[.events[].date | fromdate] | min | todate   # Earliest date
```

**Tip**: Works with ISO 8601 format only. For custom formats, use `strptime`/`strftime`.

---

### Array Functions (No Arguments)

#### `keys` / `keys_unsorted`
**Description**: Get object keys or array indices

**Examples**:
```jq
keys                          # Sorted keys: ["a","b","c"]
keys_unsorted                 # Original order preserved
keys | length                 # Count of keys
keys | map(select(startswith("_")))  # Private keys only
```

**Tip**: `keys` **sorts** output alphabetically. Use `keys_unsorted` when order matters (e.g., preserving config file order). Both work on arrays (returns indices).

---

#### `values`
**Description**: Get all values from object or array

**Examples**:
```jq
values                        # All values (strips keys)
.config | values | add        # Sum all config values
values | map(select(. != null))  # Non-null values only
```

**Tip**: Same as `.[]` but clearer intent. Does NOT filter out null values - use `map(select(. != null))` or `map(values)` on nested objects for that.

---

#### `sort` / `reverse`
**Description**: Sort array / reverse array order

**Examples**:
```jq
sort                          # Sort ascending
sort | reverse                # Sort descending
sort | unique                 # Sort and dedupe
.logs | reverse | first       # Most recent log entry
```

**Tip**: `sort` works on numbers, strings, and mixed arrays. For custom sort order, use `sort_by()`. Use `[-1]` instead of `reverse | first` for just the last element.

---

#### `unique`
**Description**: Remove duplicate values (sorts output)

**Examples**:
```jq
unique                        # Remove duplicates
[.items[].category] | unique  # Unique values of field
unique | length               # Count distinct values
sort | unique                 # Explicit: unique already sorts
```

**Tip**: Output is always **sorted**. Use `unique_by()` to dedupe by specific field while keeping whole objects. For unsorted unique, use: `reduce .[] as $x ({}; .[$x|tostring] = $x) | [.[]]`.

---

#### `flatten`
**Description**: Flatten nested arrays

**Examples**:
```jq
flatten                       # Flatten all levels
flatten(1)                    # Flatten one level only
[[1,2],[3,[4,5]]] | flatten   # [1,2,3,4,5]
.pages[].items | flatten      # Combine paginated results
```

**Tip**: Use `add` to concatenate arrays without deep flattening. `flatten(1)` is useful for combining nested arrays while preserving inner structure.

---

#### `add`
**Description**: Sum numbers or concatenate arrays/strings

**Examples**:
```jq
[.items[].price] | add        # Sum prices
["a","b","c"] | add           # "abc" (string concat)
[[1,2],[3,4]] | add           # [1,2,3,4] (array concat)
[.counts[]] | add // 0        # Sum with default 0 for empty
```

**Tip**: Returns **null** for empty arrays - use `// 0` or `// ""` for defaults. For objects, merges them (later keys win).

---

#### `length`
**Description**: Length of string/array/object, absolute value of number

**Examples**:
```jq
length                        # Element count
.items | length               # Array length
select(length > 0)            # Filter non-empty
select(.name | length <= 50)  # Max name length
```

**Tip**: Returns: array/object element count, string character count (not bytes), number's absolute value, `null` returns 0. For byte length use `utf8bytelength`.

---

#### `first` / `last`
**Description**: First/last element from array or generator

**Examples**:
```jq
first                         # First element
last                          # Last element
first(.[] | select(.valid))   # First valid item
last(inputs)                  # Last line from JSONL
```

**Tip**: For arrays, `.[0]` and `.[-1]` work too. `first()`/`last()` are essential for generators and more efficient for large streams.

---

#### `min` / `max`
**Description**: Minimum/maximum value in array

**Examples**:
```jq
min                           # Minimum value
max                           # Maximum value
[.scores[]] | max             # Highest score
[.items[].price] | min        # Lowest price
```

**Tip**: Returns the **value**, not the containing object. Returns `null` for empty arrays. For object with min/max value, use `min_by()`/`max_by()`.

---

#### `transpose`
**Description**: Transpose matrix (swap rows and columns)

**Examples**:
```jq
[[1,2],[3,4]] | transpose     # [[1,3],[2,4]]
[.names, .ages] | transpose   # Zip: [["Alice",30],["Bob",25]]
[.headers, .values] | transpose | map({(.[0]): .[1]}) | add  # Create object
```

**Tip**: Great for "zipping" multiple arrays together. Works even if arrays have different lengths (uses nulls for missing values).

---

### Type Functions

#### `type`
**Description**: Get the type name of a value

**Examples**:
```jq
type                          # "string", "number", "array", etc.
select(type == "object")      # Filter by type
.[] | select(type != "null")  # Remove nulls
group_by(type)                # Group values by type
```

**Tip**: Returns: "null", "boolean", "number", "string", "array", "object". For type filtering, use the type selectors (`arrays`, `objects`, etc.) instead - cleaner and faster.

---

#### `tostring` / `tonumber`
**Description**: Convert to string/number

**Examples**:
```jq
.id | tostring                # Ensure string type
.price | tonumber             # Parse string to number
(.count | tostring) + " items"  # String concatenation
.amount | tonumber? // 0      # Safe parse with default
```

**Tip**: `tonumber` throws error on invalid input - use `tonumber?` with `//` for safe parsing. `tostring` on strings is a no-op.

---

#### `arrays` / `objects` / `strings` / `numbers` / `booleans` / `nulls`
**Description**: Filter to keep only values of that type

**Examples**:
```jq
.[] | numbers                 # Keep only numbers
.[] | strings                 # Keep only strings
.[] | objects | .name         # Names from object children only
.. | scalars                  # All leaf values in tree
```

**Tip**: These are filters, not tests. Use instead of `select(type == "...")` for cleaner code. `scalars` = non-iterables (strings, numbers, booleans, null).

---

#### `scalars` / `iterables`
**Description**: Filter to keep scalars (primitives) or iterables (arrays/objects)

**Examples**:
```jq
.. | scalars                  # All leaf/primitive values
.[] | iterables               # Only nested structures
[.. | scalars] | unique       # All unique primitive values
```

**Tip**: `scalars` is perfect for extracting all "leaf" values from nested structures. Combine with `..` for deep extraction.

---

### Math Functions

#### `floor` / `ceil` / `round`
**Description**: Round numbers down/up/nearest

**Examples**:
```jq
floor                         # 2.7 -> 2, -2.7 -> -3
ceil                          # 2.1 -> 3, -2.1 -> -2
round                         # 2.5 -> 3, 2.4 -> 2
. * 100 | round / 100         # Round to 2 decimal places
```

**Tip**: `floor` rounds toward negative infinity, `ceil` toward positive infinity. For truncating decimals: multiply, round, divide (as shown).

---

#### `sqrt` / `abs`
**Description**: Square root / absolute value

**Examples**:
```jq
sqrt                          # 16 -> 4
abs                           # -5 -> 5
(.a - .b) | abs               # Distance between values
pow(.; 0.5)                   # Alternative: square root via pow
```

**Tip**: `sqrt` returns float even for perfect squares. For integer result, use `| floor` or `| round`.

---

### Other Functions

#### `empty`
**Description**: Produce no output (filter out current value)

**Examples**:
```jq
empty                         # Produces nothing
if .skip then empty else . end  # Conditionally omit
.[] | if . < 0 then empty else . end  # Filter negatives
select(. >= 0)                # Same as above, cleaner
```

**Tip**: Useful in conditionals to skip values. Often `select()` is cleaner for the same purpose. Also useful in custom functions.

---

#### `error`
**Description**: Raise an error and stop processing

**Examples**:
```jq
error("Invalid input")        # Error with message
if .required == null then error("Missing required field") else . end
.value | if . < 0 then error else . end  # Validate positive
try .data catch error("No data")  # Re-throw with custom message
```

**Tip**: Use `try-catch` to handle errors gracefully: `try expr catch "fallback"`. Error messages appear in stderr.

---

#### `not`
**Description**: Logical NOT

**Examples**:
```jq
not                           # Invert boolean
select(.active | not)         # Select inactive items
if (.enabled | not) then "disabled" else "enabled" end
select(has("error") | not)    # Objects without error field
```

**Tip**: Works on any value (falsey: `false`, `null`; truthy: everything else including `0` and `""`). Combine with `and`/`or` for complex conditions.

---

#### `ascii_downcase` / `ascii_upcase`
**Description**: Convert ASCII letters to lowercase/uppercase

**Examples**:
```jq
ascii_downcase                # "Hello" -> "hello"
ascii_upcase                  # "hello" -> "HELLO"
.name | ascii_downcase        # Normalize for comparison
select(.code | ascii_upcase == "USA")  # Case-insensitive compare
```

**Tip**: Only affects ASCII letters (a-z, A-Z). Non-ASCII characters unchanged. For locale-aware case conversion, jq doesn't have built-in support.

---

#### `env`
**Description**: Access environment variables

**Examples**:
```jq
env.HOME                      # Get HOME variable
env.USER                      # Get current username
env.API_KEY // error("API_KEY not set")  # Required env var
$ENV | keys                   # List all env variables
```

**Tip**: Use `env.VAR` for single variable, `$ENV` object for all variables. Useful for configuration without hardcoding values in filters.

---

## Common Patterns & Recipes

### Filtering and Selection

```jq
# Select objects where array contains value
.[] | select(.tags | index("important"))

# Select objects with specific nested value
.[] | select(.config.enabled == true)

# Multiple conditions
select(.age > 18 and .status == "active")

# Case-insensitive string match
select(.name | test("pattern"; "i"))

# Select by array length
select(.items | length > 0)
```

### Transformation

```jq
# Reshape objects
map({id, name, email: .contact.email})

# Add computed field
map(. + {fullName: (.first + " " + .last)})

# Rename key
with_entries(if .key == "old" then .key = "new" else . end)

# Convert array to object
[.items[] | {(.id): .name}] | add

# Pivot data: array of {key,value} to object
map({(.name): .value}) | add
```

### Aggregation

```jq
# Count items
[.items[] | select(.active)] | length

# Sum field
[.orders[].total] | add

# Average
[.scores[]] | add / length

# Count by category
group_by(.category) | map({category: .[0].category, count: length})

# Max/min with full object
max_by(.score)
```

### String Processing

```jq
# Split and process
.csv | split("\n") | map(split(","))

# Extract with regex
.text | capture("(?<date>\\d{4}-\\d{2}-\\d{2})")

# Clean/normalize
gsub("\\s+"; " ") | gsub("^\\s+|\\s+$"; "")

# Build string from parts
"\(.first) \(.last) <\(.email)>"
```

### Null Safety

```jq
# Default for null
.value // "default"

# Safe navigation
.config?.debug? // false

# Filter nulls
map(select(. != null))

# Remove null fields
with_entries(select(.value != null))
```

### Working with APIs

```jq
# Extract from paginated response
[.pages[].items[]] | flatten

# Transform API response
.data | map({id, name, created: .created_at | fromdate | todate})

# Handle API errors
if .error then error(.error.message) else .data end
```

---

## Deprecated/Non-Standard Functions

The following functions may not exist in all jq versions and should be avoided or used with caution:

| Function | Status | Alternative |
|----------|--------|-------------|
| `leaf_paths` | **Not in jq 1.8+** | Use `paths(scalars)` instead |

When documenting these functions, always note the compatibility issue and provide the recommended alternative.

---

## Common Gotchas

1. **`0` is truthy** - Unlike JavaScript, `0` and `""` are truthy in jq
2. **`unique` sorts** - Output is always sorted alphabetically
3. **`add` returns null** - For empty arrays, returns null not 0
4. **Regex escaping** - Use `\\d` not `\d` in jq strings
5. **Stream vs array** - Functions like `scan()`, `splits()`, `range()` return streams, wrap in `[]` for array
6. **`select()` filters** - Returns nothing if condition is false, not the value
7. **Object iteration** - `.[]` on objects gives values only, use `to_entries` for keys too
8. **`type` of null** - `null | type` returns `"null"` (string), not null

---

## Performance Tips

1. **Use `limit()` with `recurse`** - Prevent infinite loops and memory issues
2. **`first()` over `.[0]`** - More efficient for generators
3. **Stream processing** - Use `inputs` for large JSONL files instead of slurping
4. **Avoid repeated parsing** - Store intermediate results with `as $var`
5. **`[.[] | expr]` vs `map(expr)`** - Generally equivalent, but streaming form can be more memory efficient

---

## Quick Reference

| Task | jq Expression |
|------|---------------|
| Pretty print | `jq .` |
| Get field | `.fieldname` |
| Get nested | `.a.b.c` |
| Array element | `.[0]`, `.[-1]` |
| All elements | `.[]` |
| Filter | `select(.field == value)` |
| Transform all | `map(expr)` |
| Count | `length` |
| Sort | `sort`, `sort_by(.field)` |
| Unique | `unique`, `unique_by(.field)` |
| Group | `group_by(.field)` |
| Sum | `add` |
| Join strings | `join(", ")` |
| Split string | `split(",")` |
| Keys | `keys` |
| Values | `values` |
| Type check | `type == "string"` |
| Null default | `.field // "default"` |
| Null safe | `.field?` |
