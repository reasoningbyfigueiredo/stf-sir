---
id: SPEC-STF-CORE-SEMANTICS
version: 1.0.0-draft
status: draft
created: 2026-04-14
updated: 2026-04-14
owner: Rogerio Figueiredo
system: STF-SIR
type: semantic-core-spec
language: en
normative: true
tags:
  - semantics
  - coherence
  - grounding
  - theory
  - artifact-bridge
  - retention
  - auditability
---

# SPEC-STF-CORE-SEMANTICS

## 1. Purpose

This specification defines the semantic core of STF-SIR.

Its purpose is to establish the normative model for:

- semantic compilation at the artifact layer;
- semantic reasoning at the theory layer;
- the canonical bridge from `Artifact` to `Theory`;
- coherence as a structural invariant;
- grounding as a verifiable relation;
- retention as a preservation metric;
- error classification for contradiction, anomaly, and hallucination.

This specification is normative for core semantics and auditability behavior.

---

## 2. Scope

This specification governs the following internal abstractions:

- `Artifact`
- `ZToken`
- `Statement`
- `Theory`
- `Formula`
- `SIR`
- `SirGraph`
- coherence evaluation
- grounding evaluation
- retention evaluation
- semantic bridge behavior

This specification does **not** define:

- UI behavior
- transport protocols
- storage backends
- external orchestration workflows
- speculative philosophical interpretation

---

## 3. Design Principles

### 3.1 Structural Invariant Principle

Coherence SHALL be treated as a structural invariant over semantic representations.

### 3.2 Bridge Principle

Reasoning SHALL operate over theory-level objects derived from artifact-level structures through a canonical and auditable mapping.

### 3.3 Grounding Principle

No semantic admission decision SHALL be considered fully valid without a verifiable grounding relation.

### 3.4 Auditability Principle

All semantic admission, derivation, and classification steps SHALL be reconstructable from explicit evidence.

### 3.5 Preservation Principle

Transformations SHALL preserve semantic identity, provenance, and structural coherence within measurable bounds.

---

## 4. Core Semantic Model

### 4.1 Artifact

An `Artifact` is the canonical compiled semantic object.

It SHALL preserve:

- source information;
- compiler metadata;
- ordered `ztokens`;
- typed relations;
- diagnostics;
- extensions relevant to semantic auditability.

### 4.2 ZToken

A `ZToken` is the canonical semantic unit at artifact level.

A `ZToken` SHOULD preserve, when available:

- stable identifier;
- normalized text;
- node type;
- spans or coordinates;
- relation identifiers;
- source identity;
- anchors;
- semantic dimensions required by the compiler baseline.

### 4.3 Statement

A `Statement` is the canonical theory-level reasoning unit.

A `Statement` SHALL contain:

- stable identifier;
- normalized or canonical text;
- domain;
- provenance;
- structural metadata.

A `Statement` SHOULD additionally contain:

- optional `Formula`;
- relation references;
- bridge evidence back to originating `ZToken` or SIR node.

### 4.4 Theory

A `Theory` is a structured set of `Statement`s over which the system evaluates:

- logical coherence;
- computational coherence;
- operational coherence;
- grounding state;
- admissibility;
- executability;
- error class.

### 4.5 Formula

A `Formula` is the minimal logical structure used to reduce dependence on superficial text matching.

The baseline formula system SHALL support at least:

- `Atom`
- `Not`
- `Implies`

Future extensions MAY include conjunction, disjunction, quantification, and typed predicates.

### 4.6 SIR

`SIR` is the semantic intermediate representation of the compiled document.

`SirGraph` SHALL be treated as a graph-oriented semantic projection capable of supporting:

- structural correspondence checks;
- grounding verification;
- relation tracing;
- bridge auditability.

---

## 5. Canonical Bridge: Artifact -> Theory

### 5.1 Normative Mapping

The system SHALL expose a canonical bridge:

\[
\beta : Artifact \rightarrow Theory
\]

This bridge SHALL be implemented in the core library, not only in CLI or orchestration code.

### 5.2 Mapping Unit

For each `ZToken`, the bridge SHALL produce one corresponding `Statement`, unless explicitly excluded by a documented semantic rule.

### 5.3 Mapping Preservation Requirements

For each mapped token `z`, `beta(z)` SHALL preserve:

- stable identity;
- normalized text or canonical semantic text;
- node/domain classification;
- spans or equivalent source coordinates when available;
- source identity;
- anchors;
- relation identifiers relevant for semantic traceability.

### 5.4 Metadata Requirements

The bridge SHALL preserve or materialize, when available:

- `zid`
- `node_type`
- `span_start`
- `span_end`
- relation ids
- source hash or source identity
- grounding anchors

### 5.5 Formula Attachment

If a `ZToken` text or semantic payload can be interpreted as a supported logical formula, the bridge SHOULD attach `Formula` directly to the resulting `Statement`.

The engine SHOULD prefer attached formulas over reparsing free text.

### 5.6 Identity Preservation

The bridge SHALL preserve distinguishability.

Distinct source tokens SHALL NOT collapse silently into the same statement identity.

### 5.7 Loss Signaling

If the bridge cannot preserve relevant semantic fields, it SHALL emit:

- diagnostics;
- warnings;
- or explicit downgrade markers.

Silent semantic loss is forbidden.

---

## 6. Provenance and Grounding

### 6.1 Provenance Structure

Every `Statement` SHALL carry `Provenance`.

`Provenance` SHALL support, at minimum:

- `source_ids`
- `anchors`
- `generated_by`
- `grounded` status or equivalent grounding evidence marker

### 6.2 Grounding Definition

A `Statement` is grounded when there exists verifiable evidence that it is anchored to one or more internal source structures.

Valid grounding evidence MAY include:

- source identifiers;
- `ZToken` identifiers;
- SIR node identifiers;
- anchors;
- source spans;
- derivation traces from grounded premises.

### 6.3 Grounding Levels

The implementation SHOULD support at least these grounding levels:

- grounded
- ungrounded

The implementation MAY later extend this to:

- weakly grounded
- transitively grounded
- structurally grounded
- provenance-only grounded

### 6.4 SIR-Based Grounding

If `SirGraph` is available, the implementation SHOULD support structural grounding by verifying statement correspondence against SIR nodes or bridge evidence.

### 6.5 Derived Grounding

A derived statement MAY be considered grounded only if:

- its premises are grounded; and
- its derivation rule is recorded; and
- the derivation trace is reconstructable.

---

## 7. Coherence Model

The coherence of a theory SHALL be represented as:

\[
\mathrm{Coh}(T) = (C_l, C_c, C_o)
\]

### 7.1 Logical Coherence

`C_l` measures internal non-contradiction.

A theory is logically coherent when contradiction cannot be derived from the relevant statement set.

Baseline implementation MAY approximate this through minimal supported logic.

Preferred evaluation order:

1. formula-based contradiction
2. structured logical checks
3. fallback text heuristics, only during migration

### 7.2 Computational Coherence

`C_c` measures verifiability under bounded resources.

The implementation SHALL support a bounded mode in which a coherence decision is evaluated relative to a step or resource budget.

`C_c` SHALL NOT be hardcoded as permanently unknown if a budgeted evaluation is available.

### 7.3 Operational Coherence

`C_o` measures whether the theory yields non-trivial executable consequences under the active inference engine.

A theory is operationally coherent when it produces one or more admissible derived consequences.

### 7.4 Coherence Classification

The implementation SHOULD support the following interpretation:

- `C_l = 0` -> contradictory
- `C_l = 1, C_c = 0` -> intractable under budget
- `C_l = 1, C_c = 1, C_o = 0` -> sterile
- `C_l = 1, C_c = 1, C_o = 1` -> fully coherent

---

## 8. Information Semantics

### 8.1 Admissible Information

A message or statement is admissible if it:

- does not break logical coherence; and
- is grounded.

### 8.2 Executable Information

A message or statement is executable if, when admitted, it enables non-trivial consequences at the theory layer.

### 8.3 Useful Information

A message or statement is useful if it is both:

- admissible; and
- executable.

### 8.4 Auditable Information

A message or statement is auditable if the system can reconstruct:

- source origin;
- bridge evidence;
- grounding evidence;
- inferential dependencies;
- admission rationale.

---

## 9. Inference Requirements

### 9.1 Inference Engine

The system SHALL support an `InferenceEngine` abstraction.

### 9.2 Baseline Rule

The baseline implementation SHOULD support at least modus ponens over supported formulas.

### 9.3 Inference Preconditions

Inference SHALL NOT proceed from malformed formulas as though they were valid formulas.

Inference SHOULD prefer formula-aware reasoning over text-pattern reasoning.

### 9.4 Determinism

For the same theory state and the same inference engine configuration, derivation results SHOULD be deterministic.

### 9.5 Derivation Trace

Derived statements SHALL carry enough evidence to reconstruct:

- rule id;
- premises;
- derivation source.

---

## 10. Error Taxonomy

### 10.1 Contradiction

A contradiction is a logical inconsistency within the theory.

Detection mechanism:
- formula-based contradiction check
- or lower-level fallback until migration completes

### 10.2 Hallucination

A hallucination is a locally coherent but ungrounded statement.

Hallucination SHALL remain distinct from contradiction.

### 10.3 Anomaly

An anomaly is a statistical or distributional deviation relative to a domain baseline.

Anomaly SHALL remain distinct from both contradiction and hallucination.

### 10.4 Non-Executable

The implementation MAY classify coherent but operationally sterile statements or theory extensions as non-executable.

### 10.5 Error Separation Rule

The implementation SHALL NOT collapse all invalid semantic states into a generic error bucket when a more specific classification is possible.

---

## 11. Retention and Preservation

### 11.1 Retention Role

Retention SHALL be treated as a measurable preservation surface, not as a replacement for coherence.

### 11.2 Required Preservation Dimensions

Retention SHOULD support preservation evaluation for:

- lexical structure;
- semantic structure;
- formula structure;
- grounding evidence;
- SIR relation structure.

### 11.3 Unified Retention Model

The implementation SHOULD converge toward a unified retention vector capable of relating:

- artifact preservation;
- lexical preservation;
- coherence preservation.

### 11.4 Retention Safety Rule

A high retention score SHALL NOT be interpreted as semantic validity if coherence or grounding has collapsed.

---

## 12. Implementation Constraints

### 12.1 Library-Centered Semantics

Core semantic transformations SHALL live in library code.

CLI code SHOULD remain orchestration-thin.

### 12.2 Migration Strategy

During migration from text-based logic to formula-aware logic:

- formula-aware evaluation SHALL take precedence when available;
- string fallback MAY remain temporarily;
- fallback behavior SHALL be test-covered.

### 12.3 Backward Compatibility

The project SHOULD preserve compatibility with existing artifacts and tests where feasible, while preventing semantic regressions.

---

## 13. Required Test Coverage

The implementation SHALL maintain automated coverage for at least:

- formula parsing;
- artifact-to-theory bridge preservation;
- contradiction detection;
- grounding classification;
- operational coherence;
- computational coherence under budget;
- hallucination boundary behavior;
- retention/coherence interaction;
- metamorphic invariants;
- regression contracts for semantic bridge behavior.

---

## 14. Acceptance Criteria

This specification is satisfied when the implementation demonstrates all of the following:

1. `Statement` is the canonical theory-level reasoning unit.
2. `Artifact -> Theory` is implemented as a library-level canonical bridge.
3. bridge output preserves semantic identity and provenance evidence.
4. `Formula` is first-class in semantic evaluation where available.
5. coherence evaluation uses formula-aware logic when possible.
6. grounding is verifiable, not purely heuristic.
7. bounded `C_c` is supported by the engine.
8. inference results are traceable and auditable.
9. error classes remain distinct.
10. retention does not mask semantic collapse.

---

## 15. Non-Goals

This specification does not attempt to:

- solve full first-order theorem proving;
- guarantee truth in an external epistemic sense;
- replace statistical anomaly detection with logic;
- define metaphysical or physical foundations;
- encode all future STF-SIR semantics in a single version.

---

## 16. Future Extensions

Planned directions MAY include:

- richer formula AST
- typed predicates
- SAT/SMT-backed coherence checking
- stronger SIR grounding
- transitive grounding semantics
- proof objects for derivations
- explicit semantic contracts between artifact and theory layers
- unified retention + coherence scoring

---

## 17. References

This specification is aligned with the computational coherence formulation in `coherence-foundations-en` and with the current STF-SIR architecture direction toward a unified semantic bridge, formula-aware coherence, grounding, and auditable reasoning.