//! Exploratory tests for logical coherence (C_l) — both checkers.
//!
//! Goals:
//!   - Prove both checkers agree on the same inputs (false positive/negative parity).
//!   - Exercise contradiction detection at scale (large theory, late contradiction).
//!   - Verify near-misses never produce false positives.
//!   - Prove step counts grow predictably for budget reasoning.

mod common;

use stf_sir::compiler::coherence::{
    FormulaCoherenceChecker, LogicalCoherenceChecker, SimpleLogicChecker,
};
use stf_sir::model::{Statement, Theory};

fn simple() -> SimpleLogicChecker   { SimpleLogicChecker }
fn formula() -> FormulaCoherenceChecker { FormulaCoherenceChecker }

// ---------------------------------------------------------------------------
// 1. Table-driven: consistent theories

#[test]
fn consistent_theories_table() {
    let cases: &[&[(&str, &str)]] = &[
        &[],                                      // empty
        &[("s1", "A")],                           // singleton
        &[("s1", "A"), ("s2", "B")],             // two unrelated
        &[("s1", "A"), ("s2", "A -> B")],        // atom + implication
        &[("s1", "A -> B"), ("s2", "B -> C")],   // two implications
        &[("s1", "NOT A"), ("s2", "B")],         // negation + unrelated
    ];
    for &stmts in cases {
        let theory = common::atomic_theory(stmts);
        assert!(simple().check_consistency(&theory).is_ok(),
            "SimpleLogicChecker: expected consistent for {stmts:?}");
        assert!(formula().check_consistency(&theory).is_ok(),
            "FormulaCoherenceChecker: expected consistent for {stmts:?}");
    }
}

// ---------------------------------------------------------------------------
// 2. Table-driven: contradictory theories

#[test]
fn contradictory_theories_table() {
    let cases: &[&[(&str, &str)]] = &[
        &[("s1", "A"), ("s2", "NOT A")],
        &[("s1", "B"), ("s2", "NOT B")],
        &[("s1", "NOT X"), ("s2", "X")],                   // reversed order
        &[("s1", "a"), ("s2", "not a")],                   // lowercase
        &[("s1", "  P  "), ("s2", "  NOT P  ")],           // extra whitespace
    ];
    for &stmts in cases {
        let theory = common::atomic_theory(stmts);
        assert!(simple().check_consistency(&theory).is_err(),
            "SimpleLogicChecker: expected contradiction for {stmts:?}");
        assert!(formula().check_consistency(&theory).is_err(),
            "FormulaCoherenceChecker: expected contradiction for {stmts:?}");
    }
}

// ---------------------------------------------------------------------------
// 3. Near-miss strings that must NOT trigger false positive

#[test]
fn near_miss_no_false_positive_table() {
    let cases: &[&[(&str, &str)]] = &[
        &[("s1", "A"), ("s2", "NOT B")],          // different propositions
        &[("s1", "NOT A"), ("s2", "NOT B")],       // two negations of different atoms
        &[("s1", "A"), ("s2", "A -> B")],          // implication is not negation
        &[("s1", "NOT A -> B"), ("s2", "A")],      // compound LHS, not a negation
        &[("s1", "A"), ("s2", "NOTA")],            // "NOTA" is an atom, not "NOT A"
        &[("s1", "A"), ("s2", "NOT AB")],          // "NOT AB" ≠ "NOT A"
    ];
    for &stmts in cases {
        let theory = common::atomic_theory(stmts);
        assert!(formula().check_consistency(&theory).is_ok(),
            "FormulaCoherenceChecker: false positive for {stmts:?}");
    }
}

// ---------------------------------------------------------------------------
// 4. Both checkers must agree: cross-validate

#[test]
fn checkers_agree_on_consistent_case() {
    let theory = common::atomic_theory(&[("s1", "P"), ("s2", "Q"), ("s3", "P -> Q")]);
    assert_eq!(
        simple().check_consistency(&theory).is_ok(),
        formula().check_consistency(&theory).is_ok(),
        "checkers must agree on consistency"
    );
}

#[test]
fn checkers_agree_on_contradiction() {
    let theory = common::atomic_theory(&[("s1", "P"), ("s2", "NOT P")]);
    assert_eq!(
        simple().check_consistency(&theory).is_err(),
        formula().check_consistency(&theory).is_err(),
        "checkers must agree on contradiction"
    );
}

// ---------------------------------------------------------------------------
// 5. Duplicate id: BTreeMap replacement — only one statement survives

#[test]
fn inserting_same_id_twice_keeps_last_version() {
    let mut theory = Theory::new();
    theory.insert(Statement::atomic("s1", "A", "test"));
    theory.insert(Statement::atomic("s1", "NOT A", "test")); // replaces s1
    // Only "NOT A" under s1; theory has one statement and must be consistent
    // (a single statement cannot contradict itself with this checker).
    assert_eq!(theory.len(), 1);
    assert!(simple().check_consistency(&theory).is_ok());
    assert!(formula().check_consistency(&theory).is_ok());
}

// ---------------------------------------------------------------------------
// 6. Large theory with one contradiction near the end

#[test]
fn large_theory_with_late_contradiction() {
    let mut theory = Theory::new();
    // Add 50 consistent statements.
    for i in 0..50 {
        theory.insert(Statement::atomic(format!("s{i}"), format!("PROP_{i}"), "test"));
    }
    // Add contradiction at the very end.
    theory.insert(Statement::atomic("s50", "PROP_0", "test")); // duplicate text
    theory.insert(Statement::atomic("s51", "NOT PROP_0", "test"));

    assert!(simple().check_consistency(&theory).is_err(),
        "must detect late contradiction in large theory (simple)");
    assert!(formula().check_consistency(&theory).is_err(),
        "must detect late contradiction in large theory (formula)");
}

// ---------------------------------------------------------------------------
// 7. Extension checks

#[test]
fn extension_with_non_contradicting_candidate_ok() {
    let theory = common::atomic_theory(&[("s1", "A"), ("s2", "B")]);
    let candidate = Statement::atomic("c1", "C", "test");
    assert!(simple().check_extension(&theory, &candidate).is_ok());
    assert!(formula().check_extension(&theory, &candidate).is_ok());
}

#[test]
fn extension_with_contradicting_candidate_err() {
    let theory = common::atomic_theory(&[("s1", "A"), ("s2", "B")]);
    let candidate = Statement::atomic("c1", "NOT A", "test");
    assert!(simple().check_extension(&theory, &candidate).is_err());
    assert!(formula().check_extension(&theory, &candidate).is_err());
}

#[test]
fn extending_empty_theory_always_succeeds() {
    let theory = Theory::new();
    for text in &["A", "NOT A", "A -> B", "NOT NOT A"] {
        let candidate = Statement::atomic("c", *text, "test");
        assert!(simple().check_extension(&theory, &candidate).is_ok(),
            "extending empty theory with {text:?} must succeed (simple)");
        assert!(formula().check_extension(&theory, &candidate).is_ok(),
            "extending empty theory with {text:?} must succeed (formula)");
    }
}

// ---------------------------------------------------------------------------
// 8. Steps are reported and grow predictably

#[test]
fn steps_increase_with_theory_size_on_extension() {
    use stf_sir::compiler::engine::formula_engine_with_budget;

    // Build theories of increasing size (all consistent with candidate "Z").
    let candidate = Statement::atomic("c", "Z", "test");
    let mut prev_steps = 0usize;

    for n in [1, 3, 5, 10] {
        let mut theory = Theory::new();
        for i in 0..n {
            theory.insert(Statement::atomic(format!("s{i}"), format!("ATOM_{i}"), "test"));
        }
        let engine = formula_engine_with_budget(usize::MAX);
        let result = engine.evaluate_statement(&theory, &candidate);
        assert!(result.steps_used >= prev_steps,
            "steps_used must be non-decreasing as theory grows (n={n}, steps={})", result.steps_used);
        prev_steps = result.steps_used;
    }
}

// ---------------------------------------------------------------------------
// 9. Formula-based contradiction is structurally precise

#[test]
fn formula_checker_uses_ast_not_string_for_detection() {
    // "NOT A" and "A" — formula checker detects via Not(Atom("A")).contradicts(Atom("A")).
    // Ensure the pre-parsed formula in Statement is used (no re-parsing needed).
    use stf_sir::model::Formula;
    let s1 = Statement::atomic("s1", "A", "test")
        .with_formula(Formula::atom("A"));
    let s2 = Statement::atomic("s2", "NOT A", "test")
        .with_formula(Formula::not(Formula::atom("A")));
    let mut theory = Theory::new();
    theory.insert(s1);
    theory.insert(s2);
    assert!(formula().check_consistency(&theory).is_err(),
        "formula checker must detect contradiction via pre-embedded formulas");
}

// ---------------------------------------------------------------------------
// 10. Contradiction classification in error output

#[test]
fn inconsistency_names_conflicting_ids() {
    let theory = common::atomic_theory(&[("alpha", "P"), ("beta", "NOT P")]);
    let err = formula().check_consistency(&theory).unwrap_err();
    assert!(
        err.conflicting_ids.contains(&"alpha".to_string())
            || err.conflicting_ids.contains(&"beta".to_string()),
        "conflicting_ids must name at least one of the contradicting statements"
    );
    assert_eq!(err.conflicting_ids.len(), 2,
        "must name exactly two conflicting statements");
}
