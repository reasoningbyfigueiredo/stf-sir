---
id: METRICS-V2
title: KPI Definitions & Measurement — STF-SIR v2
version: 2.0.0-alpha
status: draft
roadmap: ROADMAP-STF-SIR-V2
created: 2026-04-12
---

# KPI Definitions & Measurement — STF-SIR v2

## 1. Metric Registry

All measurable KPIs for the v2 platform.

---

### M-01: Semantic Retention ρ_L (Lexical)

| Field | Value |
|---|---|
| **ID** | M-01 |
| **Name** | Lexical retention score |
| **Formula** | fraction of ztokens with valid lexical fields (non-empty source_text, valid spans, normalized_text = NFKC(plain_text)) |
| **Target** | ≥ 0.97 |
| **Measurement script** | `cargo bench retention_v2 -- --bench rho_l` |
| **Failure threshold** | < 0.97 blocks main merge |
| **Contract ref** | INV-205-4, CONTRACT-EPIC-205 |

---

### M-02: Semantic Retention ρ_S (Syntactic)

| Field | Value |
|---|---|
| **ID** | M-02 |
| **Name** | Syntactic retention score |
| **Formula** | fraction of ztokens with valid syntactic fields (non-empty node_type, path, valid parent ref) |
| **Target** | ≥ 0.97 |
| **Measurement script** | `cargo bench retention_v2 -- --bench rho_s` |
| **Failure threshold** | < 0.97 blocks main merge |
| **Contract ref** | INV-205-4 |

---

### M-03: Semantic Retention ρ_Σ_gloss (Semantic — Gloss)

| Field | Value |
|---|---|
| **ID** | M-03 |
| **Name** | Semantic gloss retention score |
| **Formula** | fraction of ztokens where Σ.gloss = normalized_text (v1 fallback rule) OR Σ.gloss is non-empty (v2 enriched mode) |
| **Target** | ≥ 0.97 |
| **Measurement script** | `cargo bench retention_v2 -- --bench rho_sigma_gloss` |
| **Failure threshold** | < 0.97 blocks main merge |

---

### M-04: Semantic Retention ρ_Σ_concepts (Semantic — Concepts)

| Field | Value |
|---|---|
| **ID** | M-04 |
| **Name** | Concept enrichment retention score |
| **Formula** | fraction of ztokens where Σ.concepts is non-empty (only when concept extractor enricher is active) |
| **Target** | ≥ 0.90 (lower than others: requires enricher) |
| **Measurement script** | `cargo bench retention_v2 -- --bench rho_sigma_concepts --features enricher` |
| **Failure threshold** | < 0.90 blocks enricher feature release |

---

### M-05: Semantic Retention ρ_Φ (Logical)

| Field | Value |
|---|---|
| **ID** | M-05 |
| **Name** | Logical retention score |
| **Formula** | Combined score: (fraction of valid relations) × (fraction of Φ.relation_ids that resolve) |
| **Target** | ≥ 0.97 |
| **Measurement script** | `cargo bench retention_v2 -- --bench rho_phi` |
| **Failure threshold** | < 0.97 blocks main merge |

---

### M-06: Corpus-Level Retention ρ_corpus

| Field | Value |
|---|---|
| **ID** | M-06 |
| **Name** | Corpus retention score |
| **Formula** | Weighted mean of per-document ρ_v2 scores over 100+ document corpus |
| **Target** | ≥ 0.97 |
| **Measurement script** | `cargo bench retention_v2 -- --bench rho_corpus` |
| **Failure threshold** | < 0.97 blocks main merge |

---

### M-07: Query p99 Latency

| Field | Value |
|---|---|
| **ID** | M-07 |
| **Name** | Query engine p99 latency |
| **Unit** | milliseconds |
| **Formula** | 99th percentile of query execution time on 10 000-ztoken synthetic artifact |
| **Target** | ≤ 50 ms |
| **Measurement script** | `cargo bench query_latency` |
| **Failure threshold** | > 50 ms blocks EPIC-203 close |
| **Contract ref** | CONTRACT-EPIC-203 postconditions |

---

### M-08: Query p50 Latency

| Field | Value |
|---|---|
| **ID** | M-08 |
| **Name** | Query engine median latency |
| **Unit** | milliseconds |
| **Target** | ≤ 10 ms (median across all patterns) |
| **Measurement script** | `cargo bench query_latency` |

---

### M-09: Query Determinism Rate

| Field | Value |
|---|---|
| **ID** | M-09 |
| **Name** | Query determinism rate |
| **Formula** | (identical_result_pairs / total_repeated_executions) × 100 |
| **Target** | 100% |
| **Measurement script** | `cargo test query_metamorphic` (100 repetitions per query) |
| **Failure threshold** | < 100% is a critical defect |
| **Contract ref** | INV-203-1 |

---

### M-10: Query Completeness Rate

| Field | Value |
|---|---|
| **ID** | M-10 |
| **Name** | Query completeness (no false negatives) |
| **Formula** | (correctly_returned / total_matching) × 100 |
| **Target** | 100% |
| **Measurement script** | `cargo test query_completeness` |
| **Failure threshold** | < 100% is a critical defect |
| **Contract ref** | INV-203-2 |

---

### M-11: DSL Operator Coverage

| Field | Value |
|---|---|
| **ID** | M-11 |
| **Name** | DSL operator implementation coverage |
| **Formula** | (operators_implemented / operators_specified) × 100 |
| **Target** | 100% |
| **Measurement script** | `scripts/audit/check-dsl-coverage.sh spec/query-dsl-v1.md src/query/` |
| **Failure threshold** | < 100% blocks EPIC-203 close |

---

### M-12: Structural Diff F1

| Field | Value |
|---|---|
| **ID** | M-12 |
| **Name** | Structural diff F1 score |
| **Formula** | 2 × (precision × recall) / (precision + recall) on golden diff corpus (20 pairs) |
| **Target** | ≥ 0.99 |
| **Measurement script** | `cargo bench diff_accuracy --features structural` |
| **Failure threshold** | < 0.99 blocks EPIC-204 close |

---

### M-13: Semantic Diff F1

| Field | Value |
|---|---|
| **ID** | M-13 |
| **Name** | Semantic diff F1 score |
| **Formula** | F1 on 50 human-labeled token pairs |
| **Target** | ≥ 0.90 |
| **Measurement script** | `cargo bench diff_accuracy --features semantic` |
| **Failure threshold** | < 0.90 blocks EPIC-204 close |

---

### M-14: Diff Determinism Rate

| Field | Value |
|---|---|
| **ID** | M-14 |
| **Name** | Diff determinism rate |
| **Formula** | identical_reports / total_repeated_diffs × 100 |
| **Target** | 100% |
| **Measurement script** | `cargo test diff_metamorphic` |
| **Contract ref** | INV-204-1 |

---

### M-15: Compile Determinism Rate

| Field | Value |
|---|---|
| **ID** | M-15 |
| **Name** | Compiler determinism rate |
| **Formula** | (compilations_with_stable_sha256 / total_compilations) × 100 |
| **Target** | 100% |
| **Measurement script** | `scripts/audit/check-determinism.sh 1000 tests/golden/sample.md` |
| **Failure threshold** | < 100% is a critical defect |
| **Contract ref** | INV-202-3, INV-207-1 |

---

### M-16: Backward Compatibility Rate

| Field | Value |
|---|---|
| **ID** | M-16 |
| **Name** | v1 artifact backward compatibility |
| **Formula** | (v1_fixtures_valid_against_v2_schema / total_v1_fixtures) × 100 |
| **Target** | 100% |
| **Measurement script** | `scripts/audit/validate-migration-v1-v2.sh tests/golden/` |
| **Failure threshold** | < 100% blocks EPIC-202 close |
| **Contract ref** | INV-202-1 |

---

### M-17: Enricher Monotonicity Rate

| Field | Value |
|---|---|
| **ID** | M-17 |
| **Name** | Enricher monotonicity violation rate |
| **Formula** | (monotonicity_violations / total_enricher_tests) × 100 |
| **Target** | 0% |
| **Measurement script** | `PROPTEST_CASES=512 cargo test enricher_monotonicity` |
| **Failure threshold** | > 0% blocks EPIC-207 close |
| **Contract ref** | INV-207-2 |

---

### M-18: Language Detection Accuracy

| Field | Value |
|---|---|
| **ID** | M-18 |
| **Name** | Language detection accuracy |
| **Formula** | (correct_language_tags / total_labeled_samples) × 100 |
| **Target** | ≥ 95% |
| **Measurement script** | `cargo test language_detection_accuracy` |
| **Failure threshold** | < 95% blocks language detection feature |

---

### M-19: Drift Detection Recall

| Field | Value |
|---|---|
| **ID** | M-19 |
| **Name** | Drift detection recall |
| **Formula** | (correctly_detected_regressions / total_injected_regressions) × 100 |
| **Target** | ≥ 95% |
| **Measurement script** | `cargo test drift_detection_recall` |
| **Failure threshold** | < 95% blocks EPIC-205 close |

---

### M-20: Provenance Completeness

| Field | Value |
|---|---|
| **ID** | M-20 |
| **Name** | RAG chunk provenance completeness |
| **Formula** | (chunks_with_full_provenance / total_emitted_chunks) × 100 |
| **Target** | 100% |
| **Measurement script** | `cargo test --features rag rag_provenance_completeness` |
| **Failure threshold** | < 100% is a critical defect |
| **Contract ref** | INV-206-1 |

---

### M-21: Plugin Isolation Rate

| Field | Value |
|---|---|
| **ID** | M-21 |
| **Name** | Plugin namespace isolation rate |
| **Formula** | 1 − (namespace_violations / total_plugin_write_attempts) |
| **Target** | 100% |
| **Measurement script** | `cargo test plugin_isolation` |
| **Failure threshold** | < 100% is a critical defect |
| **Contract ref** | INV-208-1 |

---

## 2. Metric Dashboard (CI Summary)

The CI pipeline produces a metric summary JSON file after each main merge:

```json
{
  "metrics_summary": {
    "format": "stf-sir-metrics",
    "version": "2",
    "commit_sha": "...",
    "generated_at": "...",
    "metrics": {
      "M-01_rho_l": { "value": 0.983, "target": 0.97, "status": "pass" },
      "M-07_query_p99_ms": { "value": 23.4, "target": 50, "status": "pass" },
      "M-12_structural_diff_f1": { "value": 0.997, "target": 0.99, "status": "pass" },
      "M-15_compile_determinism": { "value": 1.0, "target": 1.0, "status": "pass" }
    },
    "compliance_score": 0.97,
    "status": "pass"
  }
}
```

This file is committed to `docs/audit-reports/metrics-<date>.json` after each main merge.

---

## 3. Metric Regression Policy

| Regression | Policy |
|---|---|
| Any ρ component drops below target | Block main merge; open P1 defect |
| Query p99 increases > 20% vs baseline | Block EPIC-203 close; profile and optimize |
| Diff F1 drops below target | Block EPIC-204 close; investigate algorithm |
| Compile determinism < 100% | Block all releases; P0 critical defect |
| Backward compat < 100% | Block EPIC-202 close; revert incompatible change |
| Plugin isolation < 100% | Block EPIC-208 close; architecture review |

---

## 4. Measurement Cadence

| Metric group | When measured | Who checks |
|---|---|---|
| M-01–M-06 (retention) | Every main merge | Audit stage-4 |
| M-07–M-11 (query) | Every PR touching src/query/ | Audit stage-3 |
| M-12–M-14 (diff) | Every PR touching src/diff/ | Audit stage-3 |
| M-15 (determinism) | Every commit | Audit stage-3 |
| M-16 (backward compat) | Every PR touching schemas/ | Audit stage-3 |
| M-17 (monotonicity) | Every PR touching src/compiler/ | Audit stage-3 |
| M-18 (language) | Every PR touching src/compiler/lang.rs | Audit stage-3 |
| M-19 (drift) | Nightly | Audit stage-5 |
| M-20–M-21 (RAG/plugin) | Every PR touching src/rag/ or src/plugin/ | Audit stage-3 |
