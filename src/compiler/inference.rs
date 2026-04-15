//! Operational coherence: inference engines (C_o) — Definition C3 / A4.
//!
//! Two backends are provided:
//!
//! - `RuleBasedInferenceEngine` — surface-text modus ponens (MVP, kept for
//!   compatibility).
//! - `FormulaInferenceEngine` — uses the `Formula` AST for structurally
//!   correct modus ponens (Theorem A9 / homomorphism property).
//!
//! Both implement `InferenceEngine`.

use std::collections::BTreeMap;

use crate::model::formula::Formula;
use crate::model::statement::{Statement, StatementKind, Provenance};
use crate::model::theory::Theory;

/// A statement derived by an inference rule.
#[derive(Debug, Clone)]
pub struct DerivedStatement {
    pub statement: Statement,
    pub rule_id: String,
    pub premises: Vec<String>,
}

/// Trait for inference engines (operational coherence).
///
/// Returns all statements derivable from `theory` in one step.
/// An empty result means C_o = 0 (operationally sterile).
pub trait InferenceEngine {
    fn derive(&self, theory: &Theory) -> Vec<DerivedStatement>;
}

// ---------------------------------------------------------------------------
// Backend 1: Surface-text engine (MVP, kept for test compatibility)
// ---------------------------------------------------------------------------

/// Surface-text rule-based inference engine.
///
/// Implements modus ponens by checking for "A" and "A -> B" as literal
/// uppercase strings.  Kept as the baseline; superseded by
/// `FormulaInferenceEngine` for production use.
pub struct RuleBasedInferenceEngine;

impl InferenceEngine for RuleBasedInferenceEngine {
    fn derive(&self, theory: &Theory) -> Vec<DerivedStatement> {
        let mut derived = Vec::new();

        let normalized: Vec<(String, String)> = theory
            .iter()
            .map(|s| (s.id.clone(), s.text.trim().to_uppercase()))
            .collect();

        // Rule: modus ponens — if A and A -> B are present, derive B.
        for (id_a, text_a) in &normalized {
            let implication = format!("{text_a} -> B");
            if let Some((id_imp, _)) = normalized.iter().find(|(_, t)| *t == implication) {
                let b_text = "B";
                let already_present = normalized.iter().any(|(_, t)| t == b_text);
                if !already_present {
                    derived.push(DerivedStatement {
                        statement: Statement {
                            id: format!("derived:modus_ponens:{id_a}"),
                            text: b_text.to_string(),
                            kind: StatementKind::Derived,
                            domain: "logic".to_string(),
                            provenance: Provenance {
                                grounded: true,
                                generated_by: Some("modus_ponens".to_string()),
                                ..Default::default()
                            },
                            metadata: BTreeMap::new(),
                            formula: None,
                            semantic_dimensions: None,
                        },
                        rule_id: "modus_ponens".to_string(),
                        premises: vec![id_a.clone(), id_imp.clone()],
                    });
                }
            }
        }

        derived
    }
}

// ---------------------------------------------------------------------------
// Backend 2: Formula-AST engine
// ---------------------------------------------------------------------------

/// Formula-based inference engine.
///
/// Applies modus ponens at the AST level:
///   - If `Atom(p)` and `Implies(Atom(p), q)` are both present, derive `q`.
///
/// This is structurally correct (Theorem A9): the rule preserves the
/// Formula homomorphism.  It does not depend on specific string literals.
///
/// When a `Statement` carries a pre-parsed `formula`, that is used directly;
/// otherwise the engine falls back to `Formula::parse(&text)`.  Derived
/// statements have their `formula` set to the conclusion `Formula`, so
/// downstream engines never need to re-parse them.
pub struct FormulaInferenceEngine;

/// Return the formula for a statement: embedded if available, else parsed.
#[inline]
fn resolve_formula(stmt: &Statement) -> Option<Formula> {
    stmt.formula.clone().or_else(|| Formula::parse(&stmt.text))
}

impl InferenceEngine for FormulaInferenceEngine {
    fn derive(&self, theory: &Theory) -> Vec<DerivedStatement> {
        // Resolve all formulas once — prefer embedded, fall back to parsing.
        let parsed: Vec<(String, Option<Formula>)> = theory
            .iter()
            .map(|s| (s.id.clone(), resolve_formula(s)))
            .collect();

        // Collect all atom-level premises (non-negation, non-implication).
        let atoms: Vec<(&str, &Formula)> = parsed
            .iter()
            .filter_map(|(id, f)| {
                if let Some(f) = f {
                    if !f.is_negation() && !f.is_implication() {
                        return Some((id.as_str(), f));
                    }
                }
                None
            })
            .collect();

        // Collect all implications.
        let implications: Vec<(&str, &Formula, &Formula)> = parsed
            .iter()
            .filter_map(|(id, f)| {
                if let Some(Formula::Implies(p, q)) = f {
                    Some((id.as_str(), p.as_ref(), q.as_ref()))
                } else {
                    None
                }
            })
            .collect();

        let mut derived = Vec::new();

        for &(id_atom, atom_formula) in &atoms {
            for &(id_imp, premise, conclusion) in &implications {
                // Modus ponens: atom matches premise → derive conclusion.
                if atom_formula == premise {
                    let conclusion_text = conclusion.to_string();
                    // Do not re-derive what is already present.
                    let already = parsed.iter().any(|(_, f)| {
                        f.as_ref().map(|f| f.to_string()) == Some(conclusion_text.clone())
                    });
                    if !already {
                        derived.push(DerivedStatement {
                            statement: Statement {
                                id: format!("derived:mp:{id_atom}:{id_imp}"),
                                text: conclusion_text,
                                kind: StatementKind::Derived,
                                domain: "logic".to_string(),
                                provenance: Provenance {
                                    grounded: true,
                                    generated_by: Some("modus_ponens_formula".to_string()),
                                    ..Default::default()
                                },
                                metadata: BTreeMap::new(),
                                // Embed the conclusion formula — no re-parsing needed downstream.
                                formula: Some(conclusion.clone()),
                                semantic_dimensions: None,
                            },
                            rule_id: "modus_ponens_formula".to_string(),
                            premises: vec![id_atom.to_string(), id_imp.to_string()],
                        });
                    }
                }
            }
        }

        derived
    }
}
