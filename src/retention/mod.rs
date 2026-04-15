pub mod coherence_retention;

pub use coherence_retention::CoherenceRetention;

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

impl RetentionVector {
    /// The minimum of the four retention dimensions.
    ///
    /// `rho_alert` exposes dimensional collapse that the geometric mean can mask.
    /// For example, `(1.0, 1.0, 0.5, 0.5)` has a geometric mean of 0.707 but
    /// `rho_alert = 0.5`.  A collapsed `ρ_Φ` eliminates logical relations that
    /// underpin artefact coherence.
    ///
    /// # Invariant
    /// `rho_alert() == min(rho_l, rho_s, rho_sigma, rho_phi)`
    pub fn rho_alert(&self) -> f64 {
        self.rho_l
            .min(self.rho_s)
            .min(self.rho_sigma)
            .min(self.rho_phi)
    }

    /// Returns `true` when `rho_alert() < threshold` (dimensional collapse detected).
    ///
    /// Use [`RetentionScore::DEFAULT_THRESHOLD`] for the calibrated default.
    pub fn is_unsafe(&self, threshold: f64) -> bool {
        self.rho_alert() < threshold
    }
}

/// Unified retention vector combining artifact-level and coherence-level scores.
///
/// Distinct from `RetentionVector` (which tracks pipeline-stage retention) —
/// this structure tracks retention across the three semantic layers described
/// in the coherence paper:
///
/// - `artifact`: geometric mean of `RetentionVector` dimensions (ρ_l · ρ_s · ρ_σ · ρ_φ)^¼
/// - `lexical`: lexical coherence preservation across domain mappings
/// - `coherence`: coherence triple (C_l, C_c, C_o) preservation score
///
/// Use `UnifiedRetentionVector::scalar()` for a single composite score.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UnifiedRetentionVector {
    /// Artifact-layer retention: geometric mean of the four pipeline-stage scores.
    pub artifact: f64,
    /// Lexical-layer retention: preservation of lexical coherence across mappings.
    pub lexical: f64,
    /// Coherence-layer retention: preservation of the coherence triple.
    pub coherence: f64,
}

impl UnifiedRetentionVector {
    /// Scalar composite score: geometric mean of all three dimensions.
    pub fn scalar(&self) -> f64 {
        (self.artifact * self.lexical * self.coherence).powf(1.0 / 3.0)
    }
}

impl From<&RetentionVector> for UnifiedRetentionVector {
    /// Derive a `UnifiedRetentionVector` from a `RetentionVector`.
    ///
    /// `artifact` is set to the geometric mean of the four pipeline scores.
    /// `lexical` and `coherence` default to `1.0` (no degradation assumed
    /// unless domain-mapping or coherence data are provided separately).
    fn from(v: &RetentionVector) -> Self {
        let artifact = (v.rho_l * v.rho_s * v.rho_sigma * v.rho_phi).powf(0.25);
        Self { artifact, lexical: 1.0, coherence: 1.0 }
    }
}

/// Per-pipeline-stage retention counter (satisfied / total items).
///
/// Previously exported as `RetentionScore`; renamed to `PipelineScore` in v1.1
/// to free the name for the dimensional safety score added by EPIC-103.
/// The public API exports both names: `PipelineScore` (new) and `RetentionScore`
/// (the dimensional safety struct, also new in v1.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PipelineScore {
    pub satisfied: usize,
    pub total: usize,
}

impl PipelineScore {
    pub fn value(self) -> f64 {
        if self.total == 0 {
            1.0
        } else {
            self.satisfied as f64 / self.total as f64
        }
    }
}

/// Dimensional safety score wrapping a [`RetentionVector`] with an alert flag.
///
/// Unlike the geometric mean (which can report a high composite score even when
/// one dimension is in collapse), `RetentionScore` exposes the minimum dimension
/// (`rho_alert`) and raises `unsafe_flag` when it falls below `threshold`.
///
/// # Example
///
/// ```
/// use stf_sir::retention::{RetentionScore, RetentionVector};
///
/// let v = RetentionVector { rho_l: 1.0, rho_s: 1.0, rho_sigma: 0.8, rho_phi: 0.3 };
/// let score = RetentionScore::from_vector(v, RetentionScore::DEFAULT_THRESHOLD);
/// assert!(score.unsafe_flag);          // rho_phi=0.3 < 0.5
/// assert!(score.composite > 0.5);      // geometric mean masks the collapse
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct RetentionScore {
    /// The four pipeline-stage retention dimensions.
    pub vector: RetentionVector,
    /// Geometric mean of the four dimensions: `(ρ_l · ρ_s · ρ_σ · ρ_φ)^¼`.
    pub composite: f64,
    /// The minimum of the four dimensions — the alert indicator.
    pub rho_alert: f64,
    /// The configured safety threshold.
    pub threshold: f64,
    /// `true` when `rho_alert < threshold` (dimensional collapse detected).
    pub unsafe_flag: bool,
}

impl RetentionScore {
    /// Conservative safety threshold: any dimension below 0.5 triggers the alert.
    pub const DEFAULT_THRESHOLD: f64 = 0.5;

    /// Build a `RetentionScore` from a [`RetentionVector`] and a safety threshold.
    ///
    /// # Invariant
    /// `unsafe_flag == (rho_alert < threshold)` regardless of `composite`.
    pub fn from_vector(vector: RetentionVector, threshold: f64) -> Self {
        let rho_alert = vector.rho_alert();
        let composite =
            (vector.rho_l * vector.rho_s * vector.rho_sigma * vector.rho_phi).powf(0.25);
        Self {
            vector,
            composite,
            rho_alert,
            threshold,
            unsafe_flag: rho_alert < threshold,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionBaseline {
    pub vector: RetentionVector,
    pub lexical: PipelineScore,
    pub syntactic: PipelineScore,
    pub semantic: PipelineScore,
    pub logical: PipelineScore,
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

fn lexical_score(artifact: &Artifact) -> PipelineScore {
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

    PipelineScore { satisfied, total }
}

fn syntactic_score(artifact: &Artifact) -> PipelineScore {
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

    PipelineScore { satisfied, total }
}

fn semantic_score(artifact: &Artifact) -> PipelineScore {
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

    PipelineScore { satisfied, total }
}

fn logical_score(artifact: &Artifact) -> PipelineScore {
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

    PipelineScore {
        satisfied: relation_satisfied + phi_satisfied,
        total: relation_total + phi_total,
    }
}
