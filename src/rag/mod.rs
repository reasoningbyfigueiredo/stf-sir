//! RAG (Retrieval-Augmented Generation) integration for STF-SIR (EPIC-206).
//!
//! All items in this module are feature-gated with `#[cfg(feature = "rag")]`.
//! Compiling without `--features rag` produces zero embedding-related code.

#[cfg(feature = "rag")]
pub mod chunker;
#[cfg(feature = "rag")]
pub mod embedding;
#[cfg(feature = "rag")]
pub mod memory_store;
#[cfg(feature = "rag")]
pub mod store;

#[cfg(feature = "rag")]
pub use chunker::{Chunk, Chunker};
#[cfg(feature = "rag")]
pub use embedding::{EmbeddingError, EmbeddingProvider, MockEmbeddingProvider};
#[cfg(feature = "rag")]
pub use memory_store::MemoryVectorStore;
#[cfg(feature = "rag")]
pub use store::{SearchResult, StoreError, VectorStore};
