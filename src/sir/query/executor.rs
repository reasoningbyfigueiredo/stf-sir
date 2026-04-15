//! FEAT-203-2: Query executor.

use std::collections::{BTreeMap, BTreeSet};
use std::time::Instant;

use crate::model::Artifact;
use crate::sir::graph::SirGraph;
use crate::sir::query::ast::{Dimension, Query};
use crate::sir::query::result::QueryResult;
use crate::sir::query::traversal::{
    ancestors_of, descendants_of, nodes_in_depth_range, path_between, subgraph_nodes,
};

/// Executes queries over a `SirGraph` + `Artifact` pair (read-only, INV-203-3).
pub struct QueryExecutor<'a> {
    graph: &'a SirGraph,
    artifact: &'a Artifact,
    depth_map: BTreeMap<String, usize>,
}

impl<'a> QueryExecutor<'a> {
    /// Create a new executor.
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
    pub fn execute(&self, query: &Query) -> QueryResult {
        let query_id = format!("q-{:p}", query as *const _);
        let start = Instant::now();

        let (token_ids, relation_ids) = self.eval(query);
        let elapsed = start.elapsed().as_micros() as u64;

        QueryResult::empty(query_id)
            .with_tokens_and_relations(token_ids, relation_ids, elapsed)
    }

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

            Query::Ancestors { id } => (ancestors_of(self.graph, id), vec![]),

            Query::Descendants { id } => (descendants_of(self.graph, id), vec![]),

            Query::Path { from, to } => match path_between(self.graph, from, to) {
                Some(path) => (path, vec![]),
                None => (vec![], vec![]),
            },

            Query::Subgraph { root_id, max_depth } => {
                (subgraph_nodes(self.graph, root_id, *max_depth), vec![])
            }

            Query::DepthRange { min, max } => {
                (nodes_in_depth_range(&self.depth_map, *min, *max), vec![])
            }

            Query::RegexGloss { pattern } => {
                let ids: Vec<String> = self
                    .artifact
                    .ztokens
                    .iter()
                    .filter(|t| t.semantic.gloss.contains(pattern.as_str()))
                    .map(|t| t.id.clone())
                    .collect();
                (ids, vec![])
            }

            Query::DimensionFilter { dimension, field, value } => {
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
                let (lt, lr) = self.eval(lhs);
                let (rt, rr) = self.eval(rhs);
                let ls: BTreeSet<String> = lt.into_iter().collect();
                let rs: BTreeSet<String> = rt.into_iter().collect();
                let tokens: Vec<String> = ls.intersection(&rs).cloned().collect();
                let lrs: BTreeSet<String> = lr.into_iter().collect();
                let rrs: BTreeSet<String> = rr.into_iter().collect();
                let rels: Vec<String> = lrs.intersection(&rrs).cloned().collect();
                (tokens, rels)
            }

            Query::Or(lhs, rhs) => {
                let (lt, lr) = self.eval(lhs);
                let (rt, rr) = self.eval(rhs);
                let mut tokens: BTreeSet<String> = lt.into_iter().collect();
                tokens.extend(rt);
                let mut rels: BTreeSet<String> = lr.into_iter().collect();
                rels.extend(rr);
                (tokens.into_iter().collect(), rels.into_iter().collect())
            }

            Query::Not(inner) => {
                let (excluded, _) = self.eval(inner);
                let excl_set: BTreeSet<String> = excluded.into_iter().collect();
                let all: Vec<String> = self
                    .artifact
                    .ztokens
                    .iter()
                    .map(|t| t.id.clone())
                    .filter(|id| !excl_set.contains(id))
                    .collect();
                (all, vec![])
            }
        }
    }

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
                "concepts" => token.semantic.concepts.iter().any(|c| c == value),
                _ => false,
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
