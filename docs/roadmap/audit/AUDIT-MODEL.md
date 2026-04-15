---
id: AUDIT-MODEL-V2
title: Continuous Audit Model — STF-SIR v2
version: 2.0.0-alpha
status: draft
roadmap: ROADMAP-STF-SIR-V2
created: 2026-04-12
---

# Continuous Audit Model — STF-SIR v2

## 1. Independent Auditor Role

The **Independent Auditor** is a stateless, automated role executed by the CI pipeline.
It does not implement features; it validates that implementations conform to contracts.

### 1.1 Responsibilities

| Responsibility | Frequency | Blocking? |
|---|---|---|
| Validate all EPIC/FEATURE/TASK contracts | Every PR merge | Yes |
| Run golden gate (byte-for-byte regressions) | Every commit | Yes |
| Run invariant property tests (512+ cases) | Every PR | Yes |
| Run semantic retention benchmark | Every merge to main | Yes (if ρ < 0.97) |
| Detect semantic drift between versions | Nightly | No (advisory) |
| Produce audit report artifact | Every CI run | No |
| Check contract compliance score | Weekly | No (reported) |
| Validate schema and spec consistency | Every PR that touches spec/ or schemas/ | Yes |

### 1.2 Auditor Independence Guarantees

- The auditor scripts are in `scripts/audit/` and MUST NOT be modified by feature PRs
  without a separate PR reviewed by the project author
- Audit scripts are run in a clean matrix environment (`ubuntu-latest`, `macos-latest`)
- Audit results are uploaded as CI artifacts and committed to `docs/audit-reports/` on main merges
- No audit script depends on mutable state outside the repository

---

## 2. Audit Pipeline

### 2.1 Stage Overview

```
┌──────────────────────────────────────────────────────────────┐
│  CI Audit Pipeline (runs in parallel with main build)        │
│                                                              │
│  Stage-1: Static Analysis        [every commit]             │
│  Stage-2: Contract Validation    [every PR]                 │
│  Stage-3: Golden & Invariants    [every PR]                 │
│  Stage-4: Retention Benchmark    [every main merge]         │
│  Stage-5: Drift Detection        [nightly]                  │
│  Stage-6: Compliance Report      [weekly]                   │
└──────────────────────────────────────────────────────────────┘
```

### 2.2 Stage-1: Static Analysis

**Trigger:** Every commit on every branch.

**Scripts:**
```bash
# S1-1: Format check
cargo fmt --all -- --check

# S1-2: Clippy (all features, treat warnings as errors)
cargo clippy --all-targets --all-features -- -D warnings

# S1-3: Dependency audit (supply chain)
cargo deny check

# S1-4: Spec lint (section numbering, rule IDs)
scripts/audit/lint-spec.sh spec/stf-sir-spec-v2.md

# S1-5: Schema meta-validation
scripts/audit/validate-schema-meta.sh schemas/zmd-v2.schema.json
```

**Failure policy:** Any failure blocks the PR. No merge allowed.

**Artifacts produced:**
- `clippy.log`
- `deny.log`
- `spec-lint.log`

---

### 2.3 Stage-2: Contract Validation

**Trigger:** Every PR (runs after Stage-1 passes).

**Purpose:** Validate that all contracts (EPIC / FEATURE / TASK) are machine-checkable and
that implementations match their declared contracts.

**Scripts:**
```bash
# S2-1: Parse all contract YAML blocks from roadmap docs
scripts/audit/extract-contracts.sh docs/roadmap/ > /tmp/contracts.yaml

# S2-2: Validate contract completeness (all required fields present)
scripts/audit/validate-contract-completeness.sh /tmp/contracts.yaml

# S2-3: Check invariant implementations exist in tests
scripts/audit/check-invariant-coverage.sh /tmp/contracts.yaml tests/

# S2-4: Validate validation scripts exist and are executable
scripts/audit/check-validation-scripts.sh /tmp/contracts.yaml scripts/

# S2-5: Check failure mode handlers exist (grep for FAIL-* codes in src/)
scripts/audit/check-failure-handlers.sh /tmp/contracts.yaml src/
```

**Failure policy:**
- Missing required contract field → warning (advisory, does not block)
- Missing invariant test → error (blocks PR)
- Missing validation script → error (blocks PR)

**Artifacts produced:**
- `contracts-report.yaml` — parsed contract tree with validation status
- `invariant-coverage-report.txt`

---

### 2.4 Stage-3: Golden Gate & Invariants

**Trigger:** Every PR (can run in parallel with Stage-2).

**Purpose:** Ensure no regression in semantic representation correctness.

**Scripts:**
```bash
# S3-1: v1 golden gate (byte-for-byte)
cargo test golden -- --nocapture

# S3-2: v2 golden gate
cargo test golden_v2 -- --nocapture

# S3-3: Conformance suite (valid + invalid)
cargo test conformance -- --nocapture

# S3-4: Property tests (512 cases minimum)
PROPTEST_CASES=512 cargo test proptest_invariants -- --nocapture

# S3-5: Enricher monotonicity property test
PROPTEST_CASES=512 cargo test enricher_monotonicity -- --nocapture

# S3-6: Query determinism metamorphic test
cargo test query_metamorphic -- --nocapture

# S3-7: Diff identity and symmetry tests
cargo test diff_identity diff_symmetry -- --nocapture

# S3-8: Determinism gate (compile sample 1000×, check SHA-256)
scripts/audit/check-determinism.sh 1000 tests/golden/sample.md
```

**Failure policy:** Any failure blocks merge. All 8 scripts must exit 0.

**Artifacts produced:**
- `golden-gate-v1.log`
- `golden-gate-v2.log`
- `conformance-report.log`
- `proptest-results.txt`
- `determinism-gate.log`

---

### 2.5 Stage-4: Retention Benchmark

**Trigger:** Every merge to `main`.

**Purpose:** Ensure semantic retention ρ_v2 ≥ 0.97 on all components after any change to main.

**Scripts:**
```bash
# S4-1: Run corpus benchmark
cargo bench retention_v2 -- --output-format json > /tmp/retention-report.json

# S4-2: Assert all ρ components ≥ 0.97
scripts/audit/assert-retention.sh /tmp/retention-report.json 0.97

# S4-3: Compare to committed baseline
scripts/audit/compare-retention-baseline.sh \
  /tmp/retention-report.json \
  docs/retention-baseline-v2.md

# S4-4: Publish report to docs/audit-reports/
scripts/audit/publish-report.sh \
  /tmp/retention-report.json \
  docs/audit-reports/retention-$(date +%Y-%m-%d).json
```

**Failure policy:**
- ρ_v2 < 0.97 on any component → merge blocked; open retention defect
- ρ_v2 regresses more than 0.02 below baseline → merge blocked

**Artifacts produced:**
- `retention-report.json`
- `retention-delta-report.txt`
- Committed to `docs/audit-reports/`

---

### 2.6 Stage-5: Drift Detection

**Trigger:** Nightly (scheduled CI run).

**Purpose:** Compare today's main artifacts against the committed baseline to detect semantic
drift over time.

**Scripts:**
```bash
# S5-1: Compile benchmark corpus with current main
scripts/audit/compile-corpus.sh tests/benchmark/corpus/ /tmp/current-artifacts/

# S5-2: Run drift detection against baseline artifacts
cargo run --features benchmark -- detect-drift \
  docs/audit-reports/baseline-artifacts/ \
  /tmp/current-artifacts/ \
  --threshold 0.02 \
  --output /tmp/drift-report.json

# S5-3: Log drift report
scripts/audit/publish-report.sh \
  /tmp/drift-report.json \
  docs/audit-reports/drift-$(date +%Y-%m-%d).json

# S5-4: Alert on high-severity drift
scripts/audit/alert-on-drift.sh /tmp/drift-report.json
```

**Failure policy:** Drift above threshold = advisory warning (creates GitHub issue, does not block).

**Artifacts produced:**
- `drift-report-YYYY-MM-DD.json`
- GitHub issue if drift detected

---

### 2.7 Stage-6: Compliance Report

**Trigger:** Weekly (Sunday 00:00 UTC).

**Purpose:** Produce a comprehensive compliance score across all contracts, invariants,
and metrics defined in the roadmap.

**Scripts:**
```bash
# S6-1: Aggregate all CI artifacts from the past week
scripts/audit/aggregate-ci-artifacts.sh 7 > /tmp/weekly-artifacts.json

# S6-2: Compute compliance score per EPIC
scripts/audit/compute-compliance-score.sh \
  /tmp/weekly-artifacts.json \
  docs/roadmap/ \
  > /tmp/compliance-report.json

# S6-3: Publish weekly compliance report
scripts/audit/publish-report.sh \
  /tmp/compliance-report.json \
  docs/audit-reports/compliance-$(date +%Y-%m-%d).json

# S6-4: Update compliance badge in README
scripts/audit/update-compliance-badge.sh /tmp/compliance-report.json
```

**Failure policy:** Advisory only. Compliance score < 90% creates a GitHub issue.

**Artifacts produced:**
- `compliance-report-YYYY-MM-DD.json` (committed)
- README compliance badge updated

---

## 3. Audit Artifacts

### 3.1 Artifact Storage

All audit artifacts are stored in two places:
1. **CI artifacts** — attached to the CI run for 90 days
2. **Git-committed reports** — in `docs/audit-reports/` (JSON, size-constrained: max 100KB per report)

### 3.2 Artifact Schema

All audit reports share a common envelope:

```json
{
  "audit_report": {
    "format": "stf-sir-audit-report",
    "version": "1",
    "stage": "<stage-id>",
    "generated_at": "<ISO-8601>",
    "commit_sha": "<git-sha>",
    "pipeline_run": "<CI-run-id>",
    "status": "pass | fail | advisory",
    "score": 0.98,
    "findings": [
      {
        "id": "FINDING-001",
        "severity": "error | warning | info",
        "contract_ref": "CONTRACT-EPIC-203",
        "invariant_ref": "INV-203-1",
        "description": "...",
        "evidence": "..."
      }
    ],
    "metrics": {}
  }
}
```

### 3.3 Compliance Score Calculation

The compliance score is computed as:

```
compliance_score = (
    (passed_contracts / total_contracts) * 0.40 +
    (passed_invariants / total_invariants) * 0.30 +
    (retention_score / 1.0) * 0.20 +
    (golden_pass_rate / 1.0) * 0.10
)
```

Where:
- `passed_contracts`: contracts with all postconditions met
- `passed_invariants`: invariants with no violation in 512-case property tests
- `retention_score`: mean ρ_v2 across all components
- `golden_pass_rate`: fraction of golden tests passing byte-for-byte

### 3.4 Contract Violation Classification

| Severity | Definition | Action |
|---|---|---|
| CRITICAL | Invariant violated; data correctness at risk | Block merge immediately |
| ERROR | Contract postcondition not met | Block PR |
| WARNING | Contract precondition advisory (e.g., missing test) | Allow merge with issue |
| INFO | Metric below target but within tolerance | Record in report |

---

## 4. Audit Script Inventory

All scripts live in `scripts/audit/` and are checked into the repository.

| Script | Stage | Description |
|---|---|---|
| `lint-spec.sh` | S1 | Validates spec markdown structure |
| `validate-schema-meta.sh` | S1 | JSON Schema meta-validation |
| `extract-contracts.sh` | S2 | Parses YAML contract blocks from roadmap docs |
| `validate-contract-completeness.sh` | S2 | Checks all required contract fields |
| `check-invariant-coverage.sh` | S2 | Verifies each invariant has a test |
| `check-validation-scripts.sh` | S2 | Verifies each validation script exists |
| `check-failure-handlers.sh` | S2 | Checks FAIL-* codes referenced in src/ |
| `check-determinism.sh` | S3 | Compiles N times, compares SHA-256 |
| `assert-retention.sh` | S4 | Asserts ρ_v2 ≥ threshold on all components |
| `compare-retention-baseline.sh` | S4 | Detects retention regression vs baseline |
| `publish-report.sh` | S4–S6 | Commits report to docs/audit-reports/ |
| `compile-corpus.sh` | S5 | Batch-compiles document corpus |
| `alert-on-drift.sh` | S5 | Creates GitHub issue on drift |
| `aggregate-ci-artifacts.sh` | S6 | Collects CI artifact JSONs |
| `compute-compliance-score.sh` | S6 | Aggregates compliance score |
| `update-compliance-badge.sh` | S6 | Updates README badge |

---

## 5. Failure Policies Summary

| Condition | Stage | Policy |
|---|---|---|
| fmt / clippy failure | S1 | Block PR |
| Schema invalid | S1 | Block PR |
| Missing invariant test | S2 | Block PR |
| Missing validation script | S2 | Block PR |
| v1 golden regression | S3 | Block merge |
| Property test failure | S3 | Block merge |
| Determinism failure | S3 | Block merge |
| ρ_v2 < 0.97 | S4 | Block main merge |
| ρ_v2 regression > 0.02 | S4 | Block main merge |
| Drift > threshold | S5 | Advisory (GitHub issue) |
| Compliance score < 90% | S6 | Advisory (GitHub issue) |

---

## 6. Audit Independence Safeguards

1. **Separation of concerns:** `scripts/audit/` is reviewed independently from feature PRs
2. **Immutable baselines:** `docs/retention-baseline-v2.md` can only be updated via a separate PR with explicit justification
3. **Report archiving:** All audit reports are immutable once committed (no overwrite, only append)
4. **Audit log integrity:** Report files include the commit SHA and CI run ID for traceability
5. **External reproducibility:** Any third party with the repository can reproduce all audit results by running `scripts/audit/run-all.sh`
