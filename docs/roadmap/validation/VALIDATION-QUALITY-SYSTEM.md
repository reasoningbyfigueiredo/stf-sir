---
id: VALIDATION-QUALITY-SYSTEM-V2
title: Validation & Quality System — STF-SIR v2
version: 2.0.0-alpha
status: draft
roadmap: ROADMAP-STF-SIR-V2
created: 2026-04-12
---

# Validation & Quality System — STF-SIR v2

## 1. Overview

The STF-SIR v2 quality system has five pillars:

| Pillar | What it measures | Primary tool |
|---|---|---|
| Semantic correctness | ZToken fields conform to spec | Conformance suite + validator |
| Determinism | Identical input → identical output bytes | Golden gate + determinism script |
| Retention | ρ_v2 ≥ 0.97 on all dimensions | Retention benchmark |
| Query accuracy | DSL queries return correct, complete results | Query test suite + F1 benchmarks |
| Diff reliability | Diff F1 ≥ 0.99 structural, ≥ 0.90 semantic | Diff benchmark + labeled corpus |

---

## 2. Test Suite Architecture

```
tests/
├── golden/                     # Byte-for-byte golden tests (v1)
│   └── v2/                     # Byte-for-byte golden tests (v2)
├── conformance/                # Valid/invalid fixture suites
│   ├── valid/                  # 20+ valid fixtures
│   └── invalid_*/              # 20+ invalid fixtures per category
├── fixtures/
│   ├── valid/                  # 12+ valid inputs
│   └── invalid/                # 15+ invalid artifacts
├── query/                      # Query engine tests
│   ├── parser_tests.rs
│   ├── executor_tests.rs
│   ├── metamorphic.rs
│   ├── completeness.rs
│   └── cli_tests.rs
├── diff/                       # Diff engine tests
│   ├── golden/
│   │   ├── structural/         # 20 structural diff golden pairs
│   │   └── semantic/           # 50 human-labeled semantic pairs
│   ├── structural_tests.rs
│   ├── semantic_tests.rs
│   ├── proptest_structural.rs
│   └── cli_tests.rs
├── benchmark/                  # Retention benchmark
│   ├── corpus/                 # 100+ .md documents
│   ├── corpus-manifest.sha256
│   └── drift/
│       └── injected_regressions/
├── plugin/                     # Plugin system tests
│   ├── plugin_tests.rs
│   ├── plugin_external_tests.rs
│   └── plugin_conformance/
├── rag/                        # RAG integration tests
│   ├── embedding_tests.rs
│   ├── store_tests.rs
│   └── provenance_roundtrip.sh
├── agent/
│   └── tool_tests.rs
├── compile.rs                  # v1 compiler unit tests (≥ 30 cases)
├── compile_v2.rs               # v2 compiler integration tests
├── conformance.rs              # Conformance suite runner
├── enricher_monotonicity.rs    # Enricher property tests
├── golden.rs                   # Golden gate runner (v1)
├── invalid_matrix.rs
├── metamorphic.rs              # Compiler metamorphic tests
├── profile_tests.rs            # Profile system tests
├── proptest_invariants.rs      # Core property tests (512+ cases)
├── retention.rs                # v1 retention tests
├── retention_v2.rs             # v2 retention tests
├── sir_graph.rs                # SirGraph v1 tests
├── sir_graph_v2.rs             # SirGraph v2 tests
├── unicode_spans.rs            # Unicode edge case tests
└── common/mod.rs               # Shared test utilities
```

---

## 3. Semantic Correctness

### 3.1 Conformance Suite v2

**Purpose:** Verify that the compiler produces spec-compliant output for all valid inputs,
and correctly rejects all invalid inputs.

**Fixture categories v2:**

| Category | Count (min) | Description |
|---|---|---|
| `valid/block_v1` | 10 | All v1 conformance cases |
| `valid/block_v2` | 10 | v2 block profile, new dimensions |
| `valid/sentence_v1` | 5 | Sentence-level profile |
| `valid/language_detect` | 5 | Language detection cases (EN, PT, FR, ES, DE) |
| `valid/new_relations` | 5 | New relation types (supports, refers_to, etc.) |
| `valid/enriched` | 5 | Artifacts with concept extractor applied |
| `invalid_schema_v2` | 10 | Schema violations (new fields) |
| `invalid_semantic_v2` | 10 | Semantic rule violations (new rules VAL_19–VAL_30) |

**Pass criteria:** All valid fixtures compile and validate; all invalid fixtures fail with the expected error code.

**Script:** `cargo test conformance`

### 3.2 Validator v2

**Extended validation rules (VAL_19–VAL_30):**

| Code | Category | Description |
|---|---|---|
| VAL_19_LANGUAGE_BCP47 | Format | `document.language` must be valid BCP-47 |
| VAL_20_CONCEPTS_SORTED | Semantic | `Σ.concepts` must be sorted and deduplicated |
| VAL_21_CONFIDENCE_RANGE | Semantic | `Σ.confidence` must be in [0.0, 1.0] |
| VAL_22_EMBEDDING_REF_FORMAT | Semantic | `Σ.embedding_ref` must match `rag:<id>/<sha256>/<zid>` if present |
| VAL_23_NEW_RELATION_CATEGORY | Logical | New relation types must have correct category |
| VAL_24_SEMANTIC_LINK_TARGET | Logical | `semantic-link` relations may have external URI targets |
| VAL_25_C_SCOPE_VALID | Contextual | `C.scope` must be a non-empty string if present |
| VAL_26_TEMPORAL_RANGE | Temporal | `Δ.valid_from` must be ≤ `Δ.valid_to` if both present |
| VAL_27_COHERENCE_SCORE | Coherence | `Ω.coherence_score` must be in [0.0, 1.0] |
| VAL_28_PROFILE_FIELD_CONSISTENCY | Profile | Fields present must match declared profile |
| VAL_29_ENRICHER_IDEMPOTENT | Semantic | Applying same enricher twice must not change concepts count |
| VAL_30_PLUGIN_NAMESPACE | Extension | Extension fields must have namespaced keys (`<ns>.<field>`) |

---

## 4. Determinism

### 4.1 Golden Gate (v1 — preserved)

6 golden pairs in `tests/golden/`. Byte-for-byte comparison. Must never change.

**Script:** `cargo test golden`

**Policy:** Any byte difference is a blocking regression. No exceptions.

### 4.2 Golden Gate (v2)

12+ golden pairs in `tests/golden/v2/`. Same byte-for-byte policy.

**Script:** `cargo test golden_v2`

### 4.3 Determinism Gate

Compiles `tests/golden/sample.md` N times (default 1000 in CI), compares all output SHA-256.

**Script:** `scripts/audit/check-determinism.sh 1000 tests/golden/sample.md`

**Policy:** Any non-identical output is a critical failure.

### 4.4 Metamorphic Tests

Properties that must hold regardless of input surface form:
1. **LF vs CRLF**: same content with different line endings → identical semantic output
2. **Trailing newline**: trailing newline present or absent → identical output
3. **Byte-order mark**: BOM prefix → normalized, not included in source_text
4. **Duplicate compilation**: `compile(src)` called 1000× → identical results

**Script:** `cargo test metamorphic`

---

## 5. Retention

### 5.1 v1 Retention Baseline

Defined in `docs/retention-baseline.md`. Must not regress.

ρ_v1 = <ρ_L, ρ_S, ρ_Σ, ρ_Φ> — all ≥ 0.97.

### 5.2 v2 Retention Baseline

ρ_v2 = <ρ_L, ρ_S, ρ_Σ_gloss, ρ_Σ_concepts, ρ_Φ, ρ_corpus> — all ≥ 0.97.
Exception: ρ_Σ_concepts ≥ 0.90 (lower target because concepts require enrichment).

**Script:** `cargo bench retention_v2`

**Baseline file:** `docs/retention-baseline-v2.md`

### 5.3 Retention Test Properties

- **Bounds:** ∀ ρ ∈ ρ_v2: ρ ∈ [0.0, 1.0]
- **Vacuous completeness:** empty token set → ρ = 1.0
- **Monotone enrichment:** applying concept extractor → ρ_Σ_concepts ≥ pre-enrichment value
- **Reproducibility:** same corpus + compiler → identical ρ_v2 scores

---

## 6. Query Accuracy

### 6.1 Query Correctness

For each of the 10 named query patterns, test against the golden v2 corpus:
- Expected result set is committed as a golden file
- Test checks result set equality (sorted by ztoken ID)

**Script:** `cargo test query_executor`

### 6.2 Query Completeness

For each filter predicate P, brute-force scan all ztokens and compare with query engine result.
Must match exactly (no false negatives, no false positives).

**Script:** `cargo test query_completeness`

### 6.3 Query Determinism

For each query Q on each golden fixture:
- Execute Q 100 times
- Compare all result SHA-256 values

**Script:** `cargo test query_metamorphic`

### 6.4 Query Latency Benchmark

Benchmark queries on a 10 000-ztoken synthetic artifact:

| Pattern | p50 target | p99 target |
|---|---|---|
| by_type(heading) | ≤ 1 ms | ≤ 5 ms |
| descendants(root, 100) | ≤ 10 ms | ≤ 50 ms |
| subgraph(root) | ≤ 20 ms | ≤ 50 ms |
| gloss_match(complex_regex) | ≤ 10 ms | ≤ 50 ms |

**Script:** `cargo bench query_latency`

---

## 7. Diff Reliability

### 7.1 Structural Diff F1

Golden corpus: 20 document pairs with committed expected `StructuralDiff` JSON outputs.

**Metric:** F1 = 2 × (precision × recall) / (precision + recall) ≥ 0.99

Where:
- Precision = correct diff items / total diff items emitted
- Recall = correct diff items / total actual changes

**Script:** `cargo bench diff_accuracy --features structural`

### 7.2 Semantic Diff F1

Human-labeled corpus: 50 matched token pairs, labeled as changed/not-changed.

**Metric:** F1 ≥ 0.90

**Script:** `cargo bench diff_accuracy --features semantic`

### 7.3 Diff Identity and Symmetry Tests

- Identity: `diff(X, X)` = `EmptyDiff` for all golden fixtures
- Symmetry: `|diff(A,B).added|` = `|diff(B,A).removed|` for all pairs

**Script:** `cargo test diff_identity diff_symmetry`

### 7.4 Diff Determinism

For each pair (A, B): compute `diff(A, B)` 100 times, compare all report SHA-256 values.

**Script:** `cargo test diff_metamorphic`

---

## 8. Reproducibility Guarantees

STF-SIR v2 provides three levels of reproducibility:

### Level 1: Binary Reproducibility

The compiled binary produces byte-identical output for identical inputs. Guaranteed by:
- `BTreeMap` for all ordered structures
- `Vec` emission in deterministic insertion order
- `serde_yaml_ng` with stable struct field order
- `config_hash` locks pipeline identity

**Verified by:** golden gate + determinism script

### Level 2: Benchmark Reproducibility

The retention benchmark produces identical scores for identical corpus + binary. Guaranteed by:
- Corpus SHA-256 manifest
- Deterministic iteration order (sorted file paths)
- Reproducible `cargo bench` with fixed random seed

**Verified by:** `scripts/audit/check-benchmark-reproducibility.sh 10`

### Level 3: Audit Reproducibility

Any third party can reproduce all audit results from the public repository. Guaranteed by:
- All audit scripts in `scripts/audit/` (version-controlled)
- No external state dependencies (no network calls in core audit stages)
- `rust-toolchain.toml` pins the Rust version
- `deny.toml` locks supply chain

**Verified by:** `scripts/audit/run-all.sh` (self-contained)

---

## 9. Unicode Coverage

The v2 test suite extends the v1 Unicode golden corpus with additional edge cases:

| Test | Description |
|---|---|
| `unicode_nfkc.md` | NFKC normalization of ﬁ ligatures |
| `unicode_zwsp.md` | Zero-width space handling |
| `unicode_cjk.md` | CJK full-width characters |
| `unicode_bidi.md` | Bidirectional text (Arabic, Hebrew) |
| `unicode_emoji.md` | Emoji in paragraph text |
| `unicode_rtl_heading.md` | RTL heading with mixed direction |
| `unicode_combining.md` | Combining diacritics |
| `unicode_surrogates.md` | Surrogate pair handling |

All Unicode fixtures must: compile without error, produce valid span byte offsets, normalize correctly.

---

## 10. Design Principles Enforcement

| Principle | Enforcement Mechanism |
|---|---|
| Deterministic outputs | Golden gate + determinism script (1000 trials) |
| Semantic preservation | Retention benchmark ρ_v2 ≥ 0.97 |
| Auditability | Audit model stages 1–6; committed reports |
| Reproducibility | Three-level reproducibility guarantees |
| Extensibility without breaking core | v1 golden gate (INV-207-1); plugin isolation (INV-208-1) |
| No panic | Property test: no panic on 512 random inputs; fuzzing |
| Monotone enrichment | Enricher monotonicity property test (512 cases) |
