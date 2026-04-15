//! FEAT-207-1: Enricher trait — formally monotone post-logical enrichment passes.
//!
//! An enricher is a post-logical transformation that MAY augment ZToken semantic
//! and logical dimensions but MUST NOT remove or weaken any existing values.
//!
//! ## Monotonicity contract (INV-207-2)
//!
//! For any enricher E and artifact A:
//! - `concepts` can only grow (never shrink)
//! - `confidence` can only increase (never decrease)
//! - `gloss` is never overwritten (only supplemented)
//! - `relation_ids` can only grow (never shrink)

use crate::model::Artifact;

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

/// An error produced by an enricher during enrichment.
#[derive(Debug, Clone)]
pub struct EnricherError {
    /// Name of the enricher that produced the error.
    pub enricher: String,
    /// Human-readable error message.
    pub message: String,
}

impl std::fmt::Display for EnricherError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "enricher '{}' failed: {}", self.enricher, self.message)
    }
}

impl std::error::Error for EnricherError {}

// ---------------------------------------------------------------------------
// Enricher trait
// ---------------------------------------------------------------------------

/// A formally monotone post-logical enrichment pass.
///
/// Enrichers are applied after the logical stage in fixed registration order.
/// They MUST satisfy the monotonicity invariant: no field value may decrease
/// after enrichment.
pub trait Enricher: Send + Sync {
    /// Human-readable name for this enricher (used in error messages and logs).
    fn name(&self) -> &str;

    /// Enrich the artifact in-place.
    ///
    /// # Monotonicity contract
    ///
    /// After this call returns `Ok(())`:
    /// - every `ztoken.semantic.concepts` is a superset of its pre-call value
    /// - every `ztoken.semantic.confidence` is ≥ its pre-call value
    /// - every `ztoken.semantic.gloss` is unchanged or has only been supplemented
    /// - every `ztoken.logical.relation_ids` is a superset of its pre-call value
    fn enrich(&self, artifact: &mut Artifact) -> Result<(), EnricherError>;
}

// ---------------------------------------------------------------------------
// PassthroughEnricher — no-op identity enricher
// ---------------------------------------------------------------------------

/// A no-op enricher that leaves the artifact completely unchanged.
///
/// Used for testing pipeline wiring and as a sentinel in registration.
pub struct PassthroughEnricher;

impl Enricher for PassthroughEnricher {
    fn name(&self) -> &str {
        "passthrough"
    }

    fn enrich(&self, _artifact: &mut Artifact) -> Result<(), EnricherError> {
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// EnricherPipeline — ordered list of enrichers
// ---------------------------------------------------------------------------

/// Runs a list of enrichers in fixed registration order.
///
/// Stops on the first error and returns it. Enrichers are applied sequentially
/// (no parallel execution) to guarantee deterministic ordering (INV-207-3).
pub struct EnricherPipeline {
    enrichers: Vec<Box<dyn Enricher>>,
}

impl EnricherPipeline {
    /// Create an empty pipeline.
    pub fn new() -> Self {
        Self {
            enrichers: Vec::new(),
        }
    }

    /// Register an enricher at the end of the pipeline.
    ///
    /// Registration order determines execution order and is fixed after
    /// `apply` is called.
    pub fn register<E: Enricher + 'static>(&mut self, enricher: E) -> &mut Self {
        self.enrichers.push(Box::new(enricher));
        self
    }

    /// Apply all registered enrichers in registration order.
    ///
    /// Returns `Err` on the first enricher failure; subsequent enrichers are
    /// not executed.
    pub fn apply(&self, artifact: &mut Artifact) -> Result<(), EnricherError> {
        for enricher in &self.enrichers {
            enricher.enrich(artifact)?;
        }
        Ok(())
    }

    /// Number of registered enrichers.
    pub fn len(&self) -> usize {
        self.enrichers.len()
    }

    /// Returns `true` if no enrichers are registered.
    pub fn is_empty(&self) -> bool {
        self.enrichers.is_empty()
    }
}

impl Default for EnricherPipeline {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// ConceptExtractorEnricher — keyword-based concept extractor
// ---------------------------------------------------------------------------

/// A simple keyword-based concept extractor that populates `Σ.concepts`
/// from non-stopword tokens found in `Σ.gloss`.
///
/// Extracts lowercase alphabetic tokens, filters English and Portuguese
/// stopwords, deduplicates, sorts, and appends any new concepts to the
/// existing list (monotone: never removes existing concepts).
pub struct ConceptExtractorEnricher;

const STOPWORDS: &[&str] = &[
    // English
    "a", "an", "the", "and", "or", "but", "in", "on", "at", "to", "for",
    "of", "with", "by", "from", "is", "are", "was", "were", "be", "been",
    "being", "have", "has", "had", "do", "does", "did", "will", "would",
    "could", "should", "may", "might", "shall", "can", "not", "no", "nor",
    "so", "yet", "both", "either", "neither", "as", "if", "than", "that",
    "this", "these", "those", "it", "its", "its", "he", "she", "they",
    "we", "you", "i", "me", "him", "her", "us", "them", "my", "your",
    "his", "our", "their", "what", "which", "who", "how", "when", "where",
    "why", "all", "each", "every", "some", "any", "few", "more", "most",
    "other", "into", "through", "during", "before", "after", "above",
    "below", "up", "down", "out", "off", "over", "under", "again",
    "then", "once", "here", "there", "about", "against", "between",
    "s", "t", "re", "ve", "ll", "d", "m",
    // Portuguese
    "o", "a", "os", "as", "um", "uma", "uns", "umas", "de", "do", "da",
    "dos", "das", "em", "no", "na", "nos", "nas", "ao", "aos", "à",
    "às", "pelo", "pela", "pelos", "pelas", "num", "numa", "nuns", "numas",
    "com", "por", "para", "sem", "sob", "sobre", "entre", "até", "após",
    "e", "ou", "mas", "que", "se", "nem", "é", "são", "foi", "foram",
    "ser", "ter", "ir", "seu", "sua", "seus", "suas", "me", "te", "lhe",
    "nos", "vos", "lhes", "eu", "tu", "ele", "ela", "nós", "vós", "eles",
    "elas", "isso", "este", "esta", "esse", "essa", "aquele", "aquela",
    "não", "já", "mais", "menos", "também", "só", "ainda", "quando",
    "como", "onde", "porque", "então", "muito", "pouco", "todo", "toda",
];

impl ConceptExtractorEnricher {
    fn extract_keywords(gloss: &str) -> Vec<String> {
        let mut keywords: Vec<String> = gloss
            .split(|c: char| !c.is_alphabetic())
            .filter(|s| s.len() >= 3)
            .map(|s| s.to_lowercase())
            .filter(|s| !STOPWORDS.contains(&s.as_str()))
            .collect();

        keywords.sort();
        keywords.dedup();
        keywords
    }
}

impl Enricher for ConceptExtractorEnricher {
    fn name(&self) -> &str {
        "concept-extractor"
    }

    fn enrich(&self, artifact: &mut Artifact) -> Result<(), EnricherError> {
        for ztoken in &mut artifact.ztokens {
            let new_keywords = Self::extract_keywords(&ztoken.semantic.gloss);
            for kw in new_keywords {
                if !ztoken.semantic.concepts.contains(&kw) {
                    ztoken.semantic.concepts.push(kw);
                }
            }
            // Keep concepts sorted and deduplicated (monotone: only grows)
            ztoken.semantic.concepts.sort();
            ztoken.semantic.concepts.dedup();
        }
        Ok(())
    }
}
