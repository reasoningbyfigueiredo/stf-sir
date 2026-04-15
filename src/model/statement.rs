use std::collections::{BTreeMap, BTreeSet};

use crate::model::formula::Formula;
use crate::model::semantic_dimensions::SemanticDimensions;

pub type StatementId = String;
pub type DomainId = String;
pub type SourceId = String;

/// Classification of a statement by how it entered the theory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StatementKind {
    /// Directly observed or loaded from a source artefact.
    Atomic,
    /// Derived by an inference rule from other statements.
    Derived,
    /// An externally supplied observation (sensor / world state).
    Observation,
    /// Produced by a lexical mapping between domains.
    LexicalMapping,
}

/// Provenance record: where did this statement come from?
///
/// A statement with empty `source_ids` and `anchors` and `grounded = false`
/// is a candidate hallucination under Definition E2 of the coherence paper.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Provenance {
    /// Identifiers of source artefacts that ground this statement.
    pub source_ids: BTreeSet<SourceId>,
    /// Byte-level or token-level anchors in the source artefact.
    pub anchors: BTreeSet<String>,
    /// Name of the inference rule or transformation that generated this statement,
    /// if it was derived rather than directly observed.
    pub generated_by: Option<String>,
    /// Explicit grounding flag.  Set to `true` when the statement is directly
    /// backed by a source anchor, even if `source_ids` and `anchors` are empty
    /// (e.g., for ground-truth axioms supplied by the caller).
    pub grounded: bool,
}

/// A propositional statement within a theory.
///
/// Maps to the notion of a proposition p ∈ U in the coherence paper.
///
/// The `formula` field carries a pre-parsed `Formula` so that coherence and
/// inference engines can operate directly on the AST without re-parsing `text`
/// on every call.  It is `None` for statements whose text does not conform to
/// any recognised formula syntax, or for statements constructed without
/// explicit formula enrichment.  Use `Statement::with_formula` to attach one.
#[derive(Debug, Clone, PartialEq)]
pub struct Statement {
    pub id: StatementId,
    /// Surface text of the proposition (used for logical normalization).
    pub text: String,
    pub kind: StatementKind,
    /// Domain in which this statement is interpreted.
    pub domain: DomainId,
    pub provenance: Provenance,
    pub metadata: BTreeMap<String, String>,
    /// Pre-parsed logical formula; `None` means "not yet parsed" or
    /// "unparseable".  Engines fall back to `Formula::parse(&self.text)` when
    /// this is `None`.
    pub formula: Option<Formula>,
    /// Semantic dimensions of second order (C/P/Δ/Ω).
    ///
    /// `None` if the statement has not been evaluated by the engine.
    /// Set by calling [`SemanticDimensions::from_evaluation`] with an
    /// [`EvaluationResult`][crate::compiler::EvaluationResult].
    pub semantic_dimensions: Option<SemanticDimensions>,
}

impl Statement {
    /// Convenience constructor for atomic statements with no provenance.
    pub fn atomic(id: impl Into<String>, text: impl Into<String>, domain: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            text: text.into(),
            kind: StatementKind::Atomic,
            domain: domain.into(),
            provenance: Provenance::default(),
            metadata: BTreeMap::new(),
            formula: None,
            semantic_dimensions: None,
        }
    }

    /// Convenience constructor for grounded atomic statements (source anchor provided).
    pub fn grounded(
        id: impl Into<String>,
        text: impl Into<String>,
        domain: impl Into<String>,
        source_id: impl Into<String>,
    ) -> Self {
        let source_id = source_id.into();
        let mut provenance = Provenance::default();
        provenance.source_ids.insert(source_id);
        provenance.grounded = true;
        Self {
            id: id.into(),
            text: text.into(),
            kind: StatementKind::Atomic,
            domain: domain.into(),
            provenance,
            metadata: BTreeMap::new(),
            formula: None,
            semantic_dimensions: None,
        }
    }

    /// Builder: attach a pre-parsed formula, replacing any existing one.
    pub fn with_formula(mut self, formula: Formula) -> Self {
        self.formula = Some(formula);
        self
    }
}
