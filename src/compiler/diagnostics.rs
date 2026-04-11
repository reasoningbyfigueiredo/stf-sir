//! Typed diagnostic codes for STF-SIR v1 per spec §10.
//!
//! Each stage emits diagnostics through one of these codes so that downstream
//! consumers can match on stable identifiers rather than human-readable text.

use crate::model::{Diagnostic, DiagnosticSeverity};

/// Closed set of diagnostic codes recognised by the reference compiler.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticCode {
    /// §5.1 — Source bytes are not valid UTF-8.
    SrcUtf8Invalid,
    /// §5.2 — Markdown parser reported an unrecoverable failure.
    SynParseFailed,
    /// §5.2 — A Markdown construct is not emitted as a ztoken (e.g. raw HTML).
    SynNodeUnsupported,
    /// §5.3 — Semantic enrichment unavailable; MVP fallback applied.
    SemFallbackApplied,
    /// §5.4 — An optional relation could not be produced.
    LogRelationSkipped,
    /// §9 — Validation step rejected the artifact.
    ValSchemaFailed,
}

impl DiagnosticCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SrcUtf8Invalid => "SRC_UTF8_INVALID",
            Self::SynParseFailed => "SYN_PARSE_FAILED",
            Self::SynNodeUnsupported => "SYN_NODE_UNSUPPORTED",
            Self::SemFallbackApplied => "SEM_FALLBACK_APPLIED",
            Self::LogRelationSkipped => "LOG_RELATION_SKIPPED",
            Self::ValSchemaFailed => "VAL_SCHEMA_FAILED",
        }
    }

    pub const fn stage(self) -> &'static str {
        match self {
            Self::SrcUtf8Invalid => "lexical",
            Self::SynParseFailed | Self::SynNodeUnsupported => "syntactic",
            Self::SemFallbackApplied => "semantic",
            Self::LogRelationSkipped => "logical",
            Self::ValSchemaFailed => "validation",
        }
    }
}

pub fn make(
    code: DiagnosticCode,
    severity: DiagnosticSeverity,
    message: impl Into<String>,
    token_id: Option<String>,
) -> Diagnostic {
    Diagnostic {
        code: code.as_str().to_string(),
        severity,
        message: message.into(),
        token_id,
        stage: code.stage().to_string(),
    }
}
