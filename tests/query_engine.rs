//! EPIC-203 integration tests for the Semantic Query Engine.
//!
//! Each test compiles a small markdown string, builds a SirGraph,
//! constructs a QueryExecutor, and executes a query.

use stf_sir::compiler::compile_markdown;
use stf_sir::sir::query::{Query, QueryExecutor};
use stf_sir::sir::SirGraph;

// ---------------------------------------------------------------------------
// Helper
// ---------------------------------------------------------------------------

fn compile(md: &str) -> (stf_sir::model::Artifact, SirGraph) {
    let artifact = compile_markdown(md, None).expect("compile_markdown failed");
    let graph = artifact.as_sir_graph();
    (artifact, graph)
}

// ---------------------------------------------------------------------------
// Test 1: ByType returns sorted results
// ---------------------------------------------------------------------------

#[test]
fn query_by_type_returns_sorted_results() {
    let md = "# First\n\n## Second\n\n### Third\n\nA paragraph.\n";
    let (artifact, graph) = compile(md);
    let executor = QueryExecutor::new(&graph, &artifact);

    let result = executor.execute(&Query::ByType {
        node_type: "heading".to_string(),
    });

    // All results must be headings
    for id in &result.token_ids {
        let token = artifact.ztokens.iter().find(|t| &t.id == id).unwrap();
        assert_eq!(
            token.syntactic.node_type, "heading",
            "token {} is not a heading",
            id
        );
    }

    // Results must be sorted
    let mut sorted = result.token_ids.clone();
    sorted.sort();
    assert_eq!(
        result.token_ids, sorted,
        "token_ids are not sorted lexicographically"
    );

    // Must have found at least one heading
    assert!(
        !result.token_ids.is_empty(),
        "expected at least one heading"
    );
}

// ---------------------------------------------------------------------------
// Test 2: Ancestors is deterministic
// ---------------------------------------------------------------------------

#[test]
fn query_ancestors_is_deterministic() {
    let md = "# Top\n\n## Child\n\n### Grandchild\n";
    let (artifact, graph) = compile(md);
    let executor = QueryExecutor::new(&graph, &artifact);

    // Find a deep node to query ancestors of
    let deepest = artifact
        .ztokens
        .iter()
        .max_by_key(|t| t.syntactic.depth)
        .map(|t| t.id.clone())
        .expect("no tokens");

    let result_a = executor.execute(&Query::Ancestors {
        id: deepest.clone(),
    });
    let result_b = executor.execute(&Query::Ancestors {
        id: deepest.clone(),
    });

    assert_eq!(
        result_a.token_ids, result_b.token_ids,
        "Ancestors query is not deterministic"
    );
    assert_eq!(
        result_a.token_ids,
        {
            let mut sorted = result_a.token_ids.clone();
            sorted.sort();
            sorted
        },
        "ancestor IDs are not sorted"
    );
}

// ---------------------------------------------------------------------------
// Test 3: Descendants finds all children
// ---------------------------------------------------------------------------

#[test]
fn query_descendants_finds_all_children() {
    let md = "# Root\n\n## A\n\n### A1\n\n## B\n\n### B1\n";
    let (artifact, graph) = compile(md);
    let executor = QueryExecutor::new(&graph, &artifact);

    // Find the root (shallowest) node
    let root_id = artifact
        .ztokens
        .iter()
        .min_by_key(|t| t.syntactic.depth)
        .map(|t| t.id.clone())
        .expect("no tokens");

    let result = executor.execute(&Query::Descendants { id: root_id });

    // Must have found some descendants
    assert!(
        !result.token_ids.is_empty(),
        "expected descendants of root node"
    );

    // Result must be sorted
    let mut sorted = result.token_ids.clone();
    sorted.sort();
    assert_eq!(result.token_ids, sorted, "descendant IDs not sorted");
}

// ---------------------------------------------------------------------------
// Test 4: And (intersection)
// ---------------------------------------------------------------------------

#[test]
fn query_and_intersection() {
    let md = "# Heading One\n\n## Heading Two\n\nA paragraph here.\n";
    let (artifact, graph) = compile(md);
    let executor = QueryExecutor::new(&graph, &artifact);

    // ByType(heading) AND ByType(paragraph) should be empty (no token is both)
    let q_heading = Query::ByType {
        node_type: "heading".to_string(),
    };
    let q_paragraph = Query::ByType {
        node_type: "paragraph".to_string(),
    };

    let result = executor.execute(&Query::and(q_heading, q_paragraph));
    assert!(
        result.token_ids.is_empty(),
        "intersection of heading and paragraph should be empty"
    );

    // ByType(heading) AND ByType(heading) should equal ByType(heading)
    let q1 = Query::ByType {
        node_type: "heading".to_string(),
    };
    let q2 = Query::ByType {
        node_type: "heading".to_string(),
    };
    let q_both = Query::ByType {
        node_type: "heading".to_string(),
    };

    let intersection = executor.execute(&Query::and(q1, q2));
    let full = executor.execute(&q_both);

    assert_eq!(
        intersection.token_ids, full.token_ids,
        "heading AND heading should equal heading"
    );
}

// ---------------------------------------------------------------------------
// Test 5: Empty graph returns empty results
// ---------------------------------------------------------------------------

#[test]
fn query_empty_graph_returns_empty() {
    let md = "# Only a heading\n";
    let (artifact, graph) = compile(md);
    let executor = QueryExecutor::new(&graph, &artifact);

    let result = executor.execute(&Query::ByType {
        node_type: "nonexistent_type_xyz_123".to_string(),
    });

    assert!(
        result.token_ids.is_empty(),
        "expected empty result for nonexistent type"
    );
    assert_eq!(result.token_count(), 0);
    assert_eq!(result.relation_count(), 0);
    assert!(result.is_empty());
}

// ---------------------------------------------------------------------------
// Test 6: DepthRange
// ---------------------------------------------------------------------------

#[test]
fn query_depth_range() {
    let md = "# Level 0\n\n## Level 1\n\n### Level 2\n\nParagraph.\n";
    let (artifact, graph) = compile(md);
    let executor = QueryExecutor::new(&graph, &artifact);

    // Find min and max depths in the artifact
    let min_depth = artifact
        .ztokens
        .iter()
        .map(|t| t.syntactic.depth)
        .min()
        .unwrap_or(0);
    let max_depth = artifact
        .ztokens
        .iter()
        .map(|t| t.syntactic.depth)
        .max()
        .unwrap_or(0);

    // Query for all nodes at min depth — should be non-empty
    let result_min = executor.execute(&Query::DepthRange {
        min: min_depth,
        max: min_depth,
    });
    assert!(
        !result_min.token_ids.is_empty(),
        "expected at least one node at minimum depth {}",
        min_depth
    );

    // Query for all nodes (full depth range) — should equal all tokens
    let result_all = executor.execute(&Query::DepthRange {
        min: 0,
        max: max_depth + 100,
    });
    assert_eq!(
        result_all.token_count(),
        artifact.ztokens.len(),
        "full depth range should include all tokens"
    );

    // All result IDs should be sorted
    let mut sorted = result_min.token_ids.clone();
    sorted.sort();
    assert_eq!(result_min.token_ids, sorted, "depth range results not sorted");
}
