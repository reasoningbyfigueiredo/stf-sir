---
id: EPIC-206
title: RAG & Agent Integration
version: 2.0.0-alpha
status: implementado
roadmap: ROADMAP-STF-SIR-V2
priority: medium
created: 2026-04-12
target: 2026-12-01
depends_on:
  - EPIC-205
  - EPIC-208
blocks: []
---

# EPIC-206 — RAG & Agent Integration

## Description

Bridge the STF-SIR semantic platform with RAG (Retrieval-Augmented Generation) pipelines
and AI agents. The integration is **adapter-based**: STF-SIR provides the semantic
representation layer; external vector stores and AI inference APIs are pluggable.

Core deliverables:
1. **Embedding pipeline** — chunking strategy and embedding anchor (`Σ.embedding_ref`) population via pluggable `EmbeddingProvider` trait
2. **Vector store adapter** — serializes ztokens as embeddable chunks with provenance metadata
3. **Agent tool interface** — exposes `query`, `diff`, and `retention` as tool-call-compatible JSON functions consumable by AI agents (OpenAI tools, Anthropic tool_use)
4. **Provenance guarantees** — every retrieved chunk carries its ztoken ID, artifact SHA-256, and dimension path

## Scope

- **In scope:** EmbeddingProvider trait, chunk serializer, vector store adapter (interface + one reference impl), agent tool schema, provenance metadata
- **Out of scope:** Specific vector databases (Pinecone, Weaviate, etc.) beyond the reference impl, LLM inference, fine-tuning, training pipelines

## Deliverables

| # | Artifact | Path |
|---|---|---|
| D-206-1 | EmbeddingProvider trait | `src/rag/embedding.rs` |
| D-206-2 | Chunk serializer | `src/rag/chunker.rs` |
| D-206-3 | Vector store adapter interface | `src/rag/store.rs` |
| D-206-4 | Reference vector store adapter (in-memory) | `src/rag/memory_store.rs` |
| D-206-5 | Agent tool schema | `spec/agent-tools-v1.json` |
| D-206-6 | Agent tool server (optional) | `src/agent/` |
| D-206-7 | RAG integration guide | `docs/rag-integration-guide.md` |
| D-206-8 | Provenance spec | `spec/provenance-v1.md` |

## Success Criteria

- [x] EmbeddingProvider trait is object-safe and feature-gated (`--features rag`)
- [x] Every emitted chunk includes: ztoken_id, artifact_sha256, source_path, dimension, span
- [x] Reference in-memory store supports: insert, search (cosine similarity), delete by artifact_sha256
- [x] Agent tool schema is valid OpenAI tools JSON format (also compatible with Anthropic tool_use)
- [x] Provenance round-trip: chunk_id → ztoken_id → artifact → source span (verified by test)
- [x] Integration guide includes working examples for LangChain and raw API usage

## Risks

| ID | Risk | Mitigation |
|---|---|---|
| R-206-1 | Embedding API rate limits during testing | Use mock EmbeddingProvider in tests; real provider only in integration tests |
| R-206-2 | Agent tool schema breaks with API version changes | Version the schema file; document minimum API versions |
| R-206-3 | Embedding dimensionality varies by provider | Store dimension size in chunk metadata; reference store is dimension-agnostic |
| R-206-4 | Provenance chain breaks on artifact recompile | Use artifact SHA-256 + ztoken ID as stable provenance key |

---

## EPIC CONTRACT

```yaml
contract:
  id: CONTRACT-EPIC-206
  version: 1.0.0

  inputs:
    - id: I-206-1
      description: ZMD v2 artifacts with Σ.embedding_ref anchor
      required: true
    - id: I-206-2
      description: Query engine (EPIC-203 output)
      required: true
    - id: I-206-3
      description: Extensibility model (EPIC-208 output)
      required: true
    - id: I-206-4
      description: OpenAI tools / Anthropic tool_use JSON format specs
      required: true

  outputs:
    - id: O-206-1
      artifact: src/rag/ module
    - id: O-206-2
      artifact: spec/agent-tools-v1.json
    - id: O-206-3
      artifact: spec/provenance-v1.md
    - id: O-206-4
      artifact: docs/rag-integration-guide.md

  invariants:
    - INV-206-1: |
        Provenance invariant: every retrieved chunk MUST be traceable to
        exactly one ZToken, one artifact (by SHA-256), and one source span.
        No provenance-free chunks may be emitted.
    - INV-206-2: |
        EmbeddingProvider is optional: compiling without --features rag MUST
        produce zero embedding-related code in the binary.
    - INV-206-3: |
        Agent tools are read-only: no tool call modifies any artifact or
        SirGraph. Tool calls execute queries and return results only.
    - INV-206-4: |
        Chunk determinism: the same ZToken with the same embedding_ref
        produces the same chunk JSON on every serialization.

  preconditions:
    - PRE-206-1: EPIC-205 closed (retention metrics validated)
    - PRE-206-2: EPIC-208 closed (extensibility model provides plugin interface)
    - PRE-206-3: Σ.embedding_ref field in ZMD v2 schema (EPIC-202)

  postconditions:
    - POST-206-1: EmbeddingProvider tests pass with mock provider
    - POST-206-2: Provenance round-trip test passes
    - POST-206-3: Agent tool schema validates against OpenAI tools meta-schema
    - POST-206-4: RAG integration guide reviewed by a non-author

  validation:
    automated:
      - script: cargo test --features rag rag_integration
        description: Full RAG test suite with mock embedding provider
      - script: tests/rag/provenance_roundtrip.sh
        description: Chunk → ztoken_id → artifact → span roundtrip test
      - script: scripts/validate-agent-tools-schema.sh spec/agent-tools-v1.json
        description: Validates tool schema against OpenAI/Anthropic meta-schema
    manual:
      - review: Integration guide reviewed by a user who has not seen the code

  metrics:
    - metric: provenance_completeness
      formula: (chunks_with_full_provenance / total_chunks) * 100
      target: 100%
    - metric: chunk_determinism_rate
      target: 100%
      measurement: 100× serialization of same ZToken
    - metric: agent_tool_schema_validity
      target: pass
    - metric: feature_gate_binary_size_increase
      formula: (rag_binary_size - base_binary_size) / base_binary_size
      target: only when --features rag

  failure_modes:
    - FAIL-206-1:
        condition: INV-206-1 violated (provenance-free chunk)
        action: Critical; block RAG release; reject chunk emission
    - FAIL-206-2:
        condition: INV-206-2 violated (RAG code in base binary)
        action: Architecture defect; rearchitect feature gate
    - FAIL-206-3:
        condition: Agent tool schema invalid
        action: Fix schema before publishing guide
```

---

## Features

### FEAT-206-1: Embedding Pipeline

**Description:** Define the `EmbeddingProvider` trait and chunking pipeline that converts
ztokens into embeddable text chunks, optionally calls an external embedding provider, and
populates `Σ.embedding_ref` with a stable URI reference.

**Inputs:**
- ZMD v2 artifact
- Configurable chunking strategy (block, sentence, sliding-window)
- Optional EmbeddingProvider (mock for tests; real for production)

**Outputs:**
- `src/rag/embedding.rs` — `EmbeddingProvider` trait + mock impl
- `src/rag/chunker.rs` — chunking strategies
- Updated `src/rag/mod.rs`

**Acceptance Criteria:**
- [ ] `EmbeddingProvider` trait is object-safe: `dyn EmbeddingProvider + Send + Sync`
- [ ] Mock provider returns deterministic dummy vectors (SHA-256 of text → deterministic f32 vec)
- [ ] `block` chunking strategy: one chunk per ztoken; `sentence` requires sentence-level profile
- [ ] Sliding-window strategy: configurable window and stride in ztoken units
- [ ] Chunks are ordered by ztoken ID; emission order is stable
- [ ] All chunk text uses `normalized_text` from L dimension (not source_text)

**Feature Contract:**
```yaml
contract:
  id: CONTRACT-FEAT-206-1
  inputs: [ZMD v2, chunking strategy, EmbeddingProvider]
  outputs: [Vec<Chunk> with embedding_ref populated]
  invariants:
    - INV-206-1 (every chunk has provenance)
    - INV-206-4 (chunk determinism)
    - Chunker does not modify artifact
  postconditions:
    - embedding_ref = "rag:<provider_id>/<artifact_sha256>/<ztoken_id>"
    - All chunks serializable to JSON
  failure_modes:
    - embedding_ref collision → critical provenance failure
```

#### Tasks

**TASK-206-1-1: Define EmbeddingProvider trait**
- Description: Write trait with `embed(text: &str) -> Result<Vec<f32>, EmbeddingError>` and provider metadata
- Artifacts: `src/rag/embedding.rs`

**TASK-206-1-2: Implement mock EmbeddingProvider**
- Description: Deterministic mock: SHA-256(text) → 384-dim f32 vector (normalized)
- Artifacts: `src/rag/embedding.rs` (mock impl)

**TASK-206-1-3: Implement chunking strategies (block, sentence, sliding-window)**
- Description: Three strategies; each returns `Vec<Chunk>` in ztoken ID order
- Artifacts: `src/rag/chunker.rs`

**TASK-206-1-4: Define provenance URI scheme**
- Description: `rag:<provider_id>/<artifact_sha256>/<ztoken_id>` — document in `spec/provenance-v1.md`
- Artifacts: `spec/provenance-v1.md`

**TASK-206-1-5: Write embedding pipeline tests**
- Description: Tests for all three chunking strategies; mock provider determinism; provenance completeness
- Artifacts: `tests/rag/embedding_tests.rs`

---

### FEAT-206-2: Vector Store Adapter

**Description:** Define the `VectorStore` adapter interface and provide a reference in-memory
implementation. The interface abstracts over any vector store (Pinecone, Qdrant, etc.) and
is decoupled from STF-SIR's core.

**Inputs:**
- `Vec<Chunk>` from embedding pipeline
- Query vector for similarity search

**Outputs:**
- `src/rag/store.rs` — `VectorStore` trait
- `src/rag/memory_store.rs` — reference in-memory implementation

**Acceptance Criteria:**
- [ ] `VectorStore` trait: `insert(chunks) -> Result<()>`, `search(vector, top_k) -> Result<Vec<SearchResult>>`, `delete_by_artifact(sha256) -> Result<usize>`
- [ ] `SearchResult` includes: chunk_id, score, full provenance metadata
- [ ] In-memory store: cosine similarity search, O(n) scan acceptable for reference impl
- [ ] Store is `Send + Sync` for concurrent access
- [ ] `delete_by_artifact` removes all chunks for a given artifact SHA-256

**Feature Contract:**
```yaml
contract:
  id: CONTRACT-FEAT-206-2
  inputs: [Vec<Chunk>, query vector]
  outputs: [SearchResult with provenance]
  invariants:
    - INV-206-1 (every search result has provenance)
    - Store is read-after-write consistent
  postconditions:
    - search(embed(text)) returns same-text chunk in top-1 (for unique texts)
  failure_modes:
    - Missing provenance in search result → reject, log error
```

#### Tasks

**TASK-206-2-1: Define VectorStore trait**
- Artifacts: `src/rag/store.rs`

**TASK-206-2-2: Implement in-memory VectorStore**
- Description: BTreeMap<chunk_id, (Vec<f32>, Chunk)> + cosine similarity brute-force search
- Artifacts: `src/rag/memory_store.rs`

**TASK-206-2-3: Write VectorStore integration tests**
- Description: Insert, search, delete round-trip tests; provenance completeness test
- Artifacts: `tests/rag/store_tests.rs`

---

### FEAT-206-3: Agent Tool Interface

**Description:** Expose STF-SIR capabilities as AI agent tools, compatible with OpenAI function
calling and Anthropic tool_use formats. Tools exposed: `query_artifact`, `diff_artifacts`,
`get_retention_score`, `get_provenance`.

**Inputs:**
- Query engine (EPIC-203)
- Diff engine (EPIC-204)
- Retention metrics (EPIC-205)
- Provenance data (FEAT-206-1)

**Outputs:**
- `spec/agent-tools-v1.json` — JSON Schema for all tool definitions
- `src/agent/tools.rs` — Rust handler functions
- `docs/rag-integration-guide.md` — integration examples

**Acceptance Criteria:**
- [ ] `query_artifact`: accepts artifact path + DSL query string; returns QueryResult JSON
- [ ] `diff_artifacts`: accepts two artifact paths; returns DiffReport JSON
- [ ] `get_retention_score`: accepts artifact path; returns ρ_v2 JSON
- [ ] `get_provenance`: accepts chunk_id; returns full provenance chain JSON
- [ ] All tools return structured JSON (not plain text)
- [ ] Tool schema is valid OpenAI tools format and compatible with Anthropic tool_use
- [ ] Integration guide includes: setup, examples for LangChain/raw API, error handling

**Feature Contract:**
```yaml
contract:
  id: CONTRACT-FEAT-206-3
  inputs: [query engine, diff engine, retention metrics, provenance]
  outputs: [spec/agent-tools-v1.json, tool handlers, docs/rag-integration-guide.md]
  invariants:
    - INV-206-3 (all tools are read-only)
    - All tool outputs are valid JSON
    - Tool names are stable (semver contract)
  postconditions:
    - Schema validates against OpenAI tools meta-schema
    - Integration guide tested by non-author
  failure_modes:
    - Write-capable tool → architecture violation
    - Invalid JSON output → agent cannot parse results
```

#### Tasks

**TASK-206-3-1: Define agent tool schemas**
- Description: Write JSON Schema for all 4 tools in OpenAI tools format
- Artifacts: `spec/agent-tools-v1.json`

**TASK-206-3-2: Implement Rust tool handlers**
- Description: Write handler function per tool; returns `serde_json::Value`
- Artifacts: `src/agent/tools.rs`

**TASK-206-3-3: Write agent tool integration tests**
- Description: Test all 4 tools; verify JSON output structure; verify read-only invariant
- Artifacts: `tests/agent/tool_tests.rs`

**TASK-206-3-4: Write RAG integration guide**
- Description: Step-by-step guide: compile artifact, embed, store, query via tools
- Artifacts: `docs/rag-integration-guide.md`
