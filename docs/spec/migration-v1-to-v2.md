---
id: MIGRATION-V1-TO-V2
version: 2.0.0-alpha
status: draft
created: 2026-04-14
updated: 2026-04-14
normative: true
tags:
  - migration
  - v1
  - v2
  - backward-compat
---

# STF-SIR Migration Guide: v1 → v2

This guide covers all changes between STF-SIR v1 and v2, with concrete
examples and migration instructions for producers (compilers), consumers
(readers), and test suites.

---

## 1. Quick Summary

| Change | Category | Impact on producers | Impact on consumers |
|---|---|---|---|
| `version` field: `1` → `2` | Field value | MUST update | MUST accept both |
| Four new ZToken dimensions (C, P, Δ, Ω) | New optional fields | MAY populate | MUST accept absent fields |
| Language detection | `document.language` | MUST detect; MAY fall back to `"und"` | MUST accept `"und"` |
| Five new relation types | `relations[].type` | MAY emit | MUST accept all |
| New validation rules VAL_19–VAL_30 | Validation | MUST implement | MUST validate |
| `stf-sir-spec-v2-block` profile | `compiler.profile` | MUST declare v2 profile for v2 artifacts | MAY use either profile |
| Sentence and entity profiles | New profiles | MAY adopt | MUST accept if declared |

**No field has been removed or renamed.** A valid v1 artifact is a valid v2
artifact with `version: 1` and no new fields populated.

---

## 2. Version Field

### v1
```yaml
version: 1
```

### v2
```yaml
version: 2
```

**Migration:** Change `version: 1` to `version: 2` in produced artifacts.
Consumers MUST accept both values and gate v2-only validation rules (VAL_19–VAL_30)
on `version: 2`.

---

## 3. Language Detection

### v1 behavior
```yaml
document:
  language: und
```
Language detection was not required; the compiler unconditionally emitted `"und"`.

### v2 requirement
```yaml
document:
  language: pt    # detected: Portuguese
```

**Migration for producers:**
1. Add a BCP 47 language detection step to the compiler pipeline.
2. If detection confidence ≥ 0.7, set `document.language` to the detected tag.
3. If confidence < 0.7 or detection unavailable, set `document.language: und`.

**Migration for consumers:**
- No change required. `"und"` remains valid.
- Consumers processing multilingual corpora SHOULD check the `language` field
  before language-specific operations.

**Validation:**
- VAL_19 (ERROR): `document.language` MUST be a valid BCP 47 tag or `"und"`.
- VAL_20 (WARNING): `document.language` SHOULD NOT be `"und"` for v2 artifacts.

---

## 4. New ZToken Dimensions

The four new dimensions are entirely optional in v2. No existing ZToken
structure is modified.

### 4.1 C — Contextual Dimension

**Adding context fields to an existing token:**
```yaml
# v1 token (still valid in v2)
- id: z1
  lexical:
    source_text: "The system should respond within 5ms."
  syntactic:
    node_type: paragraph
    ...

# v2 token with C dimension populated
- id: z1
  lexical:
    source_text: "The system should respond within 5ms."
  syntactic:
    node_type: paragraph
    ...
  context:
    context_id: requirements.performance
    scope: section-3
    reference_frame: system
```

**Migration:** Populate `context.*` fields as your compiler gains discourse-
analysis capabilities. No existing code needs updating; the fields are absent
by default.

### 4.2 P — Pragmatic Dimension

```yaml
# v2 token with P dimension
- id: z2
  ...
  pragmatic:
    intent: assert
    speech_act: illocution
    register: technical
```

### 4.3 Δ — Temporal Dimension

```yaml
# v2 token with Δ dimension
- id: z3
  ...
  temporal:
    created_at: "2026-04-14T10:00:00Z"
    valid_from:  "2026-04-14T00:00:00Z"
    valid_to:    "2027-04-14T00:00:00Z"
```

**Constraint:** `valid_from ≤ valid_to` (VAL_23).

### 4.4 Ω — Coherence/Validation Dimension

```yaml
# v2 token with Ω dimension
- id: z4
  ...
  coherence_eval:
    coherence_score: 0.92
    validation_flags:
      - VAL_01
      - VAL_08
    useful_information: true
```

---

## 5. New Relation Types

### 5.1 Adding new relation types

```yaml
# v1 — only contains and precedes
relations:
  - id: r1
    type: contains
    source: z1
    target: z2
    category: structural
    stage: syntactic

# v2 — adding a supports relation
relations:
  - id: r1
    type: contains
    source: z1
    target: z2
    category: structural
    stage: syntactic
  - id: r2
    type: supports
    source: z3
    target: z4
    category: logical
    stage: semantic
```

### 5.2 New type reference

| Type | Category | Stage |
|---|---|---|
| `supports` | logical | semantic, logical |
| `contradicts` | logical | semantic, logical |
| `elaborates` | logical | semantic |
| `refers_to` | semantic-link | lexical, semantic |
| `cites` | semantic-link | semantic, logical |

**Constraint:** `supports` and `contradicts` MUST NOT both appear for the same
source-target pair (VAL_26).

**Constraint:** A `cites` relation source MUST have non-empty
`provenance.source_ids` in its bridged statement (VAL_27).

---

## 6. Profile Migration

### 6.1 Changing profile identifier

```yaml
# v1
compiler:
  profile: stf-sir-spec-v1-mvp

# v2 (block-level, same node_type set)
compiler:
  profile: stf-sir-spec-v2-block
```

### 6.2 Adopting sentence or entity profiles

To emit sentence-level tokens, declare the sentence profile and ensure all
`sentence`, `clause`, and `phrase` node_type values are used correctly (VAL_28).

```yaml
compiler:
  profile: stf-sir-spec-v2-sentence
```

---

## 7. Validation Rule Changes

### 7.1 Rules carried over from v1 (VAL_01–VAL_18)

All 18 v1 rules are preserved verbatim. No v1 rule has been removed, renamed,
or relaxed.

### 7.2 New rules gated on `version: 2`

The following rules apply only when `version: 2` is declared:

| Rule | Gated on v2 | Brief |
|---|---|---|
| VAL_19 | Yes | `document.language` must be valid BCP 47 or `"und"` |
| VAL_20 | Yes | Warning: `"und"` should not be used in v2 |
| VAL_21 | Yes | `context.context_id` pattern check |
| VAL_22 | Yes | `pragmatic.intent` enum check |
| VAL_23 | Yes | `temporal.valid_from ≤ valid_to` |
| VAL_24 | Yes | `coherence_score ∈ [0.0, 1.0]` |
| VAL_25 | Yes | `validation_flags` format check |
| VAL_26 | Yes | No simultaneous `supports` + `contradicts` for same pair |
| VAL_27 | Yes | `cites` source must have non-empty `source_ids` |
| VAL_28 | Yes | Sentence-profile `node_type` check |
| VAL_29 | Yes | Entity-profile `node_type` check |
| VAL_30 | Yes | Enricher in `config_hash` check |

---

## 8. Rust API Changes (FEAT-201-5)

### 8.1 `Statement.semantic_dimensions`

The `Statement` struct gains an optional `semantic_dimensions` field:

```rust
// v1 code — unchanged, still works
let stmt = Statement::atomic("s1", "A", "test");
// stmt.semantic_dimensions == None (default)

// v2 code — populate after evaluation
let result = engine.evaluate_statement(&theory, &stmt);
let dims = SemanticDimensions::from_evaluation(&result);
// attach to statement (builder pattern or direct assignment)
```

The field is `Option<SemanticDimensions>` and is `None` by default. No
existing code is broken.

### 8.2 `SemanticDimensions::from_evaluation`

```rust
use stf_sir::SemanticDimensions;

let dims = SemanticDimensions::from_evaluation(&result);
assert_eq!(dims.coherence, result.coherence);
assert_eq!(dims.transformation_delta, 0.0); // v1: always 0.0
```

### 8.3 Unchanged public API

All v1 public symbols remain unchanged. The following additions are purely
additive:
- `stf_sir::SemanticDimensions` (new struct)
- `stf_sir::model::SemanticDimensions` (same, via model module)
- `Statement.semantic_dimensions: Option<SemanticDimensions>` (new field)

---

## 9. Conformance Kit Migration

v1 conformance kits remain valid for testing v1 behavior. To claim v2
conformance, implementations MUST additionally pass the v2 fixture set
(see `stf-sir-spec-v2.md §10`).

Run the v2 kit against your implementation:

```bash
# reference implementation
cargo test --test conformance_v2

# external implementation
./run-conformance-kit.sh --spec v2 --impl path/to/your/impl
```

---

## 10. Checklist for Producers

- [ ] Change `version: 1` → `version: 2` in output
- [ ] Declare v2 profile in `compiler.profile`
- [ ] Implement language detection; set `document.language` correctly (VAL_19)
- [ ] Implement VAL_19–VAL_30 validation rules
- [ ] Optionally populate C, P, Δ, Ω dimensions as capabilities become available
- [ ] Pass the v2 conformance kit (≥ 95% across all categories)

## 11. Checklist for Consumers

- [ ] Accept `version: 1` and `version: 2` artifacts
- [ ] Treat absent optional v2 fields as `null` / not-present
- [ ] Gate v2-only validation (VAL_19–VAL_30) on `version: 2`
- [ ] Accept all 7 relation types (5 existing + 5 new)
- [ ] Accept `"und"` as a valid language tag
