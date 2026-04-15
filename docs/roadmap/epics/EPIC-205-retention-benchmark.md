---
id: EPIC-205
title: Retention & Benchmark
version: 2.0.0-alpha
status: implementado
roadmap: ROADMAP-STF-SIR-V2
priority: high
created: 2026-04-12
target: 2026-10-01
depends_on:
  - EPIC-203
  - EPIC-204
blocks:
  - EPIC-206
---

# EPIC-205 — Retention & Benchmark

## Description

Extend the v1 retention baseline into a comprehensive, multi-dimensional benchmark system:

1. **Corpus-level retention** — aggregate ρ across a document corpus (not just single artifacts)
2. **Semantic similarity retention** — compare Σ.concepts against human-labeled gold standards
3. **Information-theoretic retention** — mutual information between source tokens and ztokens
4. **Drift detection** — detect retention regressions across compiler versions using semantic diff
5. **Benchmark harness** — reproducible, CI-integrated benchmark with published baselines

The benchmark system must produce machine-readable reports suitable for audit trails and research publications.

## Scope

- **In scope:** Corpus benchmark harness, retention metric extensions (ρ_v2), drift detection engine, baseline management, CI integration
- **Out of scope:** External corpus licensing, human annotation tooling, production monitoring dashboards

## Deliverables

| # | Artifact | Path |
|---|---|---|
| D-205-1 | Retention v2 metric definitions | `spec/retention-v2.md` |
| D-205-2 | Corpus benchmark harness | `src/benchmark/` |
| D-205-3 | Drift detection engine | `src/benchmark/drift.rs` |
| D-205-4 | Benchmark report format | `spec/benchmark-report-v1.md` |
| D-205-5 | Published v2 baselines | `docs/retention-baseline-v2.md` |
| D-205-6 | CI benchmark stage | `.github/workflows/ci.yml` extension |

## Success Criteria

- [ ] ρ_v2 defined with 6 components (ρ_L, ρ_S, ρ_Σ_gloss, ρ_Σ_concepts, ρ_Φ, ρ_corpus)
- [ ] Corpus-level benchmark runs on ≥ 100 document corpus
- [ ] Baseline ρ_v2 ≥ 0.97 on all components for v2 compiler
- [ ] Drift detection correctly flags ≥ 95% of artificially injected regressions
- [ ] Benchmark report is reproducible: identical corpus + compiler → identical scores

## Risks

| ID | Risk | Mitigation |
|---|---|---|
| R-205-1 | 100-document corpus is too small for statistical significance | Document minimum corpus size requirements; allow extension |
| R-205-2 | Mutual information metric requires labeled data | Define proxy metric using normalized_text entropy; full MI is stretch goal |
| R-205-3 | Drift detection produces false positives | Tune detection threshold using ROC curve on injected regression corpus |

---

## EPIC CONTRACT

```yaml
contract:
  id: CONTRACT-EPIC-205
  version: 1.0.0

  inputs:
    - id: I-205-1
      description: STF-SIR compiler v2 (EPIC-207 output)
      required: true
    - id: I-205-2
      description: Semantic diff engine (EPIC-204 output)
      required: true
    - id: I-205-3
      description: Query engine (EPIC-203 output)
      required: true
    - id: I-205-4
      description: Benchmark corpus (≥ 100 Markdown documents)
      required: true

  outputs:
    - id: O-205-1
      artifact: src/benchmark/ module
    - id: O-205-2
      artifact: spec/retention-v2.md
    - id: O-205-3
      artifact: docs/retention-baseline-v2.md
    - id: O-205-4
      artifact: benchmark reports (CI artifacts)

  invariants:
    - INV-205-1: |
        Benchmark reproducibility: running the benchmark suite on the same corpus
        and the same compiler binary produces identical ρ_v2 scores.
    - INV-205-2: |
        Monotone baseline: v2 baseline scores are ≥ v1 baseline scores on the
        shared v1 corpus subset.
    - INV-205-3: |
        No benchmark run silently succeeds if the corpus is incomplete.
        Missing documents cause a hard failure, not a skipped test.
    - INV-205-4: |
        All ρ components are in [0.0, 1.0] (enforced by type constraint).
    - INV-205-5: |
        Drift detection uses semantic diff output deterministically; it never
        use source-level text comparison.

  preconditions:
    - PRE-205-1: EPIC-203 closed
    - PRE-205-2: EPIC-204 closed
    - PRE-205-3: Benchmark corpus of ≥ 100 documents assembled
    - PRE-205-4: v1 retention baseline (docs/retention-baseline.md) published

  postconditions:
    - POST-205-1: ρ_v2 ≥ 0.97 on all components recorded in CI
    - POST-205-2: Drift detection recall ≥ 0.95 on injected regression corpus
    - POST-205-3: Benchmark report format validated against spec
    - POST-205-4: Baselines committed and versioned in docs/

  validation:
    automated:
      - script: cargo bench retention_v2
        description: Runs corpus benchmark; asserts ρ_v2 ≥ 0.97 on all components
      - script: cargo test drift_detection
        description: Injects 50 artificial regressions; checks recall ≥ 0.95
      - script: scripts/check-benchmark-reproducibility.sh 10
        description: Runs benchmark 10×; diffs all report SHA-256s
    manual:
      - review: Baseline document reviewed by project author before publishing

  metrics:
    - metric: retention_rho_l
      target: ≥ 0.97
    - metric: retention_rho_s
      target: ≥ 0.97
    - metric: retention_rho_sigma_gloss
      target: ≥ 0.97
    - metric: retention_rho_sigma_concepts
      target: ≥ 0.90  # lower target: requires enrichment
    - metric: retention_rho_phi
      target: ≥ 0.97
    - metric: retention_rho_corpus
      target: ≥ 0.97
    - metric: drift_detection_recall
      target: ≥ 0.95
    - metric: drift_detection_precision
      target: ≥ 0.90
    - metric: benchmark_reproducibility
      target: 100%

  failure_modes:
    - FAIL-205-1:
        condition: INV-205-1 violated (non-reproducible benchmark)
        action: Critical; block EPIC-206 (RAG cannot trust non-reproducible retention)
    - FAIL-205-2:
        condition: Any ρ component < 0.97
        action: Regression; block main merge; open retention defect
    - FAIL-205-3:
        condition: Drift detection recall < 0.95
        action: Quality defect; tune threshold before closing EPIC
```

---

## Features

### FEAT-205-1: Retention v2 Metric Definitions

**Description:** Extend ρ(d) from the 4-component v1 vector to the 6-component v2 vector,
adding corpus-level and concept-level components. Define all formulas formally in `spec/retention-v2.md`.

**Inputs:**
- v1 retention spec (`docs/retention-baseline.md`)
- v1 `RetentionBaseline` struct (`src/retention/mod.rs`)
- Semantic dimension fields (Σ.concepts, Σ.confidence)

**Outputs:**
- `spec/retention-v2.md` (formal definition)
- Extended `RetentionBaselineV2` struct
- Updated `src/retention/mod.rs`

**Acceptance Criteria:**
- [ ] ρ_v2 = (ρ_L, ρ_S, ρ_Σ_gloss, ρ_Σ_concepts, ρ_Φ, ρ_corpus) formally defined
- [ ] All components in [0.0, 1.0] enforced by Rust type system (newtypes or asserts)
- [ ] ρ_Σ_concepts = 1.0 when concepts Vec is empty (vacuous completeness, same as v1 logic)
- [ ] ρ_corpus = weighted mean of per-document ρ scores over corpus
- [ ] v1 `RetentionBaseline` remains computable (no breaking API change)

**Metrics:** All ρ_v2 components ≥ 0.97 on golden corpus

**Feature Contract:**
```yaml
contract:
  id: CONTRACT-FEAT-205-1
  inputs: [v1 retention spec, v1 struct, Σ fields]
  outputs: [spec/retention-v2.md, RetentionBaselineV2 struct]
  invariants:
    - v1 API unchanged (backward-compatible extension)
    - All ρ values in [0.0, 1.0]
    - Empty denominator → 1.0 (vacuous completeness)
  postconditions:
    - spec doc present and reviewed
    - v1 retention tests still pass
  failure_modes:
    - Breaking v1 API → block release
```

#### Tasks

**TASK-205-1-1: Formalize ρ_Σ_concepts and ρ_corpus definitions**
- Description: Write precise mathematical definitions with edge-case handling (empty corpus, empty concepts)
- Artifacts: `spec/retention-v2.md` §2 and §3

**TASK-205-1-2: Implement RetentionBaselineV2 struct**
- Description: Extend retention module with v2 struct; add compute function; maintain v1 API
- Definition of done: `cargo test retention` passes; v1 tests unchanged
- Artifacts: Updated `src/retention/mod.rs`

**TASK-205-1-3: Write retention v2 property tests**
- Description: Property tests for all 6 components: bounds, vacuous completeness, monotonicity
- Artifacts: `tests/retention_v2.rs`

---

### FEAT-205-2: Corpus Benchmark Harness

**Description:** Build a reproducible benchmark harness that runs the STF-SIR compiler on a
corpus of ≥ 100 Markdown documents, computes ρ_v2 for each, aggregates to corpus-level scores,
and writes a structured benchmark report.

**Inputs:**
- Benchmark corpus (≥ 100 `.md` documents in `tests/benchmark/corpus/`)
- Compiler v2
- ρ_v2 metric implementation

**Outputs:**
- `src/benchmark/harness.rs` — benchmark runner
- `src/benchmark/report.rs` — `BenchmarkReport` struct and serializer
- `spec/benchmark-report-v1.md` — report format spec
- CI benchmark stage

**Acceptance Criteria:**
- [ ] Harness compiles all corpus documents, records per-document ρ_v2
- [ ] Report includes: corpus size, per-component min/max/mean/p50/p95, wall-time, compiler version, corpus SHA-256
- [ ] Harness fails hard on any compile error (no silent skips)
- [ ] Report is byte-for-byte reproducible on same corpus + compiler
- [ ] CI benchmark stage records report as artifact

**Metrics:** benchmark_reproducibility = 100%

**Feature Contract:**
```yaml
contract:
  id: CONTRACT-FEAT-205-2
  inputs: [corpus, compiler, ρ_v2 impl]
  outputs: [BenchmarkReport, CI artifact]
  invariants:
    - INV-205-1 (reproducibility)
    - INV-205-3 (no silent failures)
  postconditions:
    - Report format valid per spec
    - CI stage exits 0 iff all ρ_v2 ≥ 0.97
  failure_modes:
    - Compile error skipped → false passing report
```

#### Tasks

**TASK-205-2-1: Assemble benchmark corpus (100+ documents)**
- Description: Collect 100+ Markdown documents (can use STF-SIR own docs + synthetic variants)
- Definition of done: Corpus directory has ≥ 100 files; SHA-256 manifest committed
- Artifacts: `tests/benchmark/corpus/`, `tests/benchmark/corpus-manifest.sha256`

**TASK-205-2-2: Implement benchmark harness**
- Description: Rust binary or library function that iterates corpus, compiles, computes ρ_v2, aggregates
- Artifacts: `src/benchmark/harness.rs`

**TASK-205-2-3: Implement BenchmarkReport serializer**
- Description: Struct + JSON/YAML serializer for all report fields
- Artifacts: `src/benchmark/report.rs`

**TASK-205-2-4: Write benchmark report format spec**
- Description: Document all fields with types, semantics, and example
- Artifacts: `spec/benchmark-report-v1.md`

**TASK-205-2-5: Add CI benchmark stage**
- Description: Add benchmark job to `.github/workflows/ci.yml`; assert ρ_v2 ≥ 0.97; upload report as artifact
- Artifacts: `.github/workflows/ci.yml` update

---

### FEAT-205-3: Drift Detection Engine

**Description:** Build a drift detection engine that uses the semantic diff engine (EPIC-204)
to detect retention regressions between two compiler versions compiled on the same corpus.

**Inputs:**
- Two sets of `.zmd` artifacts (same corpus, different compiler versions)
- `SemanticDiff` and `StructuralDiff` engines (EPIC-204)
- Configurable drift threshold

**Outputs:**
- `DriftReport` type and serializer (`src/benchmark/drift.rs`)
- Drift detection test suite with injected regressions

**Acceptance Criteria:**
- [ ] Drift report lists: regressed documents, regressed tokens, ρ delta per component
- [ ] Drift recall ≥ 0.95 on artificially injected regression corpus (50 cases)
- [ ] Drift precision ≥ 0.90 (no excessive false positives)
- [ ] Threshold configurable via CLI flag `--drift-threshold`
- [ ] Drift report serializable to JSON/YAML

**Metrics:** drift_detection_recall ≥ 0.95, precision ≥ 0.90

**Feature Contract:**
```yaml
contract:
  id: CONTRACT-FEAT-205-3
  inputs: [two artifact sets, diff engines, threshold]
  outputs: [DriftReport]
  invariants:
    - INV-205-5 (uses semantic diff, not text diff)
    - Drift of identical artifact sets = empty report
  postconditions:
    - Recall/precision benchmarks pass
  failure_modes:
    - False negative drift → silently passing regression → critical
```

#### Tasks

**TASK-205-3-1: Design drift detection algorithm**
- Description: Define the algorithm: for each document pair, compute structural + semantic diff; sum ρ deltas; flag if above threshold
- Artifacts: `spec/decisions/ADR-003-drift-detection.md`

**TASK-205-3-2: Implement DriftReport and detector**
- Description: Write `detect_drift(artifacts_v1, artifacts_v2, threshold) -> DriftReport`
- Artifacts: `src/benchmark/drift.rs`

**TASK-205-3-3: Create injected regression corpus**
- Description: Generate 50 artificial regressions by modifying golden artifacts (removing relations, corrupting gloss, etc.)
- Artifacts: `tests/benchmark/drift/injected_regressions/`

**TASK-205-3-4: Write drift detection recall/precision benchmark**
- Description: Run detector on injected corpus; measure recall and precision
- Definition of done: recall ≥ 0.95, precision ≥ 0.90
- Artifacts: `tests/benchmark/drift_benchmark.rs`
