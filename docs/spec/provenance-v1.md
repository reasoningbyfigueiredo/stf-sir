# Provenance Specification v1

**Version:** 1.0.0  
**Status:** Draft  
**Date:** 2026-04-14

---

## Overview

Every chunk produced by the STF-SIR RAG pipeline carries full provenance metadata. This
enables round-trip traceability: from a retrieved chunk back to the exact source span in
the original document.

---

## Chunk ID Format

```
rag:<provider_id>/<artifact_sha256>/<ztoken_id>
```

| Component        | Description                                              | Example                   |
|------------------|----------------------------------------------------------|---------------------------|
| `provider_id`    | Identifier of the embedding provider                     | `text-embedding-3-small`  |
| `artifact_sha256`| SHA-256 hex digest of the source document                | `a3f2c1...` (64 chars)    |
| `ztoken_id`      | Unique ZToken identifier within the artifact             | `z7`                      |

### Example

```
rag:text-embedding-3-small/a3f2c1d4e5f6789012345678901234567890123456789012345678901234abcd/z7
```

The chunk ID is deterministic: given the same provider, artifact, and token, the ID is always
identical. This enables deduplication and cache lookups.

---

## Round-Trip Property

A chunk ID carries sufficient information to reconstruct the full provenance chain:

```
chunk_id
  → split on '/'  →  provider_id, artifact_sha256, ztoken_id
  → load artifact by SHA-256
  → find ZToken with id == ztoken_id
  → read ztokenl.lexical.span  →  source byte range
  → read artifact.source.path  →  source file
```

### Rust round-trip verification

```rust
fn verify_provenance(chunk: &Chunk, artifact: &Artifact) -> bool {
    // 1. chunk_id encodes the right artifact
    assert_eq!(chunk.artifact_sha256, artifact.source.sha256);

    // 2. ztoken_id references an existing token
    let token = artifact.ztokens.iter().find(|t| t.id == chunk.ztoken_id);
    assert!(token.is_some(), "ztoken_id must reference existing token");
    let token = token.unwrap();

    // 3. span is consistent
    assert_eq!(chunk.span_start, token.lexical.span.start_byte as u32);
    assert_eq!(chunk.span_end, token.lexical.span.end_byte as u32);

    // 4. chunk_id format is correct
    let expected_id = Chunker::chunk_id(&chunk_provider_id, &artifact.source.sha256, &token.id);
    assert_eq!(chunk.chunk_id, expected_id);

    true
}
```

---

## Provenance Chain

```
Chunk
  ├── chunk_id         "rag:provider/sha256/ztoken_id"
  ├── artifact_sha256  → locate artifact on disk / in store
  ├── ztoken_id        → ZToken in artifact.ztokens
  ├── source_path      → original Markdown file path
  ├── span_start       → byte offset in source
  └── span_end         → byte offset in source
```

The `source_path` may be `None` if the artifact was compiled from an in-memory string
(e.g. in tests). In that case, `artifact_sha256` alone identifies the source.

---

## Provenance Chain Verification

A provenance chain is **valid** if and only if:

1. `chunk.artifact_sha256` matches `artifact.source.sha256` for the artifact used to produce the chunk.
2. `chunk.ztoken_id` is the `id` of exactly one ZToken in `artifact.ztokens`.
3. `chunk.span_start == ztoken.lexical.span.start_byte` and `chunk.span_end == ztoken.lexical.span.end_byte`.
4. `chunk.chunk_id == Chunker::chunk_id(provider_id, artifact_sha256, ztoken_id)`.

Any chunk that fails one of these checks is considered **provenance-broken** and MUST NOT be
returned to callers.

---

## Invariants

| ID | Invariant |
|----|-----------|
| INV-PROV-1 | Every chunk emitted by `Chunker::chunk_artifact` satisfies all four verification conditions above. |
| INV-PROV-2 | `chunk_id` is deterministic: same inputs always produce the same ID. |
| INV-PROV-3 | No chunk may be returned from `VectorStore::search` without a valid `ztoken_id`. |
| INV-PROV-4 | `artifact_sha256` is the SHA-256 of the raw source bytes, not the compiled artifact. |

---

## Version History

| Version | Date       | Changes         |
|---------|------------|-----------------|
| v1      | 2026-04-14 | Initial release |
