//! Typed error surface for the STF-SIR library layer.
//!
//! The binary still uses `anyhow` for ergonomics, but library callers can
//! match on these variants — in particular to distinguish fatal diagnostics
//! (such as `SRC_UTF8_INVALID`) from I/O or serialization failures.
//!
//! ## Coherence error taxonomy
//!
//! `CoherenceError` implements the formal error taxonomy from the coherence
//! paper (§8 / Appendix A.9), using the error triple
//! `(Coh_l, Coh_g, Ground)` to distinguish:
//!
//! | `ErrorKind`       | Triple    | Detection                     |
//! |-------------------|-----------|-------------------------------|
//! | `Contradiction`   | (0,-,-)   | Formal verification           |
//! | `Hallucination`   | (1,0,0)   | Δ-tracking + grounding check  |
//! | `Anomaly`         | (1,1,1)*  | Corpus statistics             |
//! | `LexicalDrift`    | (1,0,0)   | ρ < θ threshold               |
//! | `LexicalCollapse` | (0,-,-)   | ρ < 0.1                       |
//! | `NonExecutable`   | (1,1,0)   | Inference produces ∅          |

use std::io;
use std::path::PathBuf;

use thiserror::Error;

use crate::model::Diagnostic;

// ---------------------------------------------------------------------------
// Compiler-pipeline errors (existing)
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum CompileError {
    #[error("failed to read source file {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: io::Error,
    },

    #[error("compilation produced fatal diagnostics")]
    Fatal { diagnostics: Vec<Diagnostic> },

    #[error("syntactic stage reported an unrecoverable condition: {0}")]
    Syntactic(String),

    #[error("failed to serialize artifact to YAML: {0}")]
    Serialization(#[from] serde_yaml_ng::Error),

    #[error("failed to write artifact to {path}: {source}")]
    Write {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
}

pub type CompileResult<T> = Result<T, CompileError>;

// ---------------------------------------------------------------------------
// Coherence error taxonomy (new)
// ---------------------------------------------------------------------------

/// The class of coherence failure.
///
/// Maps directly to the error triple (Coh_l, Coh_g, Ground) defined in the
/// coherence paper.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorKind {
    /// S ⊨ ⊥ — internal logical inconsistency.  Triple (0,-,-).
    Contradiction,
    /// Coh_local = 1 ∧ Ground = 0 — locally coherent, ungrounded.  Triple (1,0,0).
    Hallucination,
    /// P(x) < ε — statistically improbable under the domain distribution.  Triple (1,1,1)*.
    Anomaly,
    /// Meaning deviates without detectable contradiction (ρ < θ).  Triple (1,0,0).
    LexicalDrift,
    /// Source relations destroyed (ρ ≈ 0).  Triple (0,-,-).
    LexicalCollapse,
    /// Coherent but produces no new consequences.  Triple (1,1,0).
    NonExecutable,
}

impl std::fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorKind::Contradiction => write!(f, "CONTRADICTION"),
            ErrorKind::Hallucination => write!(f, "HALLUCINATION"),
            ErrorKind::Anomaly => write!(f, "ANOMALY"),
            ErrorKind::LexicalDrift => write!(f, "LEX_DRIFT"),
            ErrorKind::LexicalCollapse => write!(f, "LEX_COLLAPSE"),
            ErrorKind::NonExecutable => write!(f, "NON_EXECUTABLE"),
        }
    }
}

/// Severity of a coherence error.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Low => write!(f, "low"),
            Severity::Medium => write!(f, "medium"),
            Severity::High => write!(f, "high"),
            Severity::Critical => write!(f, "critical"),
        }
    }
}

/// A coherence error with kind, message, affected statement ids, and severity.
#[derive(Debug, Clone)]
pub struct CoherenceError {
    pub kind: ErrorKind,
    pub message: String,
    pub statement_ids: Vec<String>,
    pub severity: Severity,
}

impl std::fmt::Display for CoherenceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{}/{}] {}",
            self.kind, self.severity, self.message
        )
    }
}
