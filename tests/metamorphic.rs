mod common;

use std::path::Path;

use anyhow::Result;
use stf_sir::compiler;
use stf_sir::model::Artifact;

#[test]
fn recompiling_the_same_input_twice_yields_identical_output() -> Result<()> {
    let (_, first) =
        common::compile_and_serialize_fixture("tests/fixtures/valid/heading_paragraph.md")?;
    let (_, second) =
        common::compile_and_serialize_fixture("tests/fixtures/valid/heading_paragraph.md")?;

    assert_eq!(first, second);
    Ok(())
}

#[test]
fn lf_vs_crlf_preserves_structural_validity_and_shape() -> Result<()> {
    let lf = "# Title\n\nBody with line endings.\n\n- one\n- two\n";
    let crlf = lf.replace('\n', "\r\n");

    let lf_artifact = compiler::compile_markdown(lf, Some(Path::new("lf.md")))?;
    let crlf_artifact = compiler::compile_markdown(&crlf, Some(Path::new("crlf.md")))?;

    let lf_errors = compiler::validator::validate(&lf_artifact, Some(lf.as_bytes()));
    let crlf_errors = compiler::validator::validate(&crlf_artifact, Some(crlf.as_bytes()));
    assert!(
        lf_errors.is_empty(),
        "LF variant failed validation: {lf_errors:#?}"
    );
    assert!(
        crlf_errors.is_empty(),
        "CRLF variant failed validation: {crlf_errors:#?}"
    );

    assert_eq!(
        structural_signature(&lf_artifact),
        structural_signature(&crlf_artifact)
    );
    Ok(())
}

#[test]
fn trailing_newline_does_not_break_invariants_or_structure() -> Result<()> {
    let without_newline = "# Heading\n\nParagraph body.";
    let with_newline = "# Heading\n\nParagraph body.\n";

    let artifact_without =
        compiler::compile_markdown(without_newline, Some(Path::new("without.md")))?;
    let artifact_with = compiler::compile_markdown(with_newline, Some(Path::new("with.md")))?;

    let without_errors =
        compiler::validator::validate(&artifact_without, Some(without_newline.as_bytes()));
    let with_errors = compiler::validator::validate(&artifact_with, Some(with_newline.as_bytes()));
    assert!(
        without_errors.is_empty(),
        "missing trailing newline failed validation: {without_errors:#?}"
    );
    assert!(
        with_errors.is_empty(),
        "trailing newline variant failed validation: {with_errors:#?}"
    );

    assert_eq!(
        structural_signature(&artifact_without),
        structural_signature(&artifact_with)
    );
    Ok(())
}

#[test]
fn sir_graph_projection_is_deterministic_across_repeated_runs() -> Result<()> {
    let artifact_first = common::compile_fixture("tests/fixtures/valid/many_siblings.md")?;
    let artifact_second = common::compile_fixture("tests/fixtures/valid/many_siblings.md")?;

    let graph_first = artifact_first.as_sir_graph();
    let graph_second = artifact_second.as_sir_graph();

    assert_eq!(graph_first, graph_second);
    Ok(())
}

fn structural_signature(artifact: &Artifact) -> (Vec<TokenSignature>, Vec<RelationSignature>) {
    let tokens = artifact
        .ztokens
        .iter()
        .map(|token| {
            (
                token.id.clone(),
                token.syntactic.node_type.clone(),
                token.syntactic.parent_id.clone(),
                token.syntactic.depth,
                token.syntactic.sibling_index,
                token.syntactic.path.clone(),
                token.lexical.normalized_text.clone(),
                token.semantic.gloss.clone(),
            )
        })
        .collect::<Vec<_>>();
    let relations = artifact
        .relations
        .iter()
        .map(|relation| {
            (
                relation.id.clone(),
                relation.type_.clone(),
                relation.category.as_str().to_string(),
                relation.source.clone(),
                relation.target.clone(),
                relation.stage.clone(),
            )
        })
        .collect::<Vec<_>>();
    (tokens, relations)
}

type TokenSignature = (
    String,
    String,
    Option<String>,
    usize,
    usize,
    String,
    String,
    String,
);
type RelationSignature = (String, String, String, String, String, String);
