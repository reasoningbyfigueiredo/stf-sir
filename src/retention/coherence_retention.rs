//! Coherence-aware retention: extends the existing ρ vector with lexical
//! coherence dimensions derived from domain mapping results.
//!
//! Definition LC2 of the coherence paper:
//!
//!   φ is valid  ⟺  ρ(φ(S)) ≥ θ
//!
//! `CoherenceRetention` computes four sub-scores:
//!   - `rho`: overall retention (from MappingResult)
//!   - `lexical_preservation`: structure_preserved flag → 1.0 or 0.5
//!   - `structural_preservation`: structure_preserved flag → 1.0 or 0.0
//!   - `grounding_preservation`: 1.0 - semantic_drift_score

use crate::compiler::domain::MappingResult;

/// Per-transformation retention scores for coherence-preserving domain maps.
#[derive(Debug, Clone, PartialEq)]
pub struct CoherenceRetention {
    /// Overall retention ρ = MappingResult.retention_score.
    pub rho: f32,
    /// Score for lexical preservation.
    pub lexical_preservation: f32,
    /// Score for structural (logical / syntactic) preservation.
    pub structural_preservation: f32,
    /// Score for grounding preservation (1.0 - semantic_drift).
    pub grounding_preservation: f32,
}

impl CoherenceRetention {
    /// Returns `true` if overall retention meets the given threshold θ.
    pub fn is_valid(&self, theta: f32) -> bool {
        self.rho >= theta
    }

    /// Scalar coherence score: geometric mean of all four dimensions.
    pub fn scalar(&self) -> f32 {
        (self.rho
            * self.lexical_preservation
            * self.structural_preservation
            * self.grounding_preservation)
            .powf(0.25)
    }
}

impl From<&MappingResult> for CoherenceRetention {
    fn from(value: &MappingResult) -> Self {
        Self {
            rho: value.retention_score,
            lexical_preservation: if value.structure_preserved { 1.0 } else { 0.5 },
            structural_preservation: if value.structure_preserved { 1.0 } else { 0.0 },
            grounding_preservation: 1.0 - value.semantic_drift_score.clamp(0.0, 1.0),
        }
    }
}
