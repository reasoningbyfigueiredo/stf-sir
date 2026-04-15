//! FEAT-203-2: Query executor.
//!
//! Evaluates `Query` AST nodes against a `SirGraph` + `Artifact` pair
//! and returns deterministic `QueryResult` values.

use std::collections::{BTreeMap, BTreeSet};
use std::time::Instant;

use crate::model::Artifact;
use crate::query::ast::{Dimension, Query};
use crate::query::result::QueryResult;
use crate::query::traversal::{
    ancestors_of, descendants_of, nodes_in_depth_range, path_between, subgraph_nodes,
};
use crate::sir::graph::SirGraph;

// ---------------------------------------------------------------------------
// QueryExecutor
// ---------------------------------------------------------------------------

/// Executes queries over a `SirGraph` + `Artifact` pair.
///
/// The executor is read-only (INV-203-3): no query modifies the graph or
/// artifact. All results are deterministic (INV-203-1).
pub struct QueryExecutor<'a> {
    graph: &'a SirGraph,
    artifact: &'a Artifact,
    /// Precomputed depth map: token_id → syntactic depth.
    depth_map: BTreeMap<String, usize>,
}

impl<'a> QueryExecutor<'a> {
    /// Create a new executor for the given graph and artifact.
    pub fn new(graph: &'a SirGraph, artifact: &'a Artifact) -> Self {
        let depth_map = artifact
            .ztokens
            .iter()
            .map(|t| (t.id.clone(), t.syntactic.depth))
            .collect();

        Self {
            graph,
            artifact,
            depth_map,
        }
    }

    /// Execute a query and return the matching result.
    ///
    /// This is the sole entry point (INV-203-2 / INV-203-3).
    pub fn execute(&self, query: &Query) -> QueryResult {
        let query_id = format!("{:?}", query as *const _);
        let start = Instant::now();

        let (token_ids, relation_ids) = self.eval(query);
        let elapsed = start.elapsed().as_micros() as u64;

        QueryResult::empty(query_id).with_tokens_and_relations(token_ids, relation_ids, elapsed)
    }

    // -----------------------------------------------------------------------
    // Internal recursive evaluator
    // -----------------------------------------------------------------------

    /// Returns `(matched_token_ids, matched_relation_ids)` for the query.
    fn eval(&self, query: &Query) -> (Vec<String>, Vec<String>) {
        match query {
            Query::ByType { node_type } => {
                let ids: Vec<String> = self
                    .artifact
                    .ztokens
                    .iter()
                    .filter(|t| &t.syntactic.node_type == node_type)
                    .map(|t| t.id.clone())
                    .collect();
                (ids, vec![])
            }

            Query::ByCategory { category } => {
                let mut token_ids: BTreeSet<String> = BTreeSet::new();
                let mut relation_ids: Vec<String> = Vec::new();

                for relation in &self.artifact.relations {
                    if relation.category.as_str() == category.as_str() {
                        relation_ids.push(relation.id.clone());
                        token_ids.insert(relation.source.clone());
                        token_ids.insert(relation.target.clone());
                    }
                }

                relation_ids.sort();
                (token_ids.into_iter().collect(), relation_ids)
            }

            Query::Ancestors { id } => {
                let ids = ancestors_of(self.graph, id);
                (ids, vec![])
            }

            Query::Descendants { id } => {
                let ids = descendants_of(self.graph, id);
                (ids, vec![])
            }

            Query::Path { from, to } => match path_between(self.graph, from, to) {
                Some(path) => (path, vec![]),
                None => (vec![], vec![]),
            },

            Query::Subgraph { root_id, max_depth } => {
                let ids = subgraph_nodes(self.graph, root_id, *max_depth);
                (ids, vec![])
            }

            Query::DepthRange { min, max } => {
                let ids = nodes_in_depth_range(&self.depth_map, *min, *max);
                (ids, vec![])
            }

            Query::RegexGloss { pattern } => {
                // Fallback: substring matching (no external regex dependency)
                let ids: Vec<String> = self
                    .artifact
                    .ztokens
                    .iter()
                    .filter(|t| {
                        Self::gloss_matches(&t.semantic.gloss, pattern)
                    })
                    .map(|t| t.id.clone())
                    .collect();
                (ids, vec![])
            }

            Query::DimensionFilter {
                dimension,
                field,
                value,
            } => {
                let ids: Vec<String> = self
                    .artifact
                    .ztokens
                    .iter()
                    .filter(|t| Self::dimension_matches(t, dimension, field, value))
                    .map(|t| t.id.clone())
                    .collect();
                (ids, vec![])
            }

            Query::And(lhs, rhs) => {
                let (lhs_tokens, lhs_rels) = self.eval(lhs);
                let (rhs_tokens, rhs_rels) = self.eval(rhs);

                let lhs_set: BTreeSet<String> = lhs_tokens.into_iter().collect();
                let rhs_set: BTreeSet<String> = rhs_tokens.into_iter().collect();
                let tokens: Vec<String> =
                    lhs_set.intersection(&rhs_set).cloned().collect();

                let lhs_rel_set: BTreeSet<String> = lhs_rels.into_iter().collect();
                let rhs_rel_set: BTreeSet<String> = rhs_rels.into_iter().collect();
                let rels: Vec<String> =
                    lhs_rel_set.intersection(&rhs_rel_set).cloned().collect();

                (tokens, rels)
            }

            Query::Or(lhs, rhs) => {
                let (lhs_tokens, lhs_rels) = self.eval(lhs);
                let (rhs_tokens, rhs_rels) = self.eval(rhs);

                let mut tokens: BTreeSet<String> = lhs_tokens.into_iter().collect();
                tokens.extend(rhs_tokens);

                let mut rels: BTreeSet<String> = lhs_rels.into_iter().collect();
                rels.extend(rhs_rels);

                (tokens.into_iter().collect(), rels.into_iter().collect())
            }

            Query::Not(inner) => {
                let (excluded_tokens, _) = self.eval(inner);
                let excluded: BTreeSet<String> = excluded_tokens.into_iter().collect();

                let all_ids: Vec<String> = self
                    .artifact
                    .ztokens
                    .iter()
                    .map(|t| t.id.clone())
                    .filter(|id| !excluded.contains(id))
                    .collect();

                (all_ids, vec![])
            }
        }
    }

    // -----------------------------------------------------------------------
    // Predicate helpers
    // -----------------------------------------------------------------------

    /// Returns `true` if `gloss` contains `pattern` as a substring.
    ///
    /// Future: replace with full regex when the `regex` crate is available.
    fn gloss_matches(gloss: &str, pattern: &str) -> bool {
        gloss.contains(pattern.as_ref())
    }

    /// Returns `true` if the ZToken's `dimension.field` matches `value`.
    fn dimension_matches(
        token: &crate::model::ZToken,
        dimension: &Dimension,
        field: &str,
        value: &str,
    ) -> bool {
        match dimension {
            Dimension::Lexical => match field {
                "source_text" => token.lexical.source_text == value,
                "plain_text" => token.lexical.plain_text == value,
                "normalized_text" => token.lexical.normalized_text == value,
                _ => false,
            },
            Dimension::Syntactic => match field {
                "node_type" => token.syntactic.node_type == value,
                "parent_id" => token
                    .syntactic
                    .parent_id
                    .as_deref()
                    .map_or(false, |p| p == value),
                "depth" => token.syntactic.depth.to_string() == value,
                "path" => token.syntactic.path == value,
                _ => false,
            },
            Dimension::Semantic => match field {
                "gloss" => token.semantic.gloss == value,
                "confidence" => token
                    .semantic
                    .confidence
                    .map_or(false, |c| c.to_string() == value),
                _ => {
                    // check if any concept matches
                    if field == "concepts" {
                        token.semantic.concepts.iter().any(|c| c == value)
                    } else {
                        false
                    }
                }
            },
            Dimension::Logical => {
                if field == "relation_ids" {
                    token.logical.relation_ids.iter().any(|r| r == value)
                } else {
                    false
                }
            }
        }
    }
}
