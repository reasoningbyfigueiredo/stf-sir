//! Exploratory tests for computational coherence (C_c) — the step-budget dimension.
//!
//! Design goals:
//!   1. Prove the `Unknown → Violated → Satisfied` lifecycle driven by budget.
//!   2. Prove monotonicity: budget N satisfied → budget N+1 satisfied.
//!   3. Prove steps_used is stable and explainable.
//!   4. Verify migration compat: default engine still reports Unknown.

use stf_sir::compiler::engine::{default_engine, formula_engine_with_budget};
use stf_sir::model::{Statement, Theory, TruthValue};

// Helper: build a consistent theory of size N.
fn consistent_theory(n: usize) -> Theory {
    let mut t = Theory::new();
    for i in 0..n {
        t.insert(Statement::atomic(format!("s{i}"), format!("ATOM_{i}"), "test"));
    }
    t
}

// Helper: non-contradicting candidate.
fn candidate() -> Statement {
    Statement::atomic("cand", "CANDIDATE_Z", "test")
}

// ---------------------------------------------------------------------------
// 1. Default engine always reports C_c = Unknown

#[test]
fn default_engine_cc_is_unknown() {
    let engine = default_engine();
    let theory = consistent_theory(5);
    let result = engine.evaluate_statement(&theory, &candidate());
    assert_eq!(result.coherence.computational, TruthValue::Unknown,
        "default engine must report C_c = Unknown (no budget)");
}

#[test]
fn default_engine_audit_cc_is_unknown() {
    let engine = default_engine();
    let theory = consistent_theory(5);
    let result = engine.audit_theory(&theory);
    assert_eq!(result.coherence.computational, TruthValue::Unknown,
        "default engine audit must report C_c = Unknown");
}

// ---------------------------------------------------------------------------
// 2. Budget = usize::MAX gives Unknown even with explicit FormulaEngine

#[test]
fn unbounded_formula_engine_cc_is_unknown() {
    let engine = formula_engine_with_budget(usize::MAX);
    let theory = consistent_theory(3);
    let result = engine.evaluate_statement(&theory, &candidate());
    assert_eq!(result.coherence.computational, TruthValue::Unknown);
}

// ---------------------------------------------------------------------------
// 3. Budget tight: exactly at theory.len() → Satisfied

#[test]
fn budget_equal_to_steps_is_satisfied() {
    let n = 5;
    let theory = consistent_theory(n);
    let engine = formula_engine_with_budget(n); // extension check uses n steps
    let result = engine.evaluate_statement(&theory, &candidate());

    assert_eq!(result.steps_used, n,
        "steps_used must equal theory.len() for consistent extension");
    assert_eq!(result.coherence.computational, TruthValue::Satisfied,
        "budget == steps must be Satisfied");
}

// ---------------------------------------------------------------------------
// 4. Budget one below theory.len() → Violated

#[test]
fn budget_below_steps_is_violated() {
    let n = 5;
    let theory = consistent_theory(n);
    let engine = formula_engine_with_budget(n - 1);
    let result = engine.evaluate_statement(&theory, &candidate());

    assert_eq!(result.coherence.computational, TruthValue::Violated,
        "budget < steps must be Violated");
}

// ---------------------------------------------------------------------------
// 5. Budget zero → Violated for non-empty theory

#[test]
fn zero_budget_with_nonempty_theory_is_violated() {
    let theory = consistent_theory(1);
    let engine = formula_engine_with_budget(0);
    let result = engine.evaluate_statement(&theory, &candidate());
    assert_eq!(result.coherence.computational, TruthValue::Violated);
}

// ---------------------------------------------------------------------------
// 6. Empty theory: steps = 0 → always Satisfied regardless of budget

#[test]
fn empty_theory_always_satisfied_for_any_budget() {
    let theory = Theory::new();
    for budget in [0, 1, 10] {
        let engine = formula_engine_with_budget(budget);
        let result = engine.evaluate_statement(&theory, &candidate());
        assert_eq!(result.steps_used, 0, "empty theory → 0 steps");
        assert_eq!(result.coherence.computational, TruthValue::Satisfied,
            "budget={budget}: 0 steps must be Satisfied");
    }
}

// ---------------------------------------------------------------------------
// 7. Monotonicity: budget N satisfied → budget N+k satisfied

#[test]
fn computational_coherence_is_monotone_in_budget() {
    let n = 4;
    let theory = consistent_theory(n);

    // Find the threshold: budget = n should be Satisfied, budget = n-1 Violated.
    let below = formula_engine_with_budget(n - 1)
        .evaluate_statement(&theory, &candidate());
    let at = formula_engine_with_budget(n)
        .evaluate_statement(&theory, &candidate());
    let above = formula_engine_with_budget(n + 10)
        .evaluate_statement(&theory, &candidate());

    assert_eq!(below.coherence.computational, TruthValue::Violated);
    assert_eq!(at.coherence.computational, TruthValue::Satisfied);
    assert_eq!(above.coherence.computational, TruthValue::Satisfied,
        "budget above threshold must remain Satisfied");
}

// ---------------------------------------------------------------------------
// 8. steps_used is stable across identical calls

#[test]
fn steps_used_is_deterministic() {
    let theory = consistent_theory(6);
    let engine = formula_engine_with_budget(usize::MAX);

    let r1 = engine.evaluate_statement(&theory, &candidate());
    let r2 = engine.evaluate_statement(&theory, &candidate());
    assert_eq!(r1.steps_used, r2.steps_used, "steps_used must be deterministic");
}

// ---------------------------------------------------------------------------
// 9. audit_theory budget: n*n/2 steps for consistent theory of size n

#[test]
fn audit_budget_matches_n_squared_over_2() {
    let n = 4usize;
    let theory = consistent_theory(n);
    let expected_steps = n * n / 2;

    // With budget exactly at expected → Satisfied.
    let engine = formula_engine_with_budget(expected_steps);
    let result = engine.audit_theory(&theory);

    assert_eq!(result.steps_used, expected_steps,
        "audit steps must equal n*n/2 for consistent theory");
    assert_eq!(result.coherence.computational, TruthValue::Satisfied,
        "budget == audit steps must be Satisfied");
}

#[test]
fn audit_budget_below_threshold_is_violated() {
    let n = 4usize;
    let theory = consistent_theory(n);
    let threshold = n * n / 2;

    let engine = formula_engine_with_budget(threshold - 1);
    let result = engine.audit_theory(&theory);
    assert_eq!(result.coherence.computational, TruthValue::Violated);
}

// ---------------------------------------------------------------------------
// 10. C_c = Violated does not affect C_l or C_o classification

#[test]
fn violated_cc_does_not_affect_cl() {
    // Even with budget=0, C_l is still evaluated correctly.
    let theory = consistent_theory(3);
    let engine = formula_engine_with_budget(0);
    let result = engine.evaluate_statement(&theory, &candidate());

    // C_l: no contradiction → Satisfied
    assert_eq!(result.coherence.logical, TruthValue::Satisfied,
        "C_l must be independent of budget");
    // C_c: over budget → Violated
    assert_eq!(result.coherence.computational, TruthValue::Violated);
}
