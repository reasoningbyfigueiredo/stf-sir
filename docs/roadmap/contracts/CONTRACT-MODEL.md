---
id: CONTRACT-MODEL-V2
title: Contract Model — STF-SIR v2
version: 2.0.0-alpha
status: draft
roadmap: ROADMAP-STF-SIR-V2
created: 2026-04-12
---

# Contract Model — STF-SIR v2

## 1. Overview

Every unit of work in the STF-SIR v2 roadmap — EPICs, FEATUREs, and TASKs — carries a
**machine-verifiable contract**. Contracts are the primary mechanism for:

1. Ensuring implementations match specifications
2. Enabling automated audit without human review of every change
3. Providing traceability from requirements to tests to code

---

## 2. Contract Schema

All contracts use the following canonical YAML structure:

```yaml
contract:
  id: CONTRACT-<LEVEL>-<ID>     # unique identifier
  version: <semver>              # contract version (separate from feature version)

  # ─── INPUTS ───────────────────────────────────────────────────────────
  inputs:
    - id: I-<N>
      description: <human-readable description>
      required: true | false
      schema: <optional: schema ref or type>

  # ─── OUTPUTS ──────────────────────────────────────────────────────────
  outputs:
    - id: O-<N>
      artifact: <path or type name>
      constraint: <optional: free-text constraint>
      schema: <optional: schema ref>

  # ─── INVARIANTS ───────────────────────────────────────────────────────
  # Must hold at all times; machine-testable via property tests
  invariants:
    - INV-<ID>: |
        <formal statement of the invariant>
        <may reference mathematical notation>

  # ─── PRECONDITIONS ────────────────────────────────────────────────────
  # Must be true before work begins
  preconditions:
    - PRE-<N>: <statement>

  # ─── POSTCONDITIONS ───────────────────────────────────────────────────
  # Must be true when work is complete
  postconditions:
    - POST-<N>: <statement>
    # Each postcondition must be machine-checkable via the validation section

  # ─── VALIDATION ───────────────────────────────────────────────────────
  # Automated checks that verify postconditions
  validation:
    automated:
      - script: <path or cargo command>
        description: <what this checks>
        asserts: <POST-* references>
    manual:
      - review: <description of required manual review>

  # ─── METRICS ──────────────────────────────────────────────────────────
  # Measurable KPIs associated with this contract
  metrics:
    - metric: <name>
      formula: <optional: formula for computed metrics>
      target: <value or inequality>
      measurement: <how to measure>

  # ─── FAILURE MODES ────────────────────────────────────────────────────
  # Defined response to each possible failure
  failure_modes:
    - FAIL-<ID>:
        condition: <what happened>
        action: <required response>
        severity: critical | error | warning
```

---

## 3. Contract Levels

### 3.1 EPIC Contract

Scope: entire EPIC. Verified at EPIC close.

- Inputs: artifacts or states required to begin the EPIC
- Outputs: all deliverables produced
- Invariants: system-wide properties that must hold after closing
- Validation: full test suites + CI stages

### 3.2 FEATURE Contract

Scope: single feature within an EPIC. Verified at feature PR merge.

- Inputs: specific artifacts or APIs required by this feature
- Outputs: specific source files or data artifacts
- Invariants: feature-level properties (often a subset of the EPIC invariants)
- Validation: unit/integration tests for this feature
- Metrics: latency, accuracy, coverage KPIs

### 3.3 TASK Contract

Scope: single implementation task. Verified at task commit.

- Inputs: specific data/files needed
- Outputs: specific file or function produced
- Invariants: narrow (often just one or two)
- Validation: unit test or script that runs in < 30s
- Failure modes: what to do if the task artifact is wrong

---

## 4. Invariant Taxonomy

Invariants are classified by their mathematical character:

| Class | Symbol | Description | Example |
|---|---|---|---|
| Safety | INV-S-* | Something bad never happens | No query panics |
| Liveness | INV-L-* | Something good eventually happens | All tokens emitted |
| Monotone | INV-M-* | Values only increase (enrichment) | concepts.len() ≥ pre-enrichment |
| Determinism | INV-D-* | Identical input → identical output | compile(s, c) = compile(s, c) |
| Completeness | INV-C-* | No item is silently omitted | All matching tokens in result |
| Idempotence | INV-I-* | Applying twice = applying once | enrich(enrich(A)) = enrich(A) |
| Isolation | INV-X-* | No side effects outside scope | Plugin writes only to its namespace |

---

## 5. Contract Coverage Requirements

For the v2 roadmap to be considered audit-ready, the following coverage requirements apply:

| Level | Minimum contract fields | Validation scripts required? | Metrics required? |
|---|---|---|---|
| EPIC | All fields | Yes (≥ 2 automated) | Yes (≥ 2 KPIs) |
| FEATURE | inputs/outputs/invariants/postconditions/failure_modes | Yes (≥ 1 automated) | Yes (≥ 1 KPI) |
| TASK | inputs/outputs/invariants | Recommended | No |

---

## 6. Contract Validation Script

The `scripts/audit/extract-contracts.sh` script parses all contract YAML blocks from the
roadmap documents and produces a machine-readable index:

```bash
scripts/audit/extract-contracts.sh docs/roadmap/ > docs/roadmap/contracts/contract-index.yaml
```

The `scripts/audit/validate-contract-completeness.sh` script checks coverage requirements:

```bash
scripts/audit/validate-contract-completeness.sh docs/roadmap/contracts/contract-index.yaml
```

Exit codes:
- 0: all contracts meet coverage requirements
- 1: one or more EPICs missing required fields (blocks PR)
- 2: parse error in contract YAML

---

## 7. Contract Change Policy

- Contracts at TASK level may be updated freely during implementation
- Contracts at FEATURE level require EPIC owner review before change
- Contracts at EPIC level require author sign-off before change
- Invariant relaxation (making an invariant weaker) requires an explicit decision record
  in `spec/decisions/ADR-*.md` explaining why the stronger invariant was not achievable

---

## 8. Invariant → Test Mapping

Every invariant MUST have at least one automated test. The mapping is maintained in:
`docs/roadmap/contracts/invariant-test-map.yaml`

Format:
```yaml
invariants:
  - id: INV-203-1
    description: Query determinism
    tests:
      - cargo test query_metamorphic
      - tests/query/metamorphic.rs
    coverage: full   # full | partial | missing
  - id: INV-207-2
    description: Enricher monotonicity
    tests:
      - cargo test enricher_monotonicity
    coverage: full
```

The audit pipeline reads this file in Stage-2 to verify no invariant is uncovered.

---

## 9. Complete Invariant Registry (v2)

| ID | Level | Class | Description | Source |
|---|---|---|---|---|
| INV-201-1 | EPIC-201 | Safety | v1 rules VAL_01–VAL_18 preserved | CONTRACT-EPIC-201 |
| INV-201-2 | EPIC-201 | Safety | ZToken v2 is strict superset of v1 | CONTRACT-EPIC-201 |
| INV-201-3 | EPIC-201 | Monotone | Enricher trait is monotone | CONTRACT-EPIC-201 |
| INV-201-4 | EPIC-201 | Safety | New relation types have exactly one category | CONTRACT-EPIC-201 |
| INV-202-1 | EPIC-202 | Safety | v1 ZMD valid against v2 schema | CONTRACT-EPIC-202 |
| INV-202-2 | EPIC-202 | Safety | format const enforced | CONTRACT-EPIC-202 |
| INV-202-3 | EPIC-202 | Determinism | Output SHA-256 is stable | CONTRACT-EPIC-202 |
| INV-202-4 | EPIC-202 | Safety | All v2 changes are additive | CONTRACT-EPIC-202 |
| INV-202-5 | EPIC-202 | Safety | version ∈ {1, 2} | CONTRACT-EPIC-202 |
| INV-203-1 | EPIC-203 | Determinism | Query determinism | CONTRACT-EPIC-203 |
| INV-203-2 | EPIC-203 | Completeness | No matching node dropped | CONTRACT-EPIC-203 |
| INV-203-3 | EPIC-203 | Isolation | Queries are read-only | CONTRACT-EPIC-203 |
| INV-203-4 | EPIC-203 | Determinism | Result serialization is deterministic | CONTRACT-EPIC-203 |
| INV-203-5 | EPIC-203 | Safety | Engine reports error on malformed query (no panic) | CONTRACT-EPIC-203 |
| INV-204-1 | EPIC-204 | Determinism | Diff determinism | CONTRACT-EPIC-204 |
| INV-204-2 | EPIC-204 | Idempotence | diff(A, A) = EmptyDiff | CONTRACT-EPIC-204 |
| INV-204-3 | EPIC-204 | Safety | |added(A,B)| = |removed(B,A)| | CONTRACT-EPIC-204 |
| INV-204-4 | EPIC-204 | Isolation | Diff is non-destructive | CONTRACT-EPIC-204 |
| INV-204-5 | EPIC-204 | Completeness | All changed tokens in report | CONTRACT-EPIC-204 |
| INV-205-1 | EPIC-205 | Determinism | Benchmark reproducibility | CONTRACT-EPIC-205 |
| INV-205-2 | EPIC-205 | Monotone | v2 baseline ≥ v1 baseline | CONTRACT-EPIC-205 |
| INV-205-3 | EPIC-205 | Safety | No silent corpus failure | CONTRACT-EPIC-205 |
| INV-205-4 | EPIC-205 | Safety | All ρ ∈ [0.0, 1.0] | CONTRACT-EPIC-205 |
| INV-205-5 | EPIC-205 | Safety | Drift uses semantic diff (not text diff) | CONTRACT-EPIC-205 |
| INV-206-1 | EPIC-206 | Completeness | Every chunk has full provenance | CONTRACT-EPIC-206 |
| INV-206-2 | EPIC-206 | Isolation | RAG code absent without --features rag | CONTRACT-EPIC-206 |
| INV-206-3 | EPIC-206 | Isolation | Agent tools are read-only | CONTRACT-EPIC-206 |
| INV-206-4 | EPIC-206 | Determinism | Chunk determinism | CONTRACT-EPIC-206 |
| INV-207-1 | EPIC-207 | Determinism | v1 golden gate: zero byte regressions | CONTRACT-EPIC-207 |
| INV-207-2 | EPIC-207 | Monotone | Enricher monotonicity | CONTRACT-EPIC-207 |
| INV-207-3 | EPIC-207 | Safety | Pipeline order is fixed and deterministic | CONTRACT-EPIC-207 |
| INV-207-4 | EPIC-207 | Determinism | No-enricher output identical to v1 | CONTRACT-EPIC-207 |
| INV-207-5 | EPIC-207 | Safety | Compiler never panics | CONTRACT-EPIC-207 |
| INV-208-1 | EPIC-208 | Isolation | Plugin writes only to declared namespace | CONTRACT-EPIC-208 |
| INV-208-2 | EPIC-208 | Monotone | Plugins inherit enricher monotonicity | CONTRACT-EPIC-208 |
| INV-208-3 | EPIC-208 | Safety | stf-sir namespace reserved | CONTRACT-EPIC-208 |
| INV-208-4 | EPIC-208 | Safety | External protocol uses JSON only (no shell) | CONTRACT-EPIC-208 |
| INV-208-5 | EPIC-208 | Determinism | Plugin registration is deterministic | CONTRACT-EPIC-208 |
