# STF-SIR Retention Metric v2 — ρ_v2 Specification

## Overview

ρ_v2 is a six-component retention vector that extends the v1 four-component
vector (ρ_L, ρ_S, ρ_Σ, ρ_Φ) with finer-grained semantic decomposition and
a corpus-level aggregate component.

**Baseline threshold:** 0.97 (all components must reach ≥ 0.97 on the v2 compiler).

---

## Components

Let `T` = the set of ZTokens in the artifact, `|T|` = number of tokens,
`R` = the set of Relations, `|R|` = number of relations.

### 1. ρ_L — Lexical Retention

Fraction of tokens that have a non-empty `source_text` and a valid source span
(i.e., `span.start_byte < span.end_byte`).

```
ρ_L = |{ t ∈ T : t.lexical.source_text ≠ "" ∧ t.lexical.span.start_byte < t.lexical.span.end_byte }|
      ─────────────────────────────────────────────────────────────────────────────────────────────
                                          |T|
```

Edge case: `|T| = 0` → `ρ_L = 1.0` (vacuous completeness).

### 2. ρ_S — Syntactic Retention

Fraction of tokens with a non-empty `syntactic.node_type`.

```
ρ_S = |{ t ∈ T : t.syntactic.node_type ≠ "" }|
      ──────────────────────────────────────────
                         |T|
```

Edge case: `|T| = 0` → `ρ_S = 1.0`.

### 3. ρ_Σ_gloss — Semantic Gloss Retention

Fraction of tokens whose `semantic.gloss` equals `lexical.normalized_text`.
For tokens with an empty `plain_text`, the gloss must also be empty.

```
ρ_Σ_gloss = |{ t ∈ T : (t.lexical.plain_text = "" ∧ t.semantic.gloss = "")
                       ∨ (t.lexical.plain_text ≠ "" ∧ t.semantic.gloss = t.lexical.normalized_text) }|
             ─────────────────────────────────────────────────────────────────────────────────────────
                                                    |T|
```

Edge case: `|T| = 0` → `ρ_Σ_gloss = 1.0`.

### 4. ρ_Σ_concepts — Semantic Concepts Retention

Fraction of tokens that carry at least one concept in `semantic.concepts`.
If no token in the artifact has any concepts (the corpus uses no concept enrichment),
`ρ_Σ_concepts = 1.0` (vacuous completeness).

```
Let enriched = |{ t ∈ T : t.semantic.concepts ≠ [] }|

ρ_Σ_concepts = 1.0                  if enriched = 0   (vacuous)
             = enriched / |T|        otherwise
```

Edge case: `|T| = 0` → `ρ_Σ_concepts = 1.0`.

### 5. ρ_Φ — Logical Retention

Fraction of relations whose `source` and `target` token IDs both exist in the artifact.
An exception is granted for `SemanticLink` relations, which may reference external targets.

```
ρ_Φ = |{ r ∈ R : r.source ∈ T_ids ∧ (r.target ∈ T_ids ∨ r.category = SemanticLink) }|
      ───────────────────────────────────────────────────────────────────────────────────
                                          |R|
```

Edge case: `|R| = 0` → `ρ_Φ = 1.0`.

### 6. ρ_corpus — Corpus-Level Aggregate

For a single artifact, `ρ_corpus` is the geometric mean of the five artifact-level components:

```
ρ_corpus = (ρ_L · ρ_S · ρ_Σ_gloss · ρ_Σ_concepts · ρ_Φ)^(1/5)
```

For a multi-document corpus, `ρ_corpus` is the geometric mean of per-document `ρ_corpus` values:

```
ρ_corpus = (∏_{d ∈ corpus} ρ_corpus(d))^(1/|corpus|)
```

---

## Composite Score

The composite score is the geometric mean of all six components:

```
ρ_composite = (ρ_L · ρ_S · ρ_Σ_gloss · ρ_Σ_concepts · ρ_Φ · ρ_corpus)^(1/6)
```

---

## Baseline Threshold

**0.97** — all six components must reach ≥ 0.97 for the artifact/corpus to pass the v2 baseline.

The exception is `ρ_Σ_concepts`, which has a lower target of ≥ 0.90 when concept enrichment
is enabled (since concept coverage depends on external enrichment pipelines).

---

## Rust API

```rust
use stf_sir::benchmark::RetentionV2Score;

let score = RetentionV2Score::compute(&artifact);
assert!(score.is_baseline_met(0.97));
let composite = score.composite();
```

---

## Relationship to v1

| v1 component | v2 mapping |
|---|---|
| ρ_l | ρ_L (lexical) |
| ρ_s | ρ_S (syntactic) |
| ρ_sigma | split into ρ_Σ_gloss + ρ_Σ_concepts |
| ρ_phi | ρ_Φ (logical) |
| — | ρ_corpus (new) |

The v1 `RetentionBaseline` API remains unchanged (backward-compatible extension).
