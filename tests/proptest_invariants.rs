//! Property tests for STF-SIR compiler invariants.
//!
//! These tests generate small Markdown documents from a restricted grammar
//! (headings, paragraphs, and bullet lists composed of ASCII lowercase
//! words) and assert the four load-bearing invariants from spec §9 that
//! span-and-UTF-8 bugs are most likely to break:
//!
//! * **rule 5** — every ztoken id is unique,
//! * **rule 6** — every relation id is unique,
//! * **rule 7/8** — `document.token_count` and `relation_count` match the
//!   length of the serialized vectors,
//! * **rule 9--13** — parent ids, relation endpoints, and `Φ.relation_ids`
//!   resolve to existing objects,
//! * **rule 14** — every byte span satisfies
//!   `0 ≤ start_byte < end_byte ≤ source.length_bytes`,
//! * **rule 16** — `L.source_text` is byte-equal to the source slice it
//!   identifies,
//! * **graph/retention invariants** — SirGraph indexes are consistent and
//!   retention scores stay within `[0, 1]`.
//!
//! The generator is intentionally modest: it is meant to exercise the hot
//! paths (heading/paragraph/list/blockquote/code-block combinations with
//! varying whitespace) and
//! hand the deeper edge cases to the curated fixtures in
//! `tests/conformance/valid/`. Fuzzing proper is out of scope for v1.

use std::collections::HashSet;

use proptest::prelude::*;
use stf_sir::compiler;
use stf_sir::compiler::validator;
use stf_sir::compiler::recommended_engine_with_budget;
use stf_sir::model::{Provenance, RelationCategory, Statement, StatementKind, Theory};
use std::collections::BTreeMap;

/// Produce a single ASCII word of 1..6 lowercase characters.
fn word() -> impl Strategy<Value = String> {
    "[a-z]{1,6}".prop_map(String::from)
}

/// Produce a block: heading, paragraph, or short bullet list.
fn block() -> impl Strategy<Value = String> {
    let heading = (1usize..=3, prop::collection::vec(word(), 1..4))
        .prop_map(|(level, words)| format!("{} {}", "#".repeat(level), words.join(" ")));
    let paragraph = prop::collection::vec(word(), 1..8).prop_map(|words| words.join(" "));
    let bullet_list = prop::collection::vec(
        prop::collection::vec(word(), 1..4).prop_map(|ws| format!("- {}", ws.join(" "))),
        1..4,
    )
    .prop_map(|items| items.join("\n"));
    let blockquote =
        prop::collection::vec(word(), 1..6).prop_map(|words| format!("> {}", words.join(" ")));
    let code_block = prop::collection::vec(word(), 1..5)
        .prop_map(|words| format!("```\n{}\n```", words.join(" ")));

    prop_oneof![heading, paragraph, bullet_list, blockquote, code_block]
}

/// Assemble 1..5 blocks separated by blank lines and terminated by `\n`.
fn document() -> impl Strategy<Value = String> {
    prop::collection::vec(block(), 1..5).prop_map(|blocks| {
        let mut out = blocks.join("\n\n");
        out.push('\n');
        out
    })
}

// ---------------------------------------------------------------------------
// ADR-SEM-001 Rule 3.2 — INV-101-1 property test
//
// For any Statement and any Theory, the RecommendedEngine must satisfy:
//   IF result.useful_information == true THEN result.grounded == true
// ---------------------------------------------------------------------------

/// Generate a Statement with varying grounding (provenance.grounded ∈ {true, false}).
fn arb_statement() -> impl Strategy<Value = Statement> {
    // grounded flag, optional source_id, optional anchor, text
    (
        proptest::bool::ANY,
        proptest::option::of("[a-z]{3,8}"),
        proptest::option::of("[a-z]{3,8}"),
        "[a-z]{1,6}",
    )
        .prop_map(|(grounded_flag, source, anchor, text)| {
            let mut p = Provenance::default();
            p.grounded = grounded_flag;
            if let Some(s) = source {
                p.source_ids.insert(format!("sha256:{s}"));
            }
            if let Some(a) = anchor {
                p.anchors.insert(a);
            }
            Statement {
                id: "prop-stmt".into(),
                text,
                kind: StatementKind::Atomic,
                domain: "test".into(),
                provenance: p,
                metadata: BTreeMap::new(),
                formula: None,
                semantic_dimensions: None,
            }
        })
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 256,
        .. ProptestConfig::default()
    })]

    /// INV-101-1: useful_information == true → grounded == true (ADR-SEM-001 Rule 3.2).
    ///
    /// This property must hold for all possible Statement provenances.
    /// A counterexample would mean the engine reports "useful information"
    /// for an ungrounded (potentially hallucinated) statement.
    #[test]
    fn useful_information_grounding_invariant(stmt in arb_statement()) {
        let engine = recommended_engine_with_budget(usize::MAX);
        let theory = Theory::new();
        let result = engine.evaluate_statement(&theory, &stmt);
        if result.useful_information {
            prop_assert!(
                result.grounded,
                "INV-101-1 violated: useful_information=true but grounded=false \
                 for stmt with provenance: grounded={}, source_ids={}, anchors={}",
                stmt.provenance.grounded,
                stmt.provenance.source_ids.len(),
                stmt.provenance.anchors.len(),
            );
        }
    }
}

proptest! {
    #![proptest_config(ProptestConfig {
        // Keep the suite fast in CI; the curated fixtures handle breadth.
        cases: 128,
        .. ProptestConfig::default()
    })]

    #[test]
    fn compiled_artifacts_satisfy_core_invariants(src in document()) {
        let artifact = compiler::compile_markdown(&src, None)
            .expect("curated markdown must compile");

        // Rule 5 — unique ztoken ids.
        let mut ids: HashSet<&str> = HashSet::new();
        for token in &artifact.ztokens {
            prop_assert!(
                ids.insert(token.id.as_str()),
                "duplicate ztoken id {:?}",
                token.id
            );
        }

        // Rule 6 — unique relation ids.
        let mut relation_ids: HashSet<&str> = HashSet::new();
        for relation in &artifact.relations {
            prop_assert!(
                relation_ids.insert(relation.id.as_str()),
                "duplicate relation id {:?}",
                relation.id
            );
        }

        // Rules 7 and 8 — counts in sync with the serialized vectors.
        prop_assert_eq!(artifact.document.token_count, artifact.ztokens.len());
        prop_assert_eq!(artifact.document.relation_count, artifact.relations.len());

        let token_ids = artifact
            .ztokens
            .iter()
            .map(|token| token.id.as_str())
            .collect::<HashSet<_>>();

        // Rule 9 — parent references resolve.
        for token in &artifact.ztokens {
            if let Some(parent_id) = token.syntactic.parent_id.as_deref() {
                prop_assert!(token_ids.contains(parent_id));
            }
        }

        // Rules 10--13 — relation endpoints and Φ.relation_ids resolve.
        for relation in &artifact.relations {
            prop_assert!(token_ids.contains(relation.source.as_str()));
            prop_assert!(
                token_ids.contains(relation.target.as_str())
                    || matches!(relation.category, RelationCategory::SemanticLink)
            );
        }
        for token in &artifact.ztokens {
            for relation_id in &token.logical.relation_ids {
                prop_assert!(relation_ids.contains(relation_id.as_str()));
            }
        }

        // Rule 14 — byte span bounds.
        for token in &artifact.ztokens {
            let span = &token.lexical.span;
            prop_assert!(span.start_byte < span.end_byte);
            prop_assert!(span.end_byte <= artifact.source.length_bytes);
        }

        // Rule 16 — L.source_text is byte-equal to the source slice.
        let bytes = src.as_bytes();
        for token in &artifact.ztokens {
            let span = &token.lexical.span;
            prop_assert!(span.end_byte <= bytes.len());
            let slice = &bytes[span.start_byte..span.end_byte];
            prop_assert_eq!(
                slice,
                token.lexical.source_text.as_bytes(),
                "L.source_text mismatch for ztoken {}",
                token.id
            );
        }

        // Full pipeline must accept the artifact as well.
        let yaml = compiler::serializer::to_yaml_string(&artifact)
            .expect("serialization is infallible for owned model");
        let errors = validator::validate_yaml_str(&yaml, Some(bytes));
        prop_assert!(
            errors.is_empty(),
            "validator rejected a generated artifact: {:?}",
            errors
        );

        // SirGraph projection invariants.
        let graph = artifact.as_sir_graph();
        prop_assert_eq!(graph.nodes.len(), artifact.document.token_count);
        prop_assert_eq!(graph.edges.len(), artifact.document.relation_count);
        prop_assert_eq!(graph.node_by_id.len(), graph.nodes.len());
        for (id, index) in &graph.node_by_id {
            prop_assert_eq!(graph.nodes[*index].id.as_str(), id.as_str());
        }
        for (node_id, edge_indexes) in &graph.outgoing {
            for edge_index in edge_indexes {
                prop_assert_eq!(graph.edges[*edge_index].source.as_str(), node_id.as_str());
            }
        }
        for (node_id, edge_indexes) in &graph.incoming {
            for edge_index in edge_indexes {
                prop_assert_eq!(graph.edges[*edge_index].target.as_str(), node_id.as_str());
            }
        }

        // Retention values must remain within [0, 1].
        let baseline = artifact.retention_baseline();
        for value in [
            baseline.vector.rho_l,
            baseline.vector.rho_s,
            baseline.vector.rho_sigma,
            baseline.vector.rho_phi,
        ] {
            prop_assert!((0.0..=1.0).contains(&value));
        }
    }
}
