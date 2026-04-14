//! STF coherence engine: orchestrates C_l, C_c, C_o, grounding, and ICE.
//!
//! The `StfEngine` is the central evaluator described in the coherence paper.
//! It evaluates a candidate statement against a theory and returns a typed
//! `EvaluationResult` that directly corresponds to the coherence triple,
//! grounding status, ICE indicator, and error taxonomy.
//!
//! ## Mapping to the paper
//!
//! | Paper concept       | Engine field              |
//! |---------------------|---------------------------|
//! | C_l                 | `coherence.logical`       |
//! | C_c                 | `coherence.computational` |
//! | C_o                 | `coherence.operational`   |
//! | Ground(x, W)        | `grounded`                |
//! | ICE(m, A) = C1 ∧ C2 | `useful_information`      |
//! | ErrorKind           | `errors`                  |
//!
//! ## Computational coherence (C_c)
//!
//! The paper (Theorem A2 / Definition A3) establishes that C_c = 1 iff
//! coherence verification is tractable (in **P**).  The engine approximates
//! this via a *step budget*: the number of comparison steps performed by the
//! `LogicalCoherenceChecker`.  If steps ≤ `step_budget`, C_c = Satisfied.
//! If steps exceed the budget, C_c = Violated (the problem is treated as
//! intractable for this agent).  The default budget is `usize::MAX` (unbounded),
//! which preserves the previous Unknown behaviour for well-structured inputs.
//!
//! This is an honest treatment: we do not claim to solve the P vs NP question;
//! we claim only to report whether *this specific check* terminated within the
//! agent's computational budget.

use crate::compiler::coherence::LogicalCoherenceChecker;
use crate::compiler::grounding::GroundingChecker;
use crate::compiler::inference::InferenceEngine;
use crate::error::{CoherenceError, ErrorKind, Severity};
use crate::model::coherence::{CoherenceVector, TruthValue};
use crate::model::statement::Statement;
use crate::model::theory::Theory;

/// The complete evaluation of a candidate statement against a theory.
#[derive(Debug, Clone)]
pub struct EvaluationResult {
    /// The coherence triple (C_l, C_c, C_o) for the extended theory.
    pub coherence: CoherenceVector,
    /// Whether the candidate statement is referentially grounded.
    pub grounded: bool,
    /// Number of new statements derived after inserting the candidate.
    pub derived_count: usize,
    /// ICE = C_l ∧ C_o: integrable and produces new consequences.
    pub useful_information: bool,
    /// Number of comparison steps consumed by the logical coherence check.
    /// Exposed for budget tracking and C_c evaluation.
    pub steps_used: usize,
    /// All coherence errors detected.
    pub errors: Vec<CoherenceError>,
}

impl EvaluationResult {
    /// Serialize to a JSON-friendly value (for CLI --json output).
    pub fn to_json_value(&self) -> serde_json::Value {
        let errors: Vec<serde_json::Value> = self
            .errors
            .iter()
            .map(|e| {
                serde_json::json!({
                    "kind":          format!("{:?}", e.kind),
                    "severity":      format!("{:?}", e.severity),
                    "message":       e.message,
                    "statement_ids": e.statement_ids,
                })
            })
            .collect();

        serde_json::json!({
            "logical":            self.coherence.logical.to_string(),
            "computational":      self.coherence.computational.to_string(),
            "operational":        self.coherence.operational.to_string(),
            "grounded":           self.grounded,
            "useful_information": self.useful_information,
            "steps_used":         self.steps_used,
            "errors":             errors,
        })
    }
}

// ---------------------------------------------------------------------------

/// The coherence engine, generic over logic / inference / grounding backends.
///
/// ```text
/// StfEngine<L, I, G>
///   L: LogicalCoherenceChecker
///   I: InferenceEngine
///   G: GroundingChecker
/// ```
///
/// ## Step budget (C_c)
///
/// `step_budget` controls the C_c dimension:
/// - `usize::MAX` (default) → C_c = Unknown (no budget, no claim)
/// - `n < usize::MAX` → C_c = Satisfied if steps ≤ n, else Violated
pub struct StfEngine<L, I, G>
where
    L: LogicalCoherenceChecker,
    I: InferenceEngine,
    G: GroundingChecker,
{
    pub logic: L,
    pub inference: I,
    pub grounding: G,
    /// Maximum number of comparison steps before C_c is declared Violated.
    /// Set to `usize::MAX` for unbounded (C_c = Unknown).
    pub step_budget: usize,
}

impl<L, I, G> StfEngine<L, I, G>
where
    L: LogicalCoherenceChecker,
    I: InferenceEngine,
    G: GroundingChecker,
{
    fn cc_from_steps(&self, steps: usize) -> TruthValue {
        if self.step_budget == usize::MAX {
            TruthValue::Unknown
        } else if steps <= self.step_budget {
            TruthValue::Satisfied
        } else {
            TruthValue::Violated
        }
    }

    /// Evaluate whether `candidate` is coherently executable within `theory`.
    pub fn evaluate_statement(
        &self,
        theory: &Theory,
        candidate: &Statement,
    ) -> EvaluationResult {
        // --- C_l ---
        let logic_result = self.logic.check_extension(theory, candidate);
        let logical_ok = logic_result.is_ok();
        let steps = match &logic_result {
            Ok(_) => theory.len(),      // all elements scanned once
            Err(inc) => inc.steps,
        };

        // --- C_c ---
        let cc = self.cc_from_steps(steps);

        // --- Ground ---
        let grounding_result = self.grounding.check_grounding(candidate);

        // --- C_o (only meaningful when C_l holds) ---
        let derived = if logical_ok {
            let mut extended = theory.clone();
            extended.insert(candidate.clone());
            self.inference.derive(&extended)
        } else {
            Vec::new()
        };
        let operational_ok = !derived.is_empty();

        // --- ICE ---
        let useful_information = logical_ok && operational_ok;

        // --- Errors ---
        let mut errors = Vec::new();

        if !logical_ok {
            if let Err(inc) = logic_result {
                errors.push(CoherenceError {
                    kind: ErrorKind::Contradiction,
                    message: inc.message,
                    statement_ids: inc.conflicting_ids,
                    severity: Severity::Critical,
                });
            }
        }

        if logical_ok && !grounding_result.is_grounded {
            errors.push(CoherenceError {
                kind: ErrorKind::Hallucination,
                message: format!(
                    "statement '{}' is locally coherent but ungrounded (Δ-tracking failure)",
                    candidate.id
                ),
                statement_ids: vec![candidate.id.clone()],
                severity: Severity::High,
            });
        }

        if logical_ok && !operational_ok {
            errors.push(CoherenceError {
                kind: ErrorKind::NonExecutable,
                message: format!(
                    "statement '{}' is coherent but operationally sterile",
                    candidate.id
                ),
                statement_ids: vec![candidate.id.clone()],
                severity: Severity::Medium,
            });
        }

        EvaluationResult {
            coherence: CoherenceVector {
                logical: if logical_ok {
                    TruthValue::Satisfied
                } else {
                    TruthValue::Violated
                },
                computational: cc,
                operational: if operational_ok {
                    TruthValue::Satisfied
                } else {
                    TruthValue::Violated
                },
            },
            grounded: grounding_result.is_grounded,
            derived_count: derived.len(),
            useful_information,
            steps_used: steps,
            errors,
        }
    }

    /// Evaluate the coherence of an entire theory (no candidate).
    pub fn audit_theory(&self, theory: &Theory) -> EvaluationResult {
        let logic_result = self.logic.check_consistency(theory);
        let logical_ok = logic_result.is_ok();
        let steps = match &logic_result {
            Ok(_) => {
                let n = theory.len();
                n * n / 2   // O(n²) pairs checked
            }
            Err(inc) => inc.steps,
        };

        let cc = self.cc_from_steps(steps);

        let derived = if logical_ok {
            self.inference.derive(theory)
        } else {
            Vec::new()
        };
        let operational_ok = !derived.is_empty();

        let ungrounded_ids: Vec<String> = theory
            .iter()
            .filter(|s| !self.grounding.check_grounding(s).is_grounded)
            .map(|s| s.id.clone())
            .collect();

        let mut errors = Vec::new();

        if !logical_ok {
            if let Err(inc) = logic_result {
                errors.push(CoherenceError {
                    kind: ErrorKind::Contradiction,
                    message: inc.message,
                    statement_ids: inc.conflicting_ids,
                    severity: Severity::Critical,
                });
            }
        }

        for id in &ungrounded_ids {
            errors.push(CoherenceError {
                kind: ErrorKind::Hallucination,
                message: format!("statement '{id}' is ungrounded"),
                statement_ids: vec![id.clone()],
                severity: Severity::High,
            });
        }

        if logical_ok && !operational_ok {
            errors.push(CoherenceError {
                kind: ErrorKind::NonExecutable,
                message: "theory produces no non-trivial consequences".into(),
                statement_ids: vec![],
                severity: Severity::Medium,
            });
        }

        EvaluationResult {
            coherence: CoherenceVector {
                logical: if logical_ok {
                    TruthValue::Satisfied
                } else {
                    TruthValue::Violated
                },
                computational: cc,
                operational: if operational_ok {
                    TruthValue::Satisfied
                } else {
                    TruthValue::Violated
                },
            },
            grounded: ungrounded_ids.is_empty(),
            derived_count: derived.len(),
            useful_information: logical_ok && operational_ok,
            steps_used: steps,
            errors,
        }
    }
}

// ---------------------------------------------------------------------------

/// Convenience alias for the default engine (simple text backends, no budget).
pub type DefaultEngine = StfEngine<
    crate::compiler::coherence::SimpleLogicChecker,
    crate::compiler::inference::RuleBasedInferenceEngine,
    crate::compiler::grounding::ProvenanceGroundingChecker,
>;

/// Build the default engine (unbounded step budget → C_c = Unknown).
pub fn default_engine() -> DefaultEngine {
    StfEngine {
        logic: crate::compiler::coherence::SimpleLogicChecker,
        inference: crate::compiler::inference::RuleBasedInferenceEngine,
        grounding: crate::compiler::grounding::ProvenanceGroundingChecker,
        step_budget: usize::MAX,
    }
}

/// Build the formula engine with explicit budget.
///
/// C_c = Satisfied if the consistency check terminates in ≤ `budget` steps.
pub type FormulaEngine = StfEngine<
    crate::compiler::coherence::FormulaCoherenceChecker,
    crate::compiler::inference::FormulaInferenceEngine,
    crate::compiler::grounding::ProvenanceGroundingChecker,
>;

pub fn formula_engine_with_budget(budget: usize) -> FormulaEngine {
    StfEngine {
        logic: crate::compiler::coherence::FormulaCoherenceChecker,
        inference: crate::compiler::inference::FormulaInferenceEngine,
        grounding: crate::compiler::grounding::ProvenanceGroundingChecker,
        step_budget: budget,
    }
}
