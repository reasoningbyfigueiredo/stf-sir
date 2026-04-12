use std::collections::BTreeSet;

use crate::compiler::semantic::normalize_text;
use crate::model::{Artifact, RelationCategory};

const PIPELINE_STAGES: &[&str] = &["lexical", "syntactic", "semantic", "logical"];

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RetentionVector {
    pub rho_l: f64,
    pub rho_s: f64,
    pub rho_sigma: f64,
    pub rho_phi: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RetentionScore {
    pub satisfied: usize,
    pub total: usize,
}

impl RetentionScore {
    pub fn value(self) -> f64 {
        if self.total == 0 {
            1.0
        } else {
            self.satisfied as f64 / self.total as f64
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionBaseline {
    pub vector: RetentionVector,
    pub lexical: RetentionScore,
    pub syntactic: RetentionScore,
    pub semantic: RetentionScore,
    pub logical: RetentionScore,
}

impl RetentionBaseline {
    pub fn from_artifact(artifact: &Artifact) -> Self {
        let lexical = lexical_score(artifact);
        let syntactic = syntactic_score(artifact);
        let semantic = semantic_score(artifact);
        let logical = logical_score(artifact);

        Self {
            vector: RetentionVector {
                rho_l: lexical.value(),
                rho_s: syntactic.value(),
                rho_sigma: semantic.value(),
                rho_phi: logical.value(),
            },
            lexical,
            syntactic,
            semantic,
            logical,
        }
    }
}

fn lexical_score(artifact: &Artifact) -> RetentionScore {
    let total = artifact.ztokens.len();
    let satisfied = artifact
        .ztokens
        .iter()
        .filter(|token| {
            let span = &token.lexical.span;
            !token.lexical.source_text.is_empty()
                && span.start_byte < span.end_byte
                && span.end_byte <= artifact.source.length_bytes
                && span.start_line >= 1
                && span.start_line <= span.end_line
                && token.lexical.normalized_text == normalize_text(&token.lexical.plain_text)
        })
        .count();

    RetentionScore { satisfied, total }
}

fn syntactic_score(artifact: &Artifact) -> RetentionScore {
    let total = artifact.ztokens.len();
    let token_ids = artifact
        .ztokens
        .iter()
        .map(|token| token.id.as_str())
        .collect::<BTreeSet<_>>();

    let satisfied = artifact
        .ztokens
        .iter()
        .filter(|token| {
            let syntactic = &token.syntactic;
            !syntactic.node_type.is_empty()
                && !syntactic.path.is_empty()
                && match syntactic.parent_id.as_deref() {
                    Some(parent_id) => token_ids.contains(parent_id),
                    None => syntactic.depth == 0,
                }
        })
        .count();

    RetentionScore { satisfied, total }
}

fn semantic_score(artifact: &Artifact) -> RetentionScore {
    let total = artifact.ztokens.len();
    let satisfied = artifact
        .ztokens
        .iter()
        .filter(|token| {
            if token.lexical.plain_text.is_empty() {
                token.semantic.gloss.is_empty()
            } else {
                token.semantic.gloss == token.lexical.normalized_text
            }
        })
        .count();

    RetentionScore { satisfied, total }
}

fn logical_score(artifact: &Artifact) -> RetentionScore {
    let token_ids = artifact
        .ztokens
        .iter()
        .map(|token| token.id.as_str())
        .collect::<BTreeSet<_>>();
    let relation_ids = artifact
        .relations
        .iter()
        .map(|relation| relation.id.as_str())
        .collect::<BTreeSet<_>>();

    let relation_total = artifact.relations.len();
    let relation_satisfied = artifact
        .relations
        .iter()
        .filter(|relation| {
            !relation.id.is_empty()
                && !relation.type_.is_empty()
                && token_ids.contains(relation.source.as_str())
                && (token_ids.contains(relation.target.as_str())
                    || matches!(relation.category, RelationCategory::SemanticLink))
                && PIPELINE_STAGES.contains(&relation.stage.as_str())
        })
        .count();

    let phi_total = artifact
        .ztokens
        .iter()
        .map(|token| token.logical.relation_ids.len())
        .sum::<usize>();
    let phi_satisfied = artifact
        .ztokens
        .iter()
        .flat_map(|token| token.logical.relation_ids.iter())
        .filter(|relation_id| relation_ids.contains(relation_id.as_str()))
        .count();

    RetentionScore {
        satisfied: relation_satisfied + phi_satisfied,
        total: relation_total + phi_total,
    }
}
