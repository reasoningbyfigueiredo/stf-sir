---
id: SCHEMA-CHANGELOG
title: ZMD Schema Changelog
updated: 2026-04-14
---

# ZMD Schema Changelog

All notable changes to the ZMD JSON Schema are documented here.
Schema versions follow the artifact version they describe.

---

## [v2] — 2026-04-14

### Added

- `version` field: now accepts `enum: [1, 2]` (was `const: 1`). Artifacts with `version: 1` remain
  fully valid against this schema (backward-compatible superset).
- **ZToken C dimension** (`contextual`): optional object with `context_id`, `scope`, `reference_frame`.
- **ZToken P dimension** (`pragmatic`): optional object with `intent`, `speech_act`, `register`.
- **ZToken Δ dimension** (`temporal`): optional object with `created_at`, `modified_at`, `valid_from`,
  `valid_to` (all ISO 8601 date-time strings).
- **ZToken Ω dimension** (`coherence_eval`): optional object with `coherence_score ∈ [0,1]`,
  `validation_flags` (array of `VAL_XX` strings), `useful_information` (boolean).
- `semantic.embedding_ref`: nullable string; URI reference to a RAG vector store anchor.
  Format: `rag:<provider>/<artifact_sha256>/<ztoken_id>`.
- New relation types in `relations[].type` enum:
  - `supports` — logical support relation
  - `contradicts` — logical contradiction relation
  - `elaborates` — semantic elaboration relation
  - `refers_to` — cross-reference / coreference relation
  - `semantic-link` — generic semantic link
  - `cites` — provenance citation relation
- Top-level `sirgraph` property: optional graph export section (`stf-sir-sirgraph-v1` format).
  Contains `nodes` (array of `{id, node_type}`) and `edges` (array of `{id, type, source, target, category}`).
- Top-level `extensions` property: open namespace for plugin-specific metadata.
- Per-ZToken `extensions` property: open namespace at token level.

### Changed

- `$id` updated to `https://stf-sir.org/schemas/zmd-v2.schema.json`
- `title` updated to `STF-SIR .zmd v2 artifact`
- `description` updated to reflect backward-compatible superset semantics.

### Removed

Nothing removed. All v1 fields, constraints, and required properties are preserved verbatim.

---

## [v1] — initial release

- Initial ZMD schema for STF-SIR v1 artifacts.
- Required top-level fields: `format`, `version`, `source`, `compiler`, `document`, `ztokens`,
  `relations`, `diagnostics`.
- ZToken dimensions: `L` (lexical), `S` (syntactic), `Σ` (semantic), `Φ` (logical) — all required.
- Relation types: `contains`, `precedes`.
- Language: `document.language` always `"und"` in v1.
