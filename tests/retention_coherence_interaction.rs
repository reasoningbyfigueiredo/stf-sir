//! Exploratory tests for retention metrics and their interaction with the
//! coherence triple.
//!
//! Covers: CoherenceRetention from MappingResult, UnifiedRetentionVector
//! construction and scalar, boundary conditions, and misleading-score detection.

mod common;

use stf_sir::compiler::domain::{DomainMapper, IdentityDomainMapper, LexicalFailureTag};
use stf_sir::model::Statement;
use stf_sir::retention::{CoherenceRetention, RetentionVector, UnifiedRetentionVector};

// ---------------------------------------------------------------------------
// 1. IdentityDomainMapper: perfect retention

#[test]
fn identity_mapper_produces_perfect_retention() {
    let stmt = Statement::atomic("s1", "A -> B", "logic");
    let result = IdentityDomainMapper.map_statement(&stmt, "logic");

    assert_eq!(result.retention_score, 1.0);
    assert!(result.structure_preserved);
    assert_eq!(result.semantic_drift_score, 0.0);
    assert!(result.failure_tag().is_none());

    let cr = CoherenceRetention::from(&result);
    assert_eq!(cr.rho, 1.0);
    assert_eq!(cr.lexical_preservation, 1.0);
    assert_eq!(cr.structural_preservation, 1.0);
    assert_eq!(cr.grounding_preservation, 1.0);
    assert!((cr.scalar() - 1.0).abs() < 1e-6, "scalar must be 1.0 for perfect retention");
}

// ---------------------------------------------------------------------------
// 2. CoherenceRetention from MappingResult — boundary matrix

/// (rho, structure_preserved, drift) → expected failure_tag
const FAILURE_TAG_MATRIX: &[(f32, bool, f32, Option<LexicalFailureTag>)] = &[
    (1.0,  true,  0.0,  None),                              // perfect
    (0.05, false, 0.0,  Some(LexicalFailureTag::Collapse)),  // collapse: low rho + no structure
    (1.0,  true,  0.8,  Some(LexicalFailureTag::Drift)),     // high drift → Drift
    (0.4,  true,  0.0,  Some(LexicalFailureTag::Mask)),      // medium rho → Mask
    (0.9,  true,  0.4,  None),                               // drift ≤ 0.5 → no tag
];

#[test]
fn failure_tag_matrix() {
    for &(rho, structure, drift, expected) in FAILURE_TAG_MATRIX {
        let result = common::make_mapping("s", rho, structure, drift);
        let tag = result.failure_tag();
        assert_eq!(tag, expected,
            "failure_tag for (rho={rho}, structure={structure}, drift={drift}) must be {expected:?}");
    }
}

// ---------------------------------------------------------------------------
// 3. CoherenceRetention scalar is geometric mean

#[test]
fn coherence_retention_scalar_is_geometric_mean() {
    let cr = CoherenceRetention {
        rho: 0.5,
        lexical_preservation: 0.5,
        structural_preservation: 0.5,
        grounding_preservation: 0.5,
    };
    let expected = (0.5f32 * 0.5 * 0.5 * 0.5).powf(0.25);
    assert!((cr.scalar() - expected).abs() < 1e-6);
}

#[test]
fn coherence_retention_scalar_zero_if_any_dimension_zero() {
    let cr = CoherenceRetention {
        rho: 1.0,
        lexical_preservation: 1.0,
        structural_preservation: 0.0, // zero kills the product
        grounding_preservation: 1.0,
    };
    assert!((cr.scalar() - 0.0).abs() < 1e-6, "scalar must be 0 if any dimension is 0");
}

// ---------------------------------------------------------------------------
// 4. CoherenceRetention validity threshold

#[test]
fn validity_threshold_respected() {
    let cr = CoherenceRetention { rho: 0.7, lexical_preservation: 1.0,
        structural_preservation: 1.0, grounding_preservation: 1.0 };
    assert!(cr.is_valid(0.6), "0.7 >= 0.6 must be valid");
    assert!(cr.is_valid(0.7), "0.7 >= 0.7 must be valid (boundary)");
    assert!(!cr.is_valid(0.8), "0.7 < 0.8 must be invalid");
}

// ---------------------------------------------------------------------------
// 5. UnifiedRetentionVector from RetentionVector

#[test]
fn unified_from_perfect_retention_vector() {
    let rv = RetentionVector { rho_l: 1.0, rho_s: 1.0, rho_sigma: 1.0, rho_phi: 1.0 };
    let uv = UnifiedRetentionVector::from(&rv);
    assert!((uv.artifact - 1.0).abs() < 1e-9, "perfect rv → artifact=1.0");
    assert_eq!(uv.lexical, 1.0, "default lexical=1.0");
    assert_eq!(uv.coherence, 1.0, "default coherence=1.0");
    assert!((uv.scalar() - 1.0).abs() < 1e-9);
}

#[test]
fn unified_artifact_is_geometric_mean_of_four() {
    let rv = RetentionVector { rho_l: 0.8, rho_s: 0.9, rho_sigma: 0.7, rho_phi: 0.6 };
    let uv = UnifiedRetentionVector::from(&rv);
    let expected = (0.8 * 0.9 * 0.7 * 0.6f64).powf(0.25);
    assert!((uv.artifact - expected).abs() < 1e-9,
        "artifact must be geometric mean of the four rho values");
}

#[test]
fn unified_scalar_zero_if_any_dimension_zero() {
    let uv = UnifiedRetentionVector { artifact: 1.0, lexical: 0.0, coherence: 1.0 };
    assert!((uv.scalar() - 0.0).abs() < 1e-9, "scalar must be 0 if any dimension is 0");
}

#[test]
fn unified_scalar_in_unit_interval_for_valid_inputs() {
    let cases = [
        (1.0, 1.0, 1.0),
        (0.5, 0.5, 0.5),
        (0.0, 0.0, 0.0),
        (0.9, 0.8, 0.7),
    ];
    for (a, l, c) in cases {
        let uv = UnifiedRetentionVector { artifact: a, lexical: l, coherence: c };
        let s = uv.scalar();
        assert!((0.0..=1.0).contains(&s),
            "scalar {s} must be in [0,1] for ({a},{l},{c})");
    }
}

// ---------------------------------------------------------------------------
// 6. Low retention ≠ misleading "good score"

#[test]
fn collapse_mapping_scalar_is_near_zero() {
    // Structure not preserved, rho very low → CoherenceRetention scalar near zero.
    let result = common::make_mapping("s", 0.05, false, 0.0);
    let cr = CoherenceRetention::from(&result);
    // structural_preservation = 0.0 (structure not preserved) → scalar = 0
    assert_eq!(cr.structural_preservation, 0.0);
    assert!((cr.scalar() - 0.0).abs() < 1e-6, "collapse must give scalar=0");
}

#[test]
fn high_retention_vector_but_low_coherence_gives_poor_unified_scalar() {
    // Good pipeline scores, but coherence dimension is 0 → unified scalar = 0.
    let rv = RetentionVector { rho_l: 0.99, rho_s: 0.99, rho_sigma: 0.99, rho_phi: 0.99 };
    let mut uv = UnifiedRetentionVector::from(&rv);
    uv.coherence = 0.0; // coherence collapsed
    assert!((uv.scalar() - 0.0).abs() < 1e-9,
        "high artifact retention with zero coherence must give unified scalar=0");
}

// ---------------------------------------------------------------------------
// 7. Identity mapper preserves statement content

#[test]
fn identity_mapper_preserves_all_statement_fields() {
    let stmt = Statement::grounded("s1", "NOT A -> B", "logic", "sha:src");
    let result = IdentityDomainMapper.map_statement(&stmt, "other_domain");
    let mapped = &result.target_statement;

    assert_eq!(mapped.id, stmt.id, "id must be preserved");
    assert_eq!(mapped.text, stmt.text, "text must be preserved");
    // Domain is updated to target.
    assert_eq!(mapped.domain, "other_domain", "domain must be updated to target");
    // Provenance is preserved (provenance.grounded, source_ids, etc.)
    assert_eq!(mapped.provenance.grounded, stmt.provenance.grounded, "provenance.grounded preserved");
}
