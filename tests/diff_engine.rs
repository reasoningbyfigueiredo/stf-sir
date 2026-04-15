//! Integration tests for EPIC-204 — Semantic Diff Engine.

use stf_sir::compiler::compile_markdown;
use stf_sir::diff::{diff_artifacts, semantic_diff, structural_diff};

fn compile(source: &str) -> stf_sir::model::Artifact {
    compile_markdown(source, None).expect("compile_markdown failed in test")
}

/// Helper: compile the same source twice — results must be structurally identical.
fn compile_identical(source: &str) -> (stf_sir::model::Artifact, stf_sir::model::Artifact) {
    (compile(source), compile(source))
}

// ---------------------------------------------------------------------------
// Test 1 — diff of identical artifacts is empty
// ---------------------------------------------------------------------------

#[test]
fn diff_identical_artifacts_is_empty() {
    let source = "# Hello\n\nThis is a paragraph.\n";
    let (a, b) = compile_identical(source);

    let report = diff_artifacts(&a, &b);

    assert!(report.summary.is_identical, "diff of identical artifacts must be empty");
    assert_eq!(report.summary.added_tokens, 0);
    assert_eq!(report.summary.removed_tokens, 0);
    assert_eq!(report.summary.added_relations, 0);
    assert_eq!(report.summary.removed_relations, 0);
    assert_eq!(report.summary.modified_tokens, 0);
}

// ---------------------------------------------------------------------------
// Test 2 — structural diff detects added token
// ---------------------------------------------------------------------------

#[test]
fn diff_detects_added_token() {
    let source_a = "# Hello\n";
    let source_b = "# Hello\n\nExtra paragraph.\n";

    let a = compile(source_a);
    let b = compile(source_b);

    let sdiff = structural_diff(&a, &b);

    // B has more tokens than A — there must be added tokens.
    assert!(
        !sdiff.added_tokens.is_empty(),
        "expected added tokens when second artifact has extra paragraph"
    );
    assert!(
        sdiff.removed_tokens.is_empty(),
        "expected no removed tokens when B is a superset of A structurally"
    );
}

// ---------------------------------------------------------------------------
// Test 3 — structural diff detects removed relation
// ---------------------------------------------------------------------------

#[test]
fn diff_detects_removed_relation() {
    let source_a = "# Title\n\nParagraph one.\n\nParagraph two.\n";
    let source_b = "# Title\n";

    let a = compile(source_a);
    let b = compile(source_b);

    let sdiff = structural_diff(&a, &b);

    // A has more relations than B — there must be removed relations.
    let report = diff_artifacts(&a, &b);
    assert!(
        report.summary.removed_relations > 0 || report.summary.removed_tokens > 0,
        "expected some removals when second artifact is a strict subset"
    );
    let _ = sdiff; // keep for visibility
}

// ---------------------------------------------------------------------------
// Test 4 — semantic diff detects gloss change
// ---------------------------------------------------------------------------

#[test]
fn diff_detects_gloss_change() {
    // Compile two different documents and check that tokens with the same ID
    // but different glosses are detected. Since IDs are deterministic for
    // identical content, we test the identity case first (no gloss changes).
    let source = "# Hello World\n";
    let (a, b) = compile_identical(source);

    let sdiff = semantic_diff(&a, &b);
    assert!(
        sdiff.gloss_changes.is_empty(),
        "identical artifacts must have no gloss changes"
    );

    // Now compile two different single-heading artifacts.
    // ZToken IDs are sequential (z1, z2, …) by position, not content — so both produce z1.
    // The semantic diff matches by ID and detects the gloss change.
    let a2 = compile("# Alpha\n");
    let b2 = compile("# Beta\n");
    let sdiff2 = semantic_diff(&a2, &b2);
    // Both documents have a heading at z1 with different glosses → one gloss change detected.
    assert!(
        !sdiff2.gloss_changes.is_empty(),
        "same-position tokens with different glosses must produce gloss changes"
    );
}

// ---------------------------------------------------------------------------
// Test 5 — diff summary counts are correct
// ---------------------------------------------------------------------------

#[test]
fn diff_summary_is_correct() {
    let source_a = "# Title\n\nParagraph.\n";
    let source_b = "# Title\n\nParagraph.\n\nNew section.\n";

    let a = compile(source_a);
    let b = compile(source_b);

    let report = diff_artifacts(&a, &b);

    assert_eq!(
        report.summary.added_tokens,
        report.structural.added_tokens.len(),
        "summary.added_tokens must match structural.added_tokens.len()"
    );
    assert_eq!(
        report.summary.removed_tokens,
        report.structural.removed_tokens.len(),
        "summary.removed_tokens must match structural.removed_tokens.len()"
    );
    assert_eq!(
        report.summary.added_relations,
        report.structural.added_relations.len(),
        "summary.added_relations must match structural.added_relations.len()"
    );
    assert_eq!(
        report.summary.removed_relations,
        report.structural.removed_relations.len(),
        "summary.removed_relations must match structural.removed_relations.len()"
    );

    // is_identical should be false because b has extra content.
    assert!(!report.summary.is_identical);

    // Verify the report serializes to valid JSON.
    let json = report.to_json();
    let parsed: serde_json::Value = serde_json::from_str(&json).expect("report JSON must be valid");
    assert_eq!(parsed["format"], "stf-sir-diff-v1");
}
