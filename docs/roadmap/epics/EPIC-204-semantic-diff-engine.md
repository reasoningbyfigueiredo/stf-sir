---
id: EPIC-204
title: Semantic Diff Engine
version: 2.0.0-alpha
status: implementado
roadmap: ROADMAP-STF-SIR-V2
priority: high
created: 2026-04-12
target: 2026-09-15
depends_on:
  - EPIC-203
blocks:
  - EPIC-205
---

# EPIC-204 — Semantic Diff Engine

## Description

Build a deterministic, multi-level semantic diff engine that computes structured differences
between two ZMD artifacts at the structural, semantic, and logical levels. The engine produces
machine-verifiable diff reports in a canonical format, enabling:

- Automated change validation in CI pipelines
- Semantic drift detection between document versions
- Retention regression detection
- AI-agent change summaries with provenance

The diff is computed over the ZToken graph, not over source text — making it format-agnostic
and semantically meaningful.

## Scope

- **In scope:** Structural diff (graph topology), semantic diff (Σ field changes), logical diff (relation set changes), diff report format, CLI diff subcommand
- **Out of scope:** Three-way merge, diff visualization UI, source-level text diff (use `git diff` for that)

## Deliverables

| # | Artifact | Path |
|---|---|---|
| D-204-1 | Structural diff engine | `src/diff/structural.rs` |
| D-204-2 | Semantic diff engine | `src/diff/semantic.rs` |
| D-204-3 | Diff report format spec | `spec/diff-report-v1.md` |
| D-204-4 | Diff report serializer | `src/diff/report.rs` |
| D-204-5 | CLI diff subcommand | `stf-sir diff` |
| D-204-6 | Diff golden corpus | `tests/diff/golden/` |
| D-204-7 | Diff test suite | `tests/diff/` |

## Success Criteria

- [ ] Structural diff F1 ≥ 0.99 on golden diff corpus
- [ ] Semantic diff F1 ≥ 0.90 on human-labeled pairs
- [ ] Diff determinism: 100% on metamorphic suite
- [ ] Diff report is valid JSON/YAML
- [ ] `stf-sir diff a.zmd b.zmd` exits with structured report
- [ ] Diff is symmetric for addition/deletion (diff(A,B) is the inverse of diff(B,A) structurally)

## Risks

| ID | Risk | Mitigation |
|---|---|---|
| R-204-1 | Graph isomorphism for structural diff is NP-complete in general | Restrict to rooted DAG (artifact SirGraph is always a DAG); use DFS matching |
| R-204-2 | Semantic diff on `gloss` field is order-sensitive | Normalize gloss before comparison (NFKC + whitespace); use edit distance for near-matches |
| R-204-3 | Large artifacts produce enormous diff reports | Add `--summary` mode that reports counts only |
| R-204-4 | Unicode edge cases break diff equality | Extend unicode golden corpus to diff suite |

---

## EPIC CONTRACT

```yaml
contract:
  id: CONTRACT-EPIC-204
  version: 1.0.0

  inputs:
    - id: I-204-1
      description: ZMD artifact A (before)
      required: true
    - id: I-204-2
      description: ZMD artifact B (after)
      required: true
    - id: I-204-3
      description: SirGraph v2 for both artifacts (EPIC-203 infrastructure)
      required: true

  outputs:
    - id: O-204-1
      artifact: DiffReport (struct)
      constraint: Serializable to JSON and YAML
    - id: O-204-2
      artifact: src/diff/ module
    - id: O-204-3
      artifact: spec/diff-report-v1.md
    - id: O-204-4
      artifact: stf-sir diff CLI subcommand

  invariants:
    - INV-204-1: |
        Diff determinism: diff(A, B) produces identical output on every invocation
        regardless of OS, thread scheduling, or HashMap seed.
    - INV-204-2: |
        Identity: diff(A, A) = EmptyDiff (zero added, zero removed, zero changed).
    - INV-204-3: |
        Symmetry for counts: |diff(A,B).added| = |diff(B,A).removed| and vice versa.
    - INV-204-4: |
        Diff is non-destructive: neither artifact is modified.
    - INV-204-5: |
        Completeness: every changed token appears in the report.
        No changed token is silently omitted.

  preconditions:
    - PRE-204-1: EPIC-203 closed (SirGraph v2 and query engine operational)
    - PRE-204-2: Both input artifacts pass zmd-v2 schema validation

  postconditions:
    - POST-204-1: DiffReport serializes to valid JSON
    - POST-204-2: Golden diff corpus committed
    - POST-204-3: F1 benchmarks recorded in CI artifacts

  validation:
    automated:
      - script: cargo test diff
        description: Full diff test suite
      - script: cargo bench diff_accuracy
        description: F1 on golden diff corpus; asserts ≥ 0.99 structural, ≥ 0.90 semantic
      - script: tests/diff/metamorphic.sh
        description: Runs diff(A,B) 100×, diffs results
      - script: tests/diff/identity_test.sh
        description: Verifies diff(X, X) = EmptyDiff for all golden fixtures
      - script: tests/diff/symmetry_test.sh
        description: Verifies INV-204-3 for all golden fixture pairs
    manual:
      - review: Human-labeled semantic diff pairs reviewed by domain expert

  metrics:
    - metric: structural_diff_f1
      target: ≥ 0.99
      measurement: cargo bench diff_accuracy --features structural
    - metric: semantic_diff_f1
      target: ≥ 0.90
      measurement: cargo bench diff_accuracy --features semantic
    - metric: diff_determinism_rate
      target: 100%
      measurement: metamorphic suite 100× per pair
    - metric: diff_report_parse_rate
      target: 100%
      measurement: all reports parse as valid JSON/YAML

  failure_modes:
    - FAIL-204-1:
        condition: INV-204-1 violated (non-deterministic diff)
        action: Critical; block EPIC-205 drift detection
    - FAIL-204-2:
        condition: INV-204-2 violated (non-empty diff on identical artifacts)
        action: Critical defect; fix before closing EPIC
    - FAIL-204-3:
        condition: structural F1 < 0.99
        action: Performance defect; profile matching algorithm
    - FAIL-204-4:
        condition: semantic F1 < 0.90
        action: Performance defect; improve normalization or similarity metric
```

---

## Features

### FEAT-204-1: Structural Diff

**Description:** Compute the structural diff between two SirGraph instances: which ztokens were
added, removed, or repositioned; which relations were added or removed; how the tree topology changed.

**Inputs:**
- `SirGraph` A (before)
- `SirGraph` B (after)

**Outputs:**
- `StructuralDiff` type with `added_tokens`, `removed_tokens`, `moved_tokens`, `added_relations`, `removed_relations`
- `src/diff/structural.rs`

**Acceptance Criteria:**
- [ ] Added tokens: tokens in B not in A (matched by normalized_text + node_type + path, not by ID)
- [ ] Removed tokens: tokens in A not in B
- [ ] Moved tokens: tokens in both A and B but with different `path` or `parent_id`
- [ ] Relation diff: added/removed relations matched by (type, source_normalized, target_normalized)
- [ ] Identity invariant (INV-204-2) holds
- [ ] All result Vecs sorted by token normalized_text then path (deterministic)

**Metrics:** structural_diff_f1 ≥ 0.99

**Feature Contract:**
```yaml
contract:
  id: CONTRACT-FEAT-204-1
  inputs: [SirGraph A, SirGraph B]
  outputs: [StructuralDiff struct]
  invariants:
    - INV-204-1 (determinism)
    - INV-204-2 (identity)
    - INV-204-3 (symmetry for counts)
    - INV-204-4 (non-destructive)
  postconditions:
    - All result Vecs sorted
    - Matching uses content identity, not ID
  failure_modes:
    - ID-based matching → false positives on recompiled artifacts
```

#### Tasks

**TASK-204-1-1: Design token matching algorithm**
- Description: Define the matching key for structural equivalence: (normalized_text, node_type, path). Document edge cases: duplicate content at same path, content reordering.
- Definition of done: Algorithm decision document committed to `spec/decisions/ADR-002-diff-matching.md`
- Artifacts: `spec/decisions/ADR-002-diff-matching.md`

**TASK-204-1-2: Implement StructuralDiff computation**
- Description: Write `compute_structural_diff(a: &SirGraph, b: &SirGraph) -> StructuralDiff`
- Definition of done: Function returns correct results on 20 golden diff pairs; identity invariant passes
- Artifacts: `src/diff/structural.rs`

**TASK-204-1-3: Implement relation diff**
- Description: Compute added/removed relations using (type, source_key, target_key) matching
- Definition of done: Relation diff correct on all golden pairs; symmetry invariant holds
- Artifacts: Extends `src/diff/structural.rs`

**TASK-204-1-4: Create structural diff golden corpus**
- Description: 20 pairs of `.zmd` files with corresponding expected `StructuralDiff` JSON outputs
- Definition of done: All golden tests pass with `cargo test diff_structural_golden`
- Artifacts: `tests/diff/golden/structural/`

**TASK-204-1-5: Write structural diff property tests**
- Description: Property tests for identity (diff(X,X) = empty), symmetry (|added| = |removed| in reverse), completeness
- Definition of done: 512 proptest cases pass
- Artifacts: `tests/diff/proptest_structural.rs`

---

### FEAT-204-2: Semantic Diff

**Description:** Compute semantic-level differences between two artifacts: changes to `Σ.gloss`,
`Σ.concepts`, `Σ.confidence`, and `Σ.embedding_ref` for matched token pairs.

**Inputs:**
- Matched token pairs from StructuralDiff (tokens present in both A and B)
- Semantic dimension fields from both sides

**Outputs:**
- `SemanticDiff` type with `changed_glosses`, `changed_concepts`, `changed_confidence`, `changed_embedding_refs`
- `src/diff/semantic.rs`

**Acceptance Criteria:**
- [ ] Gloss changes detected using edit distance (Levenshtein) normalized to [0,1]; threshold configurable
- [ ] Concept list diff: added/removed concept strings per token
- [ ] Confidence change: if abs(a.confidence - b.confidence) > threshold (default 0.05), flagged
- [ ] embedding_ref change: any change in the URI reference is flagged
- [ ] Semantic diff F1 ≥ 0.90 on human-labeled pairs
- [ ] `SemanticDiff` is serializable; empty diff on identical semantic fields

**Metrics:** semantic_diff_f1 ≥ 0.90

**Feature Contract:**
```yaml
contract:
  id: CONTRACT-FEAT-204-2
  inputs: [matched token pairs, Σ fields from both sides]
  outputs: [SemanticDiff struct]
  invariants:
    - INV-204-1 (determinism)
    - INV-204-2 (identity: identical Σ → empty semantic diff)
    - Thresholds are configurable but have documented defaults
  postconditions:
    - SemanticDiff serializes to JSON
    - F1 benchmark passes
  failure_modes:
    - Threshold hard-coded → cannot tune for corpus
    - Non-deterministic edit distance → critical
```

#### Tasks

**TASK-204-2-1: Implement gloss change detection**
- Description: Compute normalized Levenshtein distance for each matched token pair; flag if above threshold
- Definition of done: Function returns correct change flags on golden pairs
- Artifacts: `src/diff/semantic.rs` (initial)

**TASK-204-2-2: Implement concept list diff**
- Description: Set difference for `concepts` Vec; added = in B not A; removed = in A not B
- Definition of done: Correct on all golden pairs; sorted results
- Artifacts: Extends `src/diff/semantic.rs`

**TASK-204-2-3: Create human-labeled semantic diff corpus**
- Description: 50 matched token pairs with human-labeled ground truth (changed/not changed)
- Definition of done: Corpus committed; F1 benchmark script reads it
- Artifacts: `tests/diff/golden/semantic/human_labeled.json`

**TASK-204-2-4: Write semantic diff test suite**
- Description: Tests for identity, per-field change detection, threshold sensitivity
- Artifacts: `tests/diff/semantic_tests.rs`

---

### FEAT-204-3: Diff Report Format and CLI

**Description:** Define the canonical `DiffReport` format, serializer, and the `stf-sir diff`
CLI subcommand. The report unifies structural and semantic diffs into a single structured artifact.

**Inputs:**
- `StructuralDiff` (FEAT-204-1)
- `SemanticDiff` (FEAT-204-2)
- Output format flag

**Outputs:**
- `DiffReport` type and serializer (`src/diff/report.rs`)
- `spec/diff-report-v1.md` (format spec)
- `stf-sir diff` CLI subcommand
- CLI integration tests

**Acceptance Criteria:**
- [ ] `DiffReport` has fields: `format`, `version`, `artifact_a_sha256`, `artifact_b_sha256`, `structural`, `semantic`, `summary`, `generated_at`
- [ ] `summary` field includes: `total_added`, `total_removed`, `total_moved`, `total_semantic_changes`
- [ ] `--format json|yaml` supported; default is YAML
- [ ] `--summary` flag prints only the summary block
- [ ] Exit code 0 = no diff; 1 = diff found; 2 = error
- [ ] `DiffReport` is a valid ZMD extension (top-level field, not a ZMD artifact itself)

**Feature Contract:**
```yaml
contract:
  id: CONTRACT-FEAT-204-3
  inputs: [StructuralDiff, SemanticDiff, format flag]
  outputs: [DiffReport, printed report, exit code]
  invariants:
    - INV-204-1 (determinism of report bytes)
    - Report always includes both sha256 references
    - Summary counts match actual change counts
  postconditions:
    - JSON output parses as valid JSON
    - Exit codes stable
  failure_modes:
    - Summary count mismatch → data integrity failure
```

#### Tasks

**TASK-204-3-1: Define DiffReport struct and serializer**
- Description: Write `DiffReport` with all fields; implement `to_json_string()` and `to_yaml_string()`
- Artifacts: `src/diff/report.rs`

**TASK-204-3-2: Write diff report format spec**
- Description: Document all fields, their types, semantics, and example outputs
- Artifacts: `spec/diff-report-v1.md`

**TASK-204-3-3: Add diff subcommand to CLI**
- Description: Add `Diff` variant to CLI with `--format`, `--summary`, `--threshold` flags
- Artifacts: Updated `src/cli.rs`

**TASK-204-3-4: Write CLI diff integration tests**
- Description: Test all exit codes, format outputs, and summary mode
- Artifacts: `tests/diff/cli_tests.rs`
