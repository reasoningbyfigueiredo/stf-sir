//! In-memory VectorStore implementation using cosine similarity (feature-gated: `rag`).

#[cfg(feature = "rag")]
use super::chunker::Chunk;
#[cfg(feature = "rag")]
use super::store::{SearchResult, StoreError, VectorStore};

#[cfg(feature = "rag")]
/// A simple in-memory vector store that performs brute-force cosine similarity search.
///
/// Suitable for development, testing, and small corpora. For large-scale production
/// use, replace with a purpose-built store (Qdrant, Pinecone, etc.) via the
/// `VectorStore` trait.
pub struct MemoryVectorStore {
    /// Expected dimensionality of all stored vectors.
    pub dimensions: usize,
    entries: Vec<(Chunk, Vec<f32>)>,
}

#[cfg(feature = "rag")]
impl MemoryVectorStore {
    /// Create a new empty store that expects vectors of `dimensions` elements.
    pub fn new(dimensions: usize) -> Self {
        Self {
            dimensions,
            entries: Vec::new(),
        }
    }

    /// Cosine similarity between two equal-length slices.
    ///
    /// Returns `0.0` if either vector is all-zero (avoids division by zero).
    fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
        let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm_a == 0.0 || norm_b == 0.0 {
            0.0
        } else {
            dot / (norm_a * norm_b)
        }
    }
}

#[cfg(feature = "rag")]
impl VectorStore for MemoryVectorStore {
    fn insert(&mut self, chunk: Chunk, embedding: Vec<f32>) -> Result<(), StoreError> {
        if embedding.len() != self.dimensions {
            return Err(StoreError::DimensionMismatch {
                expected: self.dimensions,
                got: embedding.len(),
            });
        }
        self.entries.push((chunk, embedding));
        Ok(())
    }

    fn search(
        &self,
        query_embedding: &[f32],
        top_k: usize,
    ) -> Result<Vec<SearchResult>, StoreError> {
        if query_embedding.len() != self.dimensions {
            return Err(StoreError::DimensionMismatch {
                expected: self.dimensions,
                got: query_embedding.len(),
            });
        }

        let mut scored: Vec<SearchResult> = self
            .entries
            .iter()
            .map(|(chunk, emb)| SearchResult {
                chunk: chunk.clone(),
                score: Self::cosine_similarity(query_embedding, emb),
            })
            .collect();

        // Sort descending by score; ties broken by chunk_id for determinism.
        scored.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.chunk.chunk_id.cmp(&b.chunk.chunk_id))
        });

        scored.truncate(top_k);
        Ok(scored)
    }

    fn delete_by_artifact(&mut self, artifact_sha256: &str) -> Result<usize, StoreError> {
        let before = self.entries.len();
        self.entries
            .retain(|(chunk, _)| chunk.artifact_sha256 != artifact_sha256);
        Ok(before - self.entries.len())
    }

    fn len(&self) -> usize {
        self.entries.len()
    }
}
