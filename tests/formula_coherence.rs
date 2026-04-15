//! Tests: FormulaCoherenceChecker operates on the Formula AST, not strings.
//!
//! Verifies that:
//!   - Contradiction is detected via structural `contradicts()`, not text matching
//!   - Pre-embedded formulas in `Statement.formula` are used directly (no re-parse)
//!   - Implications do not contradict atoms
//!   - `check_extension` enforces the same semantics

use stf_sir::compiler::coherence::{FormulaCoherenceChecker, LogicalCoherenceChecker};
use stf_sir::model::{Formula, Statement, Theory};

fn checker() -> FormulaCoherenceChecker {
    FormulaCoherenceChecker
}

// ---------------------------------------------------------------------------
// Consistent pairs

#[test]
fn atom_and_implication_are_consistent() {
    let mut theory = Theory::new();
    theory.insert(Statement::atomic("s1", "A", "logic"));
    theory.insert(Statement::atomic("s2", "A -> B", "logic"));
    assert!(checker().check_consistency(&theory).is_ok());
}

#[test]
fn two_unrelated_atoms_are_consistent() {
    let mut theory = Theory::new();
    theory.insert(Statement::atomic("s1", "P", "logic"));
    theory.insert(Statement::atomic("s2", "Q", "logic"));
    assert!(checker().check_consistency(&theory).is_ok());
}

// ---------------------------------------------------------------------------
// Contradiction via AST

#[test]
fn atom_and_not_atom_are_contradictory() {
    let mut theory = Theory::new();
    theory.insert(Statement::atomic("s1", "P", "logic"));
    theory.insert(Statement::atomic("s2", "NOT P", "logic"));
    let result = checker().check_consistency(&theory);
    assert!(result.is_err(), "P and NOT P must be contradictory");
    let inc = result.unwrap_err();
    assert!(
        inc.conflicting_ids.contains(&"s1".to_string())
            || inc.conflicting_ids.contains(&"s2".to_string())
    );
}

#[test]
fn embedded_formula_used_without_reparsing() {
    // Statements carry pre-parsed formulas; the checker must use them directly.
    let s1 = Statement::atomic("s1", "Q", "logic")
        .with_formula(Formula::atom("Q"));
    let s2 = Statement::atomic("s2", "NOT Q", "logic")
        .with_formula(Formula::not(Formula::atom("Q")));

    let mut theory = Theory::new();
    theory.insert(s1);
    theory.insert(s2);

    assert!(
        checker().check_consistency(&theory).is_err(),
        "checker must detect contradiction via embedded formulas"
    );
}

#[test]
fn not_not_p_does_not_contradict_p() {
    // NOT(NOT P) is structurally distinct from NOT P; it does not contradict P
    // directly under the `contradicts` definition (which only checks one layer).
    let mut theory = Theory::new();
    theory.insert(Statement::atomic("s1", "P", "logic"));
    // NOT NOT P is just an atom with text "NOT NOT P" at the string level,
    // but parsed as Not(Not(Atom("P"))).  Not(Not(Atom("P"))) does not equal
    // Atom("P") (no DNE in contradicts), so check must be consistent.
    let s2 = Statement::atomic("s2", "NOT P", "logic")
        .with_formula(Formula::not(Formula::not(Formula::atom("P"))));
    theory.insert(s2);
    // contradicts: (Not(inner), b) => inner == b;  Not(Not(P)) vs P => Not(P) == P? No.
    assert!(
        checker().check_consistency(&theory).is_ok(),
        "NOT NOT P does not directly contradict P in one-layer check"
    );
}

// ---------------------------------------------------------------------------
// Extension

#[test]
fn extension_detects_contradiction() {
    let mut theory = Theory::new();
    theory.insert(Statement::atomic("s1", "X", "logic"));
    let candidate = Statement::atomic("s2", "NOT X", "logic");
    assert!(checker().check_extension(&theory, &candidate).is_err());
}

#[test]
fn extension_allows_consistent_candidate() {
    let mut theory = Theory::new();
    theory.insert(Statement::atomic("s1", "A", "logic"));
    let candidate = Statement::atomic("s2", "B", "logic");
    assert!(checker().check_extension(&theory, &candidate).is_ok());
}
