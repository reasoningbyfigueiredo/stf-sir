//! Tests for referential grounding (Ground predicate) — Definition E2.

use stf_sir::model::{Statement, Provenance, StatementKind};
use stf_sir::compiler::grounding::{GroundingChecker, ProvenanceGroundingChecker};
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
        metadata: BTreeMap::new(),
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
