# Realistic Autocomplete Improvement Options for jiq
**Date**: December 1, 2025
**Context**: Alternatives to lexical parsing and tree-sitter approaches

---

## Key Research Findings

### 1. Even jq-lsp Struggles With This Problem

The official [jq-lsp language server](https://github.com/wader/jq-lsp) uses a **modified gojq parser** but explicitly notes in their TODO:

> "Better at handling broken syntax while typing"
> - Incomplete variable declarations: `$<cursor>`
> - Partial pipe chains: `a | n<cursor> f`
>
> **Planned solution**: "Create own parser or modified gojq parser to be able to recover and give more useful errors"

**Translation**: Even the LSP developers recognize that standard parsers fail at autocomplete for incomplete input.

### 2. Elasticsearch's Lesson: Command-Specific Logic Wins

[Elasticsearch rebuilt their ES|QL autocomplete](https://www.elastic.co/search-labs/blog/esql-autocomplete-rebuilt) with a critical insight:

**Old approach (failed)**:
- Declarative, generic logic
- Static command signatures
- One-size-fits-all handling

**New approach (succeeded)**:
- **Imperative, command-specific logic**
- Each command has its own `suggest()` method
- No generic routines trying to work for everything

**Quote**: "Each command gets its own logic. There is no generic routine that is supposed to work for all the commands."

**Relevance to jiq**: This validates jiq's current approach! The 13 CharType-specific insertion formulas are actually the RIGHT design pattern—they just need refinement, not replacement.

### 3. Parser Integration Challenges

From [Stack Overflow discussion on autocomplete with parsers](https://stackoverflow.com/questions/12512653/autocomplete-intergration-with-parser):

> "Parsers tend to be optimized for syntactically correct programs, and may not work at all after detecting a syntax error."
>
> "You'd need to hook into the symbol table in order to get accurate autocomplete information."

**Key insight**: Parsers are the WRONG tool for autocomplete. You need something purpose-built.

---

## Realistic Improvement Options (Ranked)

### ✅ Option 1: Enhanced Lexical Parsing (RECOMMENDED)

**Concept**: Keep lexical approach, add sophisticated pattern recognition

#### What This Means

Instead of character-by-character scanning, use **regex-based pattern matching** to recognize semantic constructs:

```rust
// Current: single character classification
CharType::classify_char(Some('|')) -> PipeOperator

// Enhanced: pattern-based classification
SyntaxContext::analyze(before_cursor) -> {
    last_complete_token: Token::Pipe,
    incomplete_token: Some(Token::FieldAccess(".ca")),
    parent_context: Context::ConditionalThen,  // recognizes "if ... then"
    nesting_level: 2,                          // tracks (), [], {}
    in_string: false,
}
```

#### Concrete Improvements

**1. Pattern-Based Keyword Recognition**

```rust
static KEYWORD_PATTERNS: &[(&str, KeywordType)] = &[
    (r"\bif\s+.*\s+then\s+$", KeywordType::ConditionalThen),
    (r"\btry\s+$", KeywordType::TryBlock),
    (r"\bas\s+\$\w*$", KeywordType::VariableBinding),
    (r"\|=\s*$", KeywordType::UpdateOperator),
    // etc.
];

fn detect_keyword_context(before_cursor: &str) -> Option<KeywordType> {
    for (pattern, keyword_type) in KEYWORD_PATTERNS {
        if Regex::new(pattern).unwrap().is_match(before_cursor) {
            return Some(*keyword_type);
        }
    }
    None
}
```

**2. Operator-Aware Context Tracking**

```rust
enum JqOperatorContext {
    Pipe,           // " | "
    UpdateAssign,   // " |= "
    Alternative,    // " // "
    Optional,       // "?"
    RecursiveDescent, // ".."
}

// Distinguish: ".field | " vs ".field |= " vs ".field // "
```

**3. Balanced Bracket Tracking**

```rust
struct BracketContext {
    parens: Vec<usize>,    // Track '(' positions
    brackets: Vec<usize>,  // Track '[' positions
    braces: Vec<usize>,    // Track '{' positions
}

// Enables: "map(select(.x | .<cursor>)"
//          → knows we're inside function call inside function call
```

**4. String Context Detection**

```rust
fn is_inside_string(text: &str, pos: usize) -> bool {
    let mut in_string = false;
    let mut escaped = false;
    for (i, ch) in text[..pos].chars().enumerate() {
        if escaped {
            escaped = false;
        } else if ch == '\\' {
            escaped = true;
        } else if ch == '"' {
            in_string = !in_string;
        }
    }
    in_string
}

// Disables autocomplete inside strings: ".field | \"text .<cursor>\""
```

#### Benefits

- **Builds on existing foundation** (320+ tests keep passing)
- **Semantic awareness** without parsing complexity
- **Handles incomplete input naturally** (lexical analysis doesn't break)
- **Low risk, incremental implementation**
- **Fast** (regex matching is microseconds)

#### Implementation Effort

**Estimated time**: 5-7 days
- Day 1-2: Implement pattern-based keyword detection
- Day 3-4: Add operator-aware context tracking
- Day 5: Integrate bracket and string context
- Day 6-7: Testing and refinement

#### Expected Code Reduction

- Consolidate 13 CharType variants → 6-8 context-aware strategies
- Extract common patterns → ~500 lines saved
- **Result**: 2,470 lines → ~1,900-2,000 lines (20% reduction)

---

### ⚠️ Option 2: Pratt Parser with Error Recovery (MODERATE RISK)

**Concept**: Lightweight operator-precedence parser designed for REPL use

#### What This Means

[Pratt parsers](https://en.wikipedia.org/wiki/Operator-precedence_parser#Pratt_parsing) are perfect for expression-heavy languages like jq:

```rust
// Pratt parser naturally handles operator precedence
".field | select(.x > 5) // default"
       ↓
    PIPE
   /    \
.field   ALT
        /   \
    SELECT  default
       |
    COMPARE
     /   \
   .x     5
```

#### Key Advantage: Designed for Incomplete Input

Unlike tree-sitter, Pratt parsers can be designed to **return partial results**:

```rust
enum ParseResult<T> {
    Complete(T),
    Partial {
        parsed: T,
        expected: Vec<TokenType>,  // What could come next
        context: ParseContext,      // Where are we in the grammar
    },
    Error(ParseError),
}

// Example:
parse(".field | select(.x ")
  → Partial {
      parsed: Pipe(.field, FunctionCall("select", [FieldAccess("x")])),
      expected: [Operator, CloseParen],
      context: InsideFunctionCall,
  }
```

#### Benefits

- **Semantic understanding** (knows operator precedence)
- **Handles incomplete input** (designed for REPL)
- **Lightweight** (~500-800 lines for basic jq subset)
- **No external dependencies** (hand-written)

#### Challenges

- **Custom implementation required** (no existing jq Pratt parser)
- **Won't cover ALL jq syntax** (focus on common expressions)
- **More complex than lexical** (but simpler than full parser)
- **Testing burden** (need to verify parsing is correct)

#### Implementation Effort

**Estimated time**: 10-15 days
- Day 1-3: Implement tokenizer with error recovery
- Day 4-7: Build Pratt parser for core jq operators
- Day 8-10: Add partial result handling
- Day 11-12: Integrate with autocomplete
- Day 13-15: Testing against existing test suite

#### Realistic Assessment

**This could work**, but it's a significant investment. Consider this if:
- You want semantic understanding of operator precedence
- You're planning other features that need parsing (formatter, linter)
- You have time for a 2-3 week project

---

### ⚠️ Option 3: Token-Stream Analysis (HYBRID)

**Concept**: Tokenize without parsing, use token patterns for context

#### What This Means

Middle ground between lexical and parsing:

```rust
// Tokenize the query
let tokens = tokenize(".services | if .x then .");
// → [Field("services"), Pipe, Keyword("if"), Field("x"),
//    Keyword("then"), Dot]

// Pattern matching on token sequences
match &tokens[tokens.len()-3..] {
    [Keyword("then"), Dot] => Context::ConditionalThen,
    [Keyword("else"), Dot] => Context::ConditionalElse,
    [Pipe, Dot] => Context::AfterPipe,
    // etc.
}
```

#### Benefits

- **Better than character-level** (recognizes keywords, operators)
- **Simpler than parsing** (no AST, just token sequences)
- **Decent error tolerance** (tokenization rarely fails)
- **Can reuse jq's lexer** (if available) or write simple one

#### Challenges

- **Limited semantic understanding** (doesn't know nesting depth)
- **Pattern matching can be brittle** (need many patterns)
- **May not be much better than Option 1** (regex can do similar things)

#### Implementation Effort

**Estimated time**: 7-10 days

#### Realistic Assessment

This is a **sideways move** from current system. Only worth it if you want a tokenizer for other purposes (syntax highlighting, formatting).

---

### ❌ Option 4: Modified Tree-Sitter with Custom Error Rules (HIGH RISK)

**Concept**: Fork tree-sitter-jq and add autocomplete-specific error recovery

#### What This Means

Modify the tree-sitter grammar to recognize common incomplete patterns:

```javascript
// In grammar.js
field_access_incomplete: $ => seq(
  '.',
  optional($.identifier),
  INCOMPLETE_MARKER,  // Special token indicating cursor position
),
```

#### Why This Might Work (Theoretically)

- Add explicit grammar rules for incomplete constructs
- Generate autocomplete hints during parsing
- Keep tree-sitter's speed and structure

#### Why This Won't Work (Practically)

1. **Requires maintaining a fork** of tree-sitter-jq grammar
2. **Every new incomplete pattern needs explicit grammar rules** (combinatorial explosion)
3. **Goes against tree-sitter's design** (it's optimized for complete code)
4. **No community support** (you're on your own)

**Estimated time**: 20-30 days (and ongoing maintenance)

**Recommendation**: ❌ Don't do this. The effort exceeds the benefit.

---

### ❌ Option 5: Full Custom Parser (VERY HIGH RISK)

**Concept**: Write a complete recursive-descent parser for jq

#### Reality Check

jq's grammar is complex:
- 50+ built-in functions
- Nested expressions, patterns, destructuring
- Module system, imports, definitions
- Format strings, string interpolation
- Complex operator precedence

**Estimated implementation**: 6-8 weeks for basic coverage, 3-6 months for complete

**Maintenance burden**: Every jq version adds new features you must support

**Recommendation**: ❌ Not worth it for a TUI tool. Let jq-lsp handle this.

---

## Recommended Path Forward

### Phase 1: Enhanced Lexical Parsing (5-7 days)

Implement **Option 1** with these specific enhancements:

#### 1. Operator Pattern Recognition

```rust
pub enum JqOperator {
    Pipe,           // " | "
    UpdatePipe,     // " |= "
    Alternative,    // " // "
    Optional,       // "?"
    Recursive,      // ".."
}

pub fn detect_operator(before_cursor: &str) -> Option<JqOperator> {
    let trimmed = before_cursor.trim_end();
    if trimmed.ends_with(" | ") { return Some(JqOperator::Pipe); }
    if trimmed.ends_with(" |= ") { return Some(JqOperator::UpdatePipe); }
    if trimmed.ends_with(" // ") { return Some(JqOperator::Alternative); }
    if trimmed.ends_with("?") { return Some(JqOperator::Optional); }
    if trimmed.ends_with("..") { return Some(JqOperator::Recursive); }
    None
}
```

#### 2. Keyword Context Detection

```rust
pub enum KeywordContext {
    ConditionalIf,
    ConditionalThen,
    ConditionalElse,
    TryBlock,
    CatchBlock,
    VariableBinding,  // "as $var"
}

pub fn detect_keyword_context(before_cursor: &str) -> Option<KeywordContext> {
    // Use regex to detect:
    // - "if ... then " → ConditionalThen
    // - "else " → ConditionalElse
    // - "try " → TryBlock
    // - "as $" → VariableBinding
}
```

#### 3. Balanced Bracket Tracking

```rust
pub struct BracketState {
    pub open_parens: usize,
    pub open_brackets: usize,
    pub open_braces: usize,
    pub in_function_call: bool,
}

pub fn analyze_brackets(text: &str) -> BracketState {
    // Track all bracket types
    // Determine if cursor is inside function call
}
```

#### 4. Consolidate Insertion Strategies

Reduce 13 CharType formulas to **6 semantic strategies**:

```rust
pub enum InsertionStrategy {
    ContinueField,      // ".services.ca" → continuing same path
    StartNewPath,       // " | ." → new path after operator
    ArrayAccess,        // ".services[]." → after array indexing
    FunctionArgument,   // "map(." → inside function
    UpdateOperation,    // " |= ." → update operator context
    ConditionalBranch,  // "then ." → inside if/then/else
}
```

#### Expected Outcome

- **Code reduction**: 2,470 → 1,900-2,000 lines (20% reduction)
- **Better semantic awareness**: Recognizes operators, keywords, nesting
- **Preserved quality**: All 320+ tests pass
- **Foundation for future**: Easy to add more patterns

---

### Phase 2 (Optional): Pratt Parser for Advanced Features (10-15 days)

If Phase 1 succeeds and you want more semantic understanding:

Implement **Option 2** (Pratt parser) for:
- Type inference through pipe chains
- Better error messages
- Query formatting/pretty-printing
- Variable tracking

This becomes a **separate module** alongside lexical autocomplete:

```rust
// app_state.rs
pub struct App {
    autocomplete: AutocompleteState,
    lexical_analyzer: LexicalAnalyzer,     // Phase 1 (always used)
    pratt_parser: Option<PrattParser>,     // Phase 2 (optional, for advanced features)
}
```

Use Pratt parser only for **complete expressions**, fall back to lexical for **incomplete input**.

---

## Comparison Table

| Approach | Complexity | Benefit | Risk | Time | Recommendation |
|----------|-----------|---------|------|------|----------------|
| **Enhanced Lexical** | Low | Medium-High | Low | 5-7 days | ✅ Do This |
| **Pratt Parser** | Medium | High | Medium | 10-15 days | ⚠️ Consider Later |
| **Token Stream** | Low-Medium | Low-Medium | Low | 7-10 days | ⚠️ Sideways Move |
| **Modified Tree-Sitter** | High | Medium | High | 20-30 days | ❌ Don't Do |
| **Full Custom Parser** | Very High | High | Very High | 2-6 months | ❌ Don't Do |

---

## Key Insights

### 1. Generic Parsing is the Wrong Approach

**Elasticsearch's lesson applies to jiq**: Command-specific (or in jiq's case, operator-specific) logic is BETTER than generic parsing. Your current 13 CharType formulas are on the right track—they just need refinement.

### 2. Incomplete Input Breaks Most Parsers

This is a **fundamental limitation** of traditional parsers. They're optimized for correct syntax, not autocomplete. Even jq-lsp struggles with this.

### 3. The Best Is Lexical + Patterns

Combining character-level analysis with regex pattern matching gives you:
- Speed of lexical analysis
- Semantic awareness of patterns
- Natural handling of incomplete input
- No external dependencies

---

## Conclusion

**The answer to your question**: Yes, there are realistic ways to improve autocomplete beyond pure character-by-character lexical parsing, but **no, there's no silver bullet that dramatically improves on the lexical approach**.

The best path forward is:

1. ✅ **Enhance your existing lexical system** with pattern matching, operator detection, and keyword awareness
2. ⚠️ **Consider a lightweight Pratt parser later** if you need semantic understanding for other features
3. ❌ **Avoid full AST parsing** (tree-sitter, custom parser) for autocomplete—the ERROR node problem is fundamental

Your current architecture is actually sound—it just needs **refinement, not replacement**.

---

## Research Sources

- [jq-lsp GitHub Repository](https://github.com/wader/jq-lsp) - Official jq language server
- [Elasticsearch: How we rebuilt autocomplete for ES|QL](https://www.elastic.co/search-labs/blog/esql-autocomplete-rebuilt) - Command-specific vs generic autocomplete
- [Stack Overflow: Autocomplete integration with parser](https://stackoverflow.com/questions/12512653/autocomplete-intergration-with-parser) - Parser limitations for autocomplete
- [What do IDEs use for code completion?](https://softwareengineering.stackexchange.com/questions/408680/what-do-ides-use-to-do-code-completion-suggestions) - General IDE completion strategies
- [Recursive Descent Parser for IDEs](https://thunderseethe.dev/posts/parser-base/) - Error recovery in parsers
