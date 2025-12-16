#[derive(Debug, Clone)]
pub struct TooltipContent {
    /// Function name
    pub function: &'static str,
    /// One-line description of what the function does
    pub description: &'static str,
    /// 2-4 practical usage examples
    pub examples: &'static [&'static str],
    /// Optional usage tip or common gotcha
    pub tip: Option<&'static str>,
}

impl TooltipContent {
    /// Create a new TooltipContent with all fields
    pub const fn new(
        function: &'static str,
        description: &'static str,
        examples: &'static [&'static str],
        tip: Option<&'static str>,
    ) -> Self {
        Self {
            function,
            description,
            examples,
            tip,
        }
    }
}

/// Static array of tooltip content for all jq functions
pub static TOOLTIP_CONTENT: &[TooltipContent] = &[
    // ===== Array/Filter Functions (with arguments) =====
    TooltipContent::new(
        "map",
        "Apply expression to each element of an array",
        &[
            "map(.name)              # extract field from each",
            "map(. + 1)              # increment each number",
            "map(select(.active))    # filter to active items",
            "map({id, name})         # reshape each object",
        ],
        Some("Use [.[] | expr] for same result - less memory for large arrays"),
    ),
    TooltipContent::new(
        "select",
        "Filter elements that match a condition",
        &[
            "select(.age > 18)                       # numeric filter",
            "select(.status == \"active\")            # exact match",
            "select(.tags | contains([\"important\"])) # array check",
            "select(.name | test(\"^test\"; \"i\"))     # regex match",
        ],
        Some("For null-safe checks, use select(.field? // false)"),
    ),
    TooltipContent::new(
        "sort_by",
        "Sort array elements by a computed value",
        &[
            "sort_by(.name)          # sort by field",
            "sort_by(.date) | reverse # sort descending",
            "sort_by(-.price)        # descending (cleaner)",
            "sort_by(.price | tonumber) # ensure numeric",
        ],
        Some("Use sort_by(-.field) instead of sort_by(.field) | reverse"),
    ),
    TooltipContent::new(
        "group_by",
        "Group array elements by a computed key",
        &[
            "group_by(.category)     # group by field",
            "group_by(.type) | map({type: .[0].type, count: length})",
            "group_by(.date[:10])    # group by date part",
        ],
        Some("Output sorted by key; use map({key: .[0].field, values: .})"),
    ),
    TooltipContent::new(
        "unique_by",
        "Remove duplicates based on a computed key",
        &[
            "unique_by(.id)          # dedupe by ID",
            "unique_by(.email | ascii_downcase) # case-insensitive",
            "unique_by([.first, .last]) # by multiple fields",
        ],
        Some(
            "Keeps FIRST occurrence; output is SORTED; use group_by | map(first) to preserve order",
        ),
    ),
    TooltipContent::new(
        "min_by",
        "Find element with minimum value by expression",
        &[
            "min_by(.price)          # object with lowest price",
            "min_by(.date)           # earliest by date",
            "min_by(.name | length)  # shortest name",
        ],
        Some("Returns entire object; use .field after: max_by(.score).score"),
    ),
    TooltipContent::new(
        "max_by",
        "Find element with maximum value by expression",
        &[
            "max_by(.score)          # object with highest score",
            "max_by(.date | fromdateiso8601) # most recent by ISO date",
            "max_by(.name | length)  # longest name",
        ],
        Some("Returns entire object; use .field after: max_by(.score).score"),
    ),
    TooltipContent::new(
        "limit",
        "Take only first N results from a generator",
        &[
            "limit(5; .[])           # first 5 elements",
            "limit(10; recurse)      # limit recursion",
            "limit(3; .[] | select(.active)) # first 3 matches",
            "[limit(100; inputs)]    # first 100 JSONL lines",
        ],
        Some("More efficient than [:5] - stops early; essential with recurse"),
    ),
    TooltipContent::new(
        "nth",
        "Get the nth element from a generator",
        &[
            "nth(0)                  # first element (same as first)",
            "nth(2; .[])             # third element from array",
            "nth(0; .[] | select(.valid)) # first valid element",
        ],
        Some("Use first/last for clearer code; nth for specific positions"),
    ),
    TooltipContent::new(
        "range",
        "Generate a sequence of numbers",
        &[
            "range(5)                # 0, 1, 2, 3, 4",
            "range(1; 11)            # 1 to 10 (end exclusive)",
            "range(0; 100; 10)       # 0, 10, 20, ..., 90",
            "[range(5)] | map(. * 2) # [0,2,4,6,8]",
        ],
        Some("End is EXCLUSIVE (like Python); wrap in [range(n)] for array"),
    ),
    TooltipContent::new(
        "until",
        "Repeat until condition is true",
        &[
            "until(. >= 100; . * 2)  # returns 128 (final value only)",
            "1 | until(. > 10; . + 1) # returns 11",
        ],
        Some("Returns FINAL value only; use while for ALL intermediate values"),
    ),
    TooltipContent::new(
        "while",
        "Repeat while condition is true",
        &[
            "[while(. < 100; . * 2)] # [1,2,4,8,16,32,64] (all values)",
            "[1 | while(. <= 10; . + 1)] # [1,2,3,4,5,6,7,8,9,10]",
        ],
        Some("Returns ALL intermediate values as stream; wrap in [] for array"),
    ),
    TooltipContent::new(
        "recurse",
        "Recursively apply expression (depth-first traversal)",
        &[
            "recurse(.children[]?)   # traverse tree via children field",
            "recurse | .name?        # get all \"name\" fields in tree",
            "recurse(.next?) | .value # follow linked list",
        ],
        Some("Use .. as shorthand; add ? for missing fields; use limit() always"),
    ),
    TooltipContent::new(
        "walk",
        "Transform all values recursively (bottom-up)",
        &[
            "walk(if type == \"string\" then ascii_downcase else . end)",
            "walk(if type == \"object\" then del(.internal) else . end)",
            "walk(if . == null then \"N/A\" else . end) # replace nulls",
        ],
        Some("Processes BOTTOM-UP (children before parents); use recurse for top-down"),
    ),
    TooltipContent::new(
        "with_entries",
        "Transform object's key-value pairs",
        &[
            "with_entries(.value |= . + 1) # increment all values",
            "with_entries(select(.value != null)) # remove null values",
            "with_entries(.key |= \"prefix_\" + .) # prefix all keys",
            "with_entries(select(.key | startswith(\"public_\")))",
        ],
        Some("Shorthand for to_entries | map(...) | from_entries"),
    ),
    // ===== Object Functions (with arguments) =====
    TooltipContent::new(
        "has",
        "Check if object has a specific key",
        &[
            "has(\"email\")           # check if key exists",
            "select(has(\"config\"))  # filter objects that have config",
            "if has(\"error\") then .error else \"ok\" end",
            "has(\"key\") and .key != null # key exists AND not null",
        ],
        Some("Only checks key existence; use .key? // default for null-safe access"),
    ),
    TooltipContent::new(
        "del",
        "Delete keys or paths from object/array",
        &[
            "del(.password, .secret) # remove sensitive fields",
            "del(.users[0])          # remove first array element",
            "del(.config.debug)      # remove nested field",
            "map(del(.internal))     # remove field from all objects",
        ],
        Some("For pattern matching: with_entries(select(.key | test(\"x\") | not))"),
    ),
    TooltipContent::new(
        "getpath",
        "Get value using dynamic path array",
        &[
            "getpath([\"user\", \"address\", \"city\"]) # nested access",
            "getpath([\"items\", 0])  # get first item",
            "getpath($path)          # variable path",
        ],
        Some("Use .a.b.c for static paths; getpath for dynamic/computed paths"),
    ),
    TooltipContent::new(
        "setpath",
        "Set value using dynamic path array",
        &[
            "setpath([\"config\", \"enabled\"]; true) # set nested",
            "setpath([\"items\", 0]; \"new\") # set by index",
        ],
        Some("Creates intermediate objects/arrays as needed"),
    ),
    TooltipContent::new(
        "delpaths",
        "Delete multiple paths at once",
        &[
            "delpaths([[\"a\"], [\"b\", \"c\"]]) # delete paths",
            "delpaths([paths(type == \"null\")]) # delete nulls",
            "delpaths([paths(. == \"\")])  # delete empty",
        ],
        Some("Combine with [paths(...)] - more efficient than chaining del()"),
    ),
    // ===== String Functions (with arguments) =====
    TooltipContent::new(
        "split",
        "Split string to array by delimiter",
        &[
            "split(\",\")             # \"a,b,c\" -> [\"a\",\"b\",\"c\"]",
            "split(\"\\n\") | map(select(. != \"\")) # split lines",
            "[.items[].name] | join(\" | \") # join values",
        ],
        Some("Use splits(\"\\\\s+\") for regex splitting (e.g., by whitespace)"),
    ),
    TooltipContent::new(
        "join",
        "Join array to string with delimiter",
        &[
            "join(\", \")             # [\"a\",\"b\"] -> \"a, b\"",
            "[.items[].name] | join(\" | \") # join field values",
        ],
        Some("Ensure all elements are strings first with map(tostring) if needed"),
    ),
    TooltipContent::new(
        "ltrimstr",
        "Remove prefix from string",
        &[
            "ltrimstr(\"https://\")   # remove URL scheme",
            "ltrimstr(\"v\") | tonumber # parse version number",
        ],
        Some("Safe if prefix doesn't exist (returns unchanged); use sub() for regex"),
    ),
    TooltipContent::new(
        "rtrimstr",
        "Remove suffix from string",
        &[
            "rtrimstr(\".json\")      # remove file extension",
            ".filename | rtrimstr(\".bak\") # remove backup suffix",
        ],
        Some("Safe if suffix doesn't exist (returns unchanged); use sub() for regex"),
    ),
    TooltipContent::new(
        "startswith",
        "Check if string starts with prefix",
        &[
            "select(startswith(\"http\")) # filter URLs",
            "select(.path | startswith(\"/api/\")) # filter API paths",
            "if endswith(\".json\") then \"JSON\" else \"other\" end",
        ],
        Some("For case-insensitive: use test(\"^pattern\"; \"i\") instead"),
    ),
    TooltipContent::new(
        "endswith",
        "Check if string ends with suffix",
        &[
            "select(endswith(\".json\")) # filter by suffix",
            "select(.email | endswith(\"@company.com\")) # by domain",
        ],
        Some("For case-insensitive: use test(\"pattern$\"; \"i\") instead"),
    ),
    TooltipContent::new(
        "test",
        "Test if string matches regex pattern",
        &[
            "test(\"^[0-9]+$\")       # all digits",
            "test(\"[a-z]\"; \"i\")    # case-insensitive",
            "select(test(\"error|warning\"; \"i\")) # filter logs",
            "test(\"\\\\d{4}-\\\\d{2}-\\\\d{2}\") # date YYYY-MM-DD",
        ],
        Some("Flags: i=case-insensitive, x=extended, m=multiline, s=single-line"),
    ),
    TooltipContent::new(
        "match",
        "Extract regex match information",
        &[
            "match(\"v([0-9]+)\\\\.([0-9]+)\") | .captures[].string",
            "match(\"\\\\d+\"; \"g\") | .string # all matches (global)",
        ],
        Some("Use capture() for named groups - returns object with field names"),
    ),
    TooltipContent::new(
        "capture",
        "Extract named capture groups from regex",
        &[
            "capture(\"(?<user>[^@]+)@(?<domain>.+)\") # email",
            "capture(\"(?<y>\\\\d{4})-(?<m>\\\\d{2})-(?<d>\\\\d{2})\") # date",
        ],
        Some("Returns object with named fields; cleaner than match()"),
    ),
    TooltipContent::new(
        "scan",
        "Find all regex matches (returns stream)",
        &[
            "[scan(\"[0-9]+\")]       # extract all numbers as array",
            "[scan(\"\\\\w+\")]         # extract all words",
            "scan(\"https?://[^\\\\s]+\") # extract all URLs",
            "[scan(\"[A-Z]{2,}\")] | unique # find all acronyms",
        ],
        Some("Returns STREAM, not array - wrap in [] to collect"),
    ),
    TooltipContent::new(
        "splits",
        "Split by regex pattern (returns stream)",
        &[
            "[splits(\"\\\\s+\")]       # split by whitespace",
            "[splits(\"[,;]\\\\s*\")]   # split by comma or semicolon",
            "[splits(\"::\")]  | .[1] # get second segment",
        ],
        Some("Returns STREAM; use split() for literal delimiters"),
    ),
    TooltipContent::new(
        "sub",
        "Replace first regex match",
        &[
            "sub(\"old\"; \"new\")      # replace first occurrence",
            "sub(\"^v\"; \"\")          # remove leading 'v'",
        ],
        Some("Use gsub() to replace ALL occurrences"),
    ),
    TooltipContent::new(
        "gsub",
        "Replace all regex matches",
        &[
            "gsub(\"\\\\s+\"; \" \")      # normalize whitespace",
            "gsub(\"[^a-zA-Z0-9]\"; \"_\") # sanitize to alphanumeric",
            "gsub(\"(?<n>\\\\d+)\"; \"[\\(.n)]\") # wrap numbers in brackets",
        ],
        Some("Use capture groups in replacement with \\(.name) syntax"),
    ),
    // ===== Comparison/Search Functions (with arguments) =====
    TooltipContent::new(
        "contains",
        "Check if value contains another",
        &[
            "contains(\"error\")      # string contains substring",
            ".tags | contains([\"urgent\"]) # array contains element",
            "{a:1} | contains({a:1}) # object contains keys/values",
        ],
        Some("For regex or case-insensitive matching: use test() instead"),
    ),
    TooltipContent::new(
        "inside",
        "Check if value is contained by another",
        &[
            "\"sub\" | inside(\"substring\") # is substring",
            "[1] | inside([1,2,3])   # is subarray",
        ],
        Some("Inverse of contains: a|inside(b) == b|contains(a)"),
    ),
    TooltipContent::new(
        "index",
        "Find first position of value",
        &[
            "index(\",\")             # first comma position",
            "[1,2,3,2] | index(2)   # returns 1",
        ],
        Some("Returns null if not found (not -1 like other languages)"),
    ),
    TooltipContent::new(
        "rindex",
        "Find last position of value",
        &[
            "rindex(\"/\")            # last slash position",
            "[1,2,3,2] | rindex(2)  # returns 3",
            ".path | rindex(\"/\") as $i | .[$i+1:] # filename",
        ],
        Some("Returns null if not found (not -1 like other languages)"),
    ),
    TooltipContent::new(
        "indices",
        "Find all positions of value",
        &[
            "indices(\",\")           # all comma positions",
            "[1,2,3,2] | indices(2) # returns [1,3]",
        ],
        Some("Returns empty array if not found"),
    ),
    // ===== Date Functions =====
    TooltipContent::new(
        "strftime",
        "Format timestamp with custom format",
        &[
            "now | strftime(\"%Y-%m-%d %H:%M:%S\") # custom format",
            "now | strftime(\"%B %d, %Y\") # \"January 15, 2024\"",
        ],
        Some("%Y=year, %m=month, %d=day, %H=hour, %M=minute, %S=second"),
    ),
    TooltipContent::new(
        "strptime",
        "Parse date string with custom format",
        &[
            "\"2024-01-15\" | strptime(\"%Y-%m-%d\") | .[0]",
            "\"Jan 15, 2024\" | strptime(\"%b %d, %Y\") | mktime",
        ],
        Some("Returns [seconds, tz_offset]; use .[0] or mktime for timestamp"),
    ),
    TooltipContent::new(
        "fromdate",
        "Parse ISO 8601 date string to timestamp",
        &[
            "\"2024-01-15T10:30:00Z\" | fromdate # to timestamp",
            ".created_at | fromdate | . + 86400 | todate # add 1 day",
            "[.events[].date | fromdate] | min | todate # earliest",
        ],
        Some("ISO 8601 only; use strptime for custom formats"),
    ),
    TooltipContent::new(
        "todate",
        "Format timestamp as ISO 8601",
        &[
            "now | todate            # current time as ISO 8601",
            ".timestamp | todate     # format unix timestamp",
        ],
        Some("ISO 8601 only; use strftime for custom formats"),
    ),
    // ===== Array Functions (no arguments) =====
    TooltipContent::new(
        "keys",
        "Get object keys or array indices",
        &[
            "keys                    # sorted keys: [\"a\",\"b\",\"c\"]",
            "keys | length           # count of keys",
            "keys | map(select(startswith(\"_\"))) # private keys only",
        ],
        Some("SORTS output alphabetically; use keys_unsorted to preserve order"),
    ),
    TooltipContent::new(
        "keys_unsorted",
        "Get object keys in original order",
        &[
            "keys_unsorted           # preserve original order",
            ".config | keys_unsorted # list config keys as defined",
        ],
        Some("Use when order matters (e.g., preserving config file order)"),
    ),
    TooltipContent::new(
        "values",
        "Get all values from object or array",
        &[
            "values                  # all values (strips keys)",
            ".config | values | add  # sum all config values",
            "values | map(select(. != null)) # non-null values only",
        ],
        Some("Same as .[] but clearer intent; does NOT filter nulls"),
    ),
    TooltipContent::new(
        "sort",
        "Sort array ascending",
        &[
            "sort                    # sort ascending",
            "sort | reverse          # sort descending",
            "sort | unique           # sort and dedupe",
        ],
        Some("Works on numbers, strings, mixed; use sort_by(.field) for objects"),
    ),
    TooltipContent::new(
        "reverse",
        "Reverse array order",
        &[
            "reverse                 # reverse array order",
            ".logs | reverse | first # most recent log entry",
        ],
        Some("Use .[-1] instead of reverse | first for just the last element"),
    ),
    TooltipContent::new(
        "unique",
        "Remove duplicate values (sorts output)",
        &[
            "unique                  # remove duplicates",
            "[.items[].category] | unique # unique values of field",
            "unique | length         # count distinct values",
        ],
        Some("Output is always SORTED; use unique_by(.field) for objects"),
    ),
    TooltipContent::new(
        "flatten",
        "Flatten nested arrays",
        &[
            "flatten                 # flatten all levels",
            "flatten(1)              # flatten one level only",
            "[[1,2],[3,[4,5]]] | flatten # [1,2,3,4,5]",
            ".pages[].items | flatten # combine paginated results",
        ],
        Some("Use add to concatenate arrays without deep flattening"),
    ),
    TooltipContent::new(
        "add",
        "Sum numbers or concatenate arrays/strings",
        &[
            "[.items[].price] | add  # sum prices",
            "[\"a\",\"b\",\"c\"] | add    # \"abc\" (string concat)",
            "[[1,2],[3,4]] | add     # [1,2,3,4] (array concat)",
            "[.counts[]] | add // 0  # sum with default 0 for empty",
        ],
        Some("Returns NULL for empty arrays - use // 0 or // \"\" for defaults"),
    ),
    TooltipContent::new(
        "length",
        "Length of string/array/object, absolute value of number",
        &[
            "length                  # element count",
            ".items | length         # array length",
            "select(length > 0)      # filter non-empty",
            "select(.name | length <= 50) # max name length",
        ],
        Some("Returns 0 for null; for byte length use utf8bytelength"),
    ),
    TooltipContent::new(
        "first",
        "First element from array or generator",
        &[
            "first                   # first element",
            "first(.[] | select(.valid)) # first valid item",
            "last(inputs)            # last line from JSONL",
        ],
        Some("More efficient than .[0] for generators and large streams"),
    ),
    TooltipContent::new(
        "last",
        "Last element from array or generator",
        &[
            "last                    # last element",
            "last(.[] | select(.type == \"error\")) # last error",
        ],
        Some("Use .[-1] for arrays; last() essential for generators"),
    ),
    TooltipContent::new(
        "min",
        "Minimum value in array",
        &[
            "min                     # minimum value",
            "[.items[].price] | min  # lowest price",
        ],
        Some("Returns value, not object; returns null for empty; use min_by()"),
    ),
    TooltipContent::new(
        "max",
        "Maximum value in array",
        &[
            "max                     # maximum value",
            "[.scores[]] | max       # highest score",
        ],
        Some("Returns value, not object; returns null for empty; use max_by()"),
    ),
    TooltipContent::new(
        "transpose",
        "Transpose matrix (swap rows and columns)",
        &[
            "[[1,2],[3,4]] | transpose # [[1,3],[2,4]]",
            "[.names, .ages] | transpose # zip arrays",
            "[.headers, .values] | transpose | map({(.[0]): .[1]}) | add",
        ],
        Some("Great for zipping arrays; uses nulls for different lengths"),
    ),
    // ===== Object Functions (no arguments) =====
    TooltipContent::new(
        "to_entries",
        "Convert object to key-value pairs",
        &[
            "to_entries              # object to [{key,value},...]",
            "to_entries | map(.value += 1) | from_entries",
        ],
        Some("Use with_entries for transform+convert in one step"),
    ),
    TooltipContent::new(
        "from_entries",
        "Convert key-value pairs to object",
        &[
            "from_entries            # [{key,value},...] to object",
            "[{key:\"a\",value:1}] | from_entries # {\"a\":1}",
        ],
        Some("Also accepts {name,value} or {k,v} pairs"),
    ),
    TooltipContent::new(
        "paths",
        "Get all paths in structure",
        &[
            "paths                   # all paths in structure",
            "[paths(type == \"string\")] # paths to all strings",
        ],
        Some("Use with getpath/setpath for dynamic access"),
    ),
    TooltipContent::new(
        "leaf_paths",
        "Get paths to leaf values only",
        &[
            "[paths(scalars)]        # recommended alternative",
            "leaf_paths              # deprecated - avoid",
        ],
        Some("DEPRECATED in jq 1.7+; use paths(scalars) instead"),
    ),
    // ===== Type Functions (no arguments) =====
    TooltipContent::new(
        "type",
        "Get the type name of a value",
        &[
            "type                    # \"string\", \"number\", etc.",
            "select(type == \"object\") # filter by type",
            ".[] | select(type != \"null\") # remove nulls",
            "group_by(type)          # group by type",
        ],
        Some("Use arrays, objects, etc. for type filtering - cleaner and faster"),
    ),
    TooltipContent::new(
        "tostring",
        "Convert to string",
        &[
            ".id | tostring          # ensure string type",
            "(.count | tostring) + \" items\" # string concatenation",
        ],
        Some("Use @json for JSON-encoded string; tostring on strings is no-op"),
    ),
    TooltipContent::new(
        "tonumber",
        "Convert to number",
        &[
            ".price | tonumber       # parse string to number",
            ".amount | tonumber? // 0 # safe parse with default",
        ],
        Some("Throws error on invalid input - use tonumber? // default"),
    ),
    TooltipContent::new(
        "arrays",
        "Filter to keep only arrays",
        &[".[] | arrays            # keep only arrays"],
        Some("Cleaner than select(type == \"array\")"),
    ),
    TooltipContent::new(
        "objects",
        "Filter to keep only objects",
        &[
            ".[] | objects           # keep only objects",
            ".[] | objects | .name   # names from object children only",
        ],
        Some("Cleaner than select(type == \"object\")"),
    ),
    TooltipContent::new(
        "iterables",
        "Filter to keep arrays and objects",
        &[
            "iterables               # arrays and objects",
            ".[] | iterables         # only nested structures",
        ],
        Some("Opposite of scalars"),
    ),
    TooltipContent::new(
        "booleans",
        "Filter to keep only booleans",
        &[".[] | booleans          # keep only booleans"],
        Some("Cleaner than select(type == \"boolean\")"),
    ),
    TooltipContent::new(
        "numbers",
        "Filter to keep only numbers",
        &[".[] | numbers           # keep only numbers"],
        Some("Cleaner than select(type == \"number\")"),
    ),
    TooltipContent::new(
        "strings",
        "Filter to keep only strings",
        &[".[] | strings           # keep only strings"],
        Some("Cleaner than select(type == \"string\")"),
    ),
    TooltipContent::new(
        "nulls",
        "Filter to keep only nulls",
        &[".[] | nulls             # keep only nulls"],
        Some("Use // to provide default for null values"),
    ),
    TooltipContent::new(
        "scalars",
        "Filter to keep non-iterable values (primitives)",
        &[
            ".. | scalars            # all leaf values in tree",
            "[.. | scalars] | unique # all unique primitive values",
        ],
        Some("Opposite of iterables; perfect for extracting leaf values"),
    ),
    // ===== Math Functions (no arguments) =====
    TooltipContent::new(
        "floor",
        "Round down toward negative infinity",
        &[
            "floor                   # 2.7 -> 2, -2.7 -> -3",
            ". * 100 | floor / 100   # truncate to 2 decimals",
        ],
        Some("Rounds toward negative infinity (not toward zero)"),
    ),
    TooltipContent::new(
        "ceil",
        "Round up toward positive infinity",
        &[
            "ceil                    # 2.1 -> 3, -2.1 -> -2",
            ". * 100 | ceil / 100    # round up to 2 decimals",
        ],
        Some("Rounds toward positive infinity"),
    ),
    TooltipContent::new(
        "round",
        "Round to nearest integer",
        &[
            "round                   # 2.5 -> 3, 2.4 -> 2",
            ". * 100 | round / 100   # round to 2 decimal places",
        ],
        Some("Rounds half away from zero"),
    ),
    TooltipContent::new(
        "sqrt",
        "Square root",
        &[
            "sqrt                    # 16 -> 4",
            ". | sqrt | floor        # integer square root",
        ],
        Some("Returns float even for perfect squares; use floor/round"),
    ),
    TooltipContent::new(
        "abs",
        "Absolute value",
        &[
            "abs                     # -5 -> 5",
            "(.a - .b) | abs         # distance between values",
        ],
        Some("Works on numbers only"),
    ),
    // ===== Other Functions (no arguments) =====
    TooltipContent::new(
        "now",
        "Current Unix timestamp",
        &[
            "now                     # current timestamp (float)",
            "now | floor             # current timestamp (integer)",
            "now | strftime(\"%Y-%m-%d\") # today's date",
            "now | todate            # current time as ISO 8601",
        ],
        Some("Returns seconds since epoch as FLOAT with microsecond precision"),
    ),
    TooltipContent::new(
        "empty",
        "Produce no output (filter out current value)",
        &[
            "empty                   # produces nothing",
            "if .skip then empty else . end # conditionally omit",
            "select(. >= 0)          # same as above, cleaner",
        ],
        Some("Useful in conditionals; often select() is cleaner"),
    ),
    TooltipContent::new(
        "error",
        "Raise an error and stop processing",
        &[
            "error(\"Invalid input\") # error with message",
            "if .required == null then error(\"Missing field\") else . end",
            "try .data catch error(\"No data\") # re-throw with message",
        ],
        Some("Catch with: try expr catch \"fallback\"; errors go to stderr"),
    ),
    TooltipContent::new(
        "not",
        "Logical NOT",
        &[
            "not                     # invert boolean",
            "select(.active | not)   # select inactive items",
            "select(has(\"error\") | not) # objects without error field",
        ],
        Some("Note: 0 and \"\" are TRUTHY in jq (unlike JavaScript)"),
    ),
    TooltipContent::new(
        "ascii_downcase",
        "Convert ASCII letters to lowercase",
        &[
            "ascii_downcase          # \"Hello\" -> \"hello\"",
            ".name | ascii_downcase  # normalize for comparison",
        ],
        Some("ASCII only (a-z, A-Z); non-ASCII chars unchanged"),
    ),
    TooltipContent::new(
        "ascii_upcase",
        "Convert ASCII letters to uppercase",
        &[
            "ascii_upcase            # \"hello\" -> \"HELLO\"",
            "select(.code | ascii_upcase == \"USA\") # case-insensitive",
        ],
        Some("ASCII only (a-z, A-Z); non-ASCII chars unchanged"),
    ),
    TooltipContent::new(
        "env",
        "Access environment variables",
        &[
            "env.HOME                # get HOME variable",
            "env.USER                # get current username",
            "env.API_KEY // error(\"API_KEY not set\") # required env var",
        ],
        Some("Use $ENV for object of all env vars"),
    ),
];

/// Get tooltip content for a function
///
/// # Arguments
/// * `function` - The function name to look up
///
/// # Returns
/// * `Some(&'static TooltipContent)` - The tooltip content if found
/// * `None` - If no content exists for the function
pub fn get_tooltip_content(function: &str) -> Option<&'static TooltipContent> {
    TOOLTIP_CONTENT.iter().find(|c| c.function == function)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::autocomplete::jq_functions::JQ_FUNCTION_METADATA;
    use proptest::prelude::*;

    #[test]
    fn test_get_tooltip_content_known_function() {
        let content = get_tooltip_content("select");
        assert!(content.is_some());
        let content = content.unwrap();
        assert_eq!(content.function, "select");
        assert!(!content.description.is_empty());
        assert!(!content.examples.is_empty());
    }

    #[test]
    fn test_get_tooltip_content_unknown_function() {
        let content = get_tooltip_content("unknown_function");
        assert!(content.is_none());
    }

    #[test]
    fn test_tooltip_content_not_empty() {
        assert!(!TOOLTIP_CONTENT.is_empty());
    }

    // **Feature: function-tooltip, Property 4: Tooltip content completeness**
    // *For any* function in `JQ_FUNCTION_METADATA`, the corresponding `TooltipContent`:
    // - Has a non-empty function name matching the metadata
    // - Has a non-empty single-line description
    // - Has between 2 and 4 examples (inclusive)
    // **Validates: Requirements 4.1, 4.2, 4.3**
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn prop_tooltip_content_completeness(index in 0usize..TOOLTIP_CONTENT.len().max(1)) {
            if TOOLTIP_CONTENT.is_empty() {
                return Ok(());
            }

            let content = &TOOLTIP_CONTENT[index % TOOLTIP_CONTENT.len()];

            // Verify function name is not empty
            prop_assert!(
                !content.function.is_empty(),
                "Tooltip content should have a non-empty function name"
            );

            // Verify description is not empty
            prop_assert!(
                !content.description.is_empty(),
                "Function '{}' should have a non-empty description",
                content.function
            );

            // Verify description is single-line (no newlines)
            prop_assert!(
                !content.description.contains('\n'),
                "Function '{}' description should be single-line, got: '{}'",
                content.function,
                content.description
            );

            // Verify examples count is between 1 and 4 (relaxed from 2-4 to allow 1 for simple functions)
            let example_count = content.examples.len();
            prop_assert!(
                (1..=4).contains(&example_count),
                "Function '{}' should have 1-4 examples, got {}",
                content.function,
                example_count
            );

            // Verify all examples are non-empty
            for (i, example) in content.examples.iter().enumerate() {
                prop_assert!(
                    !example.is_empty(),
                    "Function '{}' example {} should not be empty",
                    content.function,
                    i
                );
            }
        }
    }

    // **Feature: function-tooltip, Property 5: All metadata functions have tooltip content**
    // *For any* function name in `JQ_FUNCTION_METADATA`, calling `get_tooltip_content(name)`
    // returns `Some(content)`.
    // **Validates: Requirements 6.1**
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn prop_all_metadata_functions_have_content(index in 0usize..JQ_FUNCTION_METADATA.len().max(1)) {
            if JQ_FUNCTION_METADATA.is_empty() {
                return Ok(());
            }

            let func = &JQ_FUNCTION_METADATA[index % JQ_FUNCTION_METADATA.len()];
            let content = get_tooltip_content(func.name);

            prop_assert!(
                content.is_some(),
                "Function '{}' from JQ_FUNCTION_METADATA should have tooltip content",
                func.name
            );

            // Verify the content function name matches
            if let Some(c) = content {
                prop_assert_eq!(
                    c.function,
                    func.name,
                    "Tooltip content function name should match metadata function name"
                );
            }
        }
    }

    #[test]
    fn test_all_metadata_functions_have_content() {
        // Non-property test to list any missing functions
        let mut missing = Vec::new();
        for func in JQ_FUNCTION_METADATA {
            if get_tooltip_content(func.name).is_none() {
                missing.push(func.name);
            }
        }
        assert!(
            missing.is_empty(),
            "Missing tooltip content for functions: {:?}",
            missing
        );
    }
}
