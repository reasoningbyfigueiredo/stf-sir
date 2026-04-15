# STF-SIR Diff Report Format — v1

**Format identifier:** `stf-sir-diff-v1`

## Overview

A `DiffReport` is a structured, deterministic report that captures the differences between two
STF-SIR ZMD artifacts (A = before, B = after). The report is machine-readable (JSON/YAML) and
is suitable for audit trails, CI change validation, and semantic drift detection.

---

## Fields

| Field | Type | Description |
|---|---|---|
| `format` | `String` | Always `"stf-sir-diff-v1"`. Version sentinel for the report schema. |
| `artifact_a` | `String` | SHA-256 of the serialized artifact A (format: `sha256:<hex>`). |
| `artifact_b` | `String` | SHA-256 of the serialized artifact B (format: `sha256:<hex>`). |
| `structural` | `StructuralDiff` | Structural diff (token set and relation set differences). |
| `semantic` | `SemanticDiff` | Semantic diff (Σ field changes for matched tokens). |
| `summary` | `DiffSummary` | Aggregate counts and identity flag. |

### StructuralDiff fields

| Field | Type | Description |
|---|---|---|
| `added_tokens` | `Vec<String>` | Token IDs present in B but not in A (sorted). |
| `removed_tokens` | `Vec<String>` | Token IDs present in A but not in B (sorted). |
| `added_relations` | `Vec<String>` | Relation IDs present in B but not in A (sorted). |
| `removed_relations` | `Vec<String>` | Relation IDs present in A but not in B (sorted). |
| `modified_node_types` | `Vec<NodeTypeChange>` | Tokens in both A and B whose `syntactic.node_type` changed. |

#### NodeTypeChange fields

| Field | Type | Description |
|---|---|---|
| `token_id` | `String` | The token ID. |
| `before` | `String` | `syntactic.node_type` in A. |
| `after` | `String` | `syntactic.node_type` in B. |

### SemanticDiff fields

| Field | Type | Description |
|---|---|---|
| `gloss_changes` | `Vec<GlossChange>` | Tokens in both A and B whose `semantic.gloss` changed. |
| `concept_changes` | `Vec<ConceptChange>` | Tokens in both A and B whose `semantic.concepts` list changed. |

#### GlossChange fields

| Field | Type | Description |
|---|---|---|
| `token_id` | `String` | The token ID. |
| `before` | `String` | Gloss value in A. |
| `after` | `String` | Gloss value in B. |

#### ConceptChange fields

| Field | Type | Description |
|---|---|---|
| `token_id` | `String` | The token ID. |
| `added` | `Vec<String>` | Concepts present in B but not in A (sorted). |
| `removed` | `Vec<String>` | Concepts present in A but not in B (sorted). |

### DiffSummary fields

| Field | Type | Description |
|---|---|---|
| `added_tokens` | `usize` | `structural.added_tokens.len()` |
| `removed_tokens` | `usize` | `structural.removed_tokens.len()` |
| `modified_tokens` | `usize` | Sum of `modified_node_types.len()` + `gloss_changes.len()` + `concept_changes.len()` |
| `added_relations` | `usize` | `structural.added_relations.len()` |
| `removed_relations` | `usize` | `structural.removed_relations.len()` |
| `is_identical` | `bool` | `true` iff all counts are zero. |

---

## Invariants

### INV-1 — Identity

```
diff_artifacts(A, A).summary.is_identical == true
```

Diffing an artifact against itself must produce an empty report with `is_identical = true`.

### INV-2 — Symmetry (structural counts)

```
diff_artifacts(A, B).summary.added_tokens == diff_artifacts(B, A).summary.removed_tokens
diff_artifacts(A, B).summary.removed_tokens == diff_artifacts(B, A).summary.added_tokens
diff_artifacts(A, B).summary.added_relations == diff_artifacts(B, A).summary.removed_relations
diff_artifacts(A, B).summary.removed_relations == diff_artifacts(B, A).summary.added_relations
```

### INV-3 — Determinism

`diff_artifacts(A, B)` produces byte-identical output on every invocation (no HashMap, no
OS-dependent sorting).

### INV-4 — Non-destructive

Neither artifact is modified by the diff operation.

---

## Example JSON Report

```json
{
  "format": "stf-sir-diff-v1",
  "artifact_a": "sha256:abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890",
  "artifact_b": "sha256:fedcba0987654321fedcba0987654321fedcba0987654321fedcba0987654321",
  "structural": {
    "added_tokens": ["t-007"],
    "removed_tokens": [],
    "added_relations": ["r-003", "r-004"],
    "removed_relations": [],
    "modified_node_types": []
  },
  "semantic": {
    "gloss_changes": [
      {
        "token_id": "t-002",
        "before": "hello world",
        "after": "hello revised world"
      }
    ],
    "concept_changes": []
  },
  "summary": {
    "added_tokens": 1,
    "removed_tokens": 0,
    "modified_tokens": 1,
    "added_relations": 2,
    "removed_relations": 0,
    "is_identical": false
  }
}
```

---

## Usage

```rust
use stf_sir::diff::diff_artifacts;

let report = diff_artifacts(&artifact_a, &artifact_b);
println!("{}", report.to_json());
println!("{}", report.to_yaml());
```
