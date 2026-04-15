//! FEAT-203-1: Query DSL abstract syntax tree.
//!
//! Defines the `Query` enum representing all composable query operators
//! in the STF-SIR Query DSL v1.

// ---------------------------------------------------------------------------
// Dimension enum
// ---------------------------------------------------------------------------

/// A ZToken dimension selector for `DimensionFilter` queries.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Dimension {
    /// Lexical dimension (L): `source_text`, `plain_text`, `normalized_text`.
    Lexical,
    /// Syntactic dimension (S): `node_type`, `parent_id`, `depth`, `path`.
    Syntactic,
    /// Semantic dimension (Σ): `gloss`, `concepts`, `confidence`.
    Semantic,
    /// Logical dimension (Φ): `relation_ids`.
    Logical,
}

impl std::fmt::Display for Dimension {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Lexical => write!(f, "L"),
            Self::Syntactic => write!(f, "S"),
            Self::Semantic => write!(f, "Σ"),
            Self::Logical => write!(f, "Φ"),
        }
    }
}

// ---------------------------------------------------------------------------
// Query enum
// ---------------------------------------------------------------------------

/// A composable query expression over a `SirGraph`.
///
/// # Determinism guarantee
///
/// All query variants produce sorted, deduplicated result sets. Identical
/// graph + identical query always yields identical results (INV-203-1).
///
/// # Composition
///
/// `And`, `Or`, and `Not` combine any two queries, enabling arbitrary
/// predicate composition.
#[derive(Debug, Clone, PartialEq)]
pub enum Query {
    // -----------------------------------------------------------------------
    // Traversal operators
    // -----------------------------------------------------------------------

    /// Find a path between two nodes by ID.
    ///
    /// Returns all nodes on the shortest path from `from` to `to`,
    /// or an empty result if no path exists.
    Path { from: String, to: String },

    /// Find all ancestors of a node (transitive closure via incoming edges).
    Ancestors { id: String },

    /// Find all descendants of a node (transitive closure via outgoing edges).
    Descendants { id: String },

    /// Extract the subgraph rooted at `root_id`.
    Subgraph {
        root_id: String,
        max_depth: Option<usize>,
    },

    /// Select nodes whose syntactic depth falls in `[min, max]` (inclusive).
    DepthRange { min: usize, max: usize },

    // -----------------------------------------------------------------------
    // Predicate operators
    // -----------------------------------------------------------------------

    /// Select all nodes with the given `node_type` in their syntactic dimension.
    ByType { node_type: String },

    /// Select all nodes connected by relations of the given `category`.
    ByCategory { category: String },

    /// Select all nodes whose `semantic.gloss` contains the given pattern.
    RegexGloss { pattern: String },

    /// Select nodes where a specific dimension field matches a value.
    DimensionFilter {
        dimension: Dimension,
        field: String,
        value: String,
    },

    // -----------------------------------------------------------------------
    // Boolean combinators
    // -----------------------------------------------------------------------

    /// Intersection: nodes in both `lhs` and `rhs`.
    And(Box<Query>, Box<Query>),

    /// Union: nodes in either `lhs` or `rhs`.
    Or(Box<Query>, Box<Query>),

    /// Complement: nodes NOT returned by the inner query.
    Not(Box<Query>),
}

impl Query {
    /// Construct an `And` query without explicit boxing.
    pub fn and(lhs: Query, rhs: Query) -> Self {
        Self::And(Box::new(lhs), Box::new(rhs))
    }

    /// Construct an `Or` query without explicit boxing.
    pub fn or(lhs: Query, rhs: Query) -> Self {
        Self::Or(Box::new(lhs), Box::new(rhs))
    }

    /// Construct a `Not` query without explicit boxing.
    pub fn not(inner: Query) -> Self {
        Self::Not(Box::new(inner))
    }
}
