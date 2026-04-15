//! Integration tests for the RAG pipeline (EPIC-206).
//!
//! All tests are feature-gated with `#[cfg(feature = "rag")]`.
//! Run with: `cargo test --features rag rag_integration`
//!
//! Requires: `pub mod rag;` in `src/lib.rs` (added as part of EPIC-206 wiring).

#[cfg(feature = "rag")]
#[allow(unused_imports)]
use stf_sir::rag::{Chunk, Chunker, MemoryVectorStore, MockEmbeddingProvider, VectorStore};

#[cfg(feature = "rag")]
fn compile_sample() -> stf_sir::model::Artifact {
    stf_sir::compiler::compile_markdown(
        "# Introduction\n\nThe System is a platform.\n\n- Item one\n- Item two\n",
        None,
    )
    .expect("sample markdown should compile")
}

/// Chunker must produce exactly one chunk per ZToken.
#[cfg(feature = "rag")]
#[test]
fn chunker_produces_chunk_per_ztoken() {
    let artifact = compile_sample();
    let chunker = Chunker::new("mock-embed-v1");
    let chunks = chunker.chunk_artifact(&artifact);

    assert_eq!(
        chunks.len(),
        artifact.ztokens.len(),
        "expected one chunk per ztoken"
    );

    for (chunk, ztoken) in chunks.iter().zip(artifact.ztokens.iter()) {
        assert_eq!(chunk.ztoken_id, ztoken.id);
        assert_eq!(chunk.artifact_sha256, artifact.source.sha256);
        assert_eq!(chunk.text, ztoken.semantic.gloss);
        assert_eq!(chunk.dimension, "semantic");
        assert_eq!(chunk.span_start, ztoken.lexical.span.start_byte as u32);
        assert_eq!(chunk.span_end, ztoken.lexical.span.end_byte as u32);
    }
}

/// Chunk IDs must follow the `rag:<provider>/<sha256>/<ztoken_id>` format.
#[cfg(feature = "rag")]
#[test]
fn chunk_id_format_is_correct() {
    let artifact = compile_sample();
    let chunker = Chunker::new("mock-embed-v1");
    let chunks = chunker.chunk_artifact(&artifact);

    for chunk in &chunks {
        let expected = format!(
            "rag:{}/{}/{}",
            "mock-embed-v1", artifact.source.sha256, chunk.ztoken_id
        );
        assert_eq!(
            chunk.chunk_id, expected,
            "chunk_id format mismatch for token '{}'",
            chunk.ztoken_id
        );
    }
}

/// MemoryVectorStore: insert + search round-trip.
#[cfg(feature = "rag")]
#[test]
fn memory_store_insert_and_search() {
    let dims = 4usize;
    let mut store = MemoryVectorStore::new(dims);
    assert!(store.is_empty());

    let chunk = Chunk {
        chunk_id: "rag:mock/abc123/z1".to_string(),
        ztoken_id: "z1".to_string(),
        artifact_sha256: "abc123".to_string(),
        source_path: None,
        dimension: "semantic".to_string(),
        text: "The System is a platform.".to_string(),
        span_start: 0,
        span_end: 25,
    };

    let embedding = vec![1.0f32, 0.0, 0.0, 0.0];
    store.insert(chunk.clone(), embedding.clone()).expect("insert should succeed");
    assert_eq!(store.len(), 1);

    // Search with the same vector — should return the chunk as top-1.
    let results = store.search(&embedding, 1).expect("search should succeed");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].chunk.chunk_id, chunk.chunk_id);
    assert!(
        (results[0].score - 1.0f32).abs() < 1e-6,
        "cosine similarity of identical vectors should be 1.0"
    );
}

/// MemoryVectorStore: delete_by_artifact removes all matching entries.
#[cfg(feature = "rag")]
#[test]
fn memory_store_delete_by_artifact() {
    let dims = 2usize;
    let mut store = MemoryVectorStore::new(dims);

    let make_chunk = |id: &str, sha: &str| Chunk {
        chunk_id: format!("rag:mock/{sha}/{id}"),
        ztoken_id: id.to_string(),
        artifact_sha256: sha.to_string(),
        source_path: None,
        dimension: "semantic".to_string(),
        text: "text".to_string(),
        span_start: 0,
        span_end: 4,
    };

    let emb = vec![1.0f32, 0.0];

    // Insert 3 chunks for artifact A and 2 for artifact B.
    for id in &["z1", "z2", "z3"] {
        store.insert(make_chunk(id, "artifact-a"), emb.clone()).unwrap();
    }
    for id in &["z1", "z2"] {
        store.insert(make_chunk(id, "artifact-b"), emb.clone()).unwrap();
    }
    assert_eq!(store.len(), 5);

    let deleted = store
        .delete_by_artifact("artifact-a")
        .expect("delete should succeed");
    assert_eq!(deleted, 3, "expected 3 entries deleted for artifact-a");
    assert_eq!(store.len(), 2, "2 entries for artifact-b must remain");
}
