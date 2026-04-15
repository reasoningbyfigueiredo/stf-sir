//! EmbeddingProvider trait and mock implementation (feature-gated: `rag`).

#[cfg(feature = "rag")]
/// Trait for embedding text into dense vectors.
///
/// Object-safe: `dyn EmbeddingProvider + Send + Sync` is valid.
pub trait EmbeddingProvider: Send + Sync {
    /// Unique identifier for this model (e.g. `"text-embedding-3-small"`).
    fn model_id(&self) -> &str;

    /// Number of dimensions in the output vector.
    fn dimensions(&self) -> usize;

    /// Embed a batch of texts.
    ///
    /// Returns one vector per input text. Implementations MUST return vectors
    /// of exactly `self.dimensions()` elements; callers rely on this invariant.
    fn embed(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>, EmbeddingError>;
}

#[cfg(feature = "rag")]
/// Errors from embedding providers.
#[derive(Debug, thiserror::Error)]
pub enum EmbeddingError {
    #[error("Provider error: {0}")]
    Provider(String),

    #[error("Dimension mismatch: expected {expected}, got {got}")]
    DimensionMismatch { expected: usize, got: usize },
}

#[cfg(feature = "rag")]
/// Deterministic mock embedding provider for unit tests.
///
/// Returns a zero vector of `dimensions` elements for every input text.
/// This is sufficient for testing provenance, chunking, and store logic
/// without a real embedding API.
pub struct MockEmbeddingProvider {
    pub dimensions: usize,
}

#[cfg(feature = "rag")]
impl EmbeddingProvider for MockEmbeddingProvider {
    fn model_id(&self) -> &str {
        "mock-embed-v1"
    }

    fn dimensions(&self) -> usize {
        self.dimensions
    }

    fn embed(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>, EmbeddingError> {
        // Deterministic: every text maps to a zero vector of `dimensions` elements.
        // For a production mock you could hash text bytes → f32, but zero is
        // sufficient for structural / provenance tests.
        Ok(texts
            .iter()
            .map(|_| vec![0.0f32; self.dimensions])
            .collect())
    }
}
