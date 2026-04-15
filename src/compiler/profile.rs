//! FEAT-207-2: Compilation profile system.
//!
//! Named compilation profiles control which ZToken dimensions are populated,
//! which validation rules are applied, and which relation types are emitted.

// ---------------------------------------------------------------------------
// CompilationProfile enum
// ---------------------------------------------------------------------------

/// A named compilation profile that configures the compiler pipeline.
///
/// Profiles are ordered by capability: `BlockV1Mvp` is the minimal v1-compatible
/// profile; `BlockV2`, `SentenceV2`, and `EntityV2` progressively enable more
/// advanced features.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompilationProfile {
    /// Block-level v1 MVP profile.
    ///
    /// Produces output identical to the v1 compiler (INV-207-4). Only the four
    /// core ZToken dimensions (L, S, Σ, Φ) are populated. No contextual,
    /// pragmatic, temporal, or coherence features are active.
    BlockV1Mvp,

    /// Block-level v2 profile.
    ///
    /// Enables contextual and pragmatic dimensions in addition to core v1 output.
    /// Activates the 5 new relation type emitters from spec v2.
    BlockV2,

    /// Sentence-level v2 profile.
    ///
    /// Processes input at sentence granularity. Enables all v2 dimensions
    /// including temporal tracking.
    SentenceV2,

    /// Entity-level v2 profile.
    ///
    /// Fine-grained entity-level processing. Enables all dimensions including
    /// coherence evaluation.
    EntityV2,
}

impl CompilationProfile {
    /// Return the canonical string identifier for this profile.
    ///
    /// These identifiers appear in the compiled artifact's `compiler.profile` field.
    pub fn identifier(&self) -> &str {
        match self {
            Self::BlockV1Mvp => "stf-sir-spec-v1-mvp",
            Self::BlockV2 => "stf-sir-spec-v2-block",
            Self::SentenceV2 => "stf-sir-spec-v2-sentence",
            Self::EntityV2 => "stf-sir-spec-v2-entity",
        }
    }

    /// Return `true` if this profile populates contextual (C) dimensions.
    pub fn allows_contextual(&self) -> bool {
        matches!(self, Self::BlockV2 | Self::SentenceV2 | Self::EntityV2)
    }

    /// Return `true` if this profile populates pragmatic (P) dimensions.
    pub fn allows_pragmatic(&self) -> bool {
        matches!(self, Self::BlockV2 | Self::SentenceV2 | Self::EntityV2)
    }

    /// Return `true` if this profile populates temporal (Δ) dimensions.
    pub fn allows_temporal(&self) -> bool {
        matches!(self, Self::SentenceV2 | Self::EntityV2)
    }

    /// Return `true` if this profile enables coherence evaluation (Ω).
    pub fn allows_coherence_eval(&self) -> bool {
        matches!(self, Self::EntityV2)
    }

    /// Return the set of node types valid under this profile.
    pub fn valid_node_types(&self) -> &[&str] {
        match self {
            Self::BlockV1Mvp => &[
                "heading",
                "paragraph",
                "blockquote",
                "list",
                "list_item",
                "code_block",
                "table",
                "footnote_definition",
            ],
            Self::BlockV2 => &[
                "heading",
                "paragraph",
                "blockquote",
                "list",
                "list_item",
                "code_block",
                "table",
                "footnote_definition",
                "inline_code",
                "link",
                "image",
            ],
            Self::SentenceV2 => &[
                "heading",
                "paragraph",
                "blockquote",
                "list",
                "list_item",
                "code_block",
                "table",
                "footnote_definition",
                "inline_code",
                "link",
                "image",
                "sentence",
                "clause",
            ],
            Self::EntityV2 => &[
                "heading",
                "paragraph",
                "blockquote",
                "list",
                "list_item",
                "code_block",
                "table",
                "footnote_definition",
                "inline_code",
                "link",
                "image",
                "sentence",
                "clause",
                "entity",
                "named_entity",
                "concept_ref",
            ],
        }
    }

    /// Parse a profile identifier string into a `CompilationProfile`.
    ///
    /// Returns `None` if the string does not match any known profile identifier.
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "stf-sir-spec-v1-mvp" | "block-v1" | "block-v1-mvp" => Some(Self::BlockV1Mvp),
            "stf-sir-spec-v2-block" | "block-v2" => Some(Self::BlockV2),
            "stf-sir-spec-v2-sentence" | "sentence-v2" => Some(Self::SentenceV2),
            "stf-sir-spec-v2-entity" | "entity-v2" => Some(Self::EntityV2),
            _ => None,
        }
    }
}

impl Default for CompilationProfile {
    fn default() -> Self {
        Self::BlockV1Mvp
    }
}

impl std::fmt::Display for CompilationProfile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.identifier())
    }
}
