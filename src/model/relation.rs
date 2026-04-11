use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelationCategory {
    #[serde(rename = "structural")]
    Structural,
    #[serde(rename = "logical")]
    Logical,
    #[serde(rename = "semantic-link")]
    SemanticLink,
}

impl RelationCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Structural => "structural",
            Self::Logical => "logical",
            Self::SemanticLink => "semantic-link",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relation {
    pub id: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub category: RelationCategory,
    pub source: String,
    pub target: String,
    pub stage: String,
    #[serde(default, skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    pub attributes: std::collections::BTreeMap<String, serde_yaml_ng::Value>,
    #[serde(default, skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    pub extensions: std::collections::BTreeMap<String, serde_yaml_ng::Value>,
}
