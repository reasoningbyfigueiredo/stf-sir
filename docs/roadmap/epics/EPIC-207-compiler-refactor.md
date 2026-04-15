---
id: EPIC-207
title: Compiler Refactor
version: 2.0.0-alpha
status: implementado
roadmap: ROADMAP-STF-SIR-V2
priority: critical
created: 2026-04-12
target: 2026-08-01
depends_on:
  - EPIC-202
blocks:
  - EPIC-203
  - EPIC-205
---

# EPIC-207 — Compiler Refactor

## Description

Refactor the STF-SIR compiler to support the v2 platform capabilities while preserving
all v1 invariants and guarantees. Key changes:

1. **Enricher trait** — a formal post-logical enrichment pass with monotone semantics
2. **Multi-frontend support** — pluggable `SourceParser` trait (current Markdown frontend + future extension points)
3. **Profile system** — named compilation profiles (`block-v1`, `block-v2`, `sentence-v1`, `entity-v1`) that control which dimensions are populated
4. **Language detection** — BCP-47 language detection replacing hardcoded `"und"`
5. **SirGraph v2 serialization** — in-memory SirGraph becomes serializable (FEAT-202-3 integration)
6. **Extended relation emitters** — emit the 5 new relation types defined in EPIC-201

All v1 golden tests MUST continue to pass unchanged. The refactor is backward-compatible at
the library API level (no breaking changes to `compiler::compile_markdown` and friends).

## Scope

- **In scope:** Enricher trait, profile system, SourceParser trait, language detection, SirGraph serialization, new relation emitters, v1 golden test preservation
- **Out of scope:** New source frontends beyond Markdown (only the adapter interface is added), embedding computation (EPIC-206), external enricher plugins (EPIC-208)

## Deliverables

| # | Artifact | Path |
|---|---|---|
| D-207-1 | Enricher trait | `src/compiler/enricher.rs` |
| D-207-2 | SourceParser trait | `src/compiler/frontend.rs` |
| D-207-3 | Profile system | `src/compiler/profile.rs` |
| D-207-4 | Language detection | `src/compiler/lang.rs` |
| D-207-5 | SirGraph v2 serializer | `src/sir/serializer.rs` |
| D-207-6 | New relation emitters | Updated `src/compiler/logical.rs` |
| D-207-7 | Updated v2 golden corpus | `tests/golden/v2/` |
| D-207-8 | Compiler v2 integration tests | `tests/compile_v2.rs` |

## Success Criteria

- [x] All 6 v1 golden tests pass byte-for-byte (regression: none)
- [x] Enricher trait is formally monotone (provable by test: no field value decreases)
- [x] Profile system produces correct ZToken field sets per profile
- [x] Language detection correctly identifies language for English, Portuguese, and mixed documents
- [x] SirGraph v2 round-trips: to_json() → from_json() produces isomorphic graph
- [ ] All 5 new relation types are emitted correctly by the logical stage

## Risks

| ID | Risk | Mitigation |
|---|---|---|
| R-207-1 | Enricher ordering introduces non-determinism | Enrichers are applied in a fixed, registered order; no parallel execution |
| R-207-2 | Profile system adds complexity to the serializer | Serialize all dimensions; omit empty optional dimensions (YAML null omission) |
| R-207-3 | Language detection library is large | Use `whatlang` or `lingua` crate; feature-gate (`--features lang-detect`) |
| R-207-4 | SirGraph serialization breaks byte-stable golden tests | SirGraph export is optional and in a separate field; not included in default output |

---

## EPIC CONTRACT

```yaml
contract:
  id: CONTRACT-EPIC-207
  version: 1.0.0

  inputs:
    - id: I-207-1
      description: ZMD v2 schema (EPIC-202 output)
      required: true
    - id: I-207-2
      description: STF-SIR Spec v2 (EPIC-201 output)
      required: true
    - id: I-207-3
      description: v1 compiler source (src/compiler/)
      required: true
    - id: I-207-4
      description: v1 golden corpus (tests/golden/)
      required: true

  outputs:
    - id: O-207-1
      artifact: src/compiler/ (refactored)
    - id: O-207-2
      artifact: src/sir/ (with serializer)
    - id: O-207-3
      artifact: tests/compile_v2.rs
    - id: O-207-4
      artifact: tests/golden/v2/ (v2 golden corpus)

  invariants:
    - INV-207-1: |
        v1 golden gate: all 6 v1 golden tests produce byte-identical output
        after the refactor. Any byte difference is a regression.
    - INV-207-2: |
        Enricher monotonicity: for any enricher E and artifact A,
        no field in E(A) has a weaker value than the same field in A.
        Specifically: concepts can only grow, confidence can only increase,
        gloss cannot be overwritten (only supplemented).
    - INV-207-3: |
        Pipeline order is deterministic: lexical → syntactic → semantic →
        logical → enricher(s) → serializer. No stage may be skipped or reordered.
    - INV-207-4: |
        The base compiler (no enrichers registered) produces identical output
        to the v1 compiler for any v1-compatible input.
    - INV-207-5: |
        The compiler NEVER panics. All error cases return Err(CompilerError).

  preconditions:
    - PRE-207-1: EPIC-202 closed (ZMD v2 schema published)
    - PRE-207-2: All v1 tests pass on current HEAD
    - PRE-207-3: Spec v2 defines all new relation types

  postconditions:
    - POST-207-1: All v1 golden tests pass byte-for-byte
    - POST-207-2: All v2 integration tests pass
    - POST-207-3: Enricher trait documented with examples
    - POST-207-4: Zero clippy warnings on all targets

  validation:
    automated:
      - script: cargo test golden
        description: v1 golden gate (byte-for-byte)
      - script: cargo test golden_v2
        description: v2 golden corpus
      - script: cargo test compile_v2
        description: v2 compiler integration tests
      - script: cargo test enricher_monotonicity
        description: Property test: no field value decreases after enrichment
      - script: cargo test profile_system
        description: Tests all profiles produce correct field sets
    manual:
      - review: Enricher trait design reviewed before implementation

  metrics:
    - metric: v1_golden_regression_count
      target: 0
    - metric: enricher_monotonicity_violations
      target: 0
      measurement: 512-case property test
    - metric: profile_field_coverage
      formula: (correct_fields / expected_fields_per_profile) * 100
      target: 100%
    - metric: language_detection_accuracy
      target: ≥ 0.95 on labeled test corpus

  failure_modes:
    - FAIL-207-1:
        condition: INV-207-1 violated (v1 golden regression)
        action: Block all downstream EPICs; revert to last green state
    - FAIL-207-2:
        condition: INV-207-2 violated (non-monotone enricher)
        action: Reject enricher implementation; rewrite
    - FAIL-207-3:
        condition: INV-207-5 violated (panic)
        action: Critical; block release
```

---

## Features

### FEAT-207-1: Enricher Trait

**Description:** Define and implement the `Enricher` trait — a formally monotone post-logical
enrichment pass that may augment ZToken semantic and logical dimensions without removing or
weakening existing values.

**Inputs:**
- `Artifact` (post-logical stage output)
- Trait contract (monotonicity requirement)

**Outputs:**
- `src/compiler/enricher.rs` — `Enricher` trait + `EnricherPipeline`
- `src/compiler/enrichers/passthrough.rs` — identity enricher (no-op, for testing)
- `src/compiler/enrichers/concept_extractor.rs` — example keyword-based concept extractor

**Acceptance Criteria:**
- [x] `trait Enricher: Send + Sync { fn enrich(&self, artifact: &mut Artifact) -> Result<(), EnricherError>; fn name(&self) -> &str; }`
- [x] `EnricherPipeline` applies enrichers in fixed registration order
- [x] Passthrough enricher is a no-op: `enrich(A)` leaves A byte-identical
- [x] Concept extractor enricher populates `Σ.concepts` with keywords extracted from `Σ.gloss`
- [ ] Monotonicity property test (512 cases): `concepts.len()` only grows; `confidence` only increases; `gloss` unchanged
- [x] Enricher registration is deterministic (Vec, not HashMap)

**Metrics:** enricher_monotonicity_violations = 0

**Feature Contract:**
```yaml
contract:
  id: CONTRACT-FEAT-207-1
  inputs: [Artifact (post-logical)]
  outputs: [Artifact (enriched)]
  invariants:
    - INV-207-2 (monotonicity)
    - INV-207-3 (pipeline order)
    - INV-207-4 (no enricher → identical to v1)
  postconditions:
    - Passthrough enricher leaves artifact byte-identical
    - Concept extractor adds ≥ 0 concepts (never removes)
  failure_modes:
    - Enricher removes concept → monotonicity violation → reject
```

#### Tasks

**TASK-207-1-1: Design Enricher trait and EnricherPipeline**
- Description: Define trait signature, error type, registration API, and execution contract
- Definition of done: Trait design document reviewed and approved
- Artifacts: `spec/decisions/ADR-004-enricher-trait.md`

**TASK-207-1-2: Implement Enricher trait and EnricherPipeline**
- Artifacts: `src/compiler/enricher.rs`

**TASK-207-1-3: Implement passthrough enricher**
- Artifacts: `src/compiler/enrichers/passthrough.rs`

**TASK-207-1-4: Implement keyword concept extractor enricher**
- Description: Extract non-stopword tokens from `gloss` as concepts; deduplicate; sort
- Artifacts: `src/compiler/enrichers/concept_extractor.rs`

**TASK-207-1-5: Write enricher monotonicity property tests**
- Description: 512 proptest cases; assert no field value decreases after any registered enricher
- Artifacts: `tests/enricher_monotonicity.rs`

---

### FEAT-207-2: SourceParser Trait and Profile System

**Description:** Introduce a `SourceParser` trait that abstracts the Markdown frontend,
and a `CompilationProfile` system that controls which ZToken dimensions are populated
and which validation rules are applied.

**Inputs:**
- v1 Markdown parser (`src/compiler/lexical.rs` + `src/compiler/syntactic.rs`)
- Profile definitions from spec v2

**Outputs:**
- `src/compiler/frontend.rs` — `SourceParser` trait
- `src/compiler/profile.rs` — `CompilationProfile` enum and profile registry
- `src/compiler/frontends/markdown.rs` — Markdown frontend (refactored from v1)

**Acceptance Criteria:**
- [x] `SourceParser` trait: `parse(&self, source: &str, path: Option<&str>) -> Result<ParsedDocument, FrontendError>`
- [x] `CompilationProfile` enum: `BlockV1Mvp`, `BlockV2`, `SentenceV2`, `EntityV2`
- [x] `BlockV1Mvp` profile produces identical output to v1 compiler (INV-207-4)
- [x] `BlockV2` profile additionally populates C/P/Δ/Ω dimensions when enrichers are registered
- [ ] Profile selection via CLI flag `--profile block-v1|block-v2|sentence-v1|entity-v1`
- [x] Default profile: `BlockV1Mvp` for backward-compat mode

**Feature Contract:**
```yaml
contract:
  id: CONTRACT-FEAT-207-2
  inputs: [source bytes, profile selection]
  outputs: [ParsedDocument, CompilationProfile]
  invariants:
    - INV-207-4: BlockV1 profile → identical output to v1
    - SourceParser is object-safe
  postconditions:
    - All v1 golden tests pass with BlockV1 profile
    - Profile field sets correct per spec v2
  failure_modes:
    - BlockV1 output differs from v1 → regression
```

#### Tasks

**TASK-207-2-1: Define SourceParser trait**
- Artifacts: `src/compiler/frontend.rs`

**TASK-207-2-2: Refactor Markdown frontend as SourceParser impl**
- Description: Move `lexical.rs` + `syntactic.rs` into `src/compiler/frontends/markdown.rs` implementing `SourceParser`
- Definition of done: All v1 tests still pass after refactor
- Artifacts: `src/compiler/frontends/markdown.rs`

**TASK-207-2-3: Implement CompilationProfile system**
- Artifacts: `src/compiler/profile.rs`

**TASK-207-2-4: Add --profile CLI flag**
- Artifacts: Updated `src/cli.rs`

**TASK-207-2-5: Write profile system tests**
- Description: Test each profile produces correct field sets; test BlockV1 matches v1 golden output
- Artifacts: `tests/profile_tests.rs`

---

### FEAT-207-3: Language Detection and New Relation Emitters

**Description:** Replace hardcoded `"und"` with real BCP-47 language detection using
a lightweight detection library. Also implement the 5 new relation type emitters from spec v2.

**Inputs:**
- `document.language` field in ZMD v2
- 5 new relation types: supports, refers_to, contradicts, elaborates, cites
- Spec v2 §5.4 (relation taxonomy)

**Outputs:**
- `src/compiler/lang.rs` — language detection
- Updated `src/compiler/logical.rs` — new relation emitters
- Updated `src/compiler/semantic.rs` — semantic-link relations for refers_to, cites

**Acceptance Criteria:**
- [ ] Language detection accuracy ≥ 0.95 on labeled test corpus (English, Portuguese, French, Spanish, German, mixed)
- [ ] `document.language` set correctly in compiled artifact
- [ ] Feature-gated: `--features lang-detect` (off by default for minimal binary)
- [ ] Without feature: language = "und" (v1 behavior preserved)
- [ ] All 5 new relation types emitted correctly for appropriate syntactic patterns
- [ ] New relation emitters only active under `BlockV2` or `SentenceV1` profiles

**Feature Contract:**
```yaml
contract:
  id: CONTRACT-FEAT-207-3
  inputs: [source bytes for language detection, parsed document for relation emission]
  outputs: [BCP-47 language tag, new relations in artifact]
  invariants:
    - Without lang-detect feature, language = "und"
    - New relations not emitted under BlockV1 profile
  postconditions:
    - Language detection accuracy ≥ 0.95
    - New relation types valid per schema
  failure_modes:
    - Language detection wrong language → log warning, default to "und"
```

#### Tasks

**TASK-207-3-1: Add language detection with whatlang/lingua**
- Description: Evaluate `whatlang` and `lingua` crates; pick best accuracy/size tradeoff; implement `detect_language(text: &str) -> BCP47Tag`
- Artifacts: `src/compiler/lang.rs`

**TASK-207-3-2: Write language detection test corpus**
- Description: 50 labeled text samples in 6 languages
- Artifacts: `tests/fixtures/language_detection/`

**TASK-207-3-3: Implement supports, elaborates, contradicts relation emitters**
- Description: Logical stage emitters for `logical` category relations based on syntactic patterns (blockquote + paragraph = elaborates; etc.)
- Artifacts: Updated `src/compiler/logical.rs`

**TASK-207-3-4: Implement refers_to, cites relation emitters**
- Description: Semantic stage emitters for `semantic-link` category relations (footnote refs → cites; link tokens → refers_to)
- Artifacts: Updated `src/compiler/semantic.rs`

**TASK-207-3-5: Write new relation emitter tests**
- Description: Tests for each new relation type; verify category and stage; verify v1 output unchanged
- Artifacts: `tests/compile_v2.rs` extension
