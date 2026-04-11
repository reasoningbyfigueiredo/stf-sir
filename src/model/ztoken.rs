use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZToken {
    pub id: String,
    #[serde(rename = "L")]
    pub lexical: LexicalDimension,
    #[serde(rename = "S")]
    pub syntactic: SyntacticDimension,
    #[serde(rename = "Σ")]
    pub semantic: SemanticDimension,
    #[serde(rename = "Φ")]
    pub logical: LogicalDimension,
    #[serde(default, skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    pub extensions: std::collections::BTreeMap<String, serde_yaml_ng::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LexicalDimension {
    pub source_text: String,
    pub plain_text: String,
    pub normalized_text: String,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceSpan {
    pub start_byte: usize,
    pub end_byte: usize,
    pub start_line: usize,
    pub end_line: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyntacticDimension {
    pub node_type: String,
    pub parent_id: Option<String>,
    pub depth: usize,
    pub sibling_index: usize,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticDimension {
    pub gloss: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub concepts: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub confidence: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogicalDimension {
    pub relation_ids: Vec<String>,
}
