use std::collections::BTreeMap;

use super::statement::{Provenance, Statement, StatementId};

/// The trust level of a statement inserted into a [`Theory`] via
/// [`Theory::insert_guarded`].
///
/// Determined by a provenance heuristic (presence of `source_ids`, `anchors`,
/// or `grounded=true`). This is a lightweight inline check, not a full
/// evaluation by a [`crate::compiler::grounding::GroundingChecker`]; for
/// authoritative grounding, pass the statement through [`crate::compiler::StfEngine`].
///
/// # Note
/// `TrustLevel::Untrusted` does **not** prevent insertion. The statement is
/// always inserted; the outcome records the trust level for downstream inspection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TrustLevel {
    /// The statement has at least one source id, anchor, or `grounded=true`.
    Trusted,
    /// The statement has no verifiable provenance.
    ///
    /// It may be a hallucination candidate; use `StfEngine` for definitive
    /// classification.
    Untrusted,
}

/// The outcome of a guarded insertion into a [`Theory`].
///
/// The insertion is **always** performed regardless of [`TrustLevel`].
/// Use `trust_level` and `diagnostic` for downstream policy decisions.
///
/// # Invariants
/// - `inserted` is always `true`.
/// - `trust_level == TrustLevel::Untrusted` iff `diagnostic.is_some()`.
#[derive(Debug, Clone)]
pub struct InsertionOutcome {
    /// Always `true` — a guarded insertion never rejects.
    pub inserted: bool,
    /// The provenance-based trust classification.
    pub trust_level: TrustLevel,
    /// The id of the inserted statement.
    pub statement_id: StatementId,
    /// Diagnostic message when `trust_level == Untrusted`.
    pub diagnostic: Option<String>,
}

/// Classify a [`Provenance`] as [`TrustLevel::Trusted`] or [`TrustLevel::Untrusted`].
///
/// Mirrors the criterion used by
/// [`crate::compiler::grounding::ProvenanceGroundingChecker`] for consistency.
fn classify_provenance(p: &Provenance) -> TrustLevel {
    if !p.source_ids.is_empty() || !p.anchors.is_empty() || p.grounded {
        TrustLevel::Trusted
    } else {
        TrustLevel::Untrusted
    }
}

/// A theory M_A: the set of propositions held by an agent.
///
/// Corresponds to M_A in the coherence paper.  Internally keyed by
/// `StatementId` for O(log n) lookup.
#[derive(Debug, Clone, Default)]
pub struct Theory {
    pub statements: BTreeMap<StatementId, Statement>,
}

impl Theory {
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert a statement, replacing any existing entry with the same id.
    pub fn insert(&mut self, stmt: Statement) {
        self.statements.insert(stmt.id.clone(), stmt);
    }

    /// Remove a statement by id, returning it if present.
    pub fn remove(&mut self, id: &str) -> Option<Statement> {
        self.statements.remove(id)
    }

    /// Iterate over all statements.
    pub fn iter(&self) -> impl Iterator<Item = &Statement> {
        self.statements.values()
    }

    /// Number of statements in the theory.
    pub fn len(&self) -> usize {
        self.statements.len()
    }

    pub fn is_empty(&self) -> bool {
        self.statements.is_empty()
    }

    /// Return true if the theory contains a statement with the given id.
    pub fn contains(&self, id: &str) -> bool {
        self.statements.contains_key(id)
    }

    /// Build a Theory from an iterator of Statements.
    pub fn from_iter(iter: impl IntoIterator<Item = Statement>) -> Self {
        let mut theory = Self::new();
        for stmt in iter {
            theory.insert(stmt);
        }
        theory
    }

    /// Insert a statement and return its provenance-based trust classification.
    ///
    /// The insertion **always** occurs, regardless of trust level. Use the
    /// returned [`InsertionOutcome`] to inspect provenance quality and take
    /// downstream policy decisions (e.g., flagging statements for `StfEngine`
    /// verification).
    ///
    /// # Trust classification
    ///
    /// A statement is [`TrustLevel::Trusted`] if it has at least one of:
    /// - `provenance.source_ids` non-empty
    /// - `provenance.anchors` non-empty
    /// - `provenance.grounded == true`
    ///
    /// Otherwise it is [`TrustLevel::Untrusted`] (hallucination candidate).
    ///
    /// # Invariant (INV-104-3)
    /// After `insert_guarded`, `self.contains(&outcome.statement_id) == true`
    /// for any input statement.
    pub fn insert_guarded(&mut self, stmt: Statement) -> InsertionOutcome {
        let trust_level = classify_provenance(&stmt.provenance);
        let id = stmt.id.clone();
        let diagnostic = if trust_level == TrustLevel::Untrusted {
            Some(format!(
                "statement '{}' inserted without verifiable provenance (hallucination candidate)",
                id
            ))
        } else {
            None
        };
        self.insert(stmt); // delegates to existing insert — no logic duplication
        InsertionOutcome {
            inserted: true,
            trust_level,
            statement_id: id,
            diagnostic,
        }
    }
}
