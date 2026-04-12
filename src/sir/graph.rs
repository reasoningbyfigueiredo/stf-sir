use std::collections::BTreeMap;

use crate::model::{Artifact, RelationCategory};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SirNodeKind {
    ZToken { node_type: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SirNode {
    pub id: String,
    pub kind: SirNodeKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SirEdge {
    pub id: String,
    pub edge_type: String,
    pub category: RelationCategory,
    pub source: String,
    pub target: String,
    pub stage: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SirGraph {
    pub nodes: Vec<SirNode>,
    pub edges: Vec<SirEdge>,
    pub node_by_id: BTreeMap<String, usize>,
    pub outgoing: BTreeMap<String, Vec<usize>>,
    pub incoming: BTreeMap<String, Vec<usize>>,
}

impl SirGraph {
    pub fn from_artifact(artifact: &Artifact) -> Self {
        let nodes = artifact
            .ztokens
            .iter()
            .map(|ztoken| SirNode {
                id: ztoken.id.clone(),
                kind: SirNodeKind::ZToken {
                    node_type: ztoken.syntactic.node_type.clone(),
                },
            })
            .collect::<Vec<_>>();

        let edges = artifact
            .relations
            .iter()
            .map(|relation| SirEdge {
                id: relation.id.clone(),
                edge_type: relation.type_.clone(),
                category: relation.category,
                source: relation.source.clone(),
                target: relation.target.clone(),
                stage: relation.stage.clone(),
            })
            .collect::<Vec<_>>();

        let mut node_by_id = BTreeMap::new();
        let mut outgoing = BTreeMap::new();
        let mut incoming = BTreeMap::new();

        for (index, node) in nodes.iter().enumerate() {
            node_by_id.insert(node.id.clone(), index);
            outgoing.insert(node.id.clone(), Vec::new());
            incoming.insert(node.id.clone(), Vec::new());
        }

        for (index, edge) in edges.iter().enumerate() {
            if let Some(edge_indexes) = outgoing.get_mut(&edge.source) {
                edge_indexes.push(index);
            }
            if let Some(edge_indexes) = incoming.get_mut(&edge.target) {
                edge_indexes.push(index);
            }
        }

        Self {
            nodes,
            edges,
            node_by_id,
            outgoing,
            incoming,
        }
    }

    pub fn node(&self, id: &str) -> Option<&SirNode> {
        self.node_by_id.get(id).map(|index| &self.nodes[*index])
    }

    pub fn outgoing(&self, id: &str) -> Vec<&SirEdge> {
        self.outgoing
            .get(id)
            .into_iter()
            .flat_map(|indexes| indexes.iter().map(|index| &self.edges[*index]))
            .collect()
    }

    pub fn incoming(&self, id: &str) -> Vec<&SirEdge> {
        self.incoming
            .get(id)
            .into_iter()
            .flat_map(|indexes| indexes.iter().map(|index| &self.edges[*index]))
            .collect()
    }

    pub fn neighbors(&self, id: &str) -> Vec<&SirNode> {
        let mut neighbor_indexes = Vec::new();

        for edge in self.outgoing(id) {
            if let Some(index) = self.node_by_id.get(&edge.target) {
                if !neighbor_indexes.contains(index) {
                    neighbor_indexes.push(*index);
                }
            }
        }

        for edge in self.incoming(id) {
            if let Some(index) = self.node_by_id.get(&edge.source) {
                if !neighbor_indexes.contains(index) {
                    neighbor_indexes.push(*index);
                }
            }
        }

        neighbor_indexes
            .into_iter()
            .map(|index| &self.nodes[index])
            .collect()
    }

    pub fn edges_by_category(&self, category: RelationCategory) -> Vec<&SirEdge> {
        self.edges
            .iter()
            .filter(|edge| edge.category == category)
            .collect()
    }
}
