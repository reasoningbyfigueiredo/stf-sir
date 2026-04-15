use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::model::Artifact;

use super::semantic::{semantic_diff, SemanticDiff};
use super::structural::{structural_diff, StructuralDiff};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffSummary {
    pub added_tokens: usize,
    pub removed_tokens: usize,
    pub modified_tokens: usize,
    pub added_relations: usize,
    pub removed_relations: usize,
    pub is_identical: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffReport {
    pub format: String,
    pub artifact_a: String,
    pub artifact_b: String,
    pub structural: StructuralDiff,
    pub semantic: SemanticDiff,
    pub summary: DiffSummary,
}

impl DiffReport {
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).expect("DiffReport is always serializable")
    }

    pub fn to_yaml(&self) -> String {
        serde_yaml_ng::to_string(self).expect("DiffReport is always serializable")
    }
}

/// Compute the SHA-256 of an artifact's serialized YAML representation.
fn artifact_sha256(artifact: &Artifact) -> String {
    // Use the pre-computed sha256 from the artifact's source info when it
    // accurately represents a stable identity for the *compiled* artifact.
    // For diff purposes we use the artifact's own source SHA-256 so that
    // identical compilations produce identical IDs.
    let bytes = serde_json::to_vec(artifact).unwrap_or_default();
    let digest = Sha256::digest(&bytes);
    format!("sha256:{digest:x}")
}

/// Build a complete diff report between two artifacts.
pub fn diff_artifacts(a: &Artifact, b: &Artifact) -> DiffReport {
    let structural = structural_diff(a, b);
    let semantic = semantic_diff(a, b);

    let added_tokens = structural.added_tokens.len();
    let removed_tokens = structural.removed_tokens.len();
    let modified_tokens = structural.modified_node_types.len()
        + semantic.gloss_changes.len()
        + semantic.concept_changes.len();
    let added_relations = structural.added_relations.len();
    let removed_relations = structural.removed_relations.len();

    let is_identical = added_tokens == 0
        && removed_tokens == 0
        && modified_tokens == 0
        && added_relations == 0
        && removed_relations == 0;

    let summary = DiffSummary {
        added_tokens,
        removed_tokens,
        modified_tokens,
        added_relations,
        removed_relations,
        is_identical,
    };

    DiffReport {
        format: "stf-sir-diff-v1".to_string(),
        artifact_a: artifact_sha256(a),
        artifact_b: artifact_sha256(b),
        structural,
        semantic,
        summary,
    }
}
