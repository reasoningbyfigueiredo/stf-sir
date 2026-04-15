//! External enricher protocol v1 — JSON-over-stdin/stdout, language-agnostic.
//!
//! Defines the request/response wire types and host-side helpers for spawning
//! external enrichers written in any language (Python, TypeScript, etc.).
//!
//! Actual subprocess execution is **opt-in** (call `run` when you have a live
//! process). For tests, use `build_request` + `apply_response` directly.

use serde::{Deserialize, Serialize};

use crate::model::Artifact;

// ── Wire types ────────────────────────────────────────────────────────────────

/// Request sent to an external enricher on stdin (NDJSON line).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnricherRequest {
    /// Must be `"stf-sir-enricher-v1"`.
    pub protocol: String,
    /// SHA-256 of the source document (from `artifact.source.sha256`).
    pub artifact_id: String,
    /// Tokens to be enriched.
    pub tokens: Vec<EnricherToken>,
}

/// Minimal token representation sent to an external enricher.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnricherToken {
    /// ZToken identifier.
    pub id: String,
    /// Semantic gloss (`Σ.gloss`).
    pub gloss: String,
    /// Syntactic node type (`S.node_type`).
    pub node_type: String,
    /// Current extension data for this token (enrichers may read but must not
    /// overwrite fields outside their namespace).
    pub extensions: serde_json::Value,
}

/// Response written by the external enricher to stdout (NDJSON line).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnricherResponse {
    /// Must echo `"stf-sir-enricher-v1"`.
    pub protocol: String,
    /// One entry per token that was enriched (may be a strict subset).
    pub enrichments: Vec<TokenEnrichment>,
}

/// Per-token enrichment data returned by an external enricher.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenEnrichment {
    /// Must match a `ZToken.id` from the request.
    pub token_id: String,
    /// Extension data to merge into `token.extensions[namespace]`.
    pub extensions: serde_json::Value,
}

// ── Host-side adapter ────────────────────────────────────────────────────────

/// Host-side adapter that manages an external enricher subprocess.
///
/// Construct with `new(...)` then either:
/// - call `build_request` + `apply_response` for unit-testable dry-run usage, or
/// - call `run` (when you hold a live child process) for full round-trip.
#[allow(dead_code)]
pub struct ExternalEnricher {
    pub name: String,
    pub namespace: String,
    pub version: String,
    /// Command and arguments to spawn the enricher process.
    /// Example: `["python3", "concept_extractor.py"]`
    pub command: Vec<String>,
}

impl ExternalEnricher {
    /// Create a new external enricher descriptor.
    pub fn new(
        name: impl Into<String>,
        namespace: impl Into<String>,
        version: impl Into<String>,
        command: Vec<String>,
    ) -> Self {
        Self {
            name: name.into(),
            namespace: namespace.into(),
            version: version.into(),
            command,
        }
    }

    /// Build the protocol request for an artifact — does not spawn any process.
    pub fn build_request(artifact: &Artifact) -> EnricherRequest {
        let tokens = artifact
            .ztokens
            .iter()
            .map(|zt| EnricherToken {
                id: zt.id.clone(),
                gloss: zt.semantic.gloss.clone(),
                node_type: zt.syntactic.node_type.clone(),
                extensions: serde_json::to_value(&zt.extensions)
                    .unwrap_or(serde_json::Value::Object(Default::default())),
            })
            .collect();

        EnricherRequest {
            protocol: "stf-sir-enricher-v1".to_string(),
            artifact_id: artifact.source.sha256.clone(),
            tokens,
        }
    }

    /// Apply an enricher response to an artifact, merging each
    /// `TokenEnrichment.extensions` into `token.extensions[namespace]`.
    ///
    /// Tokens not present in the response are left unchanged.
    pub fn apply_response(artifact: &mut Artifact, response: &EnricherResponse, namespace: &str) {
        for enrichment in &response.enrichments {
            if let Some(token) = artifact
                .ztokens
                .iter_mut()
                .find(|t| t.id == enrichment.token_id)
            {
                let value = serde_yaml_ng::to_value(&enrichment.extensions)
                    .unwrap_or(serde_yaml_ng::Value::Null);
                token.extensions.insert(namespace.to_string(), value);
            }
        }
    }
}
