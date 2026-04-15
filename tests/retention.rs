use anyhow::{Context, Result};
use stf_sir::compiler;
use stf_sir::{RetentionScore, RetentionVector};

#[test]
fn canonical_sample_has_perfect_retention_baseline() -> Result<()> {
    let artifact = canonical_sample()?;
    let baseline = artifact.retention_baseline();

    assert_unit_interval(baseline.vector);
    assert_eq!(baseline.vector.rho_l, 1.0);
    assert_eq!(baseline.vector.rho_s, 1.0);
    assert_eq!(baseline.vector.rho_sigma, 1.0);
    assert_eq!(baseline.vector.rho_phi, 1.0);

    Ok(())
}

#[test]
fn empty_document_is_vacuously_complete() -> Result<()> {
    let artifact = compiler::compile_markdown("", None)?;
    let baseline = artifact.retention_baseline();

    assert_unit_interval(baseline.vector);
    assert_eq!(baseline.vector.rho_l, 1.0);
    assert_eq!(baseline.vector.rho_s, 1.0);
    assert_eq!(baseline.vector.rho_sigma, 1.0);
    assert_eq!(baseline.vector.rho_phi, 1.0);

    Ok(())
}

#[test]
fn broken_gloss_reduces_rho_sigma() -> Result<()> {
    let mut artifact = canonical_sample()?;
    let token = artifact
        .ztokens
        .get_mut(0)
        .context("canonical sample should contain a first token")?;
    token.semantic.gloss = "mismatched".to_string();

    let baseline = artifact.retention_baseline();

    assert_unit_interval(baseline.vector);
    assert!(baseline.vector.rho_sigma < 1.0);
    assert_eq!(baseline.vector.rho_l, 1.0);
    assert_eq!(baseline.vector.rho_s, 1.0);
    assert_eq!(baseline.vector.rho_phi, 1.0);

    Ok(())
}

#[test]
fn broken_lexical_field_reduces_rho_l() -> Result<()> {
    let mut artifact = canonical_sample()?;
    let token = artifact
        .ztokens
        .get_mut(0)
        .context("canonical sample should contain a first token")?;
    token.lexical.source_text.clear();

    let baseline = artifact.retention_baseline();

    assert_unit_interval(baseline.vector);
    assert!(baseline.vector.rho_l < 1.0);
    assert_eq!(baseline.vector.rho_s, 1.0);
    assert_eq!(baseline.vector.rho_sigma, 1.0);
    assert_eq!(baseline.vector.rho_phi, 1.0);

    Ok(())
}

#[test]
fn broken_relation_target_reduces_rho_phi() -> Result<()> {
    let mut artifact = canonical_sample()?;
    let relation = artifact
        .relations
        .get_mut(0)
        .context("canonical sample should contain a first relation")?;
    relation.target = "z_missing".to_string();

    let baseline = artifact.retention_baseline();

    assert_unit_interval(baseline.vector);
    assert!(baseline.vector.rho_phi < 1.0);
    assert_eq!(baseline.vector.rho_l, 1.0);
    assert_eq!(baseline.vector.rho_s, 1.0);
    assert_eq!(baseline.vector.rho_sigma, 1.0);

    Ok(())
}

#[test]
fn broken_parent_reference_reduces_rho_s() -> Result<()> {
    let mut artifact = compiler::compile_markdown("# Root\n\n- one\n", None)?;
    let list_item = artifact
        .ztokens
        .iter_mut()
        .find(|token| token.syntactic.node_type == "list_item")
        .context("expected list_item token")?;
    list_item.syntactic.parent_id = Some("z_missing".to_string());

    let baseline = artifact.retention_baseline();

    assert_unit_interval(baseline.vector);
    assert!(baseline.vector.rho_s < 1.0);
    assert_eq!(baseline.vector.rho_l, 1.0);
    assert_eq!(baseline.vector.rho_sigma, 1.0);
    assert_eq!(baseline.vector.rho_phi, 1.0);

    Ok(())
}

#[test]
fn broken_phi_reference_reduces_rho_phi_without_affecting_other_dimensions() -> Result<()> {
    let mut artifact = canonical_sample()?;
    let token = artifact
        .ztokens
        .get_mut(0)
        .context("canonical sample should contain a first token")?;
    token.logical.relation_ids = vec!["r_missing".to_string()];

    let baseline = artifact.retention_baseline();

    assert_unit_interval(baseline.vector);
    assert!(baseline.vector.rho_phi < 1.0);
    assert_eq!(baseline.vector.rho_l, 1.0);
    assert_eq!(baseline.vector.rho_s, 1.0);
    assert_eq!(baseline.vector.rho_sigma, 1.0);

    Ok(())
}

fn canonical_sample() -> Result<stf_sir::model::Artifact> {
    compiler::compile_markdown(
        "# AI is transforming software development\n\nSemantic tokenization preserves meaning across structure.",
        None,
    )
    .map_err(Into::into)
}

fn assert_unit_interval(vector: RetentionVector) {
    for value in [vector.rho_l, vector.rho_s, vector.rho_sigma, vector.rho_phi] {
        assert!(
            (0.0..=1.0).contains(&value),
            "retention component {value} was outside [0, 1]"
        );
    }
}

// ---------------------------------------------------------------------------
// EPIC-103 — rho_alert and RetentionScore tests
// ---------------------------------------------------------------------------

fn vec(rho_l: f64, rho_s: f64, rho_sigma: f64, rho_phi: f64) -> RetentionVector {
    RetentionVector { rho_l, rho_s, rho_sigma, rho_phi }
}

// UT-103-1: rho_alert returns the minimum dimension
#[test]
fn rho_alert_is_minimum() {
    assert_eq!(vec(1.0, 0.3, 1.0, 1.0).rho_alert(), 0.3);
    assert_eq!(vec(0.1, 0.5, 0.8, 0.9).rho_alert(), 0.1);
    assert_eq!(vec(1.0, 1.0, 1.0, 0.0).rho_alert(), 0.0);
    assert_eq!(vec(0.7, 0.7, 0.7, 0.7).rho_alert(), 0.7);
}

// UT-103-2: collapsed dimension triggers unsafe_flag
#[test]
fn rho_alert_triggers_on_collapsed_dimension() {
    let score = RetentionScore::from_vector(vec(1.0, 1.0, 1.0, 0.0), 0.5);
    assert!(score.unsafe_flag, "rho_phi=0.0 must trigger unsafe_flag");
    assert_eq!(score.rho_alert, 0.0);
}

// UT-103-3: geometric mean does not mask alert (INV-103-2)
#[test]
fn geometric_mean_does_not_mask_alert() {
    let v = vec(1.0, 0.8, 0.8, 0.3);
    let score = RetentionScore::from_vector(v, 0.5);
    assert!(score.unsafe_flag, "rho_phi=0.3 < threshold=0.5 → unsafe_flag must be true");
    // Geometric mean: (1.0 * 0.8 * 0.8 * 0.3)^0.25 ≈ 0.635 > 0.5
    assert!(
        score.composite > 0.5,
        "composite ({:.4}) must be above threshold — mask would hide the collapse",
        score.composite
    );
}

// UT-103-4: all dimensions above threshold → safe
#[test]
fn retention_score_safe_when_all_above_threshold() {
    let score = RetentionScore::from_vector(vec(0.9, 0.9, 0.8, 0.7), 0.5);
    assert!(!score.unsafe_flag, "all dims > 0.5 → unsafe_flag must be false");
}

// UT-103-5: rho_alert is always in [0.0, 1.0] for valid input
#[test]
fn rho_alert_in_unit_interval() {
    for v in [
        vec(0.0, 0.0, 0.0, 0.0),
        vec(1.0, 1.0, 1.0, 1.0),
        vec(0.5, 0.3, 0.8, 0.1),
    ] {
        let alert = v.rho_alert();
        assert!(
            (0.0..=1.0).contains(&alert),
            "rho_alert={alert} is outside [0.0, 1.0]"
        );
    }
}

// ADV-103-1: high composite with collapsed phi — alert must still fire
#[test]
fn high_composite_with_collapsed_phi() {
    // rho_phi=0.01, others=1.0 → composite ≈ 0.316 (>0), but alert at 0.01
    let v = vec(1.0, 1.0, 1.0, 0.01);
    let score = RetentionScore::from_vector(v, RetentionScore::DEFAULT_THRESHOLD);
    assert!(
        score.unsafe_flag,
        "ADV-103-1: rho_phi=0.01 must trigger unsafe_flag even though composite={:.4}",
        score.composite
    );
    assert_eq!(score.rho_alert, 0.01);
}

// ADV-103-2: boundary condition — exactly at threshold is NOT unsafe
#[test]
fn all_dimensions_at_threshold_boundary() {
    let v = vec(0.5, 0.5, 0.5, 0.5);
    let score = RetentionScore::from_vector(v, 0.5);
    assert!(
        !score.unsafe_flag,
        "ADV-103-2: rho_alert=0.5 == threshold=0.5 is NOT unsafe (strict <, not ≤)"
    );
}

// REG-103-1: rho_alert() does not alter existing RetentionVector fields
#[test]
fn rho_alert_does_not_mutate_vector() -> Result<()> {
    let artifact = canonical_sample()?;
    let baseline = artifact.retention_baseline();
    let v = baseline.vector;
    let _ = v.rho_alert(); // call must not mutate
    assert_eq!(v.rho_l, baseline.vector.rho_l);
    assert_eq!(v.rho_s, baseline.vector.rho_s);
    assert_eq!(v.rho_sigma, baseline.vector.rho_sigma);
    assert_eq!(v.rho_phi, baseline.vector.rho_phi);
    Ok(())
}

// RetentionScore::composite reproduces the existing geometric-mean calculation
#[test]
fn retention_score_composite_matches_geometric_mean() {
    let v = vec(0.9, 0.8, 0.7, 0.6);
    let expected = (0.9_f64 * 0.8 * 0.7 * 0.6).powf(0.25);
    let score = RetentionScore::from_vector(v, RetentionScore::DEFAULT_THRESHOLD);
    assert!(
        (score.composite - expected).abs() < 1e-12,
        "composite={} must equal geometric mean={}",
        score.composite,
        expected
    );
}
