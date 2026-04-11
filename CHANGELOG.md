# Changelog

All notable changes to this project are documented here.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

No unreleased changes.

## [1.0.0-mvp] — v1 Operational Freeze

This release declares the **STF-SIR v1 operational baseline** as frozen.
Artifacts produced under the `compiler.config_hash` recorded below are
required to remain byte-identical under the same input forever. Any
change to parser options, emitted node types, relation categories,
normalization, serialization, or stage semantics is a deliberate
`config_hash` bump and MUST be accompanied by golden regeneration.

### Contract

| Invariant | Value |
| --- | --- |
| `format` | `stf-sir.zmd` |
| `version` | `1` |
| `compiler.name` | `stf-sir-ref` |
| `compiler.profile` | `stf-sir-spec-v1-mvp` |
| Canonical `compiler.config_hash` | `sha256:930130473f2a81953293b44a228077ce3758f11fe3b286565ae1957713cd810d` |
| Canonical `examples/sample.zmd` | `sha256:57aac69eee54267ee23308aee51ca1c17e6caff32155960931ce0419214111bb` |
| `schemas/zmd-v1.schema.json` | `sha256:9bc280486ed5b5404d12c6bdc8b9028d13bd237bdb124ec64c6faaa32d2fce67` |

### Compiler

- Deterministic four-stage pipeline: lexical → syntactic → semantic → logical.
- Parser: `pulldown-cmark` with tables, footnotes, and strikethrough enabled.
- Supported block node types: heading, paragraph, blockquote, list, list_item,
  code_block, table, footnote_definition.
- Relations: `contains` and `precedes`, both `structural`, both emitted
  from the `logical` stage.
- NFKC normalization with whitespace trimming and collapsing.
- Serialization via `serde_yaml_ng` with stable struct-order field emission.
- Siblings grouped by parent's preorder index (not by string id) so
  `precedes` ordering is stable past `z9`.

### Validator

Two-pass pipeline exposed as `stf-sir validate <INPUT>`:

1. **Schema pass** — JSON Schema (Draft 2020-12) embedded at build time,
   enforcing all field-level and enum constraints.
2. **Semantic pass** — spec §9 cross-reference rules with stable codes
   `VAL_01_FORMAT` through `VAL_18_RELATION_STAGE`.

Output:

- `VALID: <path> conforms to STF-SIR v1` on success (exit 0).
- `INVALID: <path>` followed by one indented `[<rule>] <message>` per
  failure on non-success (exit non-zero).

### Diagnostics

Closed set of codes per spec §10:

- `SRC_UTF8_INVALID`
- `SYN_PARSE_FAILED`
- `SYN_NODE_UNSUPPORTED`
- `SEM_FALLBACK_APPLIED`
- `LOG_RELATION_SKIPPED`
- `VAL_SCHEMA_FAILED`

### Conformance kit

- `tests/conformance/valid/` — 10 curated `.md` ↔ `.zmd` pairs covering
  empty documents, whitespace-only input, CRLF line endings, NFKC
  compatibility forms (U+FB03), zero-width spaces, CJK fullwidth, nested
  lists, multi-line paragraphs, fenced code blocks, and heading depth.
- `tests/conformance/invalid_schema/` — 5 cases: wrong format constant,
  wrong version constant, missing category field, bad category enum,
  bad stage enum.
- `tests/conformance/invalid_semantic/` — 5 cases: mismatched
  `document.token_count`, broken relation source, broken parent ref,
  duplicate token id, dangling `Φ.relation_ids` entry.
- `tests/proptest_invariants.rs` — 128-case property test for rules 5,
  7, 8, 14, 16 over generated ASCII Markdown.

### CI

- `cargo fmt --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo test --all-features`
- Golden byte-diff gate: `examples/sample.md → sample.zmd` must match
  the checked-in copy on both Ubuntu and macOS.

### Supply chain

- `rust-toolchain.toml` pins the stable channel.
- `Cargo.toml` requires `rust-version = "1.82"`.
- `deny.toml` bans `serde_yaml` (deprecated) and restricts licenses to a
  permissive allowlist.

### Not in scope (deferred to v1.1)

- Projection operators `π_L, π_S, π_Σ, π_Φ` as an API surface.
- `Artifact::as_sir_graph()` typed attributed multigraph view.
- Information retention vector `ρ(d) = ⟨ρ_L, ρ_S, ρ_Σ, ρ_Φ⟩` baseline.
- `Enricher` trait for monotone semantic enrichment.
- Compositional `merge()` of subartifacts.
