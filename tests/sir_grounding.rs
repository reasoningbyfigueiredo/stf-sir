//! Tests: SirGroundingChecker grounds statements via SirGraph node lookup.
//!
//! Verifies that:
//!   - A statement whose id is a real ZToken id is structurally grounded
//!   - A statement with a fabricated id is ungrounded (no fallback provenance)
//!   - Provenance fallback still grounds statements outside the graph

use stf_sir::compiler;
use stf_sir::compiler::grounding::{GroundingChecker, SirGroundingChecker};
use stf_sir::model::Statement;
use stf_sir::sir::SirGraph;

// ---------------------------------------------------------------------------

#[test]
fn statement_with_real_ztoken_id_is_grounded() {
    let artifact = compiler::compile_markdown("Hello world", None).unwrap();
    let graph = SirGraph::from_artifact(&artifact);

    // Use the first actual ztoken id from the compiled artifact.
    let token_id = artifact.ztokens[0].id.clone();
    let stmt = Statement::atomic(token_id, "Hello world", "test");

    let checker = SirGroundingChecker { graph: &graph };
    assert!(
        checker.check_grounding(&stmt).is_grounded,
        "statement with a compiled ZToken id must be grounded by the SIR graph"
    );
}

#[test]
fn statement_with_fabricated_id_is_ungrounded() {
    let artifact = compiler::compile_markdown("Hello world", None).unwrap();
    let graph = SirGraph::from_artifact(&artifact);

    let stmt = Statement::atomic("z_phantom_does_not_exist", "Some claim", "test");

    let checker = SirGroundingChecker { graph: &graph };
    assert!(
        !checker.check_grounding(&stmt).is_grounded,
        "statement with unknown id must be ungrounded"
    );
}

#[test]
fn all_compiled_tokens_are_grounded_via_graph() {
    let artifact = compiler::compile_markdown("# Title\n\nBody paragraph.", None).unwrap();
    let graph = SirGraph::from_artifact(&artifact);
    let checker = SirGroundingChecker { graph: &graph };

    for token in &artifact.ztokens {
        let stmt = Statement::atomic(token.id.clone(), &token.lexical.normalized_text, "test");
        assert!(
            checker.check_grounding(&stmt).is_grounded,
            "token '{}' must be structurally grounded",
            token.id
        );
    }
}

#[test]
fn provenance_fallback_grounds_axiom_outside_graph() {
    // A statement not in the graph but with provenance should be grounded via fallback.
    let artifact = compiler::compile_markdown("Hello", None).unwrap();
    let graph = SirGraph::from_artifact(&artifact);

    let stmt = Statement::grounded("axiom:001", "provenance axiom", "test", "sha256:src");

    let checker = SirGroundingChecker { graph: &graph };
    assert!(
        checker.check_grounding(&stmt).is_grounded,
        "axiom with provenance must be grounded via fallback even if not in graph"
    );
}

#[test]
fn empty_graph_ungrounds_all_without_provenance() {
    // Empty artifact → empty graph; statements without provenance are ungrounded.
    let artifact = compiler::compile_markdown("", None).unwrap();
    let graph = SirGraph::from_artifact(&artifact);

    let stmt = Statement::atomic("x1", "no anchor", "test");
    let checker = SirGroundingChecker { graph: &graph };
    assert!(!checker.check_grounding(&stmt).is_grounded);
}
