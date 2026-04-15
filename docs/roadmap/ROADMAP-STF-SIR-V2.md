---
id: ROADMAP-STF-SIR-V2
version: 2.0.0-alpha
status: draft
created: 2026-04-12
authors:
  - Rogerio Figueiredo
  - AI Architect Auditor (claude-sonnet-4-6)
license: Apache-2.0
---

# ROADMAP-STF-SIR-V2 — Executable Semantic Platform

> **Version:** 2.0.0-alpha | **Status:** draft | **Date:** 2026-04-12

---

## 1. Objective

Transform STF-SIR from a **deterministic semantic representation baseline (v1)** into an
**executable semantic platform (v2)** that provides:

- A fully queryable semantic graph with a typed DSL
- A semantic diff engine with machine-verifiable correctness
- A canonical artifact format v2 with embedding anchors and STS extensions
- A formal extensibility model with `Enricher` trait and plugin registry
- Native RAG / AI-agent integration with deterministic provenance

**Measurable completion criterion:** All 8 EPICs closed with ≥ 95% contract compliance,
zero critical audit violations, semantic retention ρ ≥ 0.97 on the v1 golden corpus,
and query latency p99 ≤ 50 ms on artifacts ≤ 10 000 ztokens.

---

## 2. Scope

| In scope | Out of scope |
|---|---|
| Rust reference implementation | Third-party re-implementations |
| ZMD format v2 (backward-compatible with v1) | PDF / binary source frontends |
| Query DSL over compiled SirGraph | Full-text search engine |
| Semantic diff on ZMD pairs | Cross-language merges |
| RAG embedding bridge (adapter, not vector DB) | Hosting / deployment infrastructure |
| Enricher plugin system | Closed-source enrichers |
| CI audit pipeline | Production SLA monitoring |

---

## 3. Success Criteria

### 3.1 Quality

| KPI | Target | Measurement |
|---|---|---|
| Semantic retention ρ | ≥ 0.97 (all four dims) | `cargo test --features retention-bench` |
| Schema compliance | 100% on v1 corpus | Conformance kit v2 |
| Contract violations in CI | 0 (blocking) | Audit pipeline stage-3 |
| Query determinism | 100% identical on identical input | Metamorphic suite |
| Diff accuracy (structural) | ≥ 0.99 F1 on golden diff corpus | `cargo test --features diff-bench` |
| Diff accuracy (semantic) | ≥ 0.90 F1 on human-labeled pairs | Retention benchmark suite |

### 3.2 Cost

| Resource | Budget |
|---|---|
| Total new public API surface | ≤ 8 new crate-public structs per EPIC |
| Binary size increase | ≤ 2× v1 (without embedding features) |
| Compile time increase | ≤ 3× v1 full clean build |
| Test suite wall-time in CI | ≤ 10 min on ubuntu-latest |

### 3.3 Time

| Milestone | Target date |
|---|---|
| Alpha (EPICs 201–202 closed) | 2026-07-01 |
| Beta (EPICs 203–205 closed) | 2026-10-01 |
| RC (EPICs 206–208 closed) | 2027-01-01 |
| v2.0.0 stable | 2027-03-01 |

---

## 4. Risks

| ID | Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|---|
| R-01 | STS C/P/Δ/Ω dimensions break ZMD backward-compatibility | Medium | High | Additive-only fields behind `version: 2` gate |
| R-02 | Query DSL scope creep into full graph DB | High | Medium | Lock DSL grammar at FEAT-203-1 MVP; addenda require new FEAT |
| R-03 | Embedding providers API churn | High | Low | Adapter pattern; embeddings are optional extension, never required |
| R-04 | Property tests fail to cover new invariants | Medium | High | Extend proptest suite at each EPIC close; 512-case minimum for v2 |
| R-05 | Enricher trait breaks sealed compiler pipeline | Medium | High | Immutable core pipeline; Enricher operates only in post-logical pass |
| R-06 | Semantic diff non-determinism on Unicode edge cases | Low | High | Unicode golden corpus extended at FEAT-204 |
| R-07 | Audit pipeline adds ≥ 5 min to CI | Medium | Medium | Audit runs in parallel matrix, not in main build chain |

---

## 5. Dependencies

| Dependency | Type | Required by |
|---|---|---|
| STF-SIR v1.0.0 (current HEAD) | Internal baseline | All EPICs |
| `pulldown-cmark` ≥ 0.12 | Rust crate | EPIC-207 |
| `petgraph` or equivalent | Rust crate | EPIC-203, EPIC-204 |
| `serde_json` (already indirect) | Rust crate | EPIC-202 |
| `tiktoken-rs` or `tokenizers` (optional) | Rust crate | EPIC-206 |
| External embedding API (optional) | External service | EPIC-206 |
| STS formal spec (docs/sts-formalization.md) | Internal spec | EPIC-201 |
| JSON Schema Draft 2020-12 tooling | External standard | EPIC-202 |

---

## 6. Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────┐
│                      STF-SIR v2 Platform                            │
│                                                                     │
│  ┌──────────────┐    ┌───────────────────────────────────────────┐  │
│  │  Source      │    │   Compiler Pipeline (EPIC-207)            │  │
│  │  Frontends   │───▶│   Lexical → Syntactic → Semantic →        │  │
│  │  (.md, ...)  │    │   Logical → Enricher(s) → Serializer      │  │
│  └──────────────┘    └───────────────┬───────────────────────────┘  │
│                                      │                              │
│                              ZMD v2 Artifact (EPIC-202)             │
│                                      │                              │
│          ┌───────────────────────────┼───────────────────────────┐  │
│          │                           │                           │  │
│  ┌───────▼──────┐         ┌──────────▼────────┐   ┌────────────┐ │  │
│  │ Semantic     │         │  SirGraph v2       │   │ Semantic   │ │  │
│  │ Query Engine │◀────────│  (serializable)    │──▶│ Diff Engine│ │  │
│  │ (EPIC-203)   │         │  (EPIC-203 infra)  │   │ (EPIC-204) │ │  │
│  └───────┬──────┘         └────────────────────┘   └────────────┘ │  │
│          │                                                         │  │
│  ┌───────▼──────┐   ┌────────────────┐   ┌──────────────────────┐ │  │
│  │ Retention &  │   │ Extensibility  │   │ RAG / Agent Bridge   │ │  │
│  │ Benchmark    │   │ Model (EPIC-208)│   │ (EPIC-206)           │ │  │
│  │ (EPIC-205)   │   └────────────────┘   └──────────────────────┘ │  │
│  └──────────────┘                                                  │  │
│                                                                     │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │                 Continuous Audit Pipeline                     │  │
│  │   Stage-1: fmt/clippy  →  Stage-2: contracts  →              │  │
│  │   Stage-3: golden/invariants  →  Stage-4: retention          │  │
│  └──────────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────┘
```

---

## 7. Epic Index

| ID | Title | Status | Owner | Target |
|---|---|---|---|---|
| [EPIC-201](epics/EPIC-201-spec-v2.md) | Spec v2 | planned | — | 2026-06-01 |
| [EPIC-202](epics/EPIC-202-zmd-canonical-v2.md) | ZMD Canonical Format v2 | planned | — | 2026-07-01 |
| [EPIC-203](epics/EPIC-203-semantic-query-engine.md) | Semantic Query Engine | planned | — | 2026-09-01 |
| [EPIC-204](epics/EPIC-204-semantic-diff-engine.md) | Semantic Diff Engine | planned | — | 2026-09-15 |
| [EPIC-205](epics/EPIC-205-retention-benchmark.md) | Retention & Benchmark | planned | — | 2026-10-01 |
| [EPIC-206](epics/EPIC-206-rag-agent-integration.md) | RAG & Agent Integration | planned | — | 2026-12-01 |
| [EPIC-207](epics/EPIC-207-compiler-refactor.md) | Compiler Refactor | planned | — | 2026-08-01 |
| [EPIC-208](epics/EPIC-208-extensibility-model.md) | Extensibility Model | planned | — | 2026-11-01 |

---

## 8. Execution Order (Critical Path)

```
EPIC-201 ──┐
           ├──▶ EPIC-202 ──▶ EPIC-207 ──┐
                                        ├──▶ EPIC-203 ──▶ EPIC-204
                                        │                    │
                                        └──▶ EPIC-208 ────┐  │
                                                          ├──▶ EPIC-205
                                                          └──▶ EPIC-206
```

**Parallel tracks:**
- Track A (Format): EPIC-201 → EPIC-202 → EPIC-207
- Track B (Query): EPIC-203 → EPIC-204 (depends on Track A)
- Track C (Quality): EPIC-205 (can start after EPIC-207)
- Track D (Integration): EPIC-206, EPIC-208 (depends on EPIC-207)

---

## 9. Audit & Compliance

All implementation work is subject to the **[Continuous Audit Model](audit/AUDIT-MODEL.md)**.

The audit pipeline runs:
1. On every PR (stages 1–3)
2. On every merge to `main` (all stages)
3. Nightly (full regression + semantic drift detection)

---

## 10. Related Documents

| Document | Purpose |
|---|---|
| [Contract Model](contracts/CONTRACT-MODEL.md) | Canonical contract schema for all levels |
| [Validation & Quality System](validation/VALIDATION-QUALITY-SYSTEM.md) | Test harness, golden corpus, invariants |
| [Metrics Specification](metrics/METRICS.md) | KPI definitions and measurement scripts |
| [Audit Model](audit/AUDIT-MODEL.md) | Continuous auditor role, pipeline, artifacts |
| [docs/sts-formalization.md](../sts-formalization.md) | Formal STS spec (input to EPIC-201) |
| [spec/stf-sir-spec-v1.md](../../spec/stf-sir-spec-v1.md) | v1 spec (baseline) |
