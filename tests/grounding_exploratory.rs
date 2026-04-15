//! Exploratory tests for the grounding layer.
//!
//! Probes provenance-based grounding, SIR-based structural grounding,
//! grounding inheritance for derived statements, and boundary conditions.

use std::collections::BTreeMap;

use stf_sir::compiler;
use stf_sir::compiler::grounding::{GroundingChecker, ProvenanceGroundingChecker, SirGroundingChecker};
use stf_sir::model::{Provenance, Statement, StatementKind};
use stf_sir::sir::SirGraph;

fn prov_checker() -> ProvenanceGroundingChecker { ProvenanceGroundingChecker }

// Helper: build a Statement with explicit provenance fields.
fn stmt_prov(grounded: bool, source: Option<&str>, anchor: Option<&str>) -> Statement {
    let mut p = Provenance::default();
    p.grounded = grounded;
    if let Some(s) = source { p.source_ids.insert(s.into()); }
    if let Some(a) = anchor { p.anchors.insert(a.into()); }
    Statement {
        id: "t1".into(), text: "A".into(),
        kind: StatementKind::Atomic, domain: "test".into(),
        provenance: p, metadata: BTreeMap::new(), formula: None,
        semantic_dimensions: None,
    }
}

// ---------------------------------------------------------------------------
// 1. Provenance-only grounding: table-driven matrix

/// (grounded_flag, source, anchor) → expected is_grounded
const GROUNDING_MATRIX: &[(bool, Option<&str>, Option<&str>, bool)] = &[
    // flag alone
    (true,  None,             None,       true),
    (false, None,             None,       false),
    // source_id alone
    (false, Some("sha:abc"),  None,       true),
    // anchor alone
    (false, None,             Some("z1"), true),
    // flag + source
    (true,  Some("sha:abc"),  None,       true),
    // flag + anchor
    (true,  None,             Some("z2"), true),
    // all three
    (true,  Some("sha:xyz"),  Some("z3"), true),
    // none — ungrounded
    (false, None,             None,       false),
];

#[test]
fn provenance_grounding_matrix() {
    for &(flag, source, anchor, expected) in GROUNDING_MATRIX {
        let stmt = stmt_prov(flag, source, anchor);
        let result = prov_checker().check_grounding(&stmt);
        assert_eq!(result.is_grounded, expected,
            "matrix case ({flag}, {source:?}, {anchor:?}): expected {expected}");
    }
}

// ---------------------------------------------------------------------------
// 2. Multiple source_ids — all are returned in matched_sources

#[test]
fn multiple_source_ids_all_returned() {
    let mut p = Provenance::default();
    p.source_ids.insert("sha:001".into());
    p.source_ids.insert("sha:002".into());
    p.grounded = false;
    let stmt = Statement {
        id: "s".into(), text: "".into(), kind: StatementKind::Atomic,
        domain: "d".into(), provenance: p, metadata: BTreeMap::new(), formula: None,
        semantic_dimensions: None,
    };
    let result = prov_checker().check_grounding(&stmt);
    assert!(result.is_grounded);
    assert!(result.matched_sources.contains(&"sha:001".to_string()));
    assert!(result.matched_sources.contains(&"sha:002".to_string()));
}

// ---------------------------------------------------------------------------
// 3. SIR-based grounding: graph membership

#[test]
fn sir_grounds_every_compiled_token() {
    let artifact = compiler::compile_markdown("# H\n\nBody.\n", None).unwrap();
    let graph = SirGraph::from_artifact(&artifact);
    let checker = SirGroundingChecker { graph: &graph };

    for token in &artifact.ztokens {
        let stmt = Statement::atomic(token.id.clone(), &token.lexical.normalized_text, "t");
        assert!(checker.check_grounding(&stmt).is_grounded,
            "compiled token {} must be grounded by SIR graph", token.id);
    }
}

#[test]
fn sir_does_not_ground_fabricated_ids() {
    let artifact = compiler::compile_markdown("Hello.\n", None).unwrap();
    let graph = SirGraph::from_artifact(&artifact);
    let checker = SirGroundingChecker { graph: &graph };

    for fabricated in &["phantom:001", "made_up", "zz_fake", ""] {
        let stmt = Statement::atomic(*fabricated, "text", "t");
        assert!(!checker.check_grounding(&stmt).is_grounded,
            "fabricated id {fabricated:?} must not be grounded");
    }
}

// ---------------------------------------------------------------------------
// 4. SIR fallback to provenance for axioms

#[test]
fn sir_checker_falls_back_to_provenance_flag() {
    let artifact = compiler::compile_markdown("Axiom test.\n", None).unwrap();
    let graph = SirGraph::from_artifact(&artifact);
    let checker = SirGroundingChecker { graph: &graph };

    // Statement not in graph but with grounded=true → grounded via fallback.
    let axiom = Statement::grounded("axiom:999", "truth", "t", "sha:src");
    assert!(checker.check_grounding(&axiom).is_grounded,
        "axiom with provenance must be grounded even without graph node");
}

#[test]
fn sir_checker_fallback_fails_without_provenance() {
    let artifact = compiler::compile_markdown("X.\n", None).unwrap();
    let graph = SirGraph::from_artifact(&artifact);
    let checker = SirGroundingChecker { graph: &graph };

    let stmt = Statement::atomic("ghost:1", "claim", "t");
    assert!(!checker.check_grounding(&stmt).is_grounded,
        "ghost id without provenance must be ungrounded");
}

// ---------------------------------------------------------------------------
// 5. Derived statement provenance inheritance

#[test]
fn derived_statement_from_formula_engine_has_grounded_flag() {
    // FormulaInferenceEngine explicitly sets grounded=true on derived statements.
    use stf_sir::compiler::inference::{FormulaInferenceEngine, InferenceEngine};
    use stf_sir::model::Theory;

    let mut theory = Theory::new();
    theory.insert(Statement::grounded("a", "A", "logic", "sha:src"));
    theory.insert(Statement::grounded("imp", "A -> B", "logic", "sha:src"));

    let derived = FormulaInferenceEngine.derive(&theory);
    assert_eq!(derived.len(), 1);
    let d = &derived[0].statement;
    assert!(d.provenance.grounded,
        "derived statement must have grounded=true (provenance.grounded)");
    assert_eq!(d.provenance.generated_by.as_deref(), Some("modus_ponens_formula"));
}

#[test]
fn derived_statement_is_grounded_by_provenance_checker() {
    use stf_sir::compiler::inference::{FormulaInferenceEngine, InferenceEngine};
    use stf_sir::model::Theory;

    let mut theory = Theory::new();
    theory.insert(Statement::grounded("a", "A", "logic", "sha:src"));
    theory.insert(Statement::grounded("imp", "A -> B", "logic", "sha:src"));

    let derived = FormulaInferenceEngine.derive(&theory);
    let d = &derived[0].statement;
    let result = prov_checker().check_grounding(d);
    assert!(result.is_grounded,
        "provenance checker must see derived statement as grounded");
}

// ---------------------------------------------------------------------------
// 6. Grounding does NOT depend on formula content

#[test]
fn grounding_is_independent_of_formula() {
    use stf_sir::model::Formula;
    // Adding a formula must not change grounding status.
    let stmt_no_formula = Statement::atomic("s1", "A", "t");
    let stmt_with_formula = Statement::atomic("s1", "A", "t")
        .with_formula(Formula::atom("A"));

    let r1 = prov_checker().check_grounding(&stmt_no_formula);
    let r2 = prov_checker().check_grounding(&stmt_with_formula);
    assert_eq!(r1.is_grounded, r2.is_grounded,
        "formula presence must not affect grounding");
}

// ---------------------------------------------------------------------------
// 7. Boundary: empty string id is handled without panic

#[test]
fn empty_id_does_not_panic_in_sir_checker() {
    let artifact = compiler::compile_markdown("Safe test.\n", None).unwrap();
    let graph = SirGraph::from_artifact(&artifact);
    let checker = SirGroundingChecker { graph: &graph };

    let stmt = Statement::atomic("", "text", "t");
    let _ = checker.check_grounding(&stmt); // must not panic
}
