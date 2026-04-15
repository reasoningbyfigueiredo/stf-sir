//! Tests for logical coherence (C_l) — Definition C1 of the coherence paper.
//! Also covers ADR-SEM-001 I-6: RecommendedEngine invariants (INV-102-1 to INV-102-3).

use stf_sir::model::{Formula, SemanticDimensions, Statement, Theory};
use stf_sir::model::artifact_to_theory;
use stf_sir::compiler;
use stf_sir::compiler::coherence::{LogicalCoherenceChecker, SimpleLogicChecker};
use stf_sir::compiler::grounding::{GroundingChecker, ProvenanceGroundingChecker};
use stf_sir::compiler::{
    recommended_engine, recommended_engine_with_budget, recommended_engine_with_sir,
    formula_engine_with_budget, RECOMMENDED_STEP_BUDGET,
};

fn checker() -> SimpleLogicChecker { SimpleLogicChecker }

// ---------------------------------------------------------------------------
// Consistent theories

#[test]
fn empty_theory_is_consistent() {
    let theory = Theory::new();
    assert!(checker().check_consistency(&theory).is_ok());
}

#[test]
fn single_statement_is_consistent() {
    let mut theory = Theory::new();
    theory.insert(Statement::atomic("s1", "A", "test"));
    assert!(checker().check_consistency(&theory).is_ok());
}

#[test]
fn two_unrelated_statements_are_consistent() {
    let mut theory = Theory::new();
    theory.insert(Statement::atomic("s1", "A", "test"));
    theory.insert(Statement::atomic("s2", "B", "test"));
    assert!(checker().check_consistency(&theory).is_ok());
}

// ---------------------------------------------------------------------------
// Contradictory theories

#[test]
fn theory_with_a_and_not_a_is_inconsistent() {
    let mut theory = Theory::new();
    theory.insert(Statement::atomic("s1", "A", "test"));
    theory.insert(Statement::atomic("s2", "NOT A", "test"));
    let result = checker().check_consistency(&theory);
    assert!(result.is_err());
    let inc = result.unwrap_err();
    assert!(inc.conflicting_ids.contains(&"s1".to_string())
        || inc.conflicting_ids.contains(&"s2".to_string()));
}

#[test]
fn extension_with_contradiction_fails() {
    let mut theory = Theory::new();
    theory.insert(Statement::atomic("s1", "B", "test"));
    let candidate = Statement::atomic("s2", "NOT B", "test");
    let result = checker().check_extension(&theory, &candidate);
    assert!(result.is_err());
}

#[test]
fn extension_without_contradiction_succeeds() {
    let mut theory = Theory::new();
    theory.insert(Statement::atomic("s1", "A", "test"));
    let candidate = Statement::atomic("s2", "B", "test");
    assert!(checker().check_extension(&theory, &candidate).is_ok());
}

// ---------------------------------------------------------------------------
// Case-insensitive detection

#[test]
fn case_insensitive_contradiction_detected() {
    let mut theory = Theory::new();
    theory.insert(Statement::atomic("s1", "alpha", "test"));
    theory.insert(Statement::atomic("s2", "not alpha", "test"));
    assert!(checker().check_consistency(&theory).is_err());
}

// ---------------------------------------------------------------------------
// ADR-SEM-001 I-6 — RecommendedEngine invariants
// ---------------------------------------------------------------------------

// INV-102-3: RecommendedEngine is accessible (compilation test)
// If this test compiles, the type is exported correctly.
#[test]
fn recommended_engine_is_accessible() {
    let _: stf_sir::RecommendedEngine = recommended_engine();
}

// INV-102-1a: recommended_engine has an explicit step budget (not usize::MAX)
#[test]
fn recommended_engine_has_explicit_budget() {
    let engine = recommended_engine();
    assert_ne!(
        engine.step_budget,
        usize::MAX,
        "RecommendedEngine must not use usize::MAX budget (C_c would always be Unknown)"
    );
    assert_eq!(engine.step_budget, RECOMMENDED_STEP_BUDGET);
}

// INV-102-1b: recommended_engine and formula_engine_with_budget(RECOMMENDED_STEP_BUDGET)
//             produce identical results for the same input.
#[test]
fn recommended_engine_consistent_with_formula_engine() {
    let rec = recommended_engine_with_budget(usize::MAX);
    let formula = formula_engine_with_budget(usize::MAX);

    let mut theory = Theory::new();
    theory.insert(
        Statement::grounded("imp1", "A -> B", "test", "sha256:x")
            .with_formula(Formula::implies(Formula::atom("A"), Formula::atom("B"))),
    );

    let candidate = Statement::grounded("a1", "A", "test", "sha256:x")
        .with_formula(Formula::atom("A"));

    let r1 = rec.evaluate_statement(&theory, &candidate);
    let r2 = formula.evaluate_statement(&theory, &candidate);

    assert_eq!(r1.coherence.logical, r2.coherence.logical, "C_l must match");
    assert_eq!(r1.coherence.operational, r2.coherence.operational, "C_o must match");
    assert_eq!(r1.grounded, r2.grounded, "grounded must match");
    assert_eq!(r1.useful_information, r2.useful_information, "useful_information must match");
    assert_eq!(r1.derived_count, r2.derived_count, "derived_count must match");
}

// INV-102-1c: large theory exceeds RECOMMENDED_STEP_BUDGET → C_c = Violated
#[test]
fn recommended_engine_large_theory_exceeds_budget() {
    use stf_sir::TruthValue;

    let engine = recommended_engine(); // uses RECOMMENDED_STEP_BUDGET
    let mut theory = Theory::new();

    // Insert enough statements that n*(n-1)/2 > RECOMMENDED_STEP_BUDGET.
    // sqrt(2 * 1_000_000) ≈ 1414, so 1500 statements guarantees budget exceeded.
    for i in 0..1500usize {
        theory.insert(Statement::grounded(
            format!("s{i}"),
            format!("statement {i}"),
            "test",
            "sha256:src",
        ));
    }

    let result = engine.audit_theory(&theory);
    assert_eq!(
        result.coherence.computational,
        TruthValue::Violated,
        "theory with >1414 statements must exceed RECOMMENDED_STEP_BUDGET → C_c = Violated"
    );
}

// INV-102-3: DefaultEngine still compiles (backwards compatibility)
#[test]
#[allow(deprecated)]
fn deprecated_default_engine_still_compiles() {
    use stf_sir::compiler::default_engine;
    let engine = default_engine();
    let theory = Theory::new();
    let stmt = Statement::atomic("s1", "A", "test");
    // Must not panic
    let _ = engine.evaluate_statement(&theory, &stmt);
}

// ---------------------------------------------------------------------------
// IT-102-1: CLI and API produce identical coherence results for the same artifact
//
// The CLI path: read YAML → deserialize Artifact → artifact_to_theory → audit_theory.
// The API path: compile_markdown → artifact_to_theory → audit_theory.
// Both must produce the same CoherenceVector when given the same underlying data.

#[test]
fn cli_and_api_produce_same_coherence_result() {
    let src = "A -> B\n\nA\n";

    // API path.
    let artifact_api = compiler::compile_markdown(src, None)
        .expect("well-formed markdown must compile");
    let theory_api = artifact_to_theory(&artifact_api);
    let result_api = recommended_engine().audit_theory(&theory_api);

    // CLI path: serialize → deserialize (simulates CLI reading a .zmd file).
    let yaml = compiler::serializer::to_yaml_string(&artifact_api)
        .expect("serialization must not fail");
    let artifact_cli: stf_sir::model::Artifact = serde_yaml_ng::from_str(&yaml)
        .expect("deserialized artifact must be valid");
    let theory_cli = artifact_to_theory(&artifact_cli);
    let result_cli = recommended_engine().audit_theory(&theory_cli);

    assert_eq!(
        result_api.coherence.logical, result_cli.coherence.logical,
        "IT-102-1: C_l must be identical between API and CLI paths"
    );
    assert_eq!(
        result_api.coherence.computational, result_cli.coherence.computational,
        "IT-102-1: C_c must be identical between API and CLI paths"
    );
    assert_eq!(
        result_api.coherence.operational, result_cli.coherence.operational,
        "IT-102-1: C_o must be identical between API and CLI paths"
    );
    assert_eq!(
        result_api.grounded, result_cli.grounded,
        "IT-102-1: grounded must be identical between API and CLI paths"
    );
    assert_eq!(
        result_api.useful_information, result_cli.useful_information,
        "IT-102-1: useful_information must be identical between API and CLI paths"
    );
}

// ---------------------------------------------------------------------------
// ---------------------------------------------------------------------------
// EPIC-201 FEAT-201-5 — SemanticDimensions tests
// ---------------------------------------------------------------------------

// UT-201-5-1: from_evaluation preserves CoherenceVector (INV-201-5)
#[test]
fn semantic_dimensions_from_evaluation_preserves_coherence_vector() {
    use stf_sir::model::Formula;

    let engine = recommended_engine_with_budget(usize::MAX);
    let mut theory = Theory::new();
    theory.insert(
        Statement::grounded("imp1", "A -> B", "test", "sha256:src")
            .with_formula(Formula::implies(Formula::atom("A"), Formula::atom("B"))),
    );
    let candidate = Statement::grounded("a1", "A", "test", "sha256:src")
        .with_formula(Formula::atom("A"));

    let result = engine.evaluate_statement(&theory, &candidate);
    let dims = SemanticDimensions::from_evaluation(&result);

    assert_eq!(
        dims.coherence, result.coherence,
        "UT-201-5-1: from_evaluation must preserve CoherenceVector (INV-201-5)"
    );
}

// UT-201-5-2: transformation_delta is always 0.0 in v1 (INV-201-7)
#[test]
fn semantic_dimensions_transformation_delta_is_zero_in_v1() {
    let engine = recommended_engine_with_budget(usize::MAX);
    let theory = Theory::new();
    let stmt = Statement::grounded("s1", "A", "test", "sha256:src");

    let result = engine.evaluate_statement(&theory, &stmt);
    let dims = SemanticDimensions::from_evaluation(&result);

    assert_eq!(
        dims.transformation_delta, 0.0,
        "UT-201-5-2: transformation_delta must be 0.0 in v1 (INV-201-7)"
    );
}

// UT-201-5-3: Statement with semantic_dimensions = None inserts into Theory without error (INV-201-6)
#[test]
fn statement_with_none_semantic_dimensions_inserts_into_theory() {
    let mut theory = Theory::new();
    let stmt = Statement::atomic("s1", "A", "test");
    assert!(
        stmt.semantic_dimensions.is_none(),
        "UT-201-5-3: atomic constructor must produce semantic_dimensions = None"
    );
    theory.insert(stmt);
    assert!(
        theory.contains("s1"),
        "UT-201-5-3: Statement with semantic_dimensions=None must insert into Theory (INV-201-6)"
    );
}

// UT-201-5-4: is_healthy returns false when C_l = Violated
#[test]
fn semantic_dimensions_is_healthy_false_when_contradiction() {
    use stf_sir::model::Formula;

    let engine = recommended_engine_with_budget(usize::MAX);
    let mut theory = Theory::new();
    theory.insert(
        Statement::grounded("s1", "A", "test", "sha256:src")
            .with_formula(Formula::atom("A")),
    );
    let candidate = Statement::grounded("s2", "NOT A", "test", "sha256:src")
        .with_formula(Formula::not(Formula::atom("A")));

    let result = engine.evaluate_statement(&theory, &candidate);
    let dims = SemanticDimensions::from_evaluation(&result);

    assert!(
        result.coherence.logical.is_violated(),
        "UT-201-5-4 precondition: C_l must be Violated"
    );
    assert!(
        !dims.is_healthy(),
        "UT-201-5-4: is_healthy must return false when C_l = Violated"
    );
}

// ---------------------------------------------------------------------------
// ADV-102-2: recommended_engine_with_sir grounds by graph membership, not provenance
//
// A statement whose id is a real ZToken id in the SirGraph must be grounded
// by SirGroundingChecker even when it carries no provenance fields.
// ProvenanceGroundingChecker would reject the same statement.

#[test]
fn recommended_engine_with_sir_uses_graph_grounding() {
    let artifact = compiler::compile_markdown("hello world", None)
        .expect("simple source must compile");
    let token_id = artifact.ztokens[0].id.clone();

    // Build the engine backed by the SirGraph.
    let graph = artifact.as_sir_graph();
    let engine = recommended_engine_with_sir(&graph);

    // Statement: real ZToken id, NO provenance (no source_ids, no anchors, grounded=false).
    // ProvenanceGroundingChecker would reject this.
    let stmt = Statement::atomic(token_id.clone(), "any text", "test");

    let prov_result = ProvenanceGroundingChecker.check_grounding(&stmt);
    assert!(
        !prov_result.is_grounded,
        "ADV-102-2 precondition: ProvenanceGroundingChecker must reject stmt with no provenance"
    );

    // SirGroundingChecker grounds it because the id is a compiled ZToken node.
    let sir_result = engine.grounding.check_grounding(&stmt);
    assert!(
        sir_result.is_grounded,
        "ADV-102-2: SirGroundingChecker must ground stmt with real ZToken id '{}' \
         even without provenance fields",
        token_id
    );

    // The full engine reports grounded=true for this statement.
    let theory = Theory::new();
    let eval = engine.evaluate_statement(&theory, &stmt);
    assert!(
        eval.grounded,
        "ADV-102-2: recommended_engine_with_sir must report grounded=true \
         for a statement whose id is a compiled ZToken"
    );
}
