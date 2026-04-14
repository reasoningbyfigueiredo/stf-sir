use std::collections::{BTreeMap, BTreeSet};

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
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Statement {
    pub id: StatementId,
    /// Surface text of the proposition (used for logical normalization).
    pub text: String,
    pub kind: StatementKind,
    /// Domain in which this statement is interpreted.
    pub domain: DomainId,
    pub provenance: Provenance,
    pub metadata: BTreeMap<String, String>,
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
        }
    }
}
