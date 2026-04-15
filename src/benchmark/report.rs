use serde::{Deserialize, Serialize};

use super::harness::AggregateMetrics;
use super::retention_v2::RetentionV2Score;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkReport {
    pub format: String,
    pub corpus_id: String,
    pub compiler_version: String,
    pub timestamp: String,
    pub aggregate: SerializableAggregateMetrics,
    pub retention_v2: RetentionV2Score,
}

/// Serializable version of [`AggregateMetrics`] (f64 fields are always present).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableAggregateMetrics {
    pub total_documents: usize,
    pub successful_compilations: usize,
    pub mean_token_count: f64,
    pub mean_retention_v2: f64,
}

impl From<&AggregateMetrics> for SerializableAggregateMetrics {
    fn from(m: &AggregateMetrics) -> Self {
        SerializableAggregateMetrics {
            total_documents: m.total_documents,
            successful_compilations: m.successful_compilations,
            mean_token_count: m.mean_token_count,
            mean_retention_v2: m.mean_retention_v2.unwrap_or(1.0),
        }
    }
}

impl BenchmarkReport {
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).expect("BenchmarkReport is always serializable")
    }

    pub fn to_yaml(&self) -> String {
        serde_yaml_ng::to_string(self).expect("BenchmarkReport is always serializable")
    }
}
