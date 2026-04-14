//! Minimal propositional formula AST.
//!
//! Replaces the surface-text string hacks ("NOT X", "A -> B") used in the
//! MVP coherence and inference layers with a genuine algebraic representation.
//!
//! ## Grammar (subset of propositional logic)
//!
//! ```text
//! formula ::= atom
//!           | "NOT" formula
//!           | formula "->" formula
//! atom    ::= [A-Za-z0-9_:. ]+   (any non-connective text)
//! ```
//!
//! This is intentionally minimal.  The goal is to move from fragile string
//! comparison to a typed structure that supports:
//!   - structural contradiction detection (Theorem A4 / ex contradictione)
//!   - modus ponens via pattern matching (Theorem A9 / homomorphism)
//!   - display back to normalized text
//!
//! Future versions can extend this AST with And, Or, ForAll, Exists.

use std::fmt;

/// A propositional formula node.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Formula {
    /// An atomic proposition — the leaves of the formula tree.
    Atom(String),
    /// Negation: ¬φ.
    Not(Box<Formula>),
    /// Implication: φ → ψ.
    Implies(Box<Formula>, Box<Formula>),
}

impl Formula {
    // -----------------------------------------------------------------------
    // Construction helpers

    pub fn atom(s: impl Into<String>) -> Self {
        Formula::Atom(s.into())
    }

    pub fn not(inner: Formula) -> Self {
        Formula::Not(Box::new(inner))
    }

    pub fn implies(premise: Formula, conclusion: Formula) -> Self {
        Formula::Implies(Box::new(premise), Box::new(conclusion))
    }

    // -----------------------------------------------------------------------
    // Logical operations

    /// Return the logical negation of this formula, applying double-negation
    /// elimination eagerly.
    ///
    /// - `negate(Not(Not(φ))) = φ`   (DNE: strip two negations)
    /// - `negate(Not(φ))      = φ`   (complement: strip one negation)
    /// - `negate(φ)           = Not(φ)` (introduce negation)
    pub fn negate(self) -> Self {
        match self {
            Formula::Not(inner) => match *inner {
                Formula::Not(inner2) => *inner2, // ¬¬φ = φ
                other => other,
            },
            other => Formula::Not(Box::new(other)),
        }
    }

    /// Return `true` if this formula is the direct logical negation of `other`.
    ///
    /// Formally: `self.contradicts(other)` iff `self = ¬other` or `other = ¬self`.
    pub fn contradicts(&self, other: &Formula) -> bool {
        match (self, other) {
            (Formula::Not(a), b) => a.as_ref() == b,
            (a, Formula::Not(b)) => a == b.as_ref(),
            _ => false,
        }
    }

    /// Return `true` if this is a `Not` node.
    pub fn is_negation(&self) -> bool {
        matches!(self, Formula::Not(_))
    }

    /// Return `true` if this is an `Implies` node.
    pub fn is_implication(&self) -> bool {
        matches!(self, Formula::Implies(_, _))
    }

    // -----------------------------------------------------------------------
    // Parsing

    /// Parse a formula from a normalized text string.
    ///
    /// Supported surface forms (case-insensitive after trimming):
    /// - `"NOT <rest>"` → `Not(parse(rest))`
    /// - `"<lhs> -> <rhs>"` → `Implies(parse(lhs), parse(rhs))`
    /// - anything else → `Atom(normalized_uppercase)`
    ///
    /// Returns `None` only on empty input.
    pub fn parse(text: &str) -> Option<Self> {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return None;
        }

        let upper = trimmed.to_uppercase();

        // NOT <rest>
        if let Some(rest) = upper.strip_prefix("NOT ") {
            return Some(Formula::Not(Box::new(
                Formula::parse(rest).unwrap_or_else(|| Formula::Atom(rest.to_string())),
            )));
        }

        // <lhs> -> <rhs>  (split on first occurrence of " -> ")
        if let Some(pos) = find_implies_arrow(&upper) {
            let lhs = &upper[..pos].trim_end().to_string();
            let rhs = &upper[pos + 4..].trim_start().to_string();
            let lhs_f = Formula::parse(lhs).unwrap_or_else(|| Formula::Atom(lhs.clone()));
            let rhs_f = Formula::parse(rhs).unwrap_or_else(|| Formula::Atom(rhs.clone()));
            return Some(Formula::Implies(Box::new(lhs_f), Box::new(rhs_f)));
        }

        Some(Formula::Atom(upper))
    }
}

/// Find the position of ` -> ` in a string (first occurrence).
fn find_implies_arrow(s: &str) -> Option<usize> {
    s.find(" -> ")
}

// -----------------------------------------------------------------------
// Display

impl fmt::Display for Formula {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Formula::Atom(s) => write!(f, "{s}"),
            Formula::Not(inner) => write!(f, "NOT {inner}"),
            Formula::Implies(p, q) => write!(f, "{p} -> {q}"),
        }
    }
}

// -----------------------------------------------------------------------
// Tests

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_atom() {
        assert_eq!(Formula::parse("A"), Some(Formula::atom("A")));
    }

    #[test]
    fn parse_not() {
        assert_eq!(
            Formula::parse("NOT A"),
            Some(Formula::not(Formula::atom("A")))
        );
    }

    #[test]
    fn parse_implies() {
        assert_eq!(
            Formula::parse("A -> B"),
            Some(Formula::implies(Formula::atom("A"), Formula::atom("B")))
        );
    }

    #[test]
    fn contradicts_atom_and_not_atom() {
        let a = Formula::atom("A");
        let not_a = Formula::not(Formula::atom("A"));
        assert!(a.contradicts(&not_a));
        assert!(not_a.contradicts(&a));
    }

    #[test]
    fn double_negation_elimination() {
        let a = Formula::atom("A");
        let nn_a = Formula::not(Formula::not(Formula::atom("A")));
        assert_eq!(nn_a.negate(), a);
    }

    #[test]
    fn display_roundtrip() {
        let f = Formula::implies(
            Formula::not(Formula::atom("A")),
            Formula::atom("B"),
        );
        assert_eq!(f.to_string(), "NOT A -> B");
    }

    #[test]
    fn empty_parse_returns_none() {
        assert_eq!(Formula::parse(""), None);
        assert_eq!(Formula::parse("   "), None);
    }
}
