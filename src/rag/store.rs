//! VectorStore trait for pluggable vector storage backends (feature-gated: `rag`).

#[cfg(feature = "rag")]
use super::chunker::Chunk;

#[cfg(feature = "rag")]
/// Generic interface for a vector store backend.
///
/// Implementations must be `Send + Sync` so they can be shared across async tasks.
/// The reference implementation is `MemoryVectorStore`.
pub trait VectorStore: Send + Sync {
    /// Insert a chunk and its associated embedding vector.
    fn insert(&mut self, chunk: Chunk, embedding: Vec<f32>) -> Result<(), StoreError>;

    /// Retrieve the `top_k` most similar chunks to `query_embedding` (cosine similarity).
    fn search(
        &self,
        query_embedding: &[f32],
        top_k: usize,
    ) -> Result<Vec<SearchResult>, StoreError>;

    /// Delete all chunks belonging to `artifact_sha256`.
    ///
    /// Returns the number of entries deleted.
    fn delete_by_artifact(&mut self, artifact_sha256: &str) -> Result<usize, StoreError>;

    /// Total number of chunks currently stored.
    fn len(&self) -> usize;

    /// `true` when the store contains no chunks.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[cfg(feature = "rag")]
/// A single result from a similarity search, including the matched chunk and its score.
#[derive(Debug, Clone)]
pub struct SearchResult {
    /// The matched chunk (with full provenance).
    pub chunk: Chunk,

    /// Cosine similarity score in `[-1.0, 1.0]`.
    pub score: f32,
}

#[cfg(feature = "rag")]
/// Errors from vector store operations.
#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("Store error: {0}")]
    Internal(String),

    #[error("Dimension mismatch: expected {expected}, got {got}")]
    DimensionMismatch { expected: usize, got: usize },
}
