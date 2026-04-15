---
id: EPIC-202
title: ZMD Canonical Format v2
version: 2.0.0-alpha
status: implementado
roadmap: ROADMAP-STF-SIR-V2
priority: critical
created: 2026-04-12
target: 2026-07-01
depends_on:
  - EPIC-201
blocks:
  - EPIC-207
  - EPIC-203
---

# EPIC-202 — ZMD Canonical Format v2

## Description

Produce the v2 canonical artifact format (`.zmd` version 2), including:

1. JSON Schema Draft 2020-12 for ZMD v2 (`schemas/zmd-v2.schema.json`)
2. Backward-compatible version gating (`version: 1` artifacts remain valid)
3. New dimension fields (C, P, Δ, Ω) as optional extensions
4. Embedding anchor fields (`Σ.embedding_ref`) for RAG integration
5. Serialization stability guarantee (byte-for-byte identical output given identical input + config)
6. Graph export sub-format (`sirgraph-v1` export section, optional)
7. Language detection field promotion (`document.language` becomes a required field in v2 profile)

## Scope

- **In scope:** JSON Schema definition, serializer contract, backward-compat layer, schema embedding in binary
- **Out of scope:** Actual serializer implementation (EPIC-207), embedding computation (EPIC-206)

## Deliverables

| # | Artifact | Path |
|---|---|---|
| D-202-1 | ZMD v2 JSON Schema | `schemas/zmd-v2.schema.json` |
| D-202-2 | Schema changelog | `schemas/CHANGELOG.md` |
| D-202-3 | Schema migration validator script | `scripts/validate-migration-v1-v2.sh` |
| D-202-4 | Serialization stability spec | (embedded in spec v2 §7) |
| D-202-5 | Graph export sub-format spec | `spec/sirgraph-export-v1.md` |

## Success Criteria

- [x] `schemas/zmd-v2.schema.json` passes JSON Schema meta-schema validation
- [x] All 30 v1 golden `.zmd` fixtures pass `zmd-v2.schema.json` (backward-compat gate)
- [x] All new optional fields have `default` values or `nullable: true`
- [x] Serialization determinism: 1000 re-compilations of same input produce bit-identical output
- [x] Schema embedded in binary at build time (same mechanism as v1)

## Risks

| ID | Risk | Mitigation |
|---|---|---|
| R-202-1 | Embedding anchor field creates dependency on external ID system | Field is `Optional<String>` only; no external validation required |
| R-202-2 | C/P/Δ/Ω fields inflate artifact size significantly | Profile-based omission: block profile omits by default unless enricher writes them |
| R-202-3 | Version gate logic complicates the reader | Strict mode: reader rejects unknown version; permissive mode: reader ignores unknown fields |

---

## EPIC CONTRACT

```yaml
contract:
  id: CONTRACT-EPIC-202
  version: 1.0.0

  inputs:
    - id: I-202-1
      description: STF-SIR Spec v2 (EPIC-201 output)
      required: true
    - id: I-202-2
      description: ZMD v1 schema (schemas/zmd-v1.schema.json)
      required: true
    - id: I-202-3
      description: All v1 golden fixtures (tests/golden/*.zmd)
      required: true

  outputs:
    - id: O-202-1
      artifact: schemas/zmd-v2.schema.json
      constraint: Valid JSON Schema Draft 2020-12
    - id: O-202-2
      artifact: schemas/CHANGELOG.md
    - id: O-202-3
      artifact: scripts/validate-migration-v1-v2.sh
    - id: O-202-4
      artifact: spec/sirgraph-export-v1.md

  invariants:
    - INV-202-1: |
        Every v1 `.zmd` artifact (version: 1) MUST validate against zmd-v2.schema.json
        under the backward-compat profile.
    - INV-202-2: |
        The schema MUST enforce format: "stf-sir.zmd" as a const.
    - INV-202-3: |
        Serialization output is deterministic: SHA-256 of output bytes is stable across
        re-compilations of the same input with the same config_hash.
    - INV-202-4: |
        All new fields in v2 are additive (no v1 field removed or renamed).
    - INV-202-5: |
        The `version` field MUST be an integer enum [1, 2]; future versions require a
        new schema file.

  preconditions:
    - PRE-202-1: EPIC-201 is closed (spec v2 is published)
    - PRE-202-2: All v1 golden tests pass on current HEAD
    - PRE-202-3: JSON Schema tooling (jsonschema crate) supports Draft 2020-12

  postconditions:
    - POST-202-1: Schema file exists and is valid
    - POST-202-2: All v1 golden fixtures pass schema validation
    - POST-202-3: Migration script exits 0 on v1 input, exits 1 on invalid input
    - POST-202-4: CI gate validates schema at build time

  validation:
    automated:
      - script: scripts/validate-schema.sh schemas/zmd-v2.schema.json
        description: Meta-validates schema against JSON Schema Draft 2020-12 meta-schema
      - script: scripts/validate-migration-v1-v2.sh tests/golden/
        description: Runs all v1 golden fixtures through v2 schema validator
      - script: scripts/check-determinism.sh 1000
        description: Compiles sample.md 1000 times, checks SHA-256 stability
    manual:
      - review: Schema reviewer must confirm all v1 fields present with same types

  metrics:
    - metric: backward_compat_rate
      formula: (v1_fixtures_valid_against_v2 / total_v1_fixtures) * 100
      target: 100%
    - metric: schema_determinism_rate
      formula: (stable_compilations / 1000) * 100
      target: 100%
    - metric: new_field_coverage
      formula: (optional_fields_with_defaults / total_new_fields) * 100
      target: 100%

  failure_modes:
    - FAIL-202-1:
        condition: INV-202-1 violated (v1 fixture fails v2 schema)
        action: Block EPIC-207; report which fixture and which rule
    - FAIL-202-2:
        condition: INV-202-3 violated (non-deterministic output)
        action: Block EPIC-207; open defect with repro case
    - FAIL-202-3:
        condition: Schema meta-validation fails
        action: Block all downstream EPICs; fix schema before proceeding
```

---

## Features

### FEAT-202-1: ZMD v2 JSON Schema

**Description:** Author the complete `schemas/zmd-v2.schema.json` that extends v1 schema
with all new fields, maintains backward compatibility, and includes all new dimension schemas.

**Inputs:**
- `schemas/zmd-v1.schema.json`
- ZToken v2 field tables (EPIC-201 D-201-1)

**Outputs:**
- `schemas/zmd-v2.schema.json`
- Schema unit test suite

**Acceptance Criteria:**
- [ ] Schema passes JSON Schema Draft 2020-12 meta-validation
- [ ] All 6 v1 golden `.zmd` files validate against v2 schema
- [ ] New dimension objects (C, P, Δ, Ω) are defined as `$defs` with proper `$ref`
- [ ] `Σ.embedding_ref` field defined as `nullable: true, type: string, format: uri-reference`
- [ ] `document.language` is `required: true` under `profile: block-v2`
- [ ] `extensions` at all levels remain `additionalProperties: true`

**Metrics:** backward_compat_rate = 100%, schema meta-validation = pass

**Feature Contract:**
```yaml
contract:
  id: CONTRACT-FEAT-202-1
  inputs: [zmd-v1.schema.json, ZToken v2 field tables]
  outputs: [schemas/zmd-v2.schema.json]
  invariants:
    - version enum: [1, 2] only
    - All v1 required fields remain required
    - New fields: optional or with defaults
  postconditions:
    - Schema file is valid JSON
    - Meta-validation passes
    - All 6 golden v1 fixtures validate
  failure_modes:
    - Missing v1 field → backward-compat failure, block EPIC-207
```

#### Tasks

**TASK-202-1-1: Port v1 schema to v2 base**
- Description: Copy `zmd-v1.schema.json`, update `$id` and `version` enum to `[1, 2]`, add `$schema: https://json-schema.org/draft/2020-12/schema`
- Definition of done: Modified file passes meta-validation, all v1 golden fixtures still validate
- Testability: `scripts/validate-schema.sh schemas/zmd-v2.schema.json`
- Artifacts: `schemas/zmd-v2.schema.json` (initial version)
- Contract: Input = v1 schema, Output = v2 base schema, Invariant = all v1 fixtures validate

**TASK-202-1-2: Add C, P, Δ, Ω dimension schemas**
- Description: Define `$defs` for each new dimension with all fields typed and constrained; add to ztoken object as optional properties
- Definition of done: Four `$defs` added, each with required/optional fields per profile spec
- Testability: Property tests generate ZTokens with C/P/Δ/Ω fields; schema validation passes
- Artifacts: Schema `$defs` section extension
- Contract: Input = dimension field tables, Output = 4 schema defs, Invariant = all fields have types and constraints

**TASK-202-1-3: Add embedding anchor and language fields**
- Description: Add `Σ.embedding_ref` (nullable string), `document.language` (BCP-47 pattern string)
- Definition of done: Both fields defined in schema with correct types and format constraints
- Testability: Valid fixture with embedding_ref passes; invalid BCP-47 fails schema
- Artifacts: Schema update
- Contract: embedding_ref is optional; language is required under v2 profile

**TASK-202-1-4: Write schema unit test suite**
- Description: 20 valid JSON fixtures + 10 invalid fixtures specifically testing schema constraints
- Definition of done: All tests in `tests/conformance/v2/` pass
- Testability: `cargo test conformance_v2`
- Artifacts: `tests/conformance/v2/` directory with fixtures

---

### FEAT-202-2: Serialization Stability Guarantee

**Description:** Formally specify and verify the byte-for-byte serialization stability invariant
for ZMD v2 output, extending the v1 golden gate to cover all new fields.

**Inputs:**
- v1 serializer contract (stable struct field order via `serde_yaml_ng`)
- New dimension fields from FEAT-202-1

**Outputs:**
- Serialization stability specification (spec §7)
- Extended golden test suite (v2 golden corpus)
- Determinism script (`scripts/check-determinism.sh`)

**Acceptance Criteria:**
- [ ] Spec §7 formally defines: identical (source, config_hash) → identical output bytes
- [ ] All new `BTreeMap` fields are serialized in lexicographic key order
- [ ] `Vec` fields are serialized in deterministic insertion order (not hash order)
- [ ] `check-determinism.sh` runs 1000 compilations and compares SHA-256 hashes
- [ ] v2 golden corpus of ≥ 12 fixtures created

**Metrics:** determinism_rate = 100% over 1000 trials

**Feature Contract:**
```yaml
contract:
  id: CONTRACT-FEAT-202-2
  inputs: [v1 serializer, new field definitions]
  outputs: [spec §7, golden corpus v2, determinism script]
  invariants:
    - SHA-256(compile(src, cfg)) is a pure function
    - config_hash changes iff compiler config changes
  postconditions:
    - Script exits 0 after 1000 runs
    - Golden corpus committed to tests/golden/v2/
  failure_modes:
    - Non-deterministic output → open critical defect, block all downstream
```

#### Tasks

**TASK-202-2-1: Write serialization stability spec section**
- Description: Document the ordering rules for all serialized fields including new dimensions
- Artifacts: spec §7 section

**TASK-202-2-2: Create v2 golden corpus (12+ fixtures)**
- Description: Create 12+ Markdown input → ZMD v2 output golden pairs covering all new features
- Definition of done: All pairs committed to `tests/golden/v2/`, CI golden gate extended
- Testability: `cargo test golden_v2`
- Artifacts: `tests/golden/v2/` fixtures

**TASK-202-2-3: Write and validate determinism script**
- Description: Shell script that compiles a fixture N times and compares all SHA-256 outputs
- Definition of done: Script exits 0 when all N outputs are identical; exits 1 otherwise
- Artifacts: `scripts/check-determinism.sh`

---

### FEAT-202-3: Graph Export Sub-Format

**Description:** Define the optional `sirgraph` export section in ZMD v2 that serializes
the SirGraph in a standard graph interchange format, enabling external graph tools to consume
STF-SIR artifacts without a custom reader.

**Inputs:**
- SirGraph v1 in-memory structure (`src/sir/graph.rs`)
- Candidate formats: JSON-LD, GraphML, adjacency-list JSON

**Outputs:**
- Graph export format spec (`spec/sirgraph-export-v1.md`)
- Schema for export section in `zmd-v2.schema.json`

**Acceptance Criteria:**
- [ ] Format is a subset of JSON-LD or a named adjacency-list JSON format
- [ ] Export section is optional (entire section may be absent)
- [ ] Format includes node type, relation type, source, target, and all ztoken IDs
- [ ] Round-trip property: exported graph + original ztokens can reconstruct the full ZMD

**Metrics:** round-trip fidelity = 100% on all golden fixtures

**Feature Contract:**
```yaml
contract:
  id: CONTRACT-FEAT-202-3
  inputs: [SirGraph structure, graph format candidates]
  outputs: [sirgraph-export-v1.md, schema addition]
  invariants:
    - Export is optional; its absence does not affect ZMD v2 validity
    - Exported graph is isomorphic to in-memory SirGraph
  postconditions:
    - Spec doc present and reviewed
    - Schema addition does not break existing fixtures
  failure_modes:
    - Non-isomorphic export → spec defect, block FEAT-203 graph loading
```

#### Tasks

**TASK-202-3-1: Evaluate and select graph interchange format**
- Description: Compare JSON-LD, GraphML, adjacency-list JSON for round-trip fidelity and ecosystem compatibility
- Definition of done: Decision document with rationale committed to `spec/decisions/`
- Artifacts: `spec/decisions/ADR-001-graph-format.md`

**TASK-202-3-2: Write graph export sub-format spec**
- Description: Full specification of the `sirgraph` section including schema, semantics, and example
- Artifacts: `spec/sirgraph-export-v1.md`

**TASK-202-3-3: Add graph export to ZMD v2 schema**
- Description: Add optional `sirgraph` property to `zmd-v2.schema.json`
- Definition of done: Schema addition passes meta-validation; all existing fixtures still validate
- Artifacts: Schema update
