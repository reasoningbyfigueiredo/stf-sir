---
id: EPIC-203
title: Semantic Query Engine
version: 2.0.0-alpha
status: implementado
roadmap: ROADMAP-STF-SIR-V2
priority: high
created: 2026-04-12
target: 2026-09-01
depends_on:
  - EPIC-202
  - EPIC-207
blocks:
  - EPIC-204
  - EPIC-205
---

# EPIC-203 — Semantic Query Engine

## Description

Build a typed, deterministic query engine over the STF-SIR semantic graph. The engine exposes
a composable Query DSL, a Rust graph navigation API, and a CLI query interface. All queries
are deterministic: identical graph + identical query → identical result set, always.

This EPIC transforms the five primitive SirGraph methods into a production-grade query layer
capable of: path traversal, subgraph extraction, predicate filtering, dimension projection,
and transitive closure — without introducing a full graph database dependency.

## Scope

- **In scope:** Query DSL grammar, Rust API, CLI subcommand, petgraph integration, query planner, result serialization
- **Out of scope:** Distributed execution, query caching/persistence, SQL-like joins across multiple artifacts (those are EPIC-205/206)

## Deliverables

| # | Artifact | Path |
|---|---|---|
| D-203-1 | Query DSL grammar | `spec/query-dsl-v1.md` |
| D-203-2 | Graph Navigation API | `src/query/` module |
| D-203-3 | CLI query subcommand | `stf-sir query` |
| D-203-4 | Query engine test suite | `tests/query/` |
| D-203-5 | Query result format spec | (embedded in D-203-1 §5) |

## Success Criteria

- [x] Query DSL covers all 5 v1 SirGraph primitives as query operators
- [x] 10 named query patterns (path, ancestors, descendants, by-type, by-category, subgraph, depth-range, span-range, regex-gloss, dimension-filter) are expressible
- [ ] Query p99 latency ≤ 50 ms on 10 000-ztoken artifact (benchmark pending)
- [x] All queries are deterministic (BTreeMap/BTreeSet internally; sorted output)
- [x] Query results are serializable to JSON and YAML (serde derive)
- [ ] CLI `stf-sir query` subcommand documented with `--help` (FEAT-203-3 pending)

## Risks

| ID | Risk | Mitigation |
|---|---|---|
| R-203-1 | DSL scope creep into full graph query language | Lock grammar at v1 MVP; extensions require new FEAT |
| R-203-2 | petgraph introduces breaking API changes | Pin petgraph version in Cargo.toml; feature-gate |
| R-203-3 | Query determinism breaks on HashMap-ordered internals | All internal indexes use BTreeMap; Vec results sorted by ztoken id |
| R-203-4 | DSL parsing is ambiguous | Use PEG parser (pest crate) with formal grammar file |

---

## EPIC CONTRACT

```yaml
contract:
  id: CONTRACT-EPIC-203
  version: 1.0.0

  inputs:
    - id: I-203-1
      description: SirGraph v2 in-memory structure (EPIC-207 output)
      required: true
    - id: I-203-2
      description: ZMD v2 artifact (EPIC-202 output)
      required: true
    - id: I-203-3
      description: Query DSL grammar spec (D-203-1)
      required: true

  outputs:
    - id: O-203-1
      artifact: src/query/ module
      constraint: Zero unsafe blocks; all public types documented
    - id: O-203-2
      artifact: stf-sir query CLI subcommand
    - id: O-203-3
      artifact: spec/query-dsl-v1.md
    - id: O-203-4
      artifact: tests/query/ suite

  invariants:
    - INV-203-1: |
        Query determinism: for all queries Q and artifacts A,
        execute(Q, A) = execute(Q, A) on any call count and any OS.
    - INV-203-2: |
        Query completeness: the engine NEVER silently drops matching nodes.
        A result set for a filter predicate P contains ALL nodes satisfying P.
    - INV-203-3: |
        Query isolation: queries are read-only. No query modifies the artifact
        or any SirGraph index.
    - INV-203-4: |
        Result serialization is deterministic: identical result sets produce
        identical JSON/YAML bytes.
    - INV-203-5: |
        The engine MUST report an error (not panic) for any malformed query string.

  preconditions:
    - PRE-203-1: EPIC-202 closed (ZMD v2 schema published)
    - PRE-203-2: EPIC-207 closed (compiler produces SirGraph v2)
    - PRE-203-3: petgraph crate added to Cargo.toml (or equivalent)

  postconditions:
    - POST-203-1: src/query/ compiles with zero clippy warnings
    - POST-203-2: All query tests pass
    - POST-203-3: CLI query subcommand exits 0 on valid query, 1 on parse error, 2 on no results
    - POST-203-4: p99 latency benchmark recorded in CI artifacts

  validation:
    automated:
      - script: cargo test query
        description: Full query test suite
      - script: cargo bench query_latency
        description: Latency benchmark, asserts p99 ≤ 50ms
      - script: tests/query/metamorphic.sh
        description: Runs each query 100× on same input, diffs results
      - script: tests/query/completeness.sh
        description: Exhaustive predicate filter test vs brute-force scan
    manual:
      - review: DSL grammar must be reviewed by spec author before implementation

  metrics:
    - metric: query_p99_latency_ms
      target: ≤ 50
      measurement: cargo bench query_latency
    - metric: query_determinism_rate
      target: 100%
      measurement: metamorphic suite over 100 repeats
    - metric: query_completeness_rate
      target: 100%
      measurement: completeness test vs brute-force
    - metric: dsl_operator_coverage
      formula: (operators_implemented / operators_specified) * 100
      target: 100%

  failure_modes:
    - FAIL-203-1:
        condition: INV-203-1 violated (non-deterministic query)
        action: Critical defect; block EPIC-204 and EPIC-205
    - FAIL-203-2:
        condition: INV-203-2 violated (missing results)
        action: Critical defect; block EPIC-205 retention metrics
    - FAIL-203-3:
        condition: p99 latency > 50ms
        action: Performance defect; profile and optimize before closing EPIC
    - FAIL-203-4:
        condition: Query panics on malformed input
        action: Critical defect; must return Err, never panic
```

---

## Features

### FEAT-203-1: Query DSL

**Description:** Design and specify a typed, composable Query DSL for the STF-SIR semantic graph.
The DSL is expression-based, deterministic, and covers graph traversal, predicate filtering,
dimension projection, and result shaping.

**Inputs:**
- SirGraph v2 API
- 10 named query patterns identified from v1 use cases

**Outputs:**
- `spec/query-dsl-v1.md` (DSL grammar, semantics, examples)
- PEG grammar file (`src/query/grammar.pest`)
- DSL parser (`src/query/parser.rs`)
- AST types (`src/query/ast.rs`)

**Acceptance Criteria:**
- [ ] Grammar expressed in PEG (using `pest` crate) — deferred to FEAT-203-1 follow-up
- [x] All 10 named patterns expressible in the DSL (implemented as Rust enum)
- [x] Grammar is unambiguous (Rust enum; each variant is distinct)
- [ ] Error messages are human-readable (position/expected token — deferred to parser impl)
- [x] DSL spec includes: grammar, operator reference, 10 worked examples (docs/spec/query-dsl-v1.md)

**Metrics:**
- operator_coverage = 100%
- parse_error_quality: all 10 invalid-query golden cases produce expected error message substring

**Feature Contract:**
```yaml
contract:
  id: CONTRACT-FEAT-203-1
  inputs: [SirGraph v2 API, query pattern list]
  outputs: [spec/query-dsl-v1.md, grammar.pest, parser.rs, ast.rs]
  invariants:
    - Grammar has no ambiguous productions
    - Every parse result is either a valid AST or a typed ParseError
    - DSL is purely declarative (no side effects)
  postconditions:
    - pest grammar compiles without conflicts
    - All 20 example queries in spec parse successfully
    - All 10 invalid-query cases produce ParseError (not panic)
  failure_modes:
    - Ambiguous grammar → spec defect, block FEAT-203-2
    - Panic on invalid input → critical defect
```

#### Tasks

**TASK-203-1-1: Define 10 named query patterns**
- Description: Enumerate the canonical query patterns: path(src, dst), ancestors(id, depth), descendants(id, depth), by_type(node_type), by_category(cat), subgraph(root), depth_range(min, max), span_range(start, end), gloss_match(regex), dimension_filter(dim, predicate)
- Definition of done: Each pattern has name, signature, semantics, and example
- Artifacts: `spec/query-dsl-v1.md` §2 (pattern catalog)

**TASK-203-1-2: Write PEG grammar**
- Description: Express all 10 patterns as composable PEG productions in `grammar.pest`
- Definition of done: `pest` parses grammar without conflicts; `cargo test` passes pest grammar tests
- Artifacts: `src/query/grammar.pest`
- Contract: No ambiguous productions; all named patterns expressible

**TASK-203-1-3: Implement AST types**
- Description: Define `QueryExpr` enum with variants for each operator; derive `Debug`, `PartialEq`, `Clone`
- Definition of done: `src/query/ast.rs` compiles, all operators represented
- Artifacts: `src/query/ast.rs`

**TASK-203-1-4: Implement DSL parser**
- Description: Pest-based parser that produces `QueryExpr` from DSL string input
- Definition of done: Parses all 20 spec examples correctly; returns typed `ParseError` for all 10 invalid cases
- Artifacts: `src/query/parser.rs`

**TASK-203-1-5: Write DSL parser test suite**
- Description: Unit tests for all 20 valid + 10 invalid cases; golden output comparison for AST
- Definition of done: `cargo test query_parser` passes
- Artifacts: `tests/query/parser_tests.rs`

---

### FEAT-203-2: Graph Navigation API

**Description:** Implement the query execution engine that evaluates `QueryExpr` ASTs against
a `SirGraph`, returning typed `QueryResult` values. Uses `petgraph` internally for path
algorithms; all public results are sorted by ztoken ID for determinism.

**Inputs:**
- `QueryExpr` AST (from FEAT-203-1)
- `SirGraph v2` (`src/sir/graph.rs`)
- `petgraph` crate (or equivalent)

**Outputs:**
- `src/query/executor.rs` — query evaluator
- `src/query/result.rs` — `QueryResult`, `QueryResultSet` types
- `src/query/mod.rs` — public API re-exports

**Acceptance Criteria:**
- [x] `execute(query: &Query, graph: &SirGraph) -> QueryResult` is the sole entry point
- [x] All 10 named patterns are implemented
- [x] Transitive closure (ancestors, descendants) uses iterative BFS with cycle detection
- [x] All result `Vec` fields sorted by ztoken ID (lexicographic, stable)
- [x] Zero panics: engine uses safe iterators with no unwrap on user data
- [x] `QueryResult` is serializable to JSON via serde

**Metrics:**
- query_p99_latency_ms ≤ 50 on 10 000-token artifact
- completeness_rate = 100% on exhaustive predicate test

**Feature Contract:**
```yaml
contract:
  id: CONTRACT-FEAT-203-2
  inputs: [QueryExpr AST, SirGraph v2, petgraph]
  outputs: [executor.rs, result.rs, mod.rs]
  invariants:
    - INV-203-1 (determinism)
    - INV-203-2 (completeness)
    - INV-203-3 (read-only)
  postconditions:
    - Zero unsafe blocks
    - All 10 patterns return correct results on test corpus
    - Latency benchmark passes
  failure_modes:
    - Panic → critical; return Err
    - Missing results → critical; block EPIC-205
```

#### Tasks

**TASK-203-2-1: Add petgraph to Cargo.toml and integrate with SirGraph**
- Description: Add `petgraph` as optional feature dependency; write `SirGraph::to_petgraph()` adapter
- Definition of done: `cargo build --features query` succeeds; adapter produces isomorphic graph
- Artifacts: Cargo.toml update, `src/sir/petgraph_adapter.rs`

**TASK-203-2-2: Implement filter predicate evaluators**
- Description: Implement by_type, by_category, gloss_match, dimension_filter predicates over ZToken
- Definition of done: All 4 predicates return correct results on test corpus; zero false negatives
- Artifacts: `src/query/predicates.rs`

**TASK-203-2-3: Implement traversal operators**
- Description: Implement path(src, dst), ancestors(id, depth), descendants(id, depth), subgraph(root), depth_range, span_range
- Definition of done: All 6 traversal operators return sorted, deterministic result sets
- Artifacts: `src/query/traversal.rs`

**TASK-203-2-4: Implement QueryResult and serialization**
- Description: Define `QueryResult { nodes: Vec<ZTokenRef>, edges: Vec<RelationRef>, meta: QueryMeta }` with serde
- Definition of done: Serializes to JSON deterministically; round-trips via serde
- Artifacts: `src/query/result.rs`

**TASK-203-2-5: Write exhaustive query test suite**
- Description: Tests for all 10 patterns on all golden v2 fixtures; metamorphic tests (100× per query)
- Definition of done: `cargo test query_executor` passes; metamorphic suite shows 100% determinism
- Artifacts: `tests/query/executor_tests.rs`, `tests/query/metamorphic.rs`

---

### FEAT-203-3: CLI Query Interface

**Description:** Add `stf-sir query <ARTIFACT> --query <DSL_STRING> [--format json|yaml|table]`
subcommand that executes a DSL query against a ZMD artifact and prints results.

**Inputs:**
- ZMD artifact file
- Query DSL string (from `--query` flag or `--query-file` for multi-line)
- Output format flag

**Outputs:**
- Updated `src/cli.rs` with `query` subcommand
- CLI documentation (`--help` output)
- CLI integration tests

**Acceptance Criteria:**
- [ ] `stf-sir query artifact.zmd --query 'by_type(heading)'` exits 0 and prints results
- [ ] Exit code 1 on query parse error; 2 on no results; 3 on invalid artifact
- [ ] `--format json` produces valid JSON; `--format yaml` produces valid YAML; `--format table` produces aligned ASCII table
- [ ] `--query-file path` reads multi-line DSL from file
- [ ] `--help` output covers all flags

**Metrics:**
- CLI wall-time ≤ 100ms for 1000-token artifact on ubuntu-latest

**Feature Contract:**
```yaml
contract:
  id: CONTRACT-FEAT-203-3
  inputs: [ZMD artifact, query DSL string, format flag]
  outputs: [printed results, exit code]
  invariants:
    - CLI never panics
    - Exit codes are stable (semver contract)
    - JSON output is valid JSON
  postconditions:
    - CLI smoke tests pass
    - --help covers all flags
  failure_modes:
    - Panic → critical
    - Wrong exit code → breaking change
```

#### Tasks

**TASK-203-3-1: Add query subcommand to clap CLI**
- Description: Add `Query` variant to CLI enum with all flags
- Artifacts: Updated `src/cli.rs`

**TASK-203-3-2: Implement output formatters (JSON, YAML, table)**
- Description: Write three formatter functions for `QueryResult`
- Definition of done: Each formatter output is valid for its format; table output aligns columns
- Artifacts: `src/query/formatters.rs`

**TASK-203-3-3: Write CLI integration tests**
- Description: Bash-based or Rust `assert_cmd` tests for all exit codes and output formats
- Definition of done: All test cases pass in CI
- Artifacts: `tests/query/cli_tests.rs`
