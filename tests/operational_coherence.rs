//! Tests for operational coherence (C_o) and ICE — Definition C3, I3.
//!
//! A statement is operationally sterile (C_o = 0) when inserting it into the
//! theory produces no new non-trivial consequence.
//! ICE = C_l ∧ C_o: the statement is both integrable and executable.

use stf_sir::compiler::engine::default_engine;
use stf_sir::error::ErrorKind;
use stf_sir::model::coherence::TruthValue;
use stf_sir::model::{Statement, Theory};

// ---------------------------------------------------------------------------

#[test]
fn grounded_statement_without_inference_rule_is_sterile() {
    let theory = Theory::new();
    // "Hello world" triggers no inference rule → C_o = 0.
    let candidate = Statement::grounded("s1", "Hello world", "test", "sha256:src");

    let engine = default_engine();
    let result = engine.evaluate_statement(&theory, &candidate);

    assert!(result.coherence.logical.is_satisfied(), "C_l must be satisfied");
    assert_eq!(result.coherence.operational, TruthValue::Violated, "C_o must be violated");
    assert!(!result.useful_information, "sterile statement is not useful information");

    let has_non_exec = result
        .errors
        .iter()
        .any(|e| e.kind == ErrorKind::NonExecutable);
    assert!(has_non_exec, "expected NonExecutable error");
}

#[test]
fn modus_ponens_triggers_operational_coherence() {
    // Theory: "A" and "A -> B"  → derive "B" → C_o = 1.
    let mut theory = Theory::new();
    theory.insert(Statement::grounded("s1", "A", "logic", "sha256:src"));
    theory.insert(Statement::grounded("s2", "A -> B", "logic", "sha256:src"));

    // Candidate triggers the rule (any new statement added to the theory).
    // We evaluate "A -> B" as a candidate against the theory containing "A".
    let mut theory_a = Theory::new();
    theory_a.insert(Statement::grounded("s1", "A", "logic", "sha256:src"));

    let candidate = Statement::grounded("s2", "A -> B", "logic", "sha256:src");

    let engine = default_engine();
    let result = engine.evaluate_statement(&theory_a, &candidate);

    assert!(result.coherence.logical.is_satisfied(), "C_l must be satisfied");
    assert_eq!(result.coherence.operational, TruthValue::Satisfied, "C_o must be satisfied");
    assert!(result.useful_information, "modus ponens result is useful information");
    assert!(result.derived_count > 0, "expected at least one derived statement");
}

#[test]
fn ice_false_for_contradiction() {
    let mut theory = Theory::new();
    theory.insert(Statement::grounded("s1", "A", "test", "sha256:src"));
    let candidate = Statement::grounded("s2", "NOT A", "test", "sha256:src");

    let engine = default_engine();
    let result = engine.evaluate_statement(&theory, &candidate);

    assert!(!result.useful_information, "contradictory statement must have ICE = false");
}

#[test]
fn ice_true_requires_both_integrable_and_executable() {
    // Setup: theory with "A", candidate "A -> B".
    let mut theory = Theory::new();
    theory.insert(Statement::grounded("s1", "A", "logic", "sha256:src"));
    let candidate = Statement::grounded("s2", "A -> B", "logic", "sha256:src");

    let engine = default_engine();
    let result = engine.evaluate_statement(&theory, &candidate);

    // C1: integrable (no contradiction)
    assert!(result.coherence.logical.is_satisfied());
    // C2: executable (modus ponens fires)
    assert_eq!(result.coherence.operational, TruthValue::Satisfied);
    // ICE = C1 ∧ C2
    assert!(result.useful_information);
}

#[test]
fn coherence_vector_full_when_all_dimensions_satisfied() {
    let mut theory = Theory::new();
    theory.insert(Statement::grounded("s1", "A", "logic", "sha256:src"));
    let candidate = Statement::grounded("s2", "A -> B", "logic", "sha256:src");

    let engine = default_engine();
    let result = engine.evaluate_statement(&theory, &candidate);

    // C_l and C_o satisfied; C_c left Unknown by SimpleLogicChecker.
    assert!(result.coherence.logical.is_satisfied());
    assert_eq!(result.coherence.operational, TruthValue::Satisfied);
}
