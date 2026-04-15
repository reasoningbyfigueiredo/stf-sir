//! Integration tests for EPIC-205 — Retention & Benchmark.

use stf_sir::benchmark::{
    BenchmarkHarness, BenchmarkReport, CorpusEntry, DriftDetector, RetentionV2Score,
    SerializableAggregateMetrics,
};
use stf_sir::compiler::compile_markdown;

fn compile(source: &str) -> stf_sir::model::Artifact {
    compile_markdown(source, None).expect("compile_markdown failed in test")
}

// ---------------------------------------------------------------------------
// Test 1 — harness runs on a single document
// ---------------------------------------------------------------------------

#[test]
fn harness_runs_single_document() {
    let harness = BenchmarkHarness::new("test-corpus-1");
    let corpus = vec![CorpusEntry {
        document_id: "doc-1".to_string(),
        source: "# Hello\n\nThis is a test document.\n".to_string(),
        expected_token_count: None,
    }];

    let report = harness.run(&corpus);

    assert_eq!(report.corpus_id, "test-corpus-1");
    assert_eq!(report.entries.len(), 1);
    assert!(report.entries[0].compile_success);
    assert!(report.entries[0].token_count > 0);
    assert!(report.entries[0].retention_v2.is_some());
    assert_eq!(report.aggregate.total_documents, 1);
    assert_eq!(report.aggregate.successful_compilations, 1);
}

// ---------------------------------------------------------------------------
// Test 2 — harness aggregates over a corpus of multiple documents
// ---------------------------------------------------------------------------

#[test]
fn harness_aggregate_succeeds_on_corpus() {
    let harness = BenchmarkHarness::new("test-corpus-multi");
    let corpus: Vec<CorpusEntry> = (1..=5)
        .map(|i| CorpusEntry {
            document_id: format!("doc-{i}"),
            source: format!("# Document {i}\n\nContent for document {i}.\n"),
            expected_token_count: None,
        })
        .collect();

    let report = harness.run(&corpus);

    assert_eq!(report.aggregate.total_documents, 5);
    assert_eq!(report.aggregate.successful_compilations, 5);
    assert!(report.aggregate.mean_token_count > 0.0);
    assert!(report.aggregate.mean_retention_v2.is_some());
    let mean = report.aggregate.mean_retention_v2.unwrap();
    assert!((0.0..=1.0).contains(&mean), "mean retention must be in [0, 1]");
}

// ---------------------------------------------------------------------------
// Test 3 — RetentionV2Score composite is the geometric mean
// ---------------------------------------------------------------------------

#[test]
fn retention_v2_composite_is_geometric_mean() {
    let artifact = compile("# Test\n\nParagraph content.\n");
    let rv2 = RetentionV2Score::compute(&artifact);

    // All components must be in [0, 1].
    for &v in &[rv2.rho_l, rv2.rho_s, rv2.rho_sigma_gloss, rv2.rho_sigma_concepts, rv2.rho_phi, rv2.rho_corpus] {
        assert!((0.0..=1.0).contains(&v), "retention component {v} out of [0, 1]");
    }

    let composite = rv2.composite();
    assert!((0.0..=1.0).contains(&composite), "composite must be in [0, 1]");

    // Verify the geometric mean formula for the 6 components.
    let expected = (rv2.rho_l
        * rv2.rho_s
        * rv2.rho_sigma_gloss
        * rv2.rho_sigma_concepts
        * rv2.rho_phi
        * rv2.rho_corpus)
        .powf(1.0 / 6.0);

    assert!(
        (composite - expected).abs() < 1e-10,
        "composite {composite} != expected geometric mean {expected}"
    );
}

// ---------------------------------------------------------------------------
// Test 4 — DriftDetector flags regression
// ---------------------------------------------------------------------------

#[test]
fn drift_detector_flags_regression() {
    let detector = DriftDetector::new(0.02);

    // Build a "baseline" report with perfect scores.
    let baseline_rv2 = RetentionV2Score {
        rho_l: 1.0,
        rho_s: 1.0,
        rho_sigma_gloss: 1.0,
        rho_sigma_concepts: 1.0,
        rho_phi: 1.0,
        rho_corpus: 1.0,
    };
    let baseline_agg = SerializableAggregateMetrics {
        total_documents: 10,
        successful_compilations: 10,
        mean_token_count: 15.0,
        mean_retention_v2: 1.0,
    };
    let baseline = BenchmarkReport {
        format: "stf-sir-benchmark-v1".to_string(),
        corpus_id: "corpus-a".to_string(),
        compiler_version: "1.0.0".to_string(),
        timestamp: "2026-04-14T00:00:00Z".to_string(),
        aggregate: baseline_agg,
        retention_v2: baseline_rv2,
    };

    // Build a "current" report with a 5% regression in rho_phi.
    let current_rv2 = RetentionV2Score {
        rho_l: 1.0,
        rho_s: 1.0,
        rho_sigma_gloss: 1.0,
        rho_sigma_concepts: 1.0,
        rho_phi: 0.90, // regressed by 10% — well above the 2% threshold
        rho_corpus: 0.98,
    };
    let current_agg = SerializableAggregateMetrics {
        total_documents: 10,
        successful_compilations: 10,
        mean_token_count: 15.0,
        mean_retention_v2: 0.96,
    };
    let current = BenchmarkReport {
        format: "stf-sir-benchmark-v1".to_string(),
        corpus_id: "corpus-a".to_string(),
        compiler_version: "1.1.0".to_string(),
        timestamp: "2026-04-14T01:00:00Z".to_string(),
        aggregate: current_agg,
        retention_v2: current_rv2,
    };

    let drift_report = detector.detect(&baseline, &current);

    assert!(drift_report.detected, "drift must be detected when rho_phi drops by 10%");

    let phi_drift = drift_report
        .component_drifts
        .iter()
        .find(|c| c.component == "rho_phi")
        .expect("rho_phi must be in component_drifts");

    assert!(phi_drift.drift_detected, "rho_phi drift must be flagged");
    assert!((phi_drift.delta - (-0.10)).abs() < 1e-10, "delta must be -0.10");

    // rho_l should not have drifted
    let l_drift = drift_report
        .component_drifts
        .iter()
        .find(|c| c.component == "rho_l")
        .expect("rho_l must be in component_drifts");
    assert!(!l_drift.drift_detected, "rho_l must not be flagged — no change");
}
