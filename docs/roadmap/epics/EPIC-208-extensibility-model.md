---
id: EPIC-208
title: Extensibility Model
version: 2.0.0-alpha
status: implementado
roadmap: ROADMAP-STF-SIR-V2
priority: medium
created: 2026-04-12
target: 2026-11-01
depends_on:
  - EPIC-207
blocks:
  - EPIC-206
---

# EPIC-208 — Extensibility Model

## Description

Define and implement the formal extensibility model for STF-SIR v2 — the mechanism by
which third-party code can enrich, query, and transform artifacts without modifying or
forking the core compiler.

The extensibility model has three layers:

1. **Plugin interface** — a Rust trait-based plugin system using `libloading` or static
   registration, allowing external enrichers, relation emitters, and query operators
2. **Namespace registry** — a controlled namespace system for `extensions.*` fields,
   preventing collisions between plugins
3. **External enricher interface** — a language-agnostic protocol (JSON-over-stdin/stdout)
   for enrichers written in any language (Python, TypeScript, etc.)

The extensibility model enforces the same monotonicity, determinism, and provenance guarantees
as the internal compiler. No plugin can bypass these invariants.

## Scope

- **In scope:** Plugin trait API, namespace registry, external enricher protocol, extension validation rules, plugin SDK documentation
- **Out of scope:** Plugin marketplace, plugin auto-discovery (must be explicit registration), security sandboxing of external plugins beyond I/O protocol

## Deliverables

| # | Artifact | Path |
|---|---|---|
| D-208-1 | Plugin trait API | `src/plugin/mod.rs` |
| D-208-2 | Namespace registry | `src/plugin/namespace.rs` |
| D-208-3 | External enricher protocol spec | `spec/external-enricher-protocol-v1.md` |
| D-208-4 | External enricher protocol impl | `src/plugin/external.rs` |
| D-208-5 | Plugin SDK guide | `docs/plugin-sdk-guide.md` |
| D-208-6 | Reference external enricher (Python) | `examples/plugins/concept-extractor-py/` |

## Success Criteria

- [x] Plugin trait is object-safe and Send + Sync
- [x] Namespace registry prevents `extensions.stf-sir.*` from being claimed by external plugins
- [x] External enricher protocol is language-agnostic (JSON-over-stdin/stdout, no Rust FFI required)
- [x] Reference Python enricher passes round-trip test via protocol
- [x] Extension namespace collision detected and rejected at registration time
- [x] Plugin SDK guide enables a developer to write a new enricher in < 30 minutes

## Risks

| ID | Risk | Mitigation |
|---|---|---|
| R-208-1 | External enricher protocol creates shell injection surface | Protocol uses structured JSON only; no shell execution; arguments validated |
| R-208-2 | libloading for dynamic plugins is platform-fragile | Default to static registration; dynamic loading is opt-in feature |
| R-208-3 | Namespace squatting / namespace collision is hard to detect post-facto | Registry is checked at startup; duplicate namespace → hard error |
| R-208-4 | Python reference enricher falls out of sync with protocol | Integration test runs Python enricher in CI |

---

## EPIC CONTRACT

```yaml
contract:
  id: CONTRACT-EPIC-208
  version: 1.0.0

  inputs:
    - id: I-208-1
      description: Enricher trait (EPIC-207 output)
      required: true
    - id: I-208-2
      description: ZMD v2 extensions field schema (EPIC-202)
      required: true
    - id: I-208-3
      description: Query engine public API (EPIC-203)
      required: true

  outputs:
    - id: O-208-1
      artifact: src/plugin/ module
    - id: O-208-2
      artifact: spec/external-enricher-protocol-v1.md
    - id: O-208-3
      artifact: docs/plugin-sdk-guide.md
    - id: O-208-4
      artifact: examples/plugins/concept-extractor-py/

  invariants:
    - INV-208-1: |
        Plugin isolation: a plugin MUST NOT access or modify any artifact field
        outside its declared namespace. Violations cause hard errors, not warnings.
    - INV-208-2: |
        Monotonicity inheritance: all plugins inherit the Enricher monotonicity
        invariant (INV-207-2). A plugin that violates monotonicity MUST be rejected
        at registration or at test time.
    - INV-208-3: |
        Namespace integrity: the `stf-sir` namespace is reserved for the core
        compiler. External plugins MUST use namespaces of the form `<org>.<plugin>`.
    - INV-208-4: |
        External enricher communication uses structured JSON only.
        No shell expansion, no eval, no arbitrary command execution.
    - INV-208-5: |
        Plugin registration is deterministic and reproducible: same set of plugins
        registered in same order produces same pipeline behavior.

  preconditions:
    - PRE-208-1: EPIC-207 closed (Enricher trait operational)
    - PRE-208-2: EPIC-203 closed (query engine provides stable public API)
    - PRE-208-3: Namespace scheme defined in spec v2

  postconditions:
    - POST-208-1: Plugin trait tests pass
    - POST-208-2: Namespace collision detection test passes
    - POST-208-3: Python reference enricher passes round-trip CI test
    - POST-208-4: Plugin SDK guide reviewed by a developer external to the project

  validation:
    automated:
      - script: cargo test plugin
        description: Plugin unit tests including namespace collision
      - script: cargo test plugin_external
        description: External enricher protocol round-trip test with Python reference
      - script: cargo test plugin_isolation
        description: Plugin isolation: plugin cannot write to non-declared namespace
      - script: cargo test plugin_monotonicity
        description: 256-case property test of plugin monotonicity
    manual:
      - review: Plugin SDK guide tested by external developer (time-to-first-enricher < 30 min)

  metrics:
    - metric: plugin_isolation_violations
      target: 0
    - metric: monotonicity_violations_in_plugins
      target: 0
    - metric: namespace_collision_detection_rate
      target: 100%
    - metric: external_protocol_roundtrip_success
      target: 100%

  failure_modes:
    - FAIL-208-1:
        condition: INV-208-1 violated (plugin writes outside namespace)
        action: Hard error; reject plugin at registration
    - FAIL-208-2:
        condition: INV-208-4 violated (shell execution in protocol)
        action: Architecture violation; rework protocol
    - FAIL-208-3:
        condition: Python reference enricher fails CI test
        action: Fix protocol or enricher before closing EPIC
```

---

## Features

### FEAT-208-1: Plugin Trait API

**Description:** Define the plugin registration system as a Rust-native trait hierarchy
built on top of the `Enricher` trait (EPIC-207). Plugins register a namespace, declare
their field access pattern, and implement one or more of: `Enricher`, `RelationEmitter`,
`QueryOperator`.

**Inputs:**
- `Enricher` trait (EPIC-207)
- Query engine operator API (EPIC-203)
- Namespace registry design

**Outputs:**
- `src/plugin/mod.rs` — `Plugin` trait, `PluginRegistry`, `PluginManifest`
- `src/plugin/namespace.rs` — `NamespaceRegistry`

**Acceptance Criteria:**
- [ ] `Plugin` trait: `fn manifest(&self) -> PluginManifest; fn as_enricher(&self) -> Option<&dyn Enricher>; fn as_relation_emitter(&self) -> Option<&dyn RelationEmitter>`
- [ ] `PluginManifest`: name, version, namespace, declared_fields, author, description
- [ ] `PluginRegistry` maintains ordered Vec of plugins; no HashMap non-determinism
- [ ] Registering two plugins with the same namespace returns `Err(NamespaceConflict)`
- [ ] `stf-sir` namespace is hardcoded as reserved; any attempt to claim it returns error
- [ ] Plugin isolation enforced: plugin may only write to fields within its declared namespace

**Metrics:** plugin_isolation_violations = 0, namespace_collision_detection_rate = 100%

**Feature Contract:**
```yaml
contract:
  id: CONTRACT-FEAT-208-1
  inputs: [Enricher trait, namespace scheme]
  outputs: [Plugin trait, PluginRegistry, NamespaceRegistry]
  invariants:
    - INV-208-1 (plugin isolation)
    - INV-208-3 (namespace integrity)
    - INV-208-5 (deterministic registration)
  postconditions:
    - stf-sir namespace cannot be claimed
    - Duplicate namespace → Err
    - Registration order preserved
  failure_modes:
    - Namespace claimed silently → namespace squatting → critical
```

#### Tasks

**TASK-208-1-1: Design plugin manifest and namespace scheme**
- Description: Define PluginManifest fields; document namespace format `<org>.<plugin>`; define reserved namespaces
- Artifacts: `spec/decisions/ADR-005-plugin-namespace.md`

**TASK-208-1-2: Implement Plugin trait and PluginManifest**
- Artifacts: `src/plugin/mod.rs`

**TASK-208-1-3: Implement NamespaceRegistry**
- Description: Registry with reservation check, conflict detection, and reserved namespace list
- Artifacts: `src/plugin/namespace.rs`

**TASK-208-1-4: Implement PluginRegistry**
- Description: Ordered Vec of plugins with registration API and dispatch to enrichers/emitters
- Artifacts: `src/plugin/registry.rs`

**TASK-208-1-5: Write plugin isolation tests**
- Description: Test that a plugin writing to an undeclared field is rejected; test namespace collision; test reserved namespace rejection
- Artifacts: `tests/plugin_tests.rs`

---

### FEAT-208-2: External Enricher Protocol

**Description:** Define and implement a language-agnostic protocol for external enrichers
that communicate with the STF-SIR compiler via JSON-over-stdin/stdout. This allows enrichers
written in Python, TypeScript, Julia, etc. without Rust FFI.

**Inputs:**
- ZToken v2 JSON representation
- Protocol design requirements (structured JSON, no shell execution)

**Outputs:**
- `spec/external-enricher-protocol-v1.md` — protocol spec
- `src/plugin/external.rs` — host-side protocol adapter
- `examples/plugins/concept-extractor-py/` — reference Python enricher

**Acceptance Criteria:**
- [ ] Protocol: host writes ZToken JSON to enricher stdin; enricher writes enriched ZToken JSON to stdout; uses line-delimited JSON (NDJSON)
- [ ] Protocol version negotiation: first message is `{"protocol": "stf-sir-enricher", "version": "1"}`
- [ ] Enricher outputs only declared namespace fields (checked by host)
- [ ] Enricher timeout: configurable, default 5s per token; timeout returns original token unmodified
- [ ] Reference Python enricher: extracts top-3 keywords from gloss as concepts; writes to `concepts` field
- [ ] CI test runs Python enricher as subprocess; validates round-trip

**Metrics:** external_protocol_roundtrip_success = 100%

**Feature Contract:**
```yaml
contract:
  id: CONTRACT-FEAT-208-2
  inputs: [ZToken JSON, subprocess command]
  outputs: [enriched ZToken JSON]
  invariants:
    - INV-208-4 (JSON only, no shell)
    - INV-208-2 (monotonicity: host validates before accepting)
    - Timeout does not corrupt artifact
  postconditions:
    - Python enricher passes CI test
    - Protocol spec reviewed externally
  failure_modes:
    - Enricher timeout → return original token, log warning
    - Non-JSON output → hard error, reject enricher output
```

#### Tasks

**TASK-208-2-1: Write external enricher protocol spec**
- Description: Full protocol document: handshake, message format, error codes, timeout behavior
- Artifacts: `spec/external-enricher-protocol-v1.md`

**TASK-208-2-2: Implement host-side protocol adapter**
- Description: Rust code to spawn subprocess, write tokens as NDJSON, read enriched tokens, validate namespace, enforce timeout
- Artifacts: `src/plugin/external.rs`

**TASK-208-2-3: Write reference Python concept extractor**
- Description: Python script implementing the protocol; extracts top-3 keywords; handles namespace declarations
- Artifacts: `examples/plugins/concept-extractor-py/main.py`, `examples/plugins/concept-extractor-py/manifest.json`

**TASK-208-2-4: Write CI integration test for Python enricher**
- Description: Rust test that spawns Python enricher subprocess, compiles a fixture, verifies concepts populated
- Definition of done: CI test passes on ubuntu-latest with Python 3.11+
- Artifacts: `tests/plugin_external_tests.rs`

---

### FEAT-208-3: Plugin SDK Guide and Conformance

**Description:** Write the plugin SDK developer guide and define the plugin conformance
test suite that verifies any plugin for correctness, monotonicity, and namespace compliance.

**Inputs:**
- Plugin trait API (FEAT-208-1)
- External protocol spec (FEAT-208-2)

**Outputs:**
- `docs/plugin-sdk-guide.md`
- `tests/plugin_conformance/` — plugin conformance test suite

**Acceptance Criteria:**
- [ ] Guide covers: creating a Rust plugin, creating a Python/external plugin, registering, testing
- [ ] Guide includes end-to-end worked example for each plugin type
- [ ] Conformance suite tests: namespace compliance, monotonicity, isolation, error handling
- [ ] Conformance suite is runnable against any plugin via `cargo test --features plugin-conformance -- --plugin <name>`
- [ ] Guide reviewed by an external developer; time-to-first-enricher < 30 minutes

**Feature Contract:**
```yaml
contract:
  id: CONTRACT-FEAT-208-3
  inputs: [plugin API, protocol spec]
  outputs: [docs/plugin-sdk-guide.md, plugin conformance suite]
  invariants:
    - Guide does not reference internal APIs (only public surface)
  postconditions:
    - External developer review completed
    - Conformance suite passes for all reference plugins
  failure_modes:
    - Guide references private API → doc debt, fix before release
```

#### Tasks

**TASK-208-3-1: Write plugin SDK guide**
- Artifacts: `docs/plugin-sdk-guide.md`

**TASK-208-3-2: Write plugin conformance test suite**
- Description: Tests for namespace compliance, monotonicity, isolation, and protocol conformance
- Artifacts: `tests/plugin_conformance/`

**TASK-208-3-3: Validate guide with external developer**
- Description: Find one developer not on the project; have them implement the worked example; record time and blockers
- Definition of done: Time ≤ 30 minutes; all blockers resolved in guide
- Artifacts: `spec/decisions/ADR-006-plugin-sdk-usability.md` (review notes)
