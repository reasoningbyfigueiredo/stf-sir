use crate::compiler::compile_markdown;

use super::retention_v2::RetentionV2Score;

#[allow(dead_code)]
pub struct BenchmarkHarness {
    pub corpus_id: String,
    pub compiler_version: String,
}

#[allow(dead_code)]
pub struct CorpusEntry {
    pub document_id: String,
    pub source: String,
    pub expected_token_count: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct EntryReport {
    pub document_id: String,
    pub token_count: usize,
    pub relation_count: usize,
    pub compile_success: bool,
    pub retention_v2: Option<RetentionV2Score>,
}

#[derive(Debug, Clone)]
pub struct AggregateMetrics {
    pub total_documents: usize,
    pub successful_compilations: usize,
    pub mean_token_count: f64,
    pub mean_retention_v2: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct CorpusReport {
    pub corpus_id: String,
    pub compiler_version: String,
    pub entries: Vec<EntryReport>,
    pub aggregate: AggregateMetrics,
}

impl BenchmarkHarness {
    pub fn new(corpus_id: impl Into<String>) -> Self {
        Self {
            corpus_id: corpus_id.into(),
            compiler_version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }

    pub fn run(&self, corpus: &[CorpusEntry]) -> CorpusReport {
        let mut entries = Vec::with_capacity(corpus.len());

        for entry in corpus {
            let result = compile_markdown(&entry.source, None);
            match result {
                Ok(artifact) => {
                    let rv2 = RetentionV2Score::compute(&artifact);
                    entries.push(EntryReport {
                        document_id: entry.document_id.clone(),
                        token_count: artifact.ztokens.len(),
                        relation_count: artifact.relations.len(),
                        compile_success: true,
                        retention_v2: Some(rv2),
                    });
                }
                Err(_) => {
                    entries.push(EntryReport {
                        document_id: entry.document_id.clone(),
                        token_count: 0,
                        relation_count: 0,
                        compile_success: false,
                        retention_v2: None,
                    });
                }
            }
        }

        let aggregate = compute_aggregate(&self.corpus_id, &self.compiler_version, &entries);

        CorpusReport {
            corpus_id: self.corpus_id.clone(),
            compiler_version: self.compiler_version.clone(),
            entries,
            aggregate,
        }
    }
}

fn compute_aggregate(_corpus_id: &str, _compiler_version: &str, entries: &[EntryReport]) -> AggregateMetrics {
    let total_documents = entries.len();
    let successful: Vec<&EntryReport> = entries.iter().filter(|e| e.compile_success).collect();
    let successful_compilations = successful.len();

    let mean_token_count = if successful.is_empty() {
        0.0
    } else {
        successful.iter().map(|e| e.token_count as f64).sum::<f64>() / successful.len() as f64
    };

    let rv2_scores: Vec<f64> = successful
        .iter()
        .filter_map(|e| e.retention_v2.as_ref().map(|rv2| rv2.composite()))
        .collect();

    let mean_retention_v2 = if rv2_scores.is_empty() {
        None
    } else {
        Some(rv2_scores.iter().sum::<f64>() / rv2_scores.len() as f64)
    };

    AggregateMetrics {
        total_documents,
        successful_compilations,
        mean_token_count,
        mean_retention_v2,
    }
}
