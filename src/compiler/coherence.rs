//! Logical coherence checking (C_l) — Definition C1 of the coherence paper.
//!
//! Two backends are provided:
//!
//! - `SimpleLogicChecker` — surface-text normalization (MVP, kept for
//!   compatibility with existing tests and fixtures).
//! - `FormulaCoherenceChecker` — uses the `Formula` AST from `model::formula`
//!   for structurally correct contradiction detection (Theorem A4).
//!
//! Both implement `LogicalCoherenceChecker`.  Callers can choose the backend
//! or compose them via the `BudgetedChecker` wrapper (see `engine.rs`).

use crate::model::formula::Formula;
use crate::model::statement::{Statement, StatementId};
use crate::model::theory::Theory;

/// A detected logical contradiction between two statements.
#[derive(Debug, Clone)]
pub struct Inconsistency {
    /// Human-readable explanation.
    pub message: String,
    /// Ids of the conflicting pair.
    pub conflicting_ids: Vec<StatementId>,
    /// Number of comparison steps taken to reach this result.
    pub steps: usize,
}

/// Trait for logical coherence checkers.
pub trait LogicalCoherenceChecker {
    /// Check whether `theory` is logically coherent.
    fn check_consistency(&self, theory: &Theory) -> Result<(), Inconsistency>;

    /// Check whether inserting `candidate` into `theory` preserves C_l.
    fn check_extension(
        &self,
        theory: &Theory,
        candidate: &Statement,
    ) -> Result<(), Inconsistency>;
}

// ---------------------------------------------------------------------------
// Backend 1: Surface-text checker (MVP)
// ---------------------------------------------------------------------------

/// Surface-text coherence checker.  Kept as the compatibility baseline.
pub struct SimpleLogicChecker;

impl SimpleLogicChecker {
    fn normalize(text: &str) -> String {
        text.trim().to_uppercase()
    }

    fn negate(text: &str) -> String {
        let n = Self::normalize(text);
        if let Some(rest) = n.strip_prefix("NOT ") {
            rest.to_string()
        } else {
            format!("NOT {n}")
        }
    }
}

impl LogicalCoherenceChecker for SimpleLogicChecker {
    fn check_consistency(&self, theory: &Theory) -> Result<(), Inconsistency> {
        let normalized: Vec<(StatementId, String)> = theory
            .iter()
            .map(|s| (s.id.clone(), Self::normalize(&s.text)))
            .collect();

        let mut steps = 0usize;
        for (id, text) in &normalized {
            let neg = Self::negate(text);
            for (other_id, other_text) in &normalized {
                steps += 1;
                if other_text == &neg {
                    return Err(Inconsistency {
                        message: format!("statement '{id}' contradicts '{other_id}'"),
                        conflicting_ids: vec![id.clone(), other_id.clone()],
                        steps,
                    });
                }
            }
        }
        Ok(())
    }

    fn check_extension(
        &self,
        theory: &Theory,
        candidate: &Statement,
    ) -> Result<(), Inconsistency> {
        let candidate_neg = Self::negate(&candidate.text);
        let mut steps = 0usize;

        for stmt in theory.iter() {
            steps += 1;
            if Self::normalize(&stmt.text) == candidate_neg {
                return Err(Inconsistency {
                    message: format!(
                        "candidate '{}' contradicts existing statement '{}'",
                        candidate.id, stmt.id
                    ),
                    conflicting_ids: vec![candidate.id.clone(), stmt.id.clone()],
                    steps,
                });
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Backend 2: Formula-AST checker
// ---------------------------------------------------------------------------

/// Formula-based coherence checker.
///
/// Parses each statement's text into a `Formula` and uses structural
/// `Formula::contradicts` for detection, eliminating false negatives from
/// whitespace or case differences and false positives from substring matches.
///
/// Contradiction count: O(n²) pairwise checks; step count is reported in
/// `Inconsistency.steps` and used by the C_c budget estimator in the engine.
pub struct FormulaCoherenceChecker;

impl LogicalCoherenceChecker for FormulaCoherenceChecker {
    fn check_consistency(&self, theory: &Theory) -> Result<(), Inconsistency> {
        let parsed: Vec<(StatementId, Option<Formula>)> = theory
            .iter()
            .map(|s| (s.id.clone(), Formula::parse(&s.text)))
            .collect();

        let mut steps = 0usize;
        for i in 0..parsed.len() {
            for j in (i + 1)..parsed.len() {
                steps += 1;
                let (id_a, fa) = &parsed[i];
                let (id_b, fb) = &parsed[j];
                if let (Some(a), Some(b)) = (fa, fb) {
                    if a.contradicts(b) {
                        return Err(Inconsistency {
                            message: format!(
                                "formula '{a}' (statement '{id_a}') contradicts \
                                 formula '{b}' (statement '{id_b}')"
                            ),
                            conflicting_ids: vec![id_a.clone(), id_b.clone()],
                            steps,
                        });
                    }
                }
            }
        }
        Ok(())
    }

    fn check_extension(
        &self,
        theory: &Theory,
        candidate: &Statement,
    ) -> Result<(), Inconsistency> {
        let candidate_formula = Formula::parse(&candidate.text);
        let mut steps = 0usize;

        for stmt in theory.iter() {
            steps += 1;
            if let (Some(cf), Some(sf)) = (
                candidate_formula.as_ref(),
                Formula::parse(&stmt.text).as_ref(),
            ) {
                if cf.contradicts(sf) {
                    return Err(Inconsistency {
                        message: format!(
                            "candidate formula '{}' (statement '{}') contradicts \
                             existing formula '{}' (statement '{}')",
                            cf, candidate.id, sf, stmt.id
                        ),
                        conflicting_ids: vec![candidate.id.clone(), stmt.id.clone()],
                        steps,
                    });
                }
            }
        }
        Ok(())
    }
}
