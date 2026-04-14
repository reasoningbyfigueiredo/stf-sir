//! Tests for hallucination detection — Definition E2 / Theorem A10.
//!
//! A hallucination is locally coherent (C_l = 1) but ungrounded (Ground = 0).
//! The engine must emit an ErrorKind::Hallucination and NOT a Contradiction.

use stf_sir::compiler::engine::default_engine;
use stf_sir::error::ErrorKind;
use stf_sir::model::{Statement, Theory};

// ---------------------------------------------------------------------------

#[test]
fn ungrounded_coherent_statement_is_hallucination() {
    let theory = Theory::new();
    // Candidate: locally coherent (no contradiction), but no provenance.
    let candidate = Statement::atomic("h1", "The sky is green", "test");

    let engine = default_engine();
    let result = engine.evaluate_statement(&theory, &candidate);

    // C_l must be Satisfied (no contradiction with empty theory).
    assert!(result.coherence.logical.is_satisfied(), "expected C_l = satisfied");

    // Must be classified as Hallucination.
    let has_hallucination = result
        .errors
        .iter()
        .any(|e| e.kind == ErrorKind::Hallucination);
    assert!(has_hallucination, "expected Hallucination error, got: {:?}", result.errors);

    // Must NOT be a Contradiction.
    let has_contradiction = result
        .errors
        .iter()
        .any(|e| e.kind == ErrorKind::Contradiction);
    assert!(!has_contradiction, "hallucination must not be a Contradiction");
}

#[test]
fn grounded_statement_is_not_hallucination() {
    let theory = Theory::new();
    let candidate = Statement::grounded("g1", "A", "test", "sha256:source");

    let engine = default_engine();
    let result = engine.evaluate_statement(&theory, &candidate);

    let has_hallucination = result
        .errors
        .iter()
        .any(|e| e.kind == ErrorKind::Hallucination);
    assert!(!has_hallucination, "grounded statement must not be a hallucination");
}

#[test]
fn contradiction_with_existing_theory_is_not_hallucination() {
    let mut theory = Theory::new();
    theory.insert(Statement::grounded("s1", "A", "test", "sha256:src"));

    // Contradicts "A".
    let candidate = Statement::grounded("s2", "NOT A", "test", "sha256:src");

    let engine = default_engine();
    let result = engine.evaluate_statement(&theory, &candidate);

    // Must be Contradiction, not Hallucination.
    let has_contradiction = result
        .errors
        .iter()
        .any(|e| e.kind == ErrorKind::Contradiction);
    assert!(has_contradiction, "expected Contradiction error");

    let has_hallucination = result
        .errors
        .iter()
        .any(|e| e.kind == ErrorKind::Hallucination);
    assert!(!has_hallucination, "contradiction must not be classified as Hallucination");
}

#[test]
fn audit_theory_with_ungrounded_statements_reports_hallucinations() {
    let mut theory = Theory::new();
    theory.insert(Statement::grounded("s1", "A", "test", "sha256:src"));
    theory.insert(Statement::atomic("s2", "B", "test")); // ungrounded

    let engine = default_engine();
    let result = engine.audit_theory(&theory);

    let hallucination_ids: Vec<_> = result
        .errors
        .iter()
        .filter(|e| e.kind == ErrorKind::Hallucination)
        .flat_map(|e| e.statement_ids.iter().cloned())
        .collect();

    assert!(
        hallucination_ids.contains(&"s2".to_string()),
        "ungrounded statement s2 should be flagged as hallucination"
    );
    assert!(
        !hallucination_ids.contains(&"s1".to_string()),
        "grounded statement s1 should NOT be flagged"
    );
}
