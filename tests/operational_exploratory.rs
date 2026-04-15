//! Exploratory tests for operational coherence (C_o) and the inference engines.
//!
//! Probes: modus ponens correctness, chain inference, negation as antecedent,
//! duplicate premises, formula vs text fallback, premise recording,
//! and derivation determinism.

use stf_sir::compiler::inference::{
    FormulaInferenceEngine, InferenceEngine, RuleBasedInferenceEngine,
};
use stf_sir::model::{Formula, Statement, Theory};

fn formula_engine() -> FormulaInferenceEngine { FormulaInferenceEngine }
fn rule_engine()    -> RuleBasedInferenceEngine { RuleBasedInferenceEngine }

// Helper: build theory from (id, text) pairs.
fn theory(pairs: &[(&str, &str)]) -> Theory {
    let mut t = Theory::new();
    for &(id, text) in pairs {
        t.insert(Statement::atomic(id, text, "logic"));
    }
    t
}

// ---------------------------------------------------------------------------
// 1. Modus ponens fires with FormulaInferenceEngine

#[test]
fn single_modus_ponens_derives_conclusion() {
    let t = theory(&[("a", "A"), ("imp", "A -> B")]);
    let derived = formula_engine().derive(&t);
    assert_eq!(derived.len(), 1, "must derive exactly one conclusion");
    assert_eq!(derived[0].statement.text, "B");
}

#[test]
fn modus_ponens_records_correct_premises() {
    let t = theory(&[("premise", "P"), ("rule", "P -> Q")]);
    let derived = formula_engine().derive(&t);
    assert_eq!(derived.len(), 1);
    let d = &derived[0];
    assert!(d.premises.contains(&"premise".to_string()));
    assert!(d.premises.contains(&"rule".to_string()));
    assert_eq!(d.premises.len(), 2);
}

// ---------------------------------------------------------------------------
// 2. Multiple conclusions from same antecedent

#[test]
fn multiple_implications_same_antecedent_derives_all() {
    // A and A->B and A->C → derives both B and C.
    let t = theory(&[("a", "A"), ("ab", "A -> B"), ("ac", "A -> C")]);
    let derived = formula_engine().derive(&t);
    let texts: Vec<&str> = derived.iter().map(|d| d.statement.text.as_str()).collect();
    assert!(texts.contains(&"B"), "must derive B; got {texts:?}");
    assert!(texts.contains(&"C"), "must derive C; got {texts:?}");
    assert_eq!(derived.len(), 2);
}

// ---------------------------------------------------------------------------
// 3. Negation as antecedent (current engine limitation)

#[test]
fn negation_as_antecedent_does_not_fire_modus_ponens() {
    // The FormulaInferenceEngine only treats Atom nodes as antecedent candidates.
    // Not(Atom("A")) is classified as a negation and filtered from the atoms list.
    // Therefore "NOT A" + "NOT A -> B" (even with a correct pre-parsed formula
    // for the implication) produces no derivation in the current engine.
    use stf_sir::model::Formula;
    use stf_sir::model::Statement;
    let mut t = stf_sir::model::Theory::new();
    t.insert(Statement::atomic("na", "NOT A", "logic"));
    // Pre-parse the implication so the AST is Implies(Not(A), B), not Not(Implies(A,B)).
    t.insert(
        Statement::atomic("rule", "NOT A -> B", "logic")
            .with_formula(Formula::implies(
                Formula::not(Formula::atom("A")),
                Formula::atom("B"),
            ))
    );
    // The engine only collects non-negation, non-implication atoms.
    // Not(Atom("A")) is a negation → filtered out → no matching premise found.
    let derived = formula_engine().derive(&t);
    assert_eq!(derived.len(), 0,
        "engine atom filter excludes negation antecedents; negation-as-antecedent requires engine extension");
}

// ---------------------------------------------------------------------------
// 4. No derivation cases

#[test]
fn no_derivation_without_implication() {
    let t = theory(&[("a", "A"), ("b", "B"), ("c", "C")]);
    assert!(formula_engine().derive(&t).is_empty(), "no implication → no derivation");
}

#[test]
fn no_derivation_without_matching_premise() {
    let t = theory(&[("imp", "A -> B")]); // "A" not present
    assert!(formula_engine().derive(&t).is_empty());
}

#[test]
fn no_derivation_when_conclusion_already_in_theory() {
    let t = theory(&[("a", "A"), ("imp", "A -> B"), ("b", "B")]); // B already present
    assert!(formula_engine().derive(&t).is_empty(),
        "must not re-derive already-present conclusion");
}

// ---------------------------------------------------------------------------
// 5. Duplicate premise inserts — one statement wins

#[test]
fn duplicate_premise_id_does_not_double_derive() {
    // Inserting same id twice replaces the statement; only one "A" in theory.
    let mut t = Theory::new();
    t.insert(Statement::atomic("a", "A", "logic"));
    t.insert(Statement::atomic("a", "A", "logic")); // duplicate id
    t.insert(Statement::atomic("imp", "A -> B", "logic"));
    let derived = formula_engine().derive(&t);
    assert_eq!(derived.len(), 1, "duplicate premise must not cause double derivation");
}

// ---------------------------------------------------------------------------
// 6. Chain inference — only one hop per derive() call

#[test]
fn chain_produces_only_first_hop() {
    // A, A->B, B->C: in one call only B is derived (B->C needs B in theory).
    let t = theory(&[("a", "A"), ("ab", "A -> B"), ("bc", "B -> C")]);
    let derived = formula_engine().derive(&t);
    assert_eq!(derived.len(), 1);
    assert_eq!(derived[0].statement.text, "B");
}

// ---------------------------------------------------------------------------
// 7. Derived statement carries formula

#[test]
fn derived_statement_carries_conclusion_formula() {
    let t = theory(&[("a", "A"), ("imp", "A -> B")]);
    let derived = formula_engine().derive(&t);
    assert_eq!(derived.len(), 1);
    assert_eq!(derived[0].statement.formula, Some(Formula::atom("B")),
        "derived statement must carry conclusion formula");
}

// ---------------------------------------------------------------------------
// 8. Pre-embedded formula is used instead of re-parsing

#[test]
fn embedded_formula_overrides_text_for_inference() {
    // Statement text says "X" but embedded formula is Atom("Y").
    // The inference engine must use the formula, so it looks for Implies(Atom("Y"), _).
    let mut t = Theory::new();
    // atom with embedded formula = Y (despite text being "X")
    let atom = Statement::atomic("a", "X", "logic")
        .with_formula(Formula::atom("Y"));
    // implication "Y -> Z" (embedded formula and text agree)
    let imp = Statement::atomic("imp", "Y -> Z", "logic")
        .with_formula(Formula::implies(Formula::atom("Y"), Formula::atom("Z")));
    t.insert(atom);
    t.insert(imp);

    let derived = formula_engine().derive(&t);
    assert_eq!(derived.len(), 1, "must derive via embedded formula Y even though text says X");
    assert_eq!(derived[0].statement.text, "Z");
}

// ---------------------------------------------------------------------------
// 9. Rule-based engine backward-compat check

#[test]
fn rule_based_engine_fires_on_literal_pattern() {
    // RuleBasedInferenceEngine: only fires when text (uppercased) exactly matches
    // "A" + "A -> B" pattern.  Very strict string match.
    let t = theory(&[("a", "A"), ("imp", "A -> B")]);
    let derived = rule_engine().derive(&t);
    assert_eq!(derived.len(), 1, "rule-based engine must fire on literal 'A' + 'A -> B'");
    assert_eq!(derived[0].statement.text, "B");
}

#[test]
fn rule_based_fires_for_any_antecedent_with_b_conclusion() {
    // RuleBasedInferenceEngine constructs "{text_a} -> B" and searches for it.
    // Any antecedent whose implication "{X} -> B" exists in the theory will fire.
    // "HELLO" + "HELLO -> B" → derives "B".
    let t = theory(&[("h", "HELLO"), ("imp", "HELLO -> B")]);
    let derived = rule_engine().derive(&t);
    assert_eq!(derived.len(), 1,
        "rule-based engine fires whenever '{{antecedent}} -> B' exists, for any antecedent");
    assert_eq!(derived[0].statement.text, "B");
}

// ---------------------------------------------------------------------------
// 10. Determinism

#[test]
fn formula_engine_derivation_is_deterministic() {
    let t = theory(&[("a", "A"), ("imp", "A -> B")]);
    let d1 = formula_engine().derive(&t);
    let d2 = formula_engine().derive(&t);
    assert_eq!(d1.len(), d2.len(), "derivation count must be deterministic");
    for (r1, r2) in d1.iter().zip(d2.iter()) {
        assert_eq!(r1.statement.text, r2.statement.text);
        assert_eq!(r1.rule_id, r2.rule_id);
    }
}

// ---------------------------------------------------------------------------
// 11. Rule id is correctly set

#[test]
fn formula_engine_rule_id_is_modus_ponens_formula() {
    let t = theory(&[("a", "A"), ("imp", "A -> B")]);
    let derived = formula_engine().derive(&t);
    assert_eq!(derived[0].rule_id, "modus_ponens_formula");
}

#[test]
fn rule_engine_rule_id_is_modus_ponens() {
    let t = theory(&[("a", "A"), ("imp", "A -> B")]);
    let derived = rule_engine().derive(&t);
    assert_eq!(derived[0].rule_id, "modus_ponens");
}
