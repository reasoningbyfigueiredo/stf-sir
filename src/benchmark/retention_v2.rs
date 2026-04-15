use serde::{Deserialize, Serialize};

use crate::model::Artifact;

/// Six-component retention score (ρ_v2).
///
/// Components:
/// - `rho_l`:               lexical retention  — tokens with non-empty source_text and valid span
/// - `rho_s`:               syntactic retention — tokens with non-empty node_type
/// - `rho_sigma_gloss`:     semantic gloss retention — tokens whose gloss equals normalized_text
/// - `rho_sigma_concepts`:  semantic concepts retention — vacuously 1.0 when no concepts exist;
///                          otherwise the fraction of tokens that have at least one concept
/// - `rho_phi`:             logical retention — relations whose source and target IDs are valid
/// - `rho_corpus`:          corpus-level aggregate — geometric mean of the five artifact-level components
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionV2Score {
    pub rho_l: f64,
    pub rho_s: f64,
    pub rho_sigma_gloss: f64,
    pub rho_sigma_concepts: f64,
    pub rho_phi: f64,
    pub rho_corpus: f64,
}

impl RetentionV2Score {
    /// Compute ρ_v2 for a single artifact.
    ///
    /// `rho_corpus` is set to the geometric mean of the other five components,
    /// representing the corpus-level aggregate for a single-document corpus.
    pub fn compute(artifact: &Artifact) -> Self {
        let total = artifact.ztokens.len();

        let rho_l = if total == 0 {
            1.0
        } else {
            let satisfied = artifact
                .ztokens
                .iter()
                .filter(|t| {
                    !t.lexical.source_text.is_empty()
                        && t.lexical.span.start_byte < t.lexical.span.end_byte
                })
                .count();
            satisfied as f64 / total as f64
        };

        let rho_s = if total == 0 {
            1.0
        } else {
            let satisfied = artifact
                .ztokens
                .iter()
                .filter(|t| !t.syntactic.node_type.is_empty())
                .count();
            satisfied as f64 / total as f64
        };

        let rho_sigma_gloss = if total == 0 {
            1.0
        } else {
            let satisfied = artifact
                .ztokens
                .iter()
                .filter(|t| {
                    if t.lexical.plain_text.is_empty() {
                        t.semantic.gloss.is_empty()
                    } else {
                        t.semantic.gloss == t.lexical.normalized_text
                    }
                })
                .count();
            satisfied as f64 / total as f64
        };

        // ρ_Σ_concepts: vacuously 1.0 when the global concepts Vec is empty;
        // otherwise the fraction of tokens that carry at least one concept.
        let all_concepts_empty = artifact.ztokens.iter().all(|t| t.semantic.concepts.is_empty());
        let rho_sigma_concepts = if all_concepts_empty || total == 0 {
            1.0
        } else {
            let with_concepts = artifact
                .ztokens
                .iter()
                .filter(|t| !t.semantic.concepts.is_empty())
                .count();
            with_concepts as f64 / total as f64
        };

        // ρ_Φ: fraction of relations where both source and target token IDs exist.
        let token_id_set: std::collections::BTreeSet<&str> =
            artifact.ztokens.iter().map(|t| t.id.as_str()).collect();
        let rel_total = artifact.relations.len();
        let rho_phi = if rel_total == 0 {
            1.0
        } else {
            let satisfied = artifact
                .relations
                .iter()
                .filter(|r| {
                    token_id_set.contains(r.source.as_str())
                        && (token_id_set.contains(r.target.as_str())
                            || matches!(r.category, crate::model::RelationCategory::SemanticLink))
                })
                .count();
            satisfied as f64 / rel_total as f64
        };

        // ρ_corpus for a single artifact = geometric mean of the five artifact-level scores.
        let rho_corpus = (rho_l * rho_s * rho_sigma_gloss * rho_sigma_concepts * rho_phi)
            .powf(1.0 / 5.0);

        RetentionV2Score {
            rho_l,
            rho_s,
            rho_sigma_gloss,
            rho_sigma_concepts,
            rho_phi,
            rho_corpus,
        }
    }

    /// Composite score: geometric mean of all six components.
    pub fn composite(&self) -> f64 {
        (self.rho_l
            * self.rho_s
            * self.rho_sigma_gloss
            * self.rho_sigma_concepts
            * self.rho_phi
            * self.rho_corpus)
            .powf(1.0 / 6.0)
    }

    /// Returns `true` when every component is at least `threshold`.
    pub fn is_baseline_met(&self, threshold: f64) -> bool {
        self.rho_l >= threshold
            && self.rho_s >= threshold
            && self.rho_sigma_gloss >= threshold
            && self.rho_sigma_concepts >= threshold
            && self.rho_phi >= threshold
            && self.rho_corpus >= threshold
    }
}
