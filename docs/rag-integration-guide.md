# RAG Integration Guide

**Version:** 1.0.0  
**Audience:** Developers integrating STF-SIR artifacts into RAG (Retrieval-Augmented Generation) pipelines.

---

## Overview

STF-SIR produces structured `.zmd` artifacts from Markdown source documents. Each artifact
contains a list of **ZTokens** — semantically annotated chunks of the source — along with
relations and provenance metadata.

The RAG pipeline in STF-SIR converts these ZTokens into embeddable **Chunks**, stores them
in a vector store, and enables similarity search with full provenance traceability.

**Pipeline summary:**

```
Markdown source
    ↓  compile (stf_sir::compiler)
ZMD Artifact (ztokens + relations)
    ↓  chunk (stf_sir::rag::Chunker)
Vec<Chunk> (one per ZToken, with provenance)
    ↓  embed (EmbeddingProvider)
Vec<Vec<f32>> (embedding vectors)
    ↓  store (VectorStore)
MemoryVectorStore / Qdrant / Pinecone / …
    ↓  search
Vec<SearchResult> (chunk + cosine score)
    ↓  retrieve with provenance
ZToken → source span → original text
```

---

## Step-by-Step Guide

### 1. Enable the `rag` feature

Add to your `Cargo.toml` (for a downstream crate that depends on `stf-sir`):

```toml
[dependencies]
stf-sir = { version = "1.0", features = ["rag"] }
```

Or run tests with:

```bash
cargo test --features rag
```

### 2. Compile a Markdown document

```rust
use stf_sir::compiler;

let source = std::fs::read_to_string("my-document.md")?;
let artifact = compiler::compile_markdown(&source, Some("my-document.md"))?;
println!("Compiled {} tokens", artifact.ztokens.len());
```

### 3. Chunk the artifact

```rust
use stf_sir::rag::Chunker;

let chunker = Chunker::new("text-embedding-3-small");
let chunks = chunker.chunk_artifact(&artifact);
println!("Created {} chunks", chunks.len());

// Each chunk has full provenance:
for chunk in &chunks {
    println!("{}: {}", chunk.chunk_id, &chunk.text[..chunk.text.len().min(60)]);
}
```

### 4. Embed the chunks

```rust
use stf_sir::rag::{EmbeddingProvider, MockEmbeddingProvider};

// Use MockEmbeddingProvider for tests; replace with a real provider in production.
let provider = MockEmbeddingProvider { dimensions: 1536 };

let texts: Vec<&str> = chunks.iter().map(|c| c.text.as_str()).collect();
let embeddings = provider.embed(&texts)?;
```

### 5. Store in the vector store

```rust
use stf_sir::rag::{MemoryVectorStore, VectorStore};

let mut store = MemoryVectorStore::new(provider.dimensions());

for (chunk, embedding) in chunks.into_iter().zip(embeddings) {
    store.insert(chunk, embedding)?;
}
println!("Store contains {} entries", store.len());
```

### 6. Search

```rust
// Embed a query string
let query_embedding = provider.embed(&["semantic compilation platform"])?;
let results = store.search(&query_embedding[0], 5)?;

for result in &results {
    println!(
        "score={:.3}  chunk={}  text={}",
        result.score,
        result.chunk.chunk_id,
        &result.chunk.text[..result.chunk.text.len().min(80)]
    );
}
```

### 7. Retrieve with provenance

Every `SearchResult.chunk` carries:

| Field              | Description                              |
|--------------------|------------------------------------------|
| `chunk_id`         | `rag:<provider>/<sha256>/<ztoken_id>`    |
| `ztoken_id`        | The source ZToken's ID                   |
| `artifact_sha256`  | SHA-256 of the source document           |
| `source_path`      | Path to the original file (if available) |
| `span_start`       | Start byte offset in source              |
| `span_end`         | End byte offset in source                |

```rust
for result in &results {
    let chunk = &result.chunk;
    println!(
        "Source: {} bytes {}–{} (ztoken: {})",
        chunk.source_path.as_deref().unwrap_or("<memory>"),
        chunk.span_start,
        chunk.span_end,
        chunk.ztoken_id,
    );
}
```

---

## LangChain Integration (Pseudocode)

STF-SIR's `MemoryVectorStore` can be wrapped as a LangChain retriever:

```python
# pseudocode — actual implementation requires a Python FFI or subprocess call
import subprocess
import json

def compile_and_chunk(markdown_path: str, provider_id: str = "openai") -> list[dict]:
    """Compile a Markdown file with stf-sir CLI and return chunks as JSON."""
    result = subprocess.run(
        ["stf-sir", "compile", markdown_path, "--output-chunks"],
        capture_output=True, text=True, check=True
    )
    return json.loads(result.stdout)

class StfSirRetriever:
    """LangChain-compatible retriever backed by a STF-SIR vector store."""

    def __init__(self, chunks: list[dict], embed_fn):
        self.chunks = chunks
        self.embeddings = [embed_fn(c["text"]) for c in chunks]

    def get_relevant_documents(self, query: str, k: int = 5) -> list[dict]:
        from numpy import dot
        from numpy.linalg import norm

        query_emb = embed_fn(query)
        scores = [
            dot(query_emb, emb) / (norm(query_emb) * norm(emb) + 1e-8)
            for emb in self.embeddings
        ]
        top_indices = sorted(range(len(scores)), key=lambda i: -scores[i])[:k]
        return [
            {
                "page_content": self.chunks[i]["text"],
                "metadata": {
                    "chunk_id": self.chunks[i]["chunk_id"],
                    "ztoken_id": self.chunks[i]["ztoken_id"],
                    "source": self.chunks[i].get("source_path"),
                    "span_start": self.chunks[i]["span_start"],
                    "span_end": self.chunks[i]["span_end"],
                },
            }
            for i in top_indices
        ]
```

---

## Raw API Usage (Rust)

Complete example compiling, chunking, embedding, storing, and querying:

```rust
use stf_sir::compiler;
use stf_sir::rag::{Chunker, EmbeddingProvider, MemoryVectorStore, MockEmbeddingProvider, VectorStore};

fn main() -> anyhow::Result<()> {
    // 1. Compile
    let artifact = compiler::compile_markdown(
        "# Introduction\n\nSTF-SIR is a semantic compilation system.\n\n- Fast\n- Deterministic\n",
        None,
    )?;

    // 2. Chunk
    let chunker = Chunker::new("mock-embed-v1");
    let chunks = chunker.chunk_artifact(&artifact);

    // 3. Embed
    let provider = MockEmbeddingProvider { dimensions: 384 };
    let texts: Vec<&str> = chunks.iter().map(|c| c.text.as_str()).collect();
    let embeddings = provider.embed(&texts)?;

    // 4. Store
    let mut store = MemoryVectorStore::new(provider.dimensions());
    for (chunk, emb) in chunks.into_iter().zip(embeddings) {
        store.insert(chunk, emb)?;
    }

    // 5. Search
    let query_emb = provider.embed(&["semantic compilation"])?;
    let results = store.search(&query_emb[0], 3)?;

    for r in &results {
        println!("  [{:.3}] {} — {}", r.score, r.chunk.ztoken_id, r.chunk.text);
    }

    Ok(())
}
```

---

## Using Agent Tools

STF-SIR exposes three agent tools compatible with OpenAI function-calling and Anthropic
`tool_use`. Get the schema with:

```rust
use stf_sir::agent::stf_sir_tools;

let schema = stf_sir_tools();
let json = serde_json::to_string_pretty(&schema)?;
println!("{json}");
```

Pass the resulting JSON to your LLM API as the `tools` parameter. See
`docs/spec/agent-tools-v1.json` for the full schema.

---

## Deleting Stale Chunks

When a document is recompiled, delete its old chunks before inserting new ones:

```rust
let deleted = store.delete_by_artifact(&artifact.source.sha256)?;
println!("Deleted {deleted} stale chunks");

// Re-chunk and re-insert…
```

---

## Error Handling

| Error | Cause | Resolution |
|-------|-------|------------|
| `EmbeddingError::Provider` | Embedding API unavailable | Retry with backoff; use mock for tests |
| `StoreError::DimensionMismatch` | Embedding dimension changed | Rebuild the store with the new dimension |
| `CompileError` | Markdown parsing failure | Check source document; see diagnostics |
