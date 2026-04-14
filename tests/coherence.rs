//! Tests for logical coherence (C_l) — Definition C1 of the coherence paper.

use stf_sir::model::{Statement, Theory};
use stf_sir::compiler::coherence::{LogicalCoherenceChecker, SimpleLogicChecker};

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
