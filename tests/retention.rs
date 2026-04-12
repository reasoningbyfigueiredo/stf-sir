use anyhow::{Context, Result};
use stf_sir::compiler;
use stf_sir::RetentionVector;

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
