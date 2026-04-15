//! FEAT-203-2: Query result types.
//!
//! `QueryResult` is the primary output of the query executor.
//! All `Vec` fields are sorted and deduplicated for determinism (INV-203-1).

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// QueryResult
// ---------------------------------------------------------------------------

/// The result of executing a single query against a `SirGraph`.
///
/// # Determinism
///
/// `token_ids` and `relation_ids` are always sorted lexicographically and
/// deduplicated. Identical graph + identical query always produces the same
/// `QueryResult` bytes (INV-203-1 / INV-203-4).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueryResult {
    /// Opaque identifier for this query invocation (caller-assigned).
    pub query_id: String,
    /// Sorted, deduplicated ZToken IDs of nodes matching the query.
    pub token_ids: Vec<String>,
    /// Sorted, deduplicated Relation IDs of edges matching the query.
    pub relation_ids: Vec<String>,
    /// Wall-clock execution time in microseconds.
    pub execution_time_us: u64,
}

impl QueryResult {
    /// Construct an empty result.
    pub fn empty(query_id: impl Into<String>) -> Self {
        Self {
            query_id: query_id.into(),
            token_ids: Vec::new(),
            relation_ids: Vec::new(),
            execution_time_us: 0,
        }
    }

    /// Returns `true` if no tokens and no relations were matched.
    pub fn is_empty(&self) -> bool {
        self.token_ids.is_empty() && self.relation_ids.is_empty()
    }

    /// Returns the number of matched tokens.
    pub fn token_count(&self) -> usize {
        self.token_ids.len()
    }

    /// Returns the number of matched relations.
    pub fn relation_count(&self) -> usize {
        self.relation_ids.len()
    }

    /// Consume a raw list of token IDs: sort and deduplicate them.
    pub(crate) fn with_tokens(
        mut self,
        mut token_ids: Vec<String>,
        execution_time_us: u64,
    ) -> Self {
        token_ids.sort();
        token_ids.dedup();
        self.token_ids = token_ids;
        self.execution_time_us = execution_time_us;
        self
    }

    /// Consume raw token + relation ID lists: sort and deduplicate both.
    pub(crate) fn with_tokens_and_relations(
        mut self,
        mut token_ids: Vec<String>,
        mut relation_ids: Vec<String>,
        execution_time_us: u64,
    ) -> Self {
        token_ids.sort();
        token_ids.dedup();
        relation_ids.sort();
        relation_ids.dedup();
        self.token_ids = token_ids;
        self.relation_ids = relation_ids;
        self.execution_time_us = execution_time_us;
        self
    }
}
