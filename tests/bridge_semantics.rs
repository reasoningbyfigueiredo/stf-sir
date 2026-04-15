//! Tests: artifact_to_theory embeds Formula and enriched metadata into Statement.
//!
//! Verifies that the bridge is structurally complete:
//!   - Each Statement carries a pre-parsed Formula (no re-parsing needed downstream)
//!   - Metadata contains span_start, span_end, node_type, zid
//!   - Provenance grounding reflects source_text presence
//!   - artifact_to_theory_with_formulas returns the embedded formula as the second element

use stf_sir::compiler;
use stf_sir::model::{Formula, artifact_to_theory};
use stf_sir::model::bridge::artifact_to_theory_with_formulas;

// ---------------------------------------------------------------------------
// Formula embedding

#[test]
fn compiled_atom_statement_has_embedded_formula() {
    // A plain word compiles to a paragraph token; its formula is Atom("...").
    let artifact = compiler::compile_markdown("Hello", None).unwrap();
    let theory = artifact_to_theory(&artifact);
    let stmt = theory.statements.values().next().expect("at least one statement");
    assert!(
        stmt.formula.is_some(),
        "bridge must embed a formula for non-empty text"
    );
    assert!(
        matches!(&stmt.formula, Some(Formula::Atom(_))),
        "plain text must parse as Atom, got {:?}",
        stmt.formula
    );
}

#[test]
fn compiled_implication_statement_has_implies_formula() {
    // "A -> B" must produce Formula::Implies(Atom("A"), Atom("B")).
    let artifact = compiler::compile_markdown("A -> B", None).unwrap();
    let theory = artifact_to_theory(&artifact);
    let stmt = theory
        .statements
        .values()
        .find(|s| s.text.contains("->"))
        .expect("must have an implication token");
    assert!(
        matches!(&stmt.formula, Some(Formula::Implies(_, _))),
        "implication text must produce Formula::Implies, got {:?}",
        stmt.formula
    );
}

#[test]
fn compiled_negation_statement_has_not_formula() {
    let artifact = compiler::compile_markdown("NOT A", None).unwrap();
    let theory = artifact_to_theory(&artifact);
    let stmt = theory
        .statements
        .values()
        .find(|s| s.text.to_uppercase().starts_with("NOT"))
        .expect("must have a NOT token");
    assert!(
        matches!(&stmt.formula, Some(Formula::Not(_))),
        "NOT text must produce Formula::Not, got {:?}",
        stmt.formula
    );
}

// ---------------------------------------------------------------------------
// Metadata enrichment

#[test]
fn bridge_metadata_contains_span_fields() {
    let artifact = compiler::compile_markdown("Sample text", None).unwrap();
    let theory = artifact_to_theory(&artifact);
    let stmt = theory.statements.values().next().expect("at least one statement");
    assert!(stmt.metadata.contains_key("span_start"), "must have span_start in metadata");
    assert!(stmt.metadata.contains_key("span_end"),   "must have span_end in metadata");
    // span_start must be a valid integer
    stmt.metadata["span_start"]
        .parse::<u64>()
        .expect("span_start must be numeric");
}

#[test]
fn bridge_metadata_contains_node_type() {
    let artifact = compiler::compile_markdown("Sample text", None).unwrap();
    let theory = artifact_to_theory(&artifact);
    let stmt = theory.statements.values().next().expect("at least one statement");
    assert!(stmt.metadata.contains_key("node_type"), "must have node_type in metadata");
    assert!(!stmt.metadata["node_type"].is_empty(),   "node_type must be non-empty");
}

#[test]
fn bridge_metadata_contains_zid() {
    let artifact = compiler::compile_markdown("Hello world", None).unwrap();
    let theory = artifact_to_theory(&artifact);
    let stmt = theory.statements.values().next().expect("at least one statement");
    assert!(stmt.metadata.contains_key("zid"), "must have zid in metadata");
    // zid must match the statement id
    assert_eq!(
        stmt.metadata["zid"], stmt.id,
        "zid metadata must equal statement.id"
    );
}

// ---------------------------------------------------------------------------
// artifact_to_theory_with_formulas returns embedded formula

#[test]
fn with_formulas_second_element_matches_embedded_formula() {
    let artifact = compiler::compile_markdown("A -> B", None).unwrap();
    let pairs = artifact_to_theory_with_formulas(&artifact);
    for (stmt, formula_opt) in &pairs {
        assert_eq!(
            &stmt.formula, formula_opt,
            "second element must equal stmt.formula for id={}",
            stmt.id
        );
    }
}
