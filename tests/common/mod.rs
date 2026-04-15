#![allow(dead_code)]

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result};
use stf_sir::compiler;
use stf_sir::compiler::validator::{self, ValidationError};
use stf_sir::model::Artifact;
use stf_sir::sir::SirGraph;

pub fn repo_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

pub fn compile_fixture(relative: &str) -> Result<Artifact> {
    let path = repo_root().join(relative);
    compile_fixture_path(&path)
}

pub fn compile_fixture_path(path: &Path) -> Result<Artifact> {
    let source = fs::read_to_string(path)
        .with_context(|| format!("failed to read fixture {}", path.display()))?;
    let relative = path.strip_prefix(repo_root()).unwrap_or(path);
    compiler::compile_markdown(&source, Some(relative)).map_err(Into::into)
}

pub fn serialize_artifact(artifact: &Artifact) -> Result<String> {
    compiler::serializer::to_yaml_string(artifact).map_err(Into::into)
}

pub fn compile_and_serialize_fixture(relative: &str) -> Result<(Artifact, String)> {
    let artifact = compile_fixture(relative)?;
    let yaml = serialize_artifact(&artifact)?;
    Ok((artifact, yaml))
}

pub fn validate_artifact_with_fixture_source(
    artifact: &Artifact,
    relative: &str,
) -> Result<Vec<ValidationError>> {
    let source_bytes = fixture_bytes(relative)?;
    let yaml = serialize_artifact(artifact)?;
    Ok(validator::validate_yaml_str(&yaml, Some(&source_bytes)))
}

pub fn fixture_bytes(relative: &str) -> Result<Vec<u8>> {
    let path = repo_root().join(relative);
    fs::read(&path).with_context(|| format!("failed to read fixture bytes {}", path.display()))
}

pub fn read_fixture_string(relative: &str) -> Result<String> {
    let path = repo_root().join(relative);
    fs::read_to_string(&path)
        .with_context(|| format!("failed to read fixture text {}", path.display()))
}

pub fn compile_cli_relative(relative: &Path, output_path: &Path) -> Result<()> {
    let status = Command::new(env!("CARGO_BIN_EXE_stf-sir"))
        .current_dir(repo_root())
        .arg("compile")
        .arg(relative)
        .arg("-o")
        .arg(output_path)
        .status()
        .with_context(|| format!("failed to compile {}", relative.display()))?;
    anyhow::ensure!(
        status.success(),
        "compiler exited with non-zero status for {}",
        relative.display()
    );
    Ok(())
}

pub fn sorted_files_with_extension(dir: &Path, extension: &str) -> Result<Vec<PathBuf>> {
    let mut files = fs::read_dir(dir)?
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some(extension))
        .collect::<Vec<_>>();
    files.sort();
    Ok(files)
}

pub fn assert_artifact_id_uniqueness(artifact: &Artifact) {
    let token_ids = artifact
        .ztokens
        .iter()
        .map(|token| token.id.as_str())
        .collect::<BTreeSet<_>>();
    let relation_ids = artifact
        .relations
        .iter()
        .map(|relation| relation.id.as_str())
        .collect::<BTreeSet<_>>();

    assert_eq!(
        token_ids.len(),
        artifact.ztokens.len(),
        "duplicate ztoken ids"
    );
    assert_eq!(
        relation_ids.len(),
        artifact.relations.len(),
        "duplicate relation ids"
    );
}

pub fn assert_graph_indexes_consistent(graph: &SirGraph) {
    assert_eq!(graph.node_by_id.len(), graph.nodes.len());

    for (index, node) in graph.nodes.iter().enumerate() {
        assert_eq!(
            graph.node_by_id.get(&node.id).copied(),
            Some(index),
            "node_by_id missing or mismatched for {}",
            node.id
        );
    }

    for (node_id, edge_indexes) in &graph.outgoing {
        for edge_index in edge_indexes {
            let edge = &graph.edges[*edge_index];
            assert_eq!(
                edge.source, *node_id,
                "outgoing index for {node_id} contains edge {} from {}",
                edge.id, edge.source
            );
        }
    }

    for (node_id, edge_indexes) in &graph.incoming {
        for edge_index in edge_indexes {
            let edge = &graph.edges[*edge_index];
            assert_eq!(
                edge.target, *node_id,
                "incoming index for {node_id} contains edge {} to {}",
                edge.id, edge.target
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Theory builder helpers — used across multiple test suites

/// Build a `Theory` from `(id, text)` pairs; all statements are atomic.
pub fn atomic_theory(pairs: &[(&str, &str)]) -> stf_sir::model::Theory {
    let mut t = stf_sir::model::Theory::new();
    for &(id, text) in pairs {
        t.insert(stf_sir::model::Statement::atomic(id, text, "test"));
    }
    t
}

/// Build a `Theory` from `(id, text, source_id)` triples; all statements are grounded.
pub fn grounded_theory(triples: &[(&str, &str, &str)]) -> stf_sir::model::Theory {
    let mut t = stf_sir::model::Theory::new();
    for &(id, text, src) in triples {
        t.insert(stf_sir::model::Statement::grounded(id, text, "test", src));
    }
    t
}

/// Build a `MappingResult` stub for retention tests.
pub fn make_mapping(
    id: &str,
    rho: f32,
    structure_preserved: bool,
    drift: f32,
) -> stf_sir::compiler::domain::MappingResult {
    stf_sir::compiler::domain::MappingResult {
        source_statement_id: id.to_string(),
        target_statement: stf_sir::model::Statement::atomic(
            format!("{id}:mapped"),
            "mapped",
            "target",
        ),
        retention_score: rho,
        structure_preserved,
        semantic_drift_score: drift,
    }
}

pub fn assert_retention_unit_interval(artifact: &Artifact) {
    let baseline = artifact.retention_baseline();
    for value in [
        baseline.vector.rho_l,
        baseline.vector.rho_s,
        baseline.vector.rho_sigma,
        baseline.vector.rho_phi,
    ] {
        assert!(
            (0.0..=1.0).contains(&value),
            "retention component {value} was outside [0, 1]"
        );
    }
}
