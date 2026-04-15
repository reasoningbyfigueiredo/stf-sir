//! Metamorphic tests for the coherence engine.
//!
//! Metamorphic relations are input transformations that must preserve (or
//! invert) an expected output property, even when we cannot compute the exact
//! expected output directly.
//!
//! Relations tested:
//!   MR1  Adding an irrelevant grounded statement must not create contradiction.
//!   MR2  Inserting a duplicate id must not change overall coherence class.
//!   MR3  Reordering inserts must not affect the final theory or coherence.
//!   MR4  Whitespace variants of the same text must parse to the same formula.
//!   MR5  Contradiction is symmetric in check_extension.
//!   MR6  Adding grounded derived facts must not weaken prior grounding.
//!   MR7  Semantically equivalent strings must match the same contradiction.

mod common;

use stf_sir::compiler::coherence::{FormulaCoherenceChecker, LogicalCoherenceChecker, SimpleLogicChecker};
use stf_sir::compiler::engine::default_engine;
use stf_sir::model::{Formula, Statement, Theory};

// ---------------------------------------------------------------------------
// MR1: Adding irrelevant grounded statement must not create contradiction

#[test]
fn adding_irrelevant_statement_preserves_consistency() {
    let base = common::grounded_theory(&[("s1", "A", "sha:src"), ("s2", "B", "sha:src")]);
    let engine = default_engine();
    let r_base = engine.audit_theory(&base);

    let mut extended = base.clone();
    extended.insert(Statement::grounded("s3", "C", "test", "sha:src")); // irrelevant
    let r_ext = engine.audit_theory(&extended);

    assert_eq!(r_base.coherence.logical, r_ext.coherence.logical,
        "adding irrelevant statement must not change C_l");
}

// ---------------------------------------------------------------------------
// MR2: Duplicate id replaces statement — coherence class must not change
//      in ways that depend on the order of duplicate insertion.

#[test]
fn duplicate_id_insert_is_idempotent() {
    let mut t1 = Theory::new();
    t1.insert(Statement::grounded("s1", "A", "logic", "sha:src"));
    t1.insert(Statement::grounded("s2", "B", "logic", "sha:src"));

    // Insert s1 again with the same content.
    let mut t2 = t1.clone();
    t2.insert(Statement::grounded("s1", "A", "logic", "sha:src"));

    // Theory sizes must be equal.
    assert_eq!(t1.len(), t2.len());

    // Coherence result must be identical.
    let engine = default_engine();
    let r1 = engine.audit_theory(&t1);
    let r2 = engine.audit_theory(&t2);
    assert_eq!(r1.coherence.logical, r2.coherence.logical,
        "idempotent insert must not change C_l");
}

// ---------------------------------------------------------------------------
// MR3: Reordering inserts produces the same theory (BTreeMap keyed by id)

#[test]
fn reordered_inserts_yield_identical_theory() {
    let mut t_ab = Theory::new();
    t_ab.insert(Statement::atomic("a", "A", "test"));
    t_ab.insert(Statement::atomic("b", "B", "test"));

    let mut t_ba = Theory::new();
    t_ba.insert(Statement::atomic("b", "B", "test"));
    t_ba.insert(Statement::atomic("a", "A", "test"));

    assert_eq!(t_ab.len(), t_ba.len());

    let checker = FormulaCoherenceChecker;
    let r_ab = checker.check_consistency(&t_ab);
    let r_ba = checker.check_consistency(&t_ba);
    assert_eq!(r_ab.is_ok(), r_ba.is_ok(), "reordered theory must yield same coherence");
}

// ---------------------------------------------------------------------------
// MR4: Whitespace variants produce the same formula

#[test]
fn whitespace_variant_of_implication_same_formula() {
    let canonical = Formula::parse("A -> B").unwrap();
    // Note: the parser searches for " -> " (space-delimited); tab before "->" is not supported.
    for variant in &["A  ->  B", "  A -> B  ", "A ->  B"] {
        let f = Formula::parse(variant)
            .unwrap_or_else(|| panic!("must parse: {variant:?}"));
        assert_eq!(f, canonical, "whitespace variant {variant:?} must equal canonical formula");
    }
}

#[test]
fn case_variant_of_atom_same_formula() {
    let lower = Formula::parse("hello world").unwrap();
    let upper = Formula::parse("HELLO WORLD").unwrap();
    let mixed = Formula::parse("Hello World").unwrap();
    assert_eq!(lower, upper);
    assert_eq!(upper, mixed);
}

// ---------------------------------------------------------------------------
// MR5: Contradiction is symmetric in check_extension

#[test]
fn contradiction_check_extension_is_symmetric() {
    let checker = FormulaCoherenceChecker;

    for &(lhs, rhs) in &[("P", "NOT P"), ("NOT Q", "Q"), ("A -> B", "NOT (A -> B)")] {
        let mut theory_lhs = Theory::new();
        theory_lhs.insert(Statement::atomic("s1", lhs, "test"));
        let cand_rhs = Statement::atomic("c1", rhs, "test");

        let mut theory_rhs = Theory::new();
        theory_rhs.insert(Statement::atomic("s1", rhs, "test"));
        let cand_lhs = Statement::atomic("c2", lhs, "test");

        let r1 = checker.check_extension(&theory_lhs, &cand_rhs);
        let r2 = checker.check_extension(&theory_rhs, &cand_lhs);

        assert_eq!(r1.is_err(), r2.is_err(),
            "contradiction must be symmetric for ({lhs:?}, {rhs:?})");
    }
}

// ---------------------------------------------------------------------------
// MR6: Extending theory with derived facts must not weaken prior grounding

#[test]
fn adding_derived_grounded_fact_preserves_overall_grounding() {
    use stf_sir::compiler::inference::{FormulaInferenceEngine, InferenceEngine};

    let mut theory = Theory::new();
    theory.insert(Statement::grounded("a", "A", "logic", "sha:src"));
    theory.insert(Statement::grounded("ab", "A -> B", "logic", "sha:src"));

    let derived = FormulaInferenceEngine.derive(&theory);
    let was_grounded_count = theory.iter().filter(|s| s.provenance.grounded).count();

    // Add the derived statement to theory.
    let mut extended = theory.clone();
    for d in derived {
        extended.insert(d.statement);
    }

    // Original statements must still be grounded.
    for id in &["a", "ab"] {
        let stmt = extended.statements.get(*id).unwrap();
        assert!(stmt.provenance.grounded, "{id} must remain grounded after extension");
    }

    // Extended theory has more statements but at least as many grounded.
    let new_grounded_count = extended.iter().filter(|s| s.provenance.grounded).count();
    assert!(new_grounded_count >= was_grounded_count,
        "grounded count must not decrease after adding derived facts");
}

// ---------------------------------------------------------------------------
// MR7: Semantically equivalent normalization detects the same contradiction

#[test]
fn normalized_atom_contradicts_normalized_negation() {
    // "hello" vs "NOT hello" — both normalise to uppercase → same detection.
    let checker_f = FormulaCoherenceChecker;
    let checker_s = SimpleLogicChecker;

    let mut t_lower = Theory::new();
    t_lower.insert(Statement::atomic("s1", "hello", "test"));
    t_lower.insert(Statement::atomic("s2", "not hello", "test"));

    let mut t_upper = Theory::new();
    t_upper.insert(Statement::atomic("s1", "HELLO", "test"));
    t_upper.insert(Statement::atomic("s2", "NOT HELLO", "test"));

    // Both checkers must agree, and lower/upper must give the same result.
    assert!(checker_f.check_consistency(&t_lower).is_err(), "lower must be contradictory (formula)");
    assert!(checker_f.check_consistency(&t_upper).is_err(), "upper must be contradictory (formula)");
    assert!(checker_s.check_consistency(&t_lower).is_err(), "lower must be contradictory (simple)");
    assert!(checker_s.check_consistency(&t_upper).is_err(), "upper must be contradictory (simple)");
}

// ---------------------------------------------------------------------------
// MR8: audit_theory vs evaluate_statement must agree on C_l

#[test]
fn audit_and_evaluate_agree_on_c_l_for_consistent_theory() {
    let engine = default_engine();

    let theory = common::grounded_theory(&[
        ("s1", "A",      "sha:src"),
        ("s2", "B",      "sha:src"),
        ("s3", "A -> B", "sha:src"),
    ]);

    // audit_theory assesses entire theory.
    let audit_result = engine.audit_theory(&theory);
    assert!(audit_result.coherence.logical.is_satisfied(), "audit must see consistent theory");

    // evaluate_statement with a non-contradicting candidate.
    let cand = Statement::grounded("c", "C", "test", "sha:src");
    let eval_result = engine.evaluate_statement(&theory, &cand);
    assert!(eval_result.coherence.logical.is_satisfied(), "evaluate must see no contradiction");
}
