//! Semantic regression harness — frozen behaviour contracts.
//!
//! These tests pin the current intended output of the semantic pipeline so
//! that future refactors cannot silently change observable behaviour.
//!
//! Each assertion documents what the system *must* do, not merely what it
//! currently does.  Adding `#[ignore]` with a rationale is acceptable for
//! cases that are intentionally changing; removing these tests without
//! replacement is NOT acceptable.

use stf_sir::compiler;
use stf_sir::compiler::engine::default_engine;
use stf_sir::compiler::recommended_engine_with_budget;
use stf_sir::compiler::inference::{FormulaInferenceEngine, InferenceEngine};
use stf_sir::error::ErrorKind;
use stf_sir::model::{Formula, Statement, Theory, TruthValue, artifact_to_theory};
use stf_sir::retention::RetentionBaseline;

// ---------------------------------------------------------------------------
// R1: Bridge: formula attachment contracts

#[test]
fn r1_implication_text_produces_implies_formula() {
    let artifact = compiler::compile_markdown("A -> B\n", None).unwrap();
    let theory = artifact_to_theory(&artifact);
    let stmt = theory.statements.values()
        .find(|s| s.text.contains("->"))
        .expect("must have implication statement");
    assert_eq!(
        stmt.formula,
        Some(Formula::implies(Formula::atom("A"), Formula::atom("B"))),
        "R1: 'A -> B' must embed Implies(Atom(A), Atom(B))"
    );
}

#[test]
fn r1_negation_text_produces_not_formula() {
    let artifact = compiler::compile_markdown("NOT X\n", None).unwrap();
    let theory = artifact_to_theory(&artifact);
    let stmt = theory.statements.values()
        .find(|s| s.text.to_uppercase().starts_with("NOT"))
        .expect("must have negation statement");
    assert_eq!(
        stmt.formula,
        Some(Formula::not(Formula::atom("X"))),
        "R1: 'NOT X' must embed Not(Atom(X))"
    );
}

#[test]
fn r1_plain_paragraph_produces_atom_formula() {
    let artifact = compiler::compile_markdown("Hello world\n", None).unwrap();
    let theory = artifact_to_theory(&artifact);
    let stmt = theory.statements.values().next().expect("must have at least one statement");
    // "Hello world" normalizes to some text, which uppercases to its atom form.
    assert!(
        matches!(stmt.formula, Some(Formula::Atom(_))),
        "R1: plain paragraph must produce Atom formula, got {:?}", stmt.formula
    );
}

// ---------------------------------------------------------------------------
// R2: Bridge: grounding contracts

#[test]
fn r2_compiled_token_from_nonempty_source_is_grounded() {
    let artifact = compiler::compile_markdown("Ground me.\n", None).unwrap();
    let theory = artifact_to_theory(&artifact);
    for stmt in theory.statements.values() {
        assert!(stmt.provenance.grounded,
            "R2: compiled token with source_text must be grounded");
    }
}

#[test]
fn r2_source_sha256_in_provenance_source_ids() {
    let artifact = compiler::compile_markdown("SHA check.\n", None).unwrap();
    let theory = artifact_to_theory(&artifact);
    for stmt in theory.statements.values() {
        assert!(stmt.provenance.source_ids.contains(&artifact.source.sha256),
            "R2: source sha256 must be in provenance.source_ids for {}", stmt.id);
    }
}

// ---------------------------------------------------------------------------
// R3: Bridge: metadata presence contracts

#[test]
fn r3_all_metadata_fields_present() {
    let artifact = compiler::compile_markdown("Meta.\n", None).unwrap();
    let theory = artifact_to_theory(&artifact);
    for stmt in theory.statements.values() {
        for key in &["path", "depth", "zid", "node_type", "span_start", "span_end"] {
            assert!(stmt.metadata.contains_key(*key),
                "R3: metadata field '{key}' must be present for {}", stmt.id);
        }
    }
}

// ---------------------------------------------------------------------------
// R4: Inference: derivation count contracts

#[test]
fn r4_single_modus_ponens_derives_exactly_one_conclusion() {
    let mut theory = Theory::new();
    theory.insert(Statement::atomic("a", "A", "logic"));
    theory.insert(Statement::atomic("imp", "A -> B", "logic"));

    let derived = FormulaInferenceEngine.derive(&theory);
    assert_eq!(derived.len(), 1, "R4: single modus ponens must derive exactly one conclusion");
    assert_eq!(derived[0].statement.text, "B", "R4: derived text must be 'B'");
}

#[test]
fn r4_no_derivation_without_matching_premise() {
    let mut theory = Theory::new();
    theory.insert(Statement::atomic("imp", "A -> B", "logic"));

    let derived = FormulaInferenceEngine.derive(&theory);
    assert!(derived.is_empty(), "R4: no premise → no derivation");
}

#[test]
fn r4_derived_statement_carries_formula() {
    let mut theory = Theory::new();
    theory.insert(Statement::atomic("a", "A", "logic"));
    theory.insert(Statement::atomic("imp", "A -> B", "logic"));

    let derived = FormulaInferenceEngine.derive(&theory);
    assert_eq!(derived[0].statement.formula, Some(Formula::atom("B")),
        "R4: derived statement must carry conclusion formula");
}

// ---------------------------------------------------------------------------
// R5: Coherence classification contracts

#[test]
fn r5_contradiction_gives_violated_cl() {
    let mut theory = Theory::new();
    theory.insert(Statement::grounded("s1", "P", "test", "sha:src"));
    let candidate = Statement::grounded("s2", "NOT P", "test", "sha:src");

    let result = default_engine().evaluate_statement(&theory, &candidate);
    assert_eq!(result.coherence.logical, TruthValue::Violated,
        "R5: contradiction must give C_l = Violated");
    assert!(!result.useful_information, "R5: contradiction must have ICE = false");
}

#[test]
fn r5_hallucination_gives_hallucination_error() {
    let theory = Theory::new();
    let candidate = Statement::atomic("h", "ungrounded", "test");

    let result = default_engine().evaluate_statement(&theory, &candidate);
    assert!(result.errors.iter().any(|e| e.kind == ErrorKind::Hallucination),
        "R5: ungrounded coherent statement must emit Hallucination");
}

#[test]
fn r5_non_executable_coherent_grounded_gives_non_executable() {
    let theory = Theory::new();
    let candidate = Statement::grounded("e", "sterile text", "test", "sha:src");

    let result = default_engine().evaluate_statement(&theory, &candidate);
    assert!(result.errors.iter().any(|e| e.kind == ErrorKind::NonExecutable),
        "R5: sterile grounded statement must emit NonExecutable");
}

// ---------------------------------------------------------------------------
// R6: C_c = Unknown for default engine (no step budget)

#[test]
fn r6_default_engine_cc_is_unknown() {
    let theory = Theory::new();
    let candidate = Statement::grounded("c", "test", "test", "sha:src");
    let result = default_engine().evaluate_statement(&theory, &candidate);
    assert_eq!(result.coherence.computational, TruthValue::Unknown,
        "R6: default engine must report C_c = Unknown");
}

// ---------------------------------------------------------------------------
// R7: Retention baseline for empty document is vacuously perfect

#[test]
fn r7_empty_document_has_unit_retention() {
    let artifact = compiler::compile_markdown("", None).unwrap();
    let baseline = RetentionBaseline::from_artifact(&artifact);
    for value in [
        baseline.vector.rho_l,
        baseline.vector.rho_s,
        baseline.vector.rho_sigma,
        baseline.vector.rho_phi,
    ] {
        assert_eq!(value, 1.0, "R7: empty document must have perfect retention (vacuous)");
    }
}

// ---------------------------------------------------------------------------
// R8: Statement count equals ZToken count after bridge conversion

#[test]
fn r8_theory_size_equals_ztoken_count() {
    let src = "# Title\n\nParagraph one.\n\nParagraph two.\n";
    let artifact = compiler::compile_markdown(src, None).unwrap();
    let theory = artifact_to_theory(&artifact);
    assert_eq!(theory.statements.len(), artifact.ztokens.len(),
        "R8: theory statement count must equal ztoken count");
}

// ---------------------------------------------------------------------------
// R9: INV-101-1 regression guard — useful_information requires grounding
//
// Regression for ADR-SEM-001 Rule 3.2.  Guards against future regressions
// where useful_information is re-defined to ignore grounding.  The test is
// non-trivial: modus ponens actually fires (derived_count > 0), so a naive
// "logical_ok && operational_ok" fix would produce useful_information=true.

#[test]
fn r9_useful_information_requires_grounding() {
    let engine = recommended_engine_with_budget(usize::MAX);

    let mut theory = Theory::new();
    theory.insert(
        Statement::grounded("imp", "A -> B", "test", "sha:src")
            .with_formula(Formula::implies(Formula::atom("A"), Formula::atom("B"))),
    );

    // Ungrounded candidate that fires modus ponens.
    let ungrounded = Statement::atomic("u", "A", "test")
        .with_formula(Formula::atom("A"));

    let result = engine.evaluate_statement(&theory, &ungrounded);

    assert!(
        result.derived_count > 0,
        "R9: regression is non-trivial only when modus ponens fires; \
         derived_count must be > 0 (got {})",
        result.derived_count
    );
    assert!(
        !result.useful_information,
        "R9 (INV-101-1): ungrounded statement must have useful_information=false \
         even when modus ponens fires (derived_count={})",
        result.derived_count
    );
}
