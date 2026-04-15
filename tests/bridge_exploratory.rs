//! Exploratory tests for the `artifact_to_theory` bridge.
//!
//! Probes whether every field of a ZToken survives the conversion to a
//! `Statement` without silent loss, collapse of distinct tokens, or
//! incoherent metadata mapping.

use stf_sir::compiler;
use stf_sir::model::{Formula, artifact_to_theory};

// ---------------------------------------------------------------------------
// 1. Token id fidelity

#[test]
fn statement_id_equals_ztoken_id() {
    let artifact = compiler::compile_markdown("Hello world", None).unwrap();
    let theory = artifact_to_theory(&artifact);
    for token in &artifact.ztokens {
        let stmt = theory.statements.get(&token.id)
            .unwrap_or_else(|| panic!("ztoken {} not found in theory", token.id));
        assert_eq!(stmt.id, token.id);
    }
}

#[test]
fn no_duplicate_statement_ids_in_theory() {
    // Multiple paragraphs → multiple tokens; each must produce a distinct Statement.
    let artifact = compiler::compile_markdown("First.\n\nSecond.\n\nThird.\n", None).unwrap();
    let theory = artifact_to_theory(&artifact);
    // BTreeMap keying already prevents duplication, but verify count matches.
    assert_eq!(theory.statements.len(), artifact.ztokens.len(),
        "theory must have exactly as many statements as ztokens");
}

// ---------------------------------------------------------------------------
// 2. Text fidelity

#[test]
fn statement_text_equals_normalized_text() {
    let artifact = compiler::compile_markdown("Hello  world", None).unwrap();
    let theory = artifact_to_theory(&artifact);
    for token in &artifact.ztokens {
        let stmt = theory.statements.get(&token.id).unwrap();
        assert_eq!(stmt.text, token.lexical.normalized_text,
            "text must equal lexical.normalized_text for {}", token.id);
    }
}

// ---------------------------------------------------------------------------
// 3. Domain / node-type fidelity

#[test]
fn statement_domain_equals_node_type() {
    let artifact = compiler::compile_markdown("# Heading\n\nParagraph.\n", None).unwrap();
    let theory = artifact_to_theory(&artifact);
    for token in &artifact.ztokens {
        let stmt = theory.statements.get(&token.id).unwrap();
        assert_eq!(stmt.domain, token.syntactic.node_type,
            "domain must equal node_type for {}", token.id);
    }
}

// ---------------------------------------------------------------------------
// 4. Provenance: source sha256 anchor

#[test]
fn provenance_contains_source_sha256() {
    let artifact = compiler::compile_markdown("Sample text.\n", None).unwrap();
    let theory = artifact_to_theory(&artifact);
    for stmt in theory.statements.values() {
        assert!(
            stmt.provenance.source_ids.contains(&artifact.source.sha256),
            "stmt {} must have source sha256 in provenance.source_ids", stmt.id
        );
    }
}

// ---------------------------------------------------------------------------
// 5. Provenance: grounding reflects source_text presence

#[test]
fn grounded_flag_true_when_source_text_nonempty() {
    // Any non-empty, valid markdown paragraph must have source_text in the token.
    let artifact = compiler::compile_markdown("Hello world.\n", None).unwrap();
    let theory = artifact_to_theory(&artifact);
    for token in &artifact.ztokens {
        let stmt = theory.statements.get(&token.id).unwrap();
        if !token.lexical.source_text.is_empty() {
            assert!(stmt.provenance.grounded,
                "non-empty source_text must imply grounded for {}", token.id);
        }
    }
}

#[test]
fn ztoken_id_anchor_is_present_in_provenance() {
    let artifact = compiler::compile_markdown("Anchor test.\n", None).unwrap();
    let theory = artifact_to_theory(&artifact);
    for token in &artifact.ztokens {
        let stmt = theory.statements.get(&token.id).unwrap();
        assert!(
            stmt.provenance.anchors.contains(&token.id),
            "token id must be in provenance.anchors for {}", token.id
        );
    }
}

#[test]
fn span_anchor_is_present_in_provenance() {
    let artifact = compiler::compile_markdown("Span test.\n", None).unwrap();
    let theory = artifact_to_theory(&artifact);
    for token in &artifact.ztokens {
        if token.lexical.span.start_byte < token.lexical.span.end_byte {
            let stmt = theory.statements.get(&token.id).unwrap();
            let expected = format!(
                "{}:{}",
                token.lexical.span.start_byte,
                token.lexical.span.end_byte
            );
            assert!(
                stmt.provenance.anchors.contains(&expected),
                "span anchor {expected} must be present for {}", token.id
            );
        }
    }
}

// ---------------------------------------------------------------------------
// 6. Metadata field coverage

#[test]
fn metadata_contains_all_required_fields() {
    let artifact = compiler::compile_markdown("# Title\n\nBody.\n", None).unwrap();
    let theory = artifact_to_theory(&artifact);
    for stmt in theory.statements.values() {
        for field in &["path", "depth", "zid", "node_type", "span_start", "span_end"] {
            assert!(stmt.metadata.contains_key(*field),
                "stmt {} must have metadata field '{field}'", stmt.id);
        }
    }
}

#[test]
fn metadata_zid_equals_statement_id() {
    let artifact = compiler::compile_markdown("Meta check.\n", None).unwrap();
    let theory = artifact_to_theory(&artifact);
    for stmt in theory.statements.values() {
        assert_eq!(stmt.metadata.get("zid").map(String::as_str), Some(stmt.id.as_str()),
            "zid metadata must equal statement id for {}", stmt.id);
    }
}

#[test]
fn metadata_node_type_equals_domain() {
    let artifact = compiler::compile_markdown("Node type check.\n", None).unwrap();
    let theory = artifact_to_theory(&artifact);
    for stmt in theory.statements.values() {
        let meta_nt = stmt.metadata.get("node_type").map(String::as_str).unwrap_or("");
        assert_eq!(meta_nt, stmt.domain.as_str(),
            "metadata node_type must equal stmt.domain for {}", stmt.id);
    }
}

#[test]
fn metadata_span_fields_are_numeric_and_ordered() {
    let artifact = compiler::compile_markdown("Span numeric.\n", None).unwrap();
    let theory = artifact_to_theory(&artifact);
    for stmt in theory.statements.values() {
        let start: usize = stmt.metadata["span_start"].parse()
            .unwrap_or_else(|_| panic!("span_start non-numeric for {}", stmt.id));
        let end: usize = stmt.metadata["span_end"].parse()
            .unwrap_or_else(|_| panic!("span_end non-numeric for {}", stmt.id));
        assert!(start < end, "span_start must be < span_end for {}", stmt.id);
    }
}

#[test]
fn metadata_depth_is_zero_for_root_tokens() {
    let artifact = compiler::compile_markdown("Root only.\n", None).unwrap();
    let theory = artifact_to_theory(&artifact);
    for token in &artifact.ztokens {
        if token.syntactic.parent_id.is_none() {
            let stmt = theory.statements.get(&token.id).unwrap();
            assert_eq!(
                stmt.metadata.get("depth").map(String::as_str),
                Some("0"),
                "root token must have depth=0"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// 7. Formula embedding

#[test]
fn every_nonempty_token_has_embedded_formula() {
    let artifact = compiler::compile_markdown("Formula check.\n", None).unwrap();
    let theory = artifact_to_theory(&artifact);
    for token in &artifact.ztokens {
        if !token.lexical.normalized_text.is_empty() {
            let stmt = theory.statements.get(&token.id).unwrap();
            assert!(
                stmt.formula.is_some(),
                "non-empty token {} must have embedded formula", token.id
            );
        }
    }
}

#[test]
fn embedded_formula_consistent_with_text_parse() {
    // The formula embedded by bridge must equal Formula::parse(stmt.text).
    let artifact = compiler::compile_markdown("A -> B\n", None).unwrap();
    let theory = artifact_to_theory(&artifact);
    for stmt in theory.statements.values() {
        let via_text = Formula::parse(&stmt.text);
        assert_eq!(
            stmt.formula, via_text,
            "embedded formula must equal Formula::parse(text) for {}", stmt.id
        );
    }
}

#[test]
fn implication_in_source_gives_implies_formula() {
    // A paragraph literally reading "A -> B" must produce Implies(…).
    let artifact = compiler::compile_markdown("A -> B\n", None).unwrap();
    let theory = artifact_to_theory(&artifact);
    let stmt = theory.statements.values()
        .find(|s| s.text.contains("->"))
        .expect("must have a statement containing '->'");
    assert!(
        matches!(&stmt.formula, Some(Formula::Implies(_, _))),
        "implication text must produce Formula::Implies, got {:?}", stmt.formula
    );
}

// ---------------------------------------------------------------------------
// 8. Large token list — no collapse

#[test]
fn large_markdown_preserves_all_tokens() {
    let markdown: String = (1..=20)
        .map(|i| format!("Paragraph number {i}.\n"))
        .collect::<Vec<_>>()
        .join("\n");

    let artifact = compiler::compile_markdown(&markdown, None).unwrap();
    let theory = artifact_to_theory(&artifact);

    assert_eq!(
        theory.statements.len(),
        artifact.ztokens.len(),
        "large markdown must produce one Statement per ZToken"
    );

    // Every token id must appear in theory.
    for token in &artifact.ztokens {
        assert!(theory.statements.contains_key(&token.id),
            "token {} must be present in theory", token.id);
    }
}

// ---------------------------------------------------------------------------
// 9. artifact_to_theory_with_formulas: second element matches embedded formula

#[test]
fn with_formulas_returns_embedded_formula_as_second() {
    use stf_sir::model::bridge::artifact_to_theory_with_formulas;
    let artifact = compiler::compile_markdown("NOT P\n", None).unwrap();
    let pairs = artifact_to_theory_with_formulas(&artifact);
    for (stmt, formula_opt) in &pairs {
        assert_eq!(&stmt.formula, formula_opt,
            "second element must equal stmt.formula for {}", stmt.id);
    }
}
