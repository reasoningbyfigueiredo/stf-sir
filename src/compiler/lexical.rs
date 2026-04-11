use std::path::Path;

use sha2::{Digest, Sha256};

use crate::compiler::diagnostics::{make as make_diag, DiagnosticCode};
use crate::error::{CompileError, CompileResult};
use crate::model::{DiagnosticSeverity, SourceSpan};

#[derive(Debug, Clone)]
pub struct LexicalDocument {
    pub path: Option<String>,
    pub source: String,
    pub length_bytes: usize,
    pub sha256: String,
    pub line_starts: Vec<usize>,
}

pub fn load_from_path(path: &Path) -> CompileResult<LexicalDocument> {
    let bytes = std::fs::read(path).map_err(|source| CompileError::Io {
        path: path.to_path_buf(),
        source,
    })?;

    let path_display = path.to_string_lossy().into_owned();

    let source = String::from_utf8(bytes).map_err(|err| {
        let diag = make_diag(
            DiagnosticCode::SrcUtf8Invalid,
            DiagnosticSeverity::Error,
            format!(
                "source {} is not valid UTF-8 (first invalid byte at position {})",
                path_display,
                err.utf8_error().valid_up_to()
            ),
            None,
        );
        CompileError::Fatal {
            diagnostics: vec![diag],
        }
    })?;

    load_from_string(source, Some(path_display))
}

pub fn load_from_string(source: String, path: Option<String>) -> CompileResult<LexicalDocument> {
    let length_bytes = source.len();
    let sha256 = sha256_prefixed(source.as_bytes());
    let line_starts = compute_line_starts(&source);

    Ok(LexicalDocument {
        path,
        source,
        length_bytes,
        sha256,
        line_starts,
    })
}

pub fn sha256_prefixed(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    format!("sha256:{digest:x}")
}

pub fn slice_source(source: &str, start: usize, end: usize) -> anyhow::Result<String> {
    use anyhow::Context;
    source
        .get(start..end)
        .map(ToOwned::to_owned)
        .with_context(|| format!("invalid UTF-8 boundaries for slice {start}..{end}"))
}

pub fn span_from_offsets(document: &LexicalDocument, start: usize, end: usize) -> SourceSpan {
    let end_lookup = end.saturating_sub(1);

    SourceSpan {
        start_byte: start,
        end_byte: end,
        start_line: byte_offset_to_line(&document.line_starts, start),
        end_line: byte_offset_to_line(&document.line_starts, end_lookup),
    }
}

fn compute_line_starts(source: &str) -> Vec<usize> {
    let mut starts = vec![0];
    for (index, byte) in source.bytes().enumerate() {
        if byte == b'\n' {
            starts.push(index + 1);
        }
    }
    starts
}

/// Convert a byte offset into a 1-based line number.
///
/// Invariant: `byte_offset_to_line(&[0], 0) == 1`. A span of zero bytes
/// collapses to its starting line (see `span_from_offsets`).
fn byte_offset_to_line(line_starts: &[usize], offset: usize) -> usize {
    match line_starts.binary_search(&offset) {
        Ok(index) => index + 1,
        Err(index) => index,
    }
}
