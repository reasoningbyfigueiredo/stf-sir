---
id: STF-SIR-SPEC-V2
version: 2.0.0-alpha
status: draft
created: 2026-04-14
updated: 2026-04-14
owner: Rogerio Figueiredo
system: STF-SIR
type: normative-spec
language: en
normative: true
tags:
  - spec
  - v2
  - ztoken
  - dimensions
  - validation-rules
---

# STF-SIR Specification v2

## 1. Introduction

This document is the normative specification for STF-SIR v2. It extends v1
(`SPEC-STF-CORE-SEMANTICS`) with four deferred STS dimensions (C, P, Δ, Ω),
a v2 ZToken structure, an extended relation taxonomy, sentence-level and
entity-level profiles, and a mandatory language detection requirement.

**Backward compatibility guarantee:** Every valid v1 `.zmd` artifact MUST
be accepted by a v2 reader without modification. New fields are optional or
version-gated (see §8).

### 1.1 Normative Language

The key words MUST, MUST NOT, REQUIRED, SHALL, SHOULD, RECOMMENDED, MAY and
OPTIONAL in this document are to be interpreted as in RFC 2119.

### 1.2 Scope

- **In scope:** ZToken v2 structure, validation rules VAL_01–VAL_30, relation
  taxonomy v2, profile registry v2, conformance kit v2, backward-compatibility
  guarantees.
- **Out of scope:** Embedding / vector fields (EPIC-206), full Rust implementation
  (EPIC-207), SemanticDiff algorithm (EPIC-204).

---

## 2. Normative References

| ID | Reference |
|---|---|
| [STS] | docs/sts-formalization.md — STS multidimensional space formalization |
| [ADR] | docs/adr/ADR-SEM-001-positioning.md — normative architecture decisions |
| [V1] | docs/spec/SPEC-STF-CORE-SEMANTICS.md — STF-SIR v1 specification |
| [RFC2119] | RFC 2119 — Key words for use in RFCs to Indicate Requirement Levels |

---

## 3. ZToken v2 Structure

A ZToken is the atomic unit of semantic compilation. In v2 the structure
extends the v1 four-dimensional space `(L, S, Σ, Φ)` to the full STS
eight-dimensional space `z = ⟨L, S, Σ, Φ, C, P, Δ, Ω⟩`.

### 3.1 v1 Dimensions (unchanged)

All v1 fields are preserved verbatim (INV-201-2). See [V1] §3 for their
definitions. Summary:

| Dimension | Field group | Required (block) |
|---|---|---|
| L — lexical | `lexical.*` | MUST |
| S — syntactic | `syntactic.*` | MUST |
| Σ — semantic | `semantic.*` | MUST |
| Φ — logical | `logical.*` | MUST |

### 3.2 C — Contextual Dimension (new in v2)

The contextual dimension captures the discourse context and reference frame
of a token.

| Field | Type | Required (block) | Description |
|---|---|---|---|
| `context.context_id` | string | MAY | Identifier of the discourse context or thread |
| `context.scope` | string | MAY | Lexical scope or section containing this token |
| `context.reference_frame` | string | MAY | The epistemic reference frame (e.g., `"author"`, `"system"`) |

**Invariant:** If `context_id` is present, it MUST be a non-empty string matching
`/^[a-z0-9_.-]+$/`.

### 3.3 P — Pragmatic Dimension (new in v2)

The pragmatic dimension captures the speech act and communicative register of
a token.

| Field | Type | Required (block) | Description |
|---|---|---|---|
| `pragmatic.intent` | string | MAY | High-level communicative intent (e.g., `"assert"`, `"query"`, `"command"`) |
| `pragmatic.speech_act` | string | MAY | Austinian speech act type (e.g., `"illocution"`, `"perlocution"`) |
| `pragmatic.register` | string | MAY | Language register (e.g., `"formal"`, `"informal"`, `"technical"`) |

**Invariant:** `intent`, if present, MUST be one of: `assert`, `query`, `command`,
`define`, `exemplify`, `qualify`. Extension values MUST use a namespaced prefix
(e.g., `"custom:my_intent"`).

### 3.4 Δ — Temporal Dimension (new in v2)

The temporal dimension captures the provenance timestamp and validity interval
of a token.

| Field | Type | Required (block) | Description |
|---|---|---|---|
| `temporal.created_at` | ISO 8601 datetime | MAY | When the token was first compiled |
| `temporal.modified_at` | ISO 8601 datetime | MAY | When the token was last modified |
| `temporal.valid_from` | ISO 8601 datetime | MAY | Start of validity interval |
| `temporal.valid_to` | ISO 8601 datetime | MAY | End of validity interval |

**Invariant:** If both `valid_from` and `valid_to` are present, then
`valid_from ≤ valid_to` MUST hold.

### 3.5 Ω — Coherence/Validation Dimension (new in v2)

The Ω dimension exposes the coherence evaluation result as a first-class field.

| Field | Type | Required (block) | Description |
|---|---|---|---|
| `coherence_eval.coherence_score` | f64 ∈ [0.0, 1.0] | MAY | Composite coherence score |
| `coherence_eval.validation_flags` | array of string | MAY | Active validation rule IDs (e.g., `["VAL_01", "VAL_19"]`) |
| `coherence_eval.useful_information` | bool | MAY | ICE predicate: `C_l ∧ C_o ∧ Ground` |

**Invariant:** `coherence_score` MUST be in [0.0, 1.0]. If present, `validation_flags`
MUST contain only well-formed rule IDs matching `/^VAL_[0-9]{2}$/`.

---

## 4. Profile Registry v2

See `docs/spec/profiles-v2.md` for the full profile registry. Summary:

| Profile | Target granularity | Mandatory dimensions | Typical `node_type` values |
|---|---|---|---|
| `stf-sir-spec-v1-mvp` | block | L, S, Σ, Φ | heading, paragraph, blockquote, list, list_item, code_block, table |
| `stf-sir-spec-v2-block` | block | L, S, Σ, Φ, Ω | (same as v1) |
| `stf-sir-spec-v2-sentence` | sentence | L, S, Σ, Φ, C, P, Ω | sentence, clause, phrase |
| `stf-sir-spec-v2-entity` | entity/span | L, Σ, Φ, C, P | entity, span, mention |

Profile identifiers MUST be declared in the `compiler.profile` field of the
artifact.

---

## 5. Relation Taxonomy v2

### 5.1 v1 Relation Types (unchanged)

| Type | Category | Stage | Direction |
|---|---|---|---|
| `contains` | structural | syntactic | parent → child |
| `precedes` | structural | syntactic | left-sibling → right-sibling |

### 5.2 New Relation Types

Five new relation types are added in v2. Each MUST be assigned to exactly one
category and at least one valid stage (INV-201-4).

| Type | Category | Stage(s) | Source role | Target role | Description |
|---|---|---|---|---|---|
| `supports` | logical | semantic, logical | evidence | claim | Source provides evidential support for target |
| `contradicts` | logical | semantic, logical | counter-evidence | claim | Source logically contradicts target |
| `elaborates` | logical | semantic | detail | topic | Source expands on or qualifies target |
| `refers_to` | semantic-link | lexical, semantic | reference | referent | Source mentions or cites target entity |
| `cites` | semantic-link | semantic, logical | citation | source-artifact | Source cites target as its origin |

**Invariants:**
- Each type MUST appear in the `type` enum of the relation JSON Schema.
- `supports` and `contradicts` MUST NOT appear on the same source-target pair.
- `cites` source MUST have a non-empty `provenance.source_ids` set in the
  bridged statement.

### 5.3 Relation JSON Schema Fragment

```json
{
  "type": {
    "enum": [
      "contains", "precedes",
      "supports", "contradicts", "elaborates",
      "refers_to", "cites"
    ]
  }
}
```

---

## 6. Language Detection

In v1, the `document.language` field was unconditionally set to `"und"` (undetermined).

In v2, language detection is a MUST requirement:

- A conformant v2 compiler MUST detect the primary language of the source
  document and set `document.language` to a valid BCP 47 language tag.
- If detection confidence is below 0.7, the compiler MAY fall back to `"und"`.
- A v2 reader MUST accept `"und"` for backward compatibility.

---

## 7. Enricher Trait

An Enricher is a transformation `E : Artifact → Artifact` that applies
additional semantic annotation.

### 7.1 Monotonicity Contract

An Enricher MUST satisfy monotonicity:

> For any Artifact A, if E(A) = A', then for every field f ∈ A, A'.f contains
> all information present in A.f. No existing field value may be removed or
> replaced with a weaker value.

Formally: `A ⊑ A'` under the partial order where `a ⊑ b` iff b contains all
information in a.

### 7.2 Enricher Registration

Enrichers MUST be registered in `compiler.config_hash` by contributing to the
configuration string. The config hash in the output artifact MUST reflect the
enricher's identity.

---

## 8. Backward Compatibility

### 8.1 v1 Reader Rules

A v2 reader MUST accept any v1 `.zmd` artifact:
- All v1 fields are present in v2 with identical names and types.
- New v2 fields are absent in v1 artifacts; readers MUST treat absent optional
  fields as `null` / not-present.
- The `version` field in a v1 artifact is `1`; a v2 artifact uses `2`.

### 8.2 Version Gate

Features gated by `version: 2` MUST NOT be required of v1 artifacts:
- Language detection requirement (§6)
- Ω dimension coherence fields (§3.5)
- New relation types (§5.2)

---

## 9. Validation Rules v2 (VAL_01–VAL_30)

All 18 v1 rules (VAL_01–VAL_18) are preserved verbatim (INV-201-1). The
table below shows the complete set.

| Rule ID | Severity | Description | New in v2 |
|---|---|---|---|
| VAL_01 | ERROR | Token `id` MUST be a non-empty string | No |
| VAL_02 | ERROR | Token `id` MUST be unique within the artifact | No |
| VAL_03 | ERROR | `lexical.source_text` MUST be non-empty | No |
| VAL_04 | WARNING | `lexical.source_text` SHOULD match `text` after normalization | No |
| VAL_05 | ERROR | `syntactic.node_type` MUST be a recognized type for the profile | No |
| VAL_06 | ERROR | `syntactic.parent_id`, if present, MUST refer to an existing token | No |
| VAL_07 | ERROR | `syntactic.sibling_index` MUST be ≥ 0 | No |
| VAL_08 | ERROR | `semantic.gloss` MUST be a non-empty string | No |
| VAL_09 | WARNING | `semantic.gloss` SHOULD be the NFKC-normalized form of `lexical.source_text` | No |
| VAL_10 | ERROR | `logical.relation_ids` MUST refer only to existing relation IDs | No |
| VAL_11 | ERROR | `logical.formula`, if present, MUST parse successfully | No |
| VAL_12 | ERROR | `document.token_count` MUST equal the actual number of tokens | No |
| VAL_13 | ERROR | `document.relation_count` MUST equal the actual number of relations | No |
| VAL_14 | ERROR | `compiler.name` MUST be a non-empty string | No |
| VAL_15 | ERROR | `compiler.version` MUST be a semver string | No |
| VAL_16 | ERROR | `source.sha256` MUST be a valid SHA-256 hex string prefixed `sha256:` | No |
| VAL_17 | ERROR | Relation `source` and `target` MUST refer to existing token IDs | No |
| VAL_18 | ERROR | Relation `type` MUST be a value from the taxonomy | No |
| VAL_19 | ERROR | `document.language` MUST be a valid BCP 47 tag or `"und"` | Yes |
| VAL_20 | WARNING | `document.language` SHOULD NOT be `"und"` for v2 artifacts | Yes |
| VAL_21 | ERROR | `context.context_id`, if present, MUST match `/^[a-z0-9_.-]+$/` | Yes |
| VAL_22 | ERROR | `pragmatic.intent`, if present, MUST be from the allowed set or namespaced | Yes |
| VAL_23 | ERROR | `temporal.valid_from` ≤ `temporal.valid_to` when both are present | Yes |
| VAL_24 | ERROR | `coherence_eval.coherence_score`, if present, MUST be in [0.0, 1.0] | Yes |
| VAL_25 | ERROR | `coherence_eval.validation_flags` items MUST match `/^VAL_[0-9]{2}$/` | Yes |
| VAL_26 | ERROR | `supports` and `contradicts` MUST NOT both appear for the same source-target pair | Yes |
| VAL_27 | ERROR | A `cites` relation source MUST have non-empty `provenance.source_ids` | Yes |
| VAL_28 | WARNING | Sentence-profile tokens MUST use `node_type` from the sentence profile set | Yes |
| VAL_29 | WARNING | Entity-profile tokens MUST use `node_type` from the entity profile set | Yes |
| VAL_30 | WARNING | Enricher contribution MUST be reflected in `compiler.config_hash` | Yes |

---

## 10. Conformance Kit v2

### 10.1 Fixture Categories

A conformant v2 implementation MUST pass a conformance kit containing at least:

| Category | Min valid fixtures | Min invalid fixtures | Coverage target |
|---|---|---|---|
| v1 core (all VAL_01–VAL_18) | 10 | 10 | All v1 rules |
| Language detection (VAL_19, VAL_20) | 3 | 3 | BCP 47 / und fallback |
| C dimension (VAL_21) | 2 | 2 | context_id pattern |
| P dimension (VAL_22) | 2 | 2 | intent enum |
| Δ dimension (VAL_23) | 2 | 2 | valid_from ≤ valid_to |
| Ω dimension (VAL_24, VAL_25) | 2 | 2 | score range, flag format |
| New relation types (VAL_26, VAL_27) | 3 | 3 | supports/contradicts, cites |
| Sentence profile (VAL_28) | 2 | 2 | node_type set |
| Entity profile (VAL_29) | 2 | 2 | node_type set |
| Backward compat (v1 read by v2) | 5 | 0 | version gate |

**Total minimum:** ≥ 33 valid + ≥ 26 invalid = ≥ 59 fixtures.

### 10.2 Pass/Fail Criteria

A fixture test passes if:
- For valid fixtures: the implementation exits 0 and produces a structurally
  correct artifact.
- For invalid fixtures: the implementation exits non-zero and emits at least
  one error whose rule ID matches the expected rule.

### 10.3 Running the Kit

```bash
# Reference implementation (this codebase)
cargo test --test conformance_v2

# External implementations
./run-conformance-kit.sh --spec v2 --impl path/to/impl
```

### 10.4 External Implementation Guide

An external implementation wishing to claim v2 conformance MUST:
1. Accept `.md` source and produce `.zmd` YAML output with `version: 2`.
2. Run all fixtures in the conformance kit and report results in the standard
   format (exit code + JSON report).
3. Score ≥ 95% across all fixture categories.
4. Submit results to the conformance registry at `docs/conformance/registry.md`.

---

## 11. Glossary

| Term | Definition |
|---|---|
| ZToken | Atomic unit of semantic compilation; carries all 8 STS dimensions |
| Profile | Named configuration that defines which fields are mandatory for a given granularity |
| Enricher | A monotone transformation that adds annotation to an existing artifact |
| Fixture | A test case for the conformance kit (valid or invalid artifact) |
| VAL_XX | A validation rule identified by a two-digit code |
| v1 | STF-SIR version 1 — four-dimensional (L, S, Σ, Φ) |
| v2 | STF-SIR version 2 — eight-dimensional (L, S, Σ, Φ, C, P, Δ, Ω) |
| ICE | Integrable Coherent Evidence: `C_l ∧ C_o ∧ Ground` |
