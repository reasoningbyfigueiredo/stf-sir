# Retention Baseline

This document defines the first operational retention baseline for STF-SIR. It is not the full theoretical information-retention model from the formal article. Instead, it is a deterministic, implementation-ready proxy that measures whether an artifact preserves each STF-SIR dimension in an internally complete and self-consistent way.

## 1. Scope

The baseline computes the vector

\[
\rho(d) = \langle \rho_L, \rho_S, \rho_\Sigma, \rho_\Phi \rangle
\]

for an already-constructed `Artifact`.

At this stage, each component is a dimension-specific completeness score in `[0,1]`. The baseline does not attempt to measure semantic equivalence against external annotations, benchmark corpora, or human judgments.

## 2. Interpretation

The retention baseline is an operational proxy, not a final scientific claim:

- `rho_L` measures lexical completeness and lexical self-consistency.
- `rho_S` measures syntactic completeness and reference consistency.
- `rho_Sigma` measures compliance with the v1 semantic fallback rule.
- `rho_Phi` measures logical completeness and relation-reference consistency.

This gives the project a deterministic retention layer that can be computed now, while leaving room for a richer research-grade evaluation model later.

## 3. Current Definitions

### 3.1 `rho_L`

`rho_L` is the fraction of ztokens whose lexical dimension is internally valid:

- `L.source_text` is non-empty,
- byte span bounds are valid,
- line span bounds are valid,
- `L.normalized_text` equals the deterministic normalization of `L.plain_text`.

### 3.2 `rho_S`

`rho_S` is the fraction of ztokens whose syntactic dimension is internally valid:

- `S.node_type` is non-empty,
- `S.path` is non-empty,
- `S.parent_id` is either `null` with `depth = 0`, or references an existing ztoken.

### 3.3 `rho_Sigma`

`rho_Sigma` is the fraction of ztokens satisfying the STF-SIR v1 semantic fallback rule:

- if `L.plain_text` is non-empty, then `Σ.gloss = L.normalized_text`,
- if `L.plain_text` is empty, then `Σ.gloss = ""`.

### 3.4 `rho_Phi`

`rho_Phi` is the logical completeness score over both relations and local `Φ` references:

- every relation has non-empty required identity fields,
- every relation source references an existing ztoken,
- every relation target references an existing ztoken unless the category allows external targets,
- every relation stage is a valid pipeline stage,
- every `Φ.relation_ids` entry resolves to an existing relation.

## 4. Empty-Denominator Rule

When a dimension has zero evaluable items, the baseline assigns score `1.0`.

This is a deliberate vacuous-completeness rule. For example, an empty document with zero ztokens and zero relations does not violate retention completeness merely because there is nothing to score.

## 5. Rust Surface

The initial Rust API is:

- `Artifact::retention_baseline() -> RetentionBaseline`
- `RetentionBaseline.vector`
- `RetentionVector { rho_l, rho_s, rho_sigma, rho_phi }`

Each dimension also exposes its raw `satisfied` and `total` counts through `RetentionScore`.

## 6. Non-Goals

The baseline explicitly does not yet provide:

- corpus-level retention benchmarking,
- semantic similarity against human labels,
- information-theoretic mutual-information estimates,
- inference-aware retention scoring over derived graph structure,
- cross-document or cross-version retention comparison.

Those belong to later phases of STF-SIR research and engineering.
