use std::collections::BTreeMap;

use crate::model::{Relation, RelationCategory, ZToken};

/// Build the deterministic relation set for STF-SIR v1.
///
/// Emission order follows §7 of the spec:
/// 1. All `contains` relations in preorder traversal order.
/// 2. All `precedes` relations, grouped by parent and ordered by the parent's
///    position in the `ztokens` vector (which is preorder). Using the parent
///    *index* rather than the parent *id* prevents lexicographic pitfalls such
///    as `"z10" < "z2"` in documents with more than nine tokens.
///
/// Note on `stage`: STF-SIR v1 uses `stage` as relation provenance, not as a
/// semantic category. The relations emitted here are classified as
/// `category: structural`, but they still carry `stage: logical` because this
/// phase is the one that derives and emits them.
pub fn build_relations(ztokens: &mut [ZToken]) -> Vec<Relation> {
    let mut relations = Vec::new();
    let mut next_relation_id = 1usize;

    // Index every token id -> position so we can look up parent indices cheaply.
    let id_to_index: BTreeMap<&str, usize> = ztokens
        .iter()
        .enumerate()
        .map(|(index, token)| (token.id.as_str(), index))
        .collect();

    // contains: emitted in preorder of the child token.
    for token in ztokens.iter() {
        if let Some(parent_id) = &token.syntactic.parent_id {
            relations.push(Relation {
                id: format!("r{next_relation_id}"),
                type_: "contains".to_string(),
                category: RelationCategory::Structural,
                source: parent_id.clone(),
                target: token.id.clone(),
                stage: "logical".to_string(),
                attributes: BTreeMap::new(),
                extensions: BTreeMap::new(),
            });
            next_relation_id += 1;
        }
    }

    // Group siblings by parent's *index* in ztokens. None (roots) < Some(_),
    // and Some is ordered by integer index — which is preorder.
    let mut siblings: BTreeMap<Option<usize>, Vec<usize>> = BTreeMap::new();
    for (index, token) in ztokens.iter().enumerate() {
        let parent_index = token
            .syntactic
            .parent_id
            .as_deref()
            .and_then(|id| id_to_index.get(id).copied());
        siblings.entry(parent_index).or_default().push(index);
    }

    for sibling_indexes in siblings.values() {
        for window in sibling_indexes.windows(2) {
            let source = &ztokens[window[0]];
            let target = &ztokens[window[1]];

            relations.push(Relation {
                id: format!("r{next_relation_id}"),
                type_: "precedes".to_string(),
                category: RelationCategory::Structural,
                source: source.id.clone(),
                target: target.id.clone(),
                stage: "logical".to_string(),
                attributes: BTreeMap::new(),
                extensions: BTreeMap::new(),
            });
            next_relation_id += 1;
        }
    }

    // Write-back Φ.relation_ids per token.
    let mut token_relations: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for relation in &relations {
        token_relations
            .entry(relation.source.clone())
            .or_default()
            .push(relation.id.clone());
        token_relations
            .entry(relation.target.clone())
            .or_default()
            .push(relation.id.clone());
    }

    for token in ztokens {
        token.logical.relation_ids = token_relations.remove(&token.id).unwrap_or_default();
    }

    relations
}
