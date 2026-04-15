use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::model::Artifact;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NodeTypeChange {
    pub token_id: String,
    pub before: String,
    pub after: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StructuralDiff {
    pub added_tokens: Vec<String>,
    pub removed_tokens: Vec<String>,
    pub added_relations: Vec<String>,
    pub removed_relations: Vec<String>,
    pub modified_node_types: Vec<NodeTypeChange>,
}

/// Compute the structural diff between two artifacts.
///
/// Token IDs and relation IDs are compared using BTreeSet operations.
/// All result Vecs are sorted for determinism.
pub fn structural_diff(a: &Artifact, b: &Artifact) -> StructuralDiff {
    let ids_a: BTreeSet<&str> = a.ztokens.iter().map(|t| t.id.as_str()).collect();
    let ids_b: BTreeSet<&str> = b.ztokens.iter().map(|t| t.id.as_str()).collect();

    let added_tokens: Vec<String> = ids_b
        .difference(&ids_a)
        .map(|s| s.to_string())
        .collect();

    let removed_tokens: Vec<String> = ids_a
        .difference(&ids_b)
        .map(|s| s.to_string())
        .collect();

    let rel_ids_a: BTreeSet<&str> = a.relations.iter().map(|r| r.id.as_str()).collect();
    let rel_ids_b: BTreeSet<&str> = b.relations.iter().map(|r| r.id.as_str()).collect();

    let added_relations: Vec<String> = rel_ids_b
        .difference(&rel_ids_a)
        .map(|s| s.to_string())
        .collect();

    let removed_relations: Vec<String> = rel_ids_a
        .difference(&rel_ids_b)
        .map(|s| s.to_string())
        .collect();

    // Detect node type changes for tokens present in both artifacts.
    let type_map_a: std::collections::BTreeMap<&str, &str> = a
        .ztokens
        .iter()
        .map(|t| (t.id.as_str(), t.syntactic.node_type.as_str()))
        .collect();

    let mut modified_node_types: Vec<NodeTypeChange> = b
        .ztokens
        .iter()
        .filter_map(|t| {
            let before = type_map_a.get(t.id.as_str())?;
            let after = t.syntactic.node_type.as_str();
            if *before != after {
                Some(NodeTypeChange {
                    token_id: t.id.clone(),
                    before: before.to_string(),
                    after: after.to_string(),
                })
            } else {
                None
            }
        })
        .collect();

    modified_node_types.sort_by(|x, y| x.token_id.cmp(&y.token_id));

    StructuralDiff {
        added_tokens,
        removed_tokens,
        added_relations,
        removed_relations,
        modified_node_types,
    }
}
