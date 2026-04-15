//! Semantic dimensions of second order for a Statement or ZToken.
//!
//! Materializes the four C/P/Δ/Ω axes of the STS space as a first-class Rust struct.
//! See FEAT-201-5 and `docs/roadmap/epics/EPIC-201-spec-v2.md`.

use crate::model::coherence::{CoherenceVector, TruthValue};

/// Semantic dimensions of second order for a
/// [`Statement`][crate::model::statement::Statement] or ZToken.
///
/// Materializes the four STS axes C/P/Δ/Ω:
///
/// | Field | Axis | Status |
/// |---|---|---|
/// | `coherence` | C — contextual/coherence | implemented |
/// | `provenance_score` | P — pragmatic/provenance | implemented (heuristic) |
/// | `transformation_delta` | Δ — temporal/transformation | v1: always 0.0 |
/// | `execution_score` | Ω — coherence/execution | implemented (C_o proxy) |
///
/// # Aspirational note
///
/// `transformation_delta` and `execution_score` are approximations in v1.
/// Full implementation is deferred to EPIC-204 (semantic diff) and EPIC-207.
/// Each field is documented with its implementation status.
///
/// # Construction
///
/// Use [`SemanticDimensions::from_evaluation`] (available from the
/// `compiler` module) to construct from an engine evaluation result.
/// Or call [`SemanticDimensions::from_parts`] directly with the raw values.
#[derive(Debug, Clone, PartialEq)]
pub struct SemanticDimensions {
    /// C — coherence: the triple (C_l, C_c, C_o) evaluated by the
    /// [`StfEngine`][crate::compiler::RecommendedEngine].
    ///
    /// Implemented: `src/model/coherence.rs`, `src/compiler/engine.rs`.
    pub coherence: CoherenceVector,

    /// P — provenance: scalar score ∈ \[0.0, 1.0\] derived from
    /// [`Provenance`][crate::model::statement::Provenance].
    ///
    /// - `1.0` if `grounded == true`
    /// - `0.0` otherwise
    ///
    /// \[partially implemented — simple heuristic; full scoring deferred to EPIC-207\]
    pub provenance_score: f32,

    /// Δ — transformation: normalized semantic distance from the source.
    ///
    /// In v1: always `0.0` (no transformation — direct compilation from source).
    /// In v2: derived from SemanticDiff (EPIC-204) when available.
    ///
    /// **\[aspirational for v2\]**
    pub transformation_delta: f32,

    /// Ω — execution: operational executability score ∈ \[0.0, 1.0\].
    ///
    /// - `1.0` if C_o = `Satisfied`
    /// - `0.0` if C_o = `Violated`
    /// - `0.5` if C_o = `Unknown`
    ///
    /// \[partially implemented — based on C_o\]
    pub execution_score: f32,
}

impl SemanticDimensions {
    /// Constructs `SemanticDimensions` from raw component values.
    ///
    /// In v1, `transformation_delta` should always be `0.0`.
    /// Prefer [`from_evaluation`][crate::compiler::EvaluationResult] when
    /// an evaluation result is available.
    pub fn from_parts(
        coherence: CoherenceVector,
        grounded: bool,
        transformation_delta: f32,
    ) -> Self {
        let execution_score = match coherence.operational {
            TruthValue::Satisfied => 1.0,
            TruthValue::Violated => 0.0,
            TruthValue::Unknown => 0.5,
        };
        Self {
            coherence,
            provenance_score: if grounded { 1.0 } else { 0.0 },
            transformation_delta,
            execution_score,
        }
    }

    /// Returns `true` if all implemented dimensions indicate a healthy state.
    ///
    /// Healthy means:
    /// - `coherence.is_full()` — all three coherence components are `Satisfied`
    /// - `provenance_score >= 0.5` — statement is grounded
    /// - `execution_score >= 0.5` — operational coherence is not violated
    pub fn is_healthy(&self) -> bool {
        self.coherence.is_full()
            && self.provenance_score >= 0.5
            && self.execution_score >= 0.5
    }
}
