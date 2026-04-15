//! Tests for referential grounding (Ground predicate) — Definition E2.
//!
//! Also covers ADR-SEM-001 Rule 3.2 (INV-101-1, INV-101-2):
//!   useful_information = C_l ∧ C_o ∧ Ground

use stf_sir::model::{InsertionOutcome, Statement, Provenance, StatementKind, Theory, TrustLevel};
use stf_sir::model::artifact_to_theory;
use stf_sir::compiler;
use stf_sir::compiler::grounding::{GroundingChecker, ProvenanceGroundingChecker};
use stf_sir::compiler::recommended_engine_with_budget;
use std::collections::BTreeMap;

fn checker() -> ProvenanceGroundingChecker { ProvenanceGroundingChecker }

fn stmt_with_provenance(grounded: bool, source: Option<&str>, anchor: Option<&str>) -> Statement {
    let mut p = Provenance::default();
    p.grounded = grounded;
    if let Some(s) = source { p.source_ids.insert(s.to_string()); }
    if let Some(a) = anchor { p.anchors.insert(a.to_string()); }
    Statement {
        id: "test".into(),
        text: "A".into(),
        kind: StatementKind::Atomic,
        domain: "test".into(),
        provenance: p,
        semantic_dimensions: None,
        metadata: BTreeMap::new(),
        formula: None,
    }
}

// ---------------------------------------------------------------------------

#[test]
fn statement_with_source_id_is_grounded() {
    let stmt = stmt_with_provenance(false, Some("src:abc123"), None);
    let result = checker().check_grounding(&stmt);
    assert!(result.is_grounded);
    assert!(result.matched_sources.contains(&"src:abc123".to_string()));
}

#[test]
fn statement_with_anchor_is_grounded() {
    let stmt = stmt_with_provenance(false, None, Some("z1"));
    let result = checker().check_grounding(&stmt);
    assert!(result.is_grounded);
}

#[test]
fn statement_with_grounded_flag_is_grounded() {
    let stmt = stmt_with_provenance(true, None, None);
    let result = checker().check_grounding(&stmt);
    assert!(result.is_grounded);
}

#[test]
fn statement_with_no_provenance_is_ungrounded() {
    let stmt = Statement::atomic("u1", "ungrounded claim", "test");
    let result = checker().check_grounding(&stmt);
    assert!(!result.is_grounded);
    assert!(!result.missing_anchors.is_empty());
}

#[test]
fn grounded_constructor_produces_grounded_statement() {
    let stmt = Statement::grounded("g1", "claim", "domain", "sha256:abc");
    let result = checker().check_grounding(&stmt);
    assert!(result.is_grounded);
}

// ---------------------------------------------------------------------------
// ADR-SEM-001 Rule 3.2 — useful_information invariants (INV-101-1, INV-101-2)
// ---------------------------------------------------------------------------

// UT-101-1: coherent + NOT grounded → useful_information = false
#[test]
fn coherent_ungrounded_not_useful() {
    let theory = Theory::new();
    let engine = recommended_engine_with_budget(usize::MAX);
    // Ungrounded: no source_ids, no anchors, grounded=false
    let stmt = Statement::atomic("u1", "A", "test");
    let result = engine.evaluate_statement(&theory, &stmt);
    assert!(
        !result.useful_information,
        "ungrounded statement must not be useful_information (INV-101-1)"
    );
    assert!(!result.grounded);
}

// UT-101-2: grounded + operationally sterile → useful_information = false
#[test]
fn grounded_non_executable_not_useful() {
    let theory = Theory::new();
    let engine = recommended_engine_with_budget(usize::MAX);
    // Grounded (source_id present) but no implication in theory → no modus ponens fires
    let stmt = Statement::grounded("g1", "A", "test", "sha256:abc");
    let result = engine.evaluate_statement(&theory, &stmt);
    assert!(result.grounded, "statement with source_id must be grounded");
    // C_o = false: no derivation possible (empty theory, no implications)
    assert!(
        !result.useful_information,
        "grounded but sterile statement must not be useful_information"
    );
}

// UT-101-3: grounded + coherent + executable → useful_information = true
#[test]
fn grounded_coherent_executable_is_useful() {
    use stf_sir::model::Formula;

    let engine = recommended_engine_with_budget(usize::MAX);

    // Theory already contains an implication "A -> B" (grounded)
    let mut theory = Theory::new();
    let impl_stmt = Statement::grounded("imp1", "A -> B", "test", "sha256:src")
        .with_formula(Formula::implies(Formula::atom("A"), Formula::atom("B")));
    theory.insert(impl_stmt);

    // Candidate: grounded "A" — triggers modus ponens → derives "B" → C_o = true
    let candidate = Statement::grounded("a1", "A", "test", "sha256:src")
        .with_formula(Formula::atom("A"));

    let result = engine.evaluate_statement(&theory, &candidate);

    assert!(result.grounded, "candidate must be grounded");
    assert!(
        result.coherence.logical.is_satisfied(),
        "no contradiction expected"
    );
    assert_eq!(result.derived_count, 1, "modus ponens should derive B");
    assert!(
        result.useful_information,
        "grounded + coherent + executable must be useful_information (INV-101-1)"
    );
}

// UT-101-4: contradictory → useful_information = false regardless of grounding
#[test]
fn contradictory_not_useful() {
    use stf_sir::model::Formula;

    let engine = recommended_engine_with_budget(usize::MAX);

    let mut theory = Theory::new();
    theory.insert(
        Statement::grounded("s1", "A", "test", "sha256:src")
            .with_formula(Formula::atom("A")),
    );

    // Candidate contradicts existing "A"
    let candidate = Statement::grounded("s2", "NOT A", "test", "sha256:src")
        .with_formula(Formula::not(Formula::atom("A")));

    let result = engine.evaluate_statement(&theory, &candidate);

    assert!(
        result.coherence.logical.is_violated(),
        "contradiction must set C_l = Violated"
    );
    assert!(
        !result.useful_information,
        "contradictory statement must not be useful_information"
    );
}

// UT-101-5: audit_theory with mixed grounding → useful_information = false
#[test]
fn audit_theory_with_mixed_grounding() {
    use stf_sir::model::Formula;

    let engine = recommended_engine_with_budget(usize::MAX);

    let mut theory = Theory::new();
    // Grounded statement with an implication
    theory.insert(
        Statement::grounded("imp1", "A -> B", "test", "sha256:src")
            .with_formula(Formula::implies(Formula::atom("A"), Formula::atom("B"))),
    );
    theory.insert(
        Statement::grounded("a1", "A", "test", "sha256:src")
            .with_formula(Formula::atom("A")),
    );
    // Ungrounded statement (no source, no anchor, grounded=false)
    theory.insert(Statement::atomic("u1", "C", "test"));

    let result = engine.audit_theory(&theory);

    assert!(
        !result.grounded,
        "theory with ungrounded statement must report grounded=false"
    );
    assert!(
        !result.useful_information,
        "theory with ungrounded statement must report useful_information=false (INV-101-2)"
    );
    // Errors must include at least one Hallucination
    assert!(
        result.errors.iter().any(|e| matches!(e.kind, stf_sir::ErrorKind::Hallucination)),
        "ungrounded statement must produce Hallucination error"
    );
}

// ---------------------------------------------------------------------------
// IT-101-1: compile real artifact → bridge → audit_theory
// Verifies that all bridged statements are grounded and the theory as a whole
// reports useful_information=true when there is a derivable consequence.
// ---------------------------------------------------------------------------

#[test]
fn bridge_derived_theory_useful_information() {
    // "A -> B" and "A" are separate paragraphs.
    // Formula::parse extracts Implies(A, B) and Atom(A) respectively.
    // artifact_to_theory populates provenance from the source sha256 and span anchors.
    let artifact = compiler::compile_markdown("A -> B\n\nA\n", None)
        .expect("well-formed markdown must compile");

    let theory = artifact_to_theory(&artifact);

    // All bridged statements must be grounded (non-empty source_text sets grounded=true
    // and the sha256 is placed in source_ids).
    let engine = recommended_engine_with_budget(usize::MAX);
    for stmt in theory.statements.values() {
        let gr = engine.grounding.check_grounding(stmt);
        assert!(
            gr.is_grounded,
            "IT-101-1: bridged statement '{}' must be grounded (source_text is non-empty)",
            stmt.id
        );
    }

    // Auditing the full theory must produce at least one derivation
    // ("A" + "A -> B" → FormulaInferenceEngine derives "B") and, because
    // all statements are grounded, useful_information must be true.
    let result = engine.audit_theory(&theory);
    assert!(
        result.grounded,
        "IT-101-1: theory from compiled artifact must be fully grounded"
    );
    assert!(
        result.derived_count > 0,
        "IT-101-1: FormulaInferenceEngine must derive at least one consequence \
         from 'A -> B' + 'A'; got derived_count={}",
        result.derived_count
    );
    assert!(
        result.useful_information,
        "IT-101-1: fully grounded + executable theory must report useful_information=true"
    );
}

// UT-101-6: fully grounded theory with derivation → useful_information = true
#[test]
fn audit_theory_fully_grounded_is_useful() {
    use stf_sir::model::Formula;

    let engine = recommended_engine_with_budget(usize::MAX);

    let mut theory = Theory::new();
    theory.insert(
        Statement::grounded("imp1", "A -> B", "test", "sha256:src")
            .with_formula(Formula::implies(Formula::atom("A"), Formula::atom("B"))),
    );
    theory.insert(
        Statement::grounded("a1", "A", "test", "sha256:src")
            .with_formula(Formula::atom("A")),
    );

    let result = engine.audit_theory(&theory);

    assert!(result.grounded, "fully grounded theory must report grounded=true");
    assert!(
        result.useful_information,
        "fully grounded + executable theory must be useful_information"
    );
}

// ---------------------------------------------------------------------------
// EPIC-104 — Theory::insert_guarded tests
// ---------------------------------------------------------------------------

// UT-104-1: statement with source_ids → TrustLevel::Trusted
#[test]
fn insert_guarded_with_source_ids_is_trusted() {
    let mut theory = Theory::new();
    let stmt = Statement::grounded("s1", "A", "test", "sha:src");
    let outcome = theory.insert_guarded(stmt);
    assert_eq!(outcome.trust_level, TrustLevel::Trusted);
    assert!(outcome.inserted);
    assert!(outcome.diagnostic.is_none());
}

// UT-104-2: statement with anchors → TrustLevel::Trusted
#[test]
fn insert_guarded_with_anchors_is_trusted() {
    let mut theory = Theory::new();
    let mut p = Provenance::default();
    p.anchors.insert("z1".to_string());
    let stmt = Statement {
        id: "s1".into(), text: "A".into(),
        kind: StatementKind::Atomic, domain: "test".into(),
        provenance: p, metadata: Default::default(), formula: None,
        semantic_dimensions: None,
    };
    let outcome = theory.insert_guarded(stmt);
    assert_eq!(outcome.trust_level, TrustLevel::Trusted);
    assert!(outcome.diagnostic.is_none());
}

// UT-104-3: statement with grounded=true flag → TrustLevel::Trusted
#[test]
fn insert_guarded_with_grounded_flag_is_trusted() {
    let mut theory = Theory::new();
    let mut p = Provenance::default();
    p.grounded = true;
    let stmt = Statement {
        id: "s1".into(), text: "A".into(),
        kind: StatementKind::Atomic, domain: "test".into(),
        provenance: p, metadata: Default::default(), formula: None,
        semantic_dimensions: None,
    };
    let outcome = theory.insert_guarded(stmt);
    assert_eq!(outcome.trust_level, TrustLevel::Trusted);
    assert!(outcome.diagnostic.is_none());
}

// UT-104-4: statement without any provenance → TrustLevel::Untrusted + diagnostic
#[test]
fn insert_guarded_without_provenance_is_untrusted() {
    let mut theory = Theory::new();
    let stmt = Statement::atomic("u1", "hallucination candidate", "test");
    let outcome = theory.insert_guarded(stmt);
    assert_eq!(outcome.trust_level, TrustLevel::Untrusted);
    assert!(outcome.inserted, "INV-104-3: must still insert even when Untrusted");
    assert!(outcome.diagnostic.is_some(), "Untrusted must carry a diagnostic message");
}

// UT-104-5: insert_guarded always inserts (INV-104-3)
#[test]
fn insert_guarded_always_inserts() {
    let mut theory = Theory::new();

    // Trusted insertion
    let trusted = Statement::grounded("t1", "trusted", "test", "sha:src");
    let out_t = theory.insert_guarded(trusted);
    assert!(out_t.inserted);
    assert!(theory.contains("t1"), "trusted statement must be present after insert_guarded");

    // Untrusted insertion — MUST still be inserted
    let untrusted = Statement::atomic("u1", "untrusted", "test");
    let out_u = theory.insert_guarded(untrusted);
    assert!(out_u.inserted, "INV-104-3: inserted must be true even for Untrusted");
    assert!(theory.contains("u1"), "INV-104-3: untrusted statement must be present in theory");
}

// UT-104-6: Theory::insert is unchanged (INV-104-1)
#[test]
fn theory_insert_unchanged_after_epic() {
    let mut theory = Theory::new();
    let stmt = Statement::atomic("bare", "no provenance", "test");
    theory.insert(stmt); // must not panic; no return value
    assert!(theory.contains("bare"), "INV-104-1: Theory::insert must still work");
}

// ADV-104-1: duplicate insert_guarded with different provenance — second wins (BTreeMap semantics)
#[test]
fn insert_guarded_conflicting_ids() {
    let mut theory = Theory::new();

    let first = Statement::atomic("dup", "first version — untrusted", "test");
    let out1 = theory.insert_guarded(first);
    assert_eq!(out1.trust_level, TrustLevel::Untrusted);

    let second = Statement::grounded("dup", "second version — trusted", "test", "sha:src");
    let out2 = theory.insert_guarded(second);
    assert_eq!(out2.trust_level, TrustLevel::Trusted);

    // BTreeMap::insert replaces: the second statement wins.
    let present = theory.statements.get("dup").unwrap();
    assert_eq!(present.text, "second version — trusted",
        "ADV-104-1: second insert_guarded must overwrite the first");
}

// ADV-104-2: source_ids with an empty-string entry — non-empty set → Trusted
#[test]
fn insert_guarded_empty_source_id_string() {
    let mut theory = Theory::new();
    let mut p = Provenance::default();
    p.source_ids.insert(String::new()); // empty string, but set is non-empty
    let stmt = Statement {
        id: "s1".into(), text: "A".into(),
        kind: StatementKind::Atomic, domain: "test".into(),
        provenance: p, metadata: Default::default(), formula: None,
        semantic_dimensions: None,
    };
    let outcome = theory.insert_guarded(stmt);
    assert_eq!(
        outcome.trust_level, TrustLevel::Trusted,
        "ADV-104-2: non-empty source_ids set (even with empty string) → Trusted"
    );
}
