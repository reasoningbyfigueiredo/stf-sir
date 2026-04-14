//! Lexical coherence: cross-domain mapping (LC1 / LC2).
//!
//! Implements Definition LC1–LC2 of the coherence paper:
//!
//!   φ is lexically coherent  ⟺  Coh(S) = (1,1,1) ⟹ Coh(φ(S)) = (1,1,1)
//!   φ is operationally valid ⟺  ρ(φ(S)) ≥ θ
//!
//! The `DomainMapper` trait is the extension point; the `IdentityDomainMapper`
//! ships as a zero-cost baseline (same domain, ρ = 1.0).

use crate::model::statement::Statement;

/// Result of mapping a statement to a target domain.
#[derive(Debug, Clone)]
pub struct MappingResult {
    pub source_statement_id: String,
    /// The mapped statement in the target domain.
    pub target_statement: Statement,
    /// Retention score ρ ∈ [0.0, 1.0] measuring structural preservation.
    pub retention_score: f32,
    /// Whether the logical/syntactic structure of the source was preserved.
    pub structure_preserved: bool,
    /// Semantic drift score δ ∈ [0.0, 1.0].  0.0 = no drift.
    pub semantic_drift_score: f32,
}

impl MappingResult {
    /// The mapping failure class based on the error triple approximation.
    pub fn failure_tag(&self) -> Option<LexicalFailureTag> {
        if !self.structure_preserved && self.retention_score < 0.1 {
            Some(LexicalFailureTag::Collapse)
        } else if self.semantic_drift_score > 0.5 {
            Some(LexicalFailureTag::Drift)
        } else if self.retention_score < 0.5 {
            Some(LexicalFailureTag::Mask)
        } else {
            None
        }
    }
}

/// Lexical failure taxonomy from the coherence paper (§7 Lexical Coherence).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LexicalFailureTag {
    /// `LEX_COLLAPSE`: Source relations destroyed.  Error triple (0,-,-).
    Collapse,
    /// `LEX_DRIFT`: Meaning deviates without detectable contradiction.  (1,0,0).
    Drift,
    /// `LEX_SPLIT`: One proposition maps to multiple interpretations.  (1,1,0).
    Split,
    /// `LEX_MASK`: Mapping appears coherent but conceals incoherence.  (1,0,0).
    Mask,
}

impl std::fmt::Display for LexicalFailureTag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LexicalFailureTag::Collapse => write!(f, "LEX_COLLAPSE"),
            LexicalFailureTag::Drift => write!(f, "LEX_DRIFT"),
            LexicalFailureTag::Split => write!(f, "LEX_SPLIT"),
            LexicalFailureTag::Mask => write!(f, "LEX_MASK"),
        }
    }
}

/// Trait for cross-domain mappers.
///
/// Implementors translate a statement from its source domain into a target
/// domain, returning a `MappingResult` that includes the retention score.
pub trait DomainMapper {
    fn map_statement(&self, stmt: &Statement, target_domain: &str) -> MappingResult;
}

// ---------------------------------------------------------------------------

/// Identity mapper: the target domain is the same as the source.
/// ρ = 1.0, zero drift, structure fully preserved.
pub struct IdentityDomainMapper;

impl DomainMapper for IdentityDomainMapper {
    fn map_statement(&self, stmt: &Statement, target_domain: &str) -> MappingResult {
        let mut mapped = stmt.clone();
        mapped.domain = target_domain.to_string();

        MappingResult {
            source_statement_id: stmt.id.clone(),
            target_statement: mapped,
            retention_score: 1.0,
            structure_preserved: true,
            semantic_drift_score: 0.0,
        }
    }
}
