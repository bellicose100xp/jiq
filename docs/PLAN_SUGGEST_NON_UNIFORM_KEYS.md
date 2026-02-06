# Plan: Suggest Non-Uniform Keys

## Problem Statement

I have a JSON document with non-uniform keys across array elements. As I type and refine a jq filter, I expect autocomplete to show the keys that are actually available at the level I am currently navigating.

Today, suggestions are often biased toward the first available element, or toward a parsed streaming cache that only contains one object. That makes suggestions feel inconsistent across equivalent query shapes.

**NOTE**: This first available element bias was a deliberate performance decision, not a bug. This plan explores a hybrid approach: keep that default fast path, and add bounded enrichment only where it meaningfully improves correctness for non-uniform and streamed data.

## Canonical JSON Fixture

```json
{
  "services": [
    {
      "name": "api",
      "deployments": [
        {
          "id": 1,
          "tasks": [
            {
              "tid": 1
            },
            {
              "tid": 2,
              "extra_task_key": true
            }
          ]
        },
        {
          "id": 2,
          "extra_deploy_key": 42,
          "tasks": [
            {
              "tid": 3
            },
            {
              "tid": 4,
              "payload": "foo"
            }
          ]
        }
      ]
    },
    {
      "name": "worker",
      "extra_service_key": true,
      "deployments": [
        {
          "id": 3,
          "tasks": [
            {
              "tid": 5
            },
            {
              "tid": 6,
              "payload": "bar"
            }
          ]
        }
      ]
    }
  ]
}
```

## User Journey: Expected Suggestions

### Step 1
Query as typed:
`.services[].`

Expectation:
- Show keys from all sampled service elements at that level
- Includes at least:
  - `name`
  - `deployments`
  - `extra_service_key` (actual: this is missing)

### Step 2
Query as typed:
`.services[] | select(.`

Expectation:
- Same logical level as Step 1
- Suggestions should still include:
  - `name`
  - `deployments`
  - `extra_service_key` (actual: this is missing)

### Step 3
Query as typed:
`.services[] | .`

Expectation:
- Same logical level as Step 1 and Step 2
- Suggestions should be equivalent at this level

### Step 4
Query as typed:
`.services | map(select(.extra_service_key)) | .[].deployments[].tasks.`

Expectation:
- Suggest task element keys at this level
- Includes:
  - `.[].tid`
  - `.[].payload` (actual: this is missing)

### Step 5
Query as typed:
`.services | map(select(.extra_service_key)) | .[].deployments[].tasks[].`

Expectation:
- Same task element level as Step 4
- Suggest element keys:
  - `tid`
  - `payload` (actual: this is missing)

## Why Current Behavior Misses These Cases

### 1) First-element bias in array navigation
- Core navigation path tends to use first array element semantics.
- This hides keys that only appear in later elements.

### 2) Streamed result cache only parses first object
- For destructured output, cached parsed JSON may only represent one object.
- At `tasks[]` element-level suggestions, this can surface `tid` but miss `payload`.

### 3) Equivalent query shapes route differently
- `.services[].`, `.services[] | select(.`, and `.services[] | .` can take different internal paths.
- Without shared routing semantics, equivalent levels produce non-equivalent suggestions.

### 4) Pipe-relative context is ambiguous without ancestry
- After a pipe and empty local path (`| .`), the current level is not recoverable from local syntax alone.
- We need iterator ancestry (provenance) to recover intended level.

## Proposed Design Model
These concepts are proposed design additions. Together, they define the minimal architecture needed to make iterator-level suggestions consistent across query shapes while preserving predictable autocomplete performance.

## 1) Target Level Resolution
Purpose:
- Decide what JSON level (the exact object/array node in the tree the cursor is currently targeting for suggestions) autocomplete is targeting before generating suggestions.

Outputs:
- source selection (result cache, original JSON, all-known-fields fallback)
- target kind (value-at-path, array-elements-at-path, fallback)

Why needed:
- Prevents query-shape-specific ad hoc logic.
- Makes equivalent cursor scenarios resolve consistently.

## 2) Iterator Provenance
Purpose:
- Recover last relevant iterator ancestry from the query before cursor.

Usage:
- When local path is empty or pipe-relative, provenance maps cursor back to intended array level.

Why needed:
- Makes `.services[] | .` align with `.services[] | select(.` at suggestion level.

## 3) Key Enrichment Strategy
Purpose:
- Determine how keys are collected once target level is known.

Modes:
- first-object (default conservative behavior)
- scan-ahead union of first N elements
- stream scan-ahead for destructured outputs

Why needed:
- Makes non-uniform keys visible without requiring full scan every time.

## 4) Stream-Aware Enrichment
Purpose:
- Handle destructured stream outputs where parsed cache only contains first object.

Approach:
- Sample first N streamed objects from unformatted output and union keys.

Why needed:
- Required to make Step 5 show `payload` reliably.

## Strategy Summary (Target State)

| Scenario | Primary strategy | Fallback |
|---|---|---|
| explicit path to object level | value-at-path routing | original JSON, then all-known-fields |
| explicit iterator tail (`[]`) | array-elements-at-path routing | first-object keys |
| empty local path after pipe with iterator ancestry | provenance-resolved array-elements target | all-known-fields |
| array element key discovery | scan-ahead union (N) | first-object |
| streamed element key discovery | stream scan-ahead union (N) | first streamed object |

## Implementation Phases

### Phase 1: Routing Foundation
- Introduce target-level resolver and router entrypoint.
- Normalize source/target decisions in one place.

### Phase 2: Provenance Integration
- Extract iterator ancestry from query-before-cursor.
- Feed provenance into target-level resolver for pipe-relative/empty local path cases.

### Phase 3: Array Key Enrichment
- Add scan-ahead union behavior with configurable size.
- Keep first-object fallback behavior.

### Phase 4: Streaming Gap Closure
- Add stream scan-ahead for destructured outputs.
- Unify behavior between `tasks.` and `tasks[].` element-level suggestions.

### Phase 5: Optional-Operator Parity
- Ensure optional forms (`[]?`, `.foo?`) map to same semantic level behavior.

## Rollout and Gating

- Gate non-uniform key enrichment behind an environment variable first, and use the same variable to control scan-ahead size (N), where invalid/missing values keep default first-object behavior (e.g. `JIQ_AUTOCOMPLETE_ARRAY_SCAN_AHEAD=14`)
- Keep this as an opt-in safety valve during rollout so behavior can be disabled instantly without code rollback.
- If the feature proves valuable, stable, and perfomant, migrate this setting to the main config file for discoverability and persistence.
- Preserve tiny seams in core architecture:
  - decision seam: target/source resolution in one resolver layer
  - enrichment seam: key-union logic isolated from navigation/routing logic
  - fallback seam: default first-object behavior remains intact and callable
- Prefer additive integration over broad rewrites so removal/revert is low-risk and localized.

## Acceptance Criteria

1. `.services[].` includes `extra_service_key`.
2. `.services[] | select(.` and `.services[] | .` produce equivalent level suggestions.
3. `.services | map(select(.extra_service_key)) | .[].deployments[].tasks.` includes `.[].payload`.
4. `.services | map(select(.extra_service_key)) | .[].deployments[].tasks[].` includes `payload` after streaming enrichment is introduced.
5. Invalid/missing enrichment config falls back cleanly to first-object behavior.

## Out of Scope (For This Plan)

- UI affordances (spinner/timeouts/badges for eventual consistency)
- Full-query jq re-execution for autocomplete-only paths
- Full AST parser for jq syntax (referencing the jq parser and lexer logic itself)
