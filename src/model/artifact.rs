use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::model::{Relation, ZToken};
use crate::retention::RetentionBaseline;
use crate::sir::SirGraph;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artifact {
    pub format: String,
    pub version: u32,
    pub source: SourceInfo,
    pub compiler: CompilerInfo,
    pub document: DocumentInfo,
    pub ztokens: Vec<ZToken>,
    pub relations: Vec<Relation>,
    pub diagnostics: Vec<Diagnostic>,
    #[serde(default)]
    pub extensions: BTreeMap<String, serde_yaml_ng::Value>,
}

impl Artifact {
    pub fn as_sir_graph(&self) -> SirGraph {
        SirGraph::from_artifact(self)
    }

    pub fn retention_baseline(&self) -> RetentionBaseline {
        RetentionBaseline::from_artifact(self)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceInfo {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    pub media_type: String,
    pub encoding: String,
    pub length_bytes: usize,
    pub sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompilerInfo {
    pub name: String,
    pub version: String,
    pub config_hash: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub profile: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentInfo {
    pub language: String,
    pub token_count: usize,
    pub relation_count: usize,
    pub root_token_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnostic {
    pub code: String,
    pub severity: DiagnosticSeverity,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub token_id: Option<String>,
    pub stage: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DiagnosticSeverity {
    Info,
    Warning,
    Error,
}
