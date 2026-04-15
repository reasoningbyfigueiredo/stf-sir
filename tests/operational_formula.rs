//! Tests: FormulaInferenceEngine derives via the Formula AST.
//!
//! Verifies that:
//!   - Modus ponens fires when Atom(p) and Implies(Atom(p), q) are both present
//!   - Derived statement carries the conclusion Formula directly (no re-parsing)
//!   - No derivation when premise is absent
//!   - No re-derivation when conclusion is already in theory
//!   - Pre-embedded formulas are used instead of re-parsing text

use stf_sir::compiler::inference::{FormulaInferenceEngine, InferenceEngine};
use stf_sir::model::{Formula, Statement, Theory};

fn engine() -> FormulaInferenceEngine {
    FormulaInferenceEngine
}

// ---------------------------------------------------------------------------

#[test]
fn modus_ponens_derives_conclusion() {
    let mut theory = Theory::new();
    theory.insert(Statement::atomic("a", "A", "logic"));
    theory.insert(Statement::atomic("imp", "A -> B", "logic"));

    let derived = engine().derive(&theory);
    assert_eq!(derived.len(), 1, "exactly one consequence must be derived");
    assert_eq!(derived[0].statement.text, "B");
    assert_eq!(derived[0].rule_id, "modus_ponens_formula");
    assert_eq!(derived[0].premises, vec!["a".to_string(), "imp".to_string()]);
}

#[test]
fn derived_statement_carries_conclusion_formula() {
    let mut theory = Theory::new();
    theory.insert(Statement::atomic("a", "A", "logic"));
    theory.insert(Statement::atomic("imp", "A -> B", "logic"));

    let derived = engine().derive(&theory);
    assert_eq!(derived.len(), 1);
    // Conclusion formula must be Atom("B") — embedded, not re-parsed.
    assert_eq!(
        derived[0].statement.formula,
        Some(Formula::atom("B")),
        "derived statement must carry conclusion formula"
    );
}

#[test]
fn no_derivation_when_premise_absent() {
    let mut theory = Theory::new();
    theory.insert(Statement::atomic("imp", "A -> B", "logic"));
    // "A" is not in theory

    let derived = engine().derive(&theory);
    assert!(derived.is_empty(), "no derivation without premise");
}

#[test]
fn no_rederivation_when_conclusion_already_present() {
    let mut theory = Theory::new();
    theory.insert(Statement::atomic("a", "A", "logic"));
    theory.insert(Statement::atomic("imp", "A -> B", "logic"));
    theory.insert(Statement::atomic("b", "B", "logic")); // already present

    let derived = engine().derive(&theory);
    assert!(derived.is_empty(), "must not re-derive existing conclusion");
}

#[test]
fn embedded_formula_used_for_premise_matching() {
    // Attach a pre-parsed formula to the atom; engine must match via formula, not text.
    let mut theory = Theory::new();
    let atom = Statement::atomic("a", "X", "logic")
        .with_formula(Formula::atom("X"));
    let imp = Statement::atomic("imp", "X -> Y", "logic")
        .with_formula(Formula::implies(Formula::atom("X"), Formula::atom("Y")));
    theory.insert(atom);
    theory.insert(imp);

    let derived = engine().derive(&theory);
    assert_eq!(derived.len(), 1, "engine must match via embedded formula");
    assert_eq!(derived[0].statement.text, "Y");
}

#[test]
fn chain_does_not_fire_in_single_pass() {
    // A → B → C: in one pass only B is derived (C requires B in theory).
    let mut theory = Theory::new();
    theory.insert(Statement::atomic("a",   "A",      "logic"));
    theory.insert(Statement::atomic("ab",  "A -> B", "logic"));
    theory.insert(Statement::atomic("bc",  "B -> C", "logic"));

    let derived = engine().derive(&theory);
    // Only B is derivable in one step; B→C fires only when B is in theory.
    assert_eq!(derived.len(), 1);
    assert_eq!(derived[0].statement.text, "B");
}
