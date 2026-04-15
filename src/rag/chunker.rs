//! Chunk serializer — converts ZTokens to embeddable chunks (feature-gated: `rag`).

#[cfg(feature = "rag")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "rag")]
use crate::model::Artifact;

#[cfg(feature = "rag")]
/// An embeddable chunk derived from a single ZToken.
///
/// Every chunk carries full provenance so a retrieved chunk can be traced back
/// to its source span in the original document (INV-206-1).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chunk {
    /// Globally unique chunk identifier.
    /// Format: `rag:<provider_id>/<artifact_sha256>/<ztoken_id>`
    pub chunk_id: String,

    /// The ZToken this chunk was derived from.
    pub ztoken_id: String,

    /// SHA-256 of the source document.
    pub artifact_sha256: String,

    /// Optional path to the source file.
    pub source_path: Option<String>,

    /// Which semantic dimension was embedded (`"semantic"` for Σ.gloss).
    pub dimension: String,

    /// The text that was (or will be) embedded.
    pub text: String,

    /// Start byte offset in the original source.
    pub span_start: u32,

    /// End byte offset in the original source.
    pub span_end: u32,
}

#[cfg(feature = "rag")]
/// Converts an `Artifact` into a `Vec<Chunk>` — one chunk per ZToken.
pub struct Chunker {
    pub provider_id: String,
}

#[cfg(feature = "rag")]
impl Chunker {
    /// Create a new chunker bound to a specific embedding provider.
    pub fn new(provider_id: impl Into<String>) -> Self {
        Self {
            provider_id: provider_id.into(),
        }
    }

    /// Produce one chunk per ZToken in the artifact.
    ///
    /// Chunks are ordered by ZToken position (as they appear in `artifact.ztokens`).
    /// The embedded text is `Σ.gloss` (normalised, language-model-friendly).
    pub fn chunk_artifact(&self, artifact: &Artifact) -> Vec<Chunk> {
        artifact
            .ztokens
            .iter()
            .map(|zt| {
                let chunk_id = Self::chunk_id(
                    &self.provider_id,
                    &artifact.source.sha256,
                    &zt.id,
                );
                Chunk {
                    chunk_id,
                    ztoken_id: zt.id.clone(),
                    artifact_sha256: artifact.source.sha256.clone(),
                    source_path: artifact.source.path.clone(),
                    dimension: "semantic".to_string(),
                    text: zt.semantic.gloss.clone(),
                    span_start: zt.lexical.span.start_byte as u32,
                    span_end: zt.lexical.span.end_byte as u32,
                }
            })
            .collect()
    }

    /// Build the canonical chunk ID for a given (provider, artifact, token) triple.
    pub fn chunk_id(provider_id: &str, artifact_sha256: &str, ztoken_id: &str) -> String {
        format!("rag:{}/{}/{}", provider_id, artifact_sha256, ztoken_id)
    }
}
