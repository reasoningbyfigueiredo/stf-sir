//! Referential grounding checks (Ground predicate).
//!
//! Implements the grounding component of the Error triple in the coherence
//! paper (Definition E2 / Theorem A10):
//!
//!   Ground(x, W) = 0  ⟺  hallucination candidate
//!
//! A statement is considered grounded if it has at least one of:
//!   - a non-empty `source_ids` set in its Provenance,
//!   - a non-empty `anchors` set in its Provenance, or
//!   - the `grounded` flag explicitly set to `true`.
//!
//! This maps directly to Δ-tracking in the STF-SIR token model.

use crate::model::statement::{SourceId, Statement};

/// The result of a grounding check for a single statement.
#[derive(Debug, Clone)]
pub struct GroundingResult {
    /// Whether the statement is considered grounded.
    pub is_grounded: bool,
    /// Anchors that were expected but missing.
    pub missing_anchors: Vec<String>,
    /// Source ids that were matched.
    pub matched_sources: Vec<SourceId>,
}

impl GroundingResult {
    pub fn grounded(matched_sources: Vec<SourceId>) -> Self {
        Self {
            is_grounded: true,
            missing_anchors: vec![],
            matched_sources,
        }
    }

    pub fn ungrounded() -> Self {
        Self {
            is_grounded: false,
            missing_anchors: vec!["missing_source_anchor".into()],
            matched_sources: vec![],
        }
    }
}

/// Trait for grounding checkers.
///
/// Implementations decide whether a statement has adequate referential
/// grounding in the source artefact or world model.
pub trait GroundingChecker {
    fn check_grounding(&self, stmt: &Statement) -> GroundingResult;
}

// ---------------------------------------------------------------------------

/// Checks grounding via the Provenance fields of a Statement.
///
/// A statement is grounded iff it has at least one source id, at least one
/// anchor, or the `grounded` flag is `true`.
pub struct ProvenanceGroundingChecker;

impl GroundingChecker for ProvenanceGroundingChecker {
    fn check_grounding(&self, stmt: &Statement) -> GroundingResult {
        let p = &stmt.provenance;
        let has_grounding =
            !p.source_ids.is_empty() || !p.anchors.is_empty() || p.grounded;

        if has_grounding {
            GroundingResult::grounded(p.source_ids.iter().cloned().collect())
        } else {
            GroundingResult::ungrounded()
        }
    }
}
