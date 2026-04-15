//! Exploratory tests for hallucination boundary classification.
//!
//! The goal is to make every error-kind boundary explicit and prevent
//! future category drift as the formal model evolves.
//!
//! Boundary map:
//!   Contradiction  : C_l = Violated (logical conflict with existing theory)
//!   Hallucination  : C_l = Satisfied, Ground = 0 (coherent but ungrounded)
//!   NonExecutable  : C_l = Satisfied, Ground = 1, C_o = Violated (sterile)
//!   ICE (useful)   : C_l = Satisfied, Ground = 1, C_o = Satisfied

use stf_sir::compiler::engine::default_engine;
use stf_sir::compiler::recommended_engine_with_budget;
use stf_sir::compiler::grounding::{GroundingChecker, ProvenanceGroundingChecker, SirGroundingChecker};
use stf_sir::compiler;
use stf_sir::error::ErrorKind;
use stf_sir::model::{Formula, Provenance, Statement, StatementKind, Theory, TruthValue};
use stf_sir::sir::SirGraph;
use std::collections::BTreeMap;

fn engine() -> stf_sir::compiler::engine::DefaultEngine { default_engine() }

// Helper: collect all error kinds from a result.
fn error_kinds(result: &stf_sir::compiler::engine::EvaluationResult) -> Vec<ErrorKind> {
    result.errors.iter().map(|e| e.kind.clone()).collect()
}

// ---------------------------------------------------------------------------
// 1. Clean case: no errors

#[test]
fn fully_coherent_grounded_executable_has_no_errors() {
    let mut theory = Theory::new();
    theory.insert(Statement::grounded("s1", "A", "logic", "sha:src"));
    let candidate = Statement::grounded("s2", "A -> B", "logic", "sha:src");

    let result = engine().evaluate_statement(&theory, &candidate);
    assert!(result.errors.is_empty(), "ICE case must produce no errors");
    assert!(result.useful_information);
}

// ---------------------------------------------------------------------------
// 2. Contradiction — must be Contradiction not Hallucination

#[test]
fn contradiction_classified_as_contradiction_not_hallucination() {
    let mut theory = Theory::new();
    theory.insert(Statement::grounded("s1", "P", "test", "sha:src"));
    let candidate = Statement::grounded("s2", "NOT P", "test", "sha:src");

    let result = engine().evaluate_statement(&theory, &candidate);
    let kinds = error_kinds(&result);

    assert!(kinds.contains(&ErrorKind::Contradiction), "must be Contradiction; got {kinds:?}");
    assert!(!kinds.contains(&ErrorKind::Hallucination), "must NOT be Hallucination");
    assert_eq!(result.coherence.logical, TruthValue::Violated);
    assert!(!result.useful_information);
}

#[test]
fn contradiction_of_ungrounded_candidate_is_still_contradiction() {
    // Even without provenance, if the candidate contradicts the theory, it's Contradiction.
    let mut theory = Theory::new();
    theory.insert(Statement::grounded("s1", "Q", "test", "sha:src"));
    let candidate = Statement::atomic("s2", "NOT Q", "test"); // ungrounded

    let result = engine().evaluate_statement(&theory, &candidate);
    let kinds = error_kinds(&result);

    assert!(kinds.contains(&ErrorKind::Contradiction), "ungrounded contradiction must be Contradiction");
    // Hallucination check only runs when C_l = Satisfied; contradiction short-circuits it.
    assert!(!kinds.contains(&ErrorKind::Hallucination), "must NOT be Hallucination");
}

// ---------------------------------------------------------------------------
// 3. Hallucination: coherent but ungrounded

#[test]
fn ungrounded_coherent_statement_is_hallucination() {
    let theory = Theory::new();
    let candidate = Statement::atomic("h1", "The moon is made of cheese", "test");

    let result = engine().evaluate_statement(&theory, &candidate);
    let kinds = error_kinds(&result);

    assert!(kinds.contains(&ErrorKind::Hallucination), "ungrounded must be Hallucination; got {kinds:?}");
    assert!(!kinds.contains(&ErrorKind::Contradiction));
    assert_eq!(result.coherence.logical, TruthValue::Satisfied, "C_l must be Satisfied");
    assert!(!result.grounded);
}

#[test]
fn grounded_coherent_statement_is_not_hallucination() {
    let theory = Theory::new();
    let candidate = Statement::grounded("g1", "A", "test", "sha:src");

    let result = engine().evaluate_statement(&theory, &candidate);
    let kinds = error_kinds(&result);

    assert!(!kinds.contains(&ErrorKind::Hallucination), "grounded must NOT be Hallucination");
    assert!(result.grounded);
}

// ---------------------------------------------------------------------------
// 4. NonExecutable: coherent, grounded, but operationally sterile

#[test]
fn grounded_coherent_but_sterile_is_non_executable() {
    let theory = Theory::new();
    // "Hello world" triggers no inference rule → sterile.
    let candidate = Statement::grounded("e1", "Hello world", "test", "sha:src");

    let result = engine().evaluate_statement(&theory, &candidate);
    let kinds = error_kinds(&result);

    assert!(kinds.contains(&ErrorKind::NonExecutable), "sterile must be NonExecutable; got {kinds:?}");
    assert!(!kinds.contains(&ErrorKind::Hallucination));
    assert!(!kinds.contains(&ErrorKind::Contradiction));
    assert_eq!(result.coherence.operational, TruthValue::Violated, "C_o must be Violated");
}

// ---------------------------------------------------------------------------
// 5. Hallucination + NonExecutable simultaneously

#[test]
fn ungrounded_sterile_has_both_hallucination_and_non_executable() {
    // Ungrounded AND sterile → both errors should appear.
    let theory = Theory::new();
    let candidate = Statement::atomic("u1", "Random ungrounded text", "test");

    let result = engine().evaluate_statement(&theory, &candidate);
    let kinds = error_kinds(&result);

    assert!(kinds.contains(&ErrorKind::Hallucination),
        "ungrounded sterile must emit Hallucination; got {kinds:?}");
    assert!(kinds.contains(&ErrorKind::NonExecutable),
        "ungrounded sterile must also emit NonExecutable; got {kinds:?}");
}

// ---------------------------------------------------------------------------
// 6. audit_theory boundary classification

#[test]
fn audit_flags_hallucinations_and_not_grounded_statements() {
    let mut theory = Theory::new();
    theory.insert(Statement::grounded("s1", "A", "test", "sha:src"));
    theory.insert(Statement::atomic("s2", "B", "test")); // ungrounded

    let result = engine().audit_theory(&theory);
    let hallucination_ids: Vec<_> = result.errors.iter()
        .filter(|e| e.kind == ErrorKind::Hallucination)
        .flat_map(|e| e.statement_ids.iter().cloned())
        .collect();

    assert!(!hallucination_ids.contains(&"s1".to_string()), "s1 grounded must not be flagged");
    assert!(hallucination_ids.contains(&"s2".to_string()), "s2 ungrounded must be flagged");
}

#[test]
fn audit_contradiction_reports_contradiction_not_hallucination() {
    let mut theory = Theory::new();
    theory.insert(Statement::grounded("s1", "X", "test", "sha:src"));
    theory.insert(Statement::grounded("s2", "NOT X", "test", "sha:src"));

    let result = engine().audit_theory(&theory);
    let kinds = error_kinds(&result);

    assert!(kinds.contains(&ErrorKind::Contradiction), "audit must detect contradiction");
    // In contradictory theory, C_l = Violated so inference does not run →
    // NonExecutable may not be emitted.  We should NOT see Hallucination
    // for grounded statements in a contradictory theory.
    assert!(!kinds.contains(&ErrorKind::Hallucination),
        "grounded contradictory statements must NOT be Hallucination");
}

// ---------------------------------------------------------------------------
// 7. Severity ordering: Contradiction > Hallucination > NonExecutable

#[test]
fn contradiction_has_critical_severity() {
    use stf_sir::error::Severity;
    let mut theory = Theory::new();
    theory.insert(Statement::grounded("s1", "Y", "test", "sha:src"));
    let candidate = Statement::grounded("s2", "NOT Y", "test", "sha:src");

    let result = engine().evaluate_statement(&theory, &candidate);
    let crit = result.errors.iter()
        .filter(|e| e.kind == ErrorKind::Contradiction)
        .all(|e| e.severity == Severity::Critical);
    assert!(crit, "contradiction errors must be Critical severity");
}

#[test]
fn hallucination_has_high_severity() {
    use stf_sir::error::Severity;
    let theory = Theory::new();
    let candidate = Statement::atomic("h", "ungrounded", "test");

    let result = engine().evaluate_statement(&theory, &candidate);
    let high = result.errors.iter()
        .filter(|e| e.kind == ErrorKind::Hallucination)
        .all(|e| e.severity == Severity::High);
    assert!(high, "hallucination errors must be High severity");
}

#[test]
fn non_executable_has_medium_severity() {
    use stf_sir::error::Severity;
    let theory = Theory::new();
    let candidate = Statement::grounded("e", "sterile grounded", "test", "sha:src");

    let result = engine().evaluate_statement(&theory, &candidate);
    let medium = result.errors.iter()
        .filter(|e| e.kind == ErrorKind::NonExecutable)
        .all(|e| e.severity == Severity::Medium);
    assert!(medium, "non-executable errors must be Medium severity");
}

// ---------------------------------------------------------------------------
// 8. ADV-101-1: Ungrounded statement fires modus ponens — still not useful
//
// An ungrounded statement can trigger inference (C_o = Satisfied) yet
// useful_information MUST remain false (INV-101-1).  This is the core
// adversarial case: an LLM can hallucinate "A" which, together with a
// grounded rule "A → B", appears productive — but it is not useful
// information because the premise is ungrounded.

#[test]
fn hallucination_with_modus_ponens() {
    let engine = recommended_engine_with_budget(usize::MAX);

    // Theory: grounded implication "A -> B".
    let mut theory = Theory::new();
    theory.insert(
        Statement::grounded("impl1", "A -> B", "test", "sha:src")
            .with_formula(Formula::implies(Formula::atom("A"), Formula::atom("B"))),
    );

    // Candidate: UNGROUNDED atom "A" (no source_id, no anchor, grounded=false).
    // FormulaInferenceEngine will derive "B" → C_o = Satisfied, derived_count > 0.
    let candidate = Statement::atomic("u1", "A", "test")
        .with_formula(Formula::atom("A"));

    let result = engine.evaluate_statement(&theory, &candidate);

    assert!(!result.grounded, "ungrounded candidate must have grounded=false");
    assert!(
        result.derived_count > 0,
        "ADV-101-1 requires modus ponens to fire; got derived_count={}",
        result.derived_count
    );
    assert_eq!(
        result.coherence.operational,
        TruthValue::Satisfied,
        "C_o must be Satisfied (modus ponens fired)"
    );
    assert!(
        !result.useful_information,
        "ADV-101-1: modus ponens fires but grounded=false → useful_information must be false (INV-101-1)"
    );
    assert!(
        result.errors.iter().any(|e| e.kind == ErrorKind::Hallucination),
        "ADV-101-1: must emit Hallucination for ungrounded candidate"
    );
}

// ---------------------------------------------------------------------------
// 9. ADV-101-2: Fabricated grounded flag
//
// Documents the trust boundary: setting Provenance.grounded=true (without
// source_ids or anchors) is accepted by ProvenanceGroundingChecker at face
// value.  SirGroundingChecker's fallback also accepts the flag.  The only
// strong rejection comes from SirGroundingChecker when neither the ID is in
// the graph NOR any provenance evidence is present.

#[test]
fn fabricated_grounded_flag() {
    // Build a statement with grounded=true but NO source_ids and NO anchors.
    let mut provenance = Provenance::default();
    provenance.grounded = true;
    // source_ids and anchors intentionally left empty.

    let stmt = Statement {
        id: "fabricated-001".into(),
        text: "claimed to be grounded by flag alone".into(),
        kind: StatementKind::Atomic,
        domain: "test".into(),
        provenance,
        metadata: BTreeMap::new(),
        formula: None,
        semantic_dimensions: None,
    };

    // ProvenanceGroundingChecker accepts the grounded=true flag alone.
    let prov_result = ProvenanceGroundingChecker.check_grounding(&stmt);
    assert!(
        prov_result.is_grounded,
        "ADV-101-2: ProvenanceGroundingChecker must accept grounded=true flag alone"
    );
    assert!(
        prov_result.matched_sources.is_empty(),
        "ADV-101-2: no source_ids → matched_sources must be empty"
    );

    // SirGroundingChecker: 'fabricated-001' is not in any compiled graph.
    // The fallback checks p.grounded, so it also accepts the fabricated flag.
    // This documents a known limitation: the flag is not verified against the graph.
    let artifact = compiler::compile_markdown("hello world", None)
        .expect("simple markdown must compile");
    let graph = SirGraph::from_artifact(&artifact);
    let sir_checker = SirGroundingChecker { graph: &graph };
    let sir_result = sir_checker.check_grounding(&stmt);

    assert!(
        sir_result.is_grounded,
        "ADV-101-2 (documented limitation): SirGroundingChecker fallback also accepts \
         grounded=true flag for an id not in the graph — flag is not graph-verified"
    );

    // Contrast: without ANY grounding evidence, SirGroundingChecker rejects.
    let bare = Statement::atomic("fabricated-002", "no evidence at all", "test");
    assert!(
        !sir_checker.check_grounding(&bare).is_grounded,
        "ADV-101-2: without any provenance evidence, SirGroundingChecker must reject"
    );
}
