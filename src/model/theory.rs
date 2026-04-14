use std::collections::BTreeMap;

use super::statement::{Statement, StatementId};

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
}
