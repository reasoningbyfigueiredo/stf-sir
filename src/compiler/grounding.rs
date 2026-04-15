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
use crate::sir::SirGraph;

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

// ---------------------------------------------------------------------------

/// SIR-graph-backed grounding checker.
///
/// A statement is structurally grounded when its `id` resolves to a node in
/// the `SirGraph` — i.e., it was compiled from a real ZToken in the source
/// artefact.  This eliminates the heuristic provenance check for statements
/// that originate from a compiled artefact.
///
/// For statements that were not compiled from an artefact (e.g., derived or
/// axiom statements), falls back to `ProvenanceGroundingChecker`.
pub struct SirGroundingChecker<'a> {
    /// The SIR graph for the artefact that produced the theory.
    pub graph: &'a SirGraph,
}

impl<'a> GroundingChecker for SirGroundingChecker<'a> {
    fn check_grounding(&self, stmt: &Statement) -> GroundingResult {
        // Primary: graph membership — the statement was compiled from a ZToken.
        if self.graph.node(&stmt.id).is_some() {
            return GroundingResult::grounded(
                stmt.provenance.source_ids.iter().cloned().collect(),
            );
        }

        // Fallback: provenance-based grounding (for axioms, derived statements, etc.).
        let p = &stmt.provenance;
        if !p.source_ids.is_empty() || !p.anchors.is_empty() || p.grounded {
            GroundingResult::grounded(p.source_ids.iter().cloned().collect())
        } else {
            GroundingResult::ungrounded()
        }
    }
}
