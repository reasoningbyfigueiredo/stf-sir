//! FEAT-207-5: SirGraph v2 serialization.
//!
//! Provides a stable, deterministic serialization of `SirGraph` to/from
//! a JSON/YAML-compatible export structure (`SirGraphExport`).
//!
//! The export format is `"stf-sir-sirgraph-v1"`. It is stored in the
//! `extensions.sirgraph` field of the ZMD artifact and does NOT affect
//! the byte-stable golden test output.

use serde::{Deserialize, Serialize};

use crate::sir::graph::{SirGraph, SirNodeKind};

// ---------------------------------------------------------------------------
// Export types
// ---------------------------------------------------------------------------

/// A serializable snapshot of a `SirGraph`.
///
/// Format identifier: `"stf-sir-sirgraph-v1"`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SirGraphExport {
    /// Always `"stf-sir-sirgraph-v1"`.
    pub format: String,
    /// All nodes in the graph, sorted by `id` for determinism.
    pub nodes: Vec<ExportNode>,
    /// All edges in the graph, sorted by `id` for determinism.
    pub edges: Vec<ExportEdge>,
}

/// A serializable representation of a single graph node.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct ExportNode {
    /// Stable node identifier (ZToken ID).
    pub id: String,
    /// Node type derived from the ZToken syntactic dimension.
    pub node_type: String,
}

/// A serializable representation of a single graph edge.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct ExportEdge {
    /// Stable edge identifier (Relation ID).
    pub id: String,
    /// Relation type (e.g. `"contains"`, `"precedes"`).
    #[serde(rename = "type")]
    pub type_: String,
    /// Source node ID.
    pub source: String,
    /// Target node ID.
    pub target: String,
    /// Relation category (`"structural"`, `"logical"`, `"semantic-link"`).
    pub category: String,
}

// ---------------------------------------------------------------------------
// SirGraphExport implementation
// ---------------------------------------------------------------------------

impl SirGraphExport {
    /// Construct a `SirGraphExport` from a `SirGraph`.
    ///
    /// Nodes and edges are sorted by `id` for deterministic serialization
    /// (INV-203-4 / INV-203-1).
    pub fn from_graph(graph: &SirGraph) -> Self {
        let mut nodes: Vec<ExportNode> = graph
            .nodes
            .iter()
            .map(|node| {
                let node_type = match &node.kind {
                    SirNodeKind::ZToken { node_type } => node_type.clone(),
                };
                ExportNode {
                    id: node.id.clone(),
                    node_type,
                }
            })
            .collect();

        nodes.sort();

        let mut edges: Vec<ExportEdge> = graph
            .edges
            .iter()
            .map(|edge| ExportEdge {
                id: edge.id.clone(),
                type_: edge.edge_type.clone(),
                source: edge.source.clone(),
                target: edge.target.clone(),
                category: edge.category.as_str().to_string(),
            })
            .collect();

        edges.sort();

        Self {
            format: "stf-sir-sirgraph-v1".to_string(),
            nodes,
            edges,
        }
    }

    /// Serialize this export to a JSON string.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Serialize this export to a pretty-printed JSON string.
    pub fn to_json_pretty(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Deserialize a `SirGraphExport` from a JSON string.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compiler::compile_markdown;

    #[test]
    fn export_round_trips_via_json() {
        let artifact = compile_markdown("# Hello\n\nWorld.\n", None).unwrap();
        let graph = artifact.as_sir_graph();
        let export = SirGraphExport::from_graph(&graph);

        let json = export.to_json().expect("serialization failed");
        let restored = SirGraphExport::from_json(&json).expect("deserialization failed");

        assert_eq!(export, restored);
    }

    #[test]
    fn export_format_is_v1() {
        let artifact = compile_markdown("# Test\n", None).unwrap();
        let graph = artifact.as_sir_graph();
        let export = SirGraphExport::from_graph(&graph);
        assert_eq!(export.format, "stf-sir-sirgraph-v1");
    }

    #[test]
    fn nodes_are_sorted() {
        let artifact = compile_markdown("# A\n\n## B\n\n### C\n", None).unwrap();
        let graph = artifact.as_sir_graph();
        let export = SirGraphExport::from_graph(&graph);

        let ids: Vec<&str> = export.nodes.iter().map(|n| n.id.as_str()).collect();
        let mut sorted = ids.clone();
        sorted.sort();
        assert_eq!(ids, sorted, "node IDs should be sorted");
    }
}
