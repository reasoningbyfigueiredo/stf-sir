//! Typed error surface for the STF-SIR library layer.
//!
//! The binary still uses `anyhow` for ergonomics, but library callers can
//! match on these variants — in particular to distinguish fatal diagnostics
//! (such as `SRC_UTF8_INVALID`) from I/O or serialization failures.

use std::io;
use std::path::PathBuf;

use thiserror::Error;

use crate::model::Diagnostic;

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
