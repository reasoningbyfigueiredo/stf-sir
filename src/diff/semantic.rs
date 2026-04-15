use std::collections::BTreeMap;
use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::model::Artifact;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GlossChange {
    pub token_id: String,
    pub before: String,
    pub after: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConceptChange {
    pub token_id: String,
    pub added: Vec<String>,
    pub removed: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SemanticDiff {
    pub gloss_changes: Vec<GlossChange>,
    pub concept_changes: Vec<ConceptChange>,
}

/// Compute the semantic diff between two artifacts.
///
/// Only tokens that exist in both artifacts (matched by ID) are compared.
/// Results are sorted by token_id for determinism.
pub fn semantic_diff(a: &Artifact, b: &Artifact) -> SemanticDiff {
    let sem_map_a: BTreeMap<&str, &crate::model::SemanticDimension> = a
        .ztokens
        .iter()
        .map(|t| (t.id.as_str(), &t.semantic))
        .collect();

    let mut gloss_changes = Vec::new();
    let mut concept_changes = Vec::new();

    // Only compare tokens present in both artifacts (matched by ID).
    let mut tokens_b: Vec<&crate::model::ZToken> = b
        .ztokens
        .iter()
        .filter(|t| sem_map_a.contains_key(t.id.as_str()))
        .collect();
    tokens_b.sort_by(|x, y| x.id.cmp(&y.id));

    for token_b in tokens_b {
        let sem_a = sem_map_a[token_b.id.as_str()];
        let sem_b = &token_b.semantic;

        if sem_a.gloss != sem_b.gloss {
            gloss_changes.push(GlossChange {
                token_id: token_b.id.clone(),
                before: sem_a.gloss.clone(),
                after: sem_b.gloss.clone(),
            });
        }

        let concepts_a: BTreeSet<&str> = sem_a.concepts.iter().map(|s| s.as_str()).collect();
        let concepts_b: BTreeSet<&str> = sem_b.concepts.iter().map(|s| s.as_str()).collect();

        let added: Vec<String> = concepts_b
            .difference(&concepts_a)
            .map(|s| s.to_string())
            .collect();
        let removed: Vec<String> = concepts_a
            .difference(&concepts_b)
            .map(|s| s.to_string())
            .collect();

        if !added.is_empty() || !removed.is_empty() {
            concept_changes.push(ConceptChange {
                token_id: token_b.id.clone(),
                added,
                removed,
            });
        }
    }

    SemanticDiff { gloss_changes, concept_changes }
}
