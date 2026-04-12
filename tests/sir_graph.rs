mod common;

use anyhow::{Context, Result};
use stf_sir::compiler;
use stf_sir::model::RelationCategory;
use stf_sir::sir::SirNodeKind;

#[test]
fn graph_node_count_matches_document_token_count() -> Result<()> {
    let artifact = canonical_sample()?;
    let graph = artifact.as_sir_graph();

    assert_eq!(graph.nodes.len(), artifact.document.token_count);
    Ok(())
}

#[test]
fn graph_edge_count_matches_document_relation_count() -> Result<()> {
    let artifact = canonical_sample()?;
    let graph = artifact.as_sir_graph();

    assert_eq!(graph.edges.len(), artifact.document.relation_count);
    Ok(())
}

#[test]
fn canonical_graph_contains_z1_and_z2_nodes() -> Result<()> {
    let artifact = canonical_sample()?;
    let graph = artifact.as_sir_graph();

    let z1 = graph.node("z1").context("missing z1 node")?;
    let z2 = graph.node("z2").context("missing z2 node")?;

    assert!(matches!(
        z1.kind,
        SirNodeKind::ZToken {
            ref node_type
        } if node_type == "heading"
    ));
    assert!(matches!(
        z2.kind,
        SirNodeKind::ZToken {
            ref node_type
        } if node_type == "paragraph"
    ));

    Ok(())
}

#[test]
fn precedes_edge_exists_from_z1_to_z2() -> Result<()> {
    let artifact = canonical_sample()?;
    let graph = artifact.as_sir_graph();

    let edge = graph
        .outgoing("z1")
        .into_iter()
        .find(|edge| edge.edge_type == "precedes" && edge.target == "z2")
        .context("missing precedes edge from z1 to z2")?;

    assert_eq!(edge.category, RelationCategory::Structural);
    assert_eq!(edge.stage, "logical");
    Ok(())
}

#[test]
fn outgoing_and_incoming_indexes_are_consistent() -> Result<()> {
    let artifact = canonical_sample()?;
    let graph = artifact.as_sir_graph();

    for edge in &graph.edges {
        assert!(
            graph
                .outgoing(&edge.source)
                .into_iter()
                .any(|candidate| candidate.id == edge.id),
            "edge {} missing from outgoing index for {}",
            edge.id,
            edge.source
        );
        assert!(
            graph
                .incoming(&edge.target)
                .into_iter()
                .any(|candidate| candidate.id == edge.id),
            "edge {} missing from incoming index for {}",
            edge.id,
            edge.target
        );
    }

    Ok(())
}

#[test]
fn structural_edges_query_returns_expected_precedes_edge() -> Result<()> {
    let artifact = canonical_sample()?;
    let graph = artifact.as_sir_graph();

    let edges = graph.edges_by_category(RelationCategory::Structural);
    assert_eq!(edges.len(), 1);
    assert_eq!(edges[0].edge_type, "precedes");
    assert_eq!(edges[0].source, "z1");
    assert_eq!(edges[0].target, "z2");

    Ok(())
}

#[test]
fn graph_construction_is_deterministic() -> Result<()> {
    let artifact = canonical_sample()?;

    let first = artifact.as_sir_graph();
    let second = artifact.as_sir_graph();

    assert_eq!(first, second);
    Ok(())
}

#[test]
fn neighbors_return_adjacent_nodes_without_duplicates() -> Result<()> {
    let artifact = compiler::compile_markdown("# Root\n\n- one\n- two\n", None)?;
    let graph = artifact.as_sir_graph();

    let neighbors = graph.neighbors("z2");
    let neighbor_ids = neighbors
        .iter()
        .map(|node| node.id.as_str())
        .collect::<Vec<_>>();

    assert_eq!(neighbor_ids, vec!["z3", "z4", "z1"]);
    Ok(())
}

#[test]
fn single_node_graph_has_no_edges_and_empty_neighbors() -> Result<()> {
    let artifact = common::compile_fixture("tests/fixtures/valid/paragraph.md")?;
    let graph = artifact.as_sir_graph();

    assert_eq!(graph.nodes.len(), 1);
    assert!(graph.edges.is_empty());
    assert!(graph.outgoing("z1").is_empty());
    assert!(graph.incoming("z1").is_empty());
    assert!(graph.neighbors("z1").is_empty());

    let node = graph.node("z1").context("missing z1")?;
    assert!(matches!(
        node.kind,
        SirNodeKind::ZToken { ref node_type } if node_type == "paragraph"
    ));

    Ok(())
}

#[test]
fn nested_list_graph_has_expected_larger_adjacency_without_duplicate_neighbors() -> Result<()> {
    let artifact = common::compile_fixture("tests/fixtures/valid/nested_list.md")?;
    let graph = artifact.as_sir_graph();

    let outgoing_ids = graph
        .outgoing("z2")
        .into_iter()
        .map(|edge| edge.id.as_str())
        .collect::<Vec<_>>();
    assert_eq!(outgoing_ids, vec!["r1", "r6"]);

    let incoming_ids = graph
        .incoming("z2")
        .into_iter()
        .map(|edge| edge.id.as_str())
        .collect::<Vec<_>>();
    assert_eq!(incoming_ids, vec!["r7"]);

    let neighbor_ids = graph
        .neighbors("z2")
        .into_iter()
        .map(|node| node.id.as_str())
        .collect::<Vec<_>>();
    assert_eq!(neighbor_ids, vec!["z3", "z8", "z1"]);

    Ok(())
}

#[test]
fn larger_fixture_graph_indexes_are_consistent() -> Result<()> {
    let artifact = common::compile_fixture("tests/fixtures/valid/nested_list.md")?;
    let graph = artifact.as_sir_graph();

    common::assert_graph_indexes_consistent(&graph);
    assert_eq!(graph.nodes.len(), artifact.document.token_count);
    assert_eq!(graph.edges.len(), artifact.document.relation_count);

    Ok(())
}

#[test]
fn many_siblings_graph_is_deterministic_and_structural() -> Result<()> {
    let artifact = common::compile_fixture("tests/fixtures/valid/many_siblings.md")?;

    let first = artifact.as_sir_graph();
    let second = artifact.as_sir_graph();

    assert_eq!(first, second);
    assert_eq!(first.nodes.len(), 12);
    assert_eq!(first.edges.len(), 11);

    let structural = first.edges_by_category(RelationCategory::Structural);
    assert_eq!(structural.len(), 11);
    assert!(structural.iter().all(|edge| edge.edge_type == "precedes"));

    let z6_outgoing = first.outgoing("z6");
    assert_eq!(z6_outgoing.len(), 1);
    assert_eq!(z6_outgoing[0].target, "z7");

    let z7_incoming = first.incoming("z7");
    assert_eq!(z7_incoming.len(), 1);
    assert_eq!(z7_incoming[0].source, "z6");

    Ok(())
}

#[test]
fn ztoken_projection_helpers_return_original_dimensions() -> Result<()> {
    let artifact = canonical_sample()?;
    let token = artifact
        .ztokens
        .first()
        .context("canonical sample should contain at least one token")?;

    assert!(std::ptr::eq(token.pi_l(), &token.lexical));
    assert!(std::ptr::eq(token.pi_s(), &token.syntactic));
    assert!(std::ptr::eq(token.pi_sigma(), &token.semantic));
    assert!(std::ptr::eq(token.pi_phi(), &token.logical));

    Ok(())
}

fn canonical_sample() -> Result<stf_sir::model::Artifact> {
    compiler::compile_markdown(
        "# AI is transforming software development\n\nSemantic tokenization preserves meaning across structure.",
        None,
    )
    .map_err(Into::into)
}
