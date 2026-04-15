//! FEAT-207-2: SourceParser trait — pluggable source frontend abstraction.
//!
//! Abstracts the Markdown parser (and future parsers) behind a common trait,
//! enabling multi-frontend compilation pipelines.

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

/// An error produced by a source parser frontend.
#[derive(Debug, Clone)]
pub struct FrontendError {
    /// Human-readable error message.
    pub message: String,
}

impl std::fmt::Display for FrontendError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "frontend parse error: {}", self.message)
    }
}

impl std::error::Error for FrontendError {}

// ---------------------------------------------------------------------------
// ParsedDocument
// ---------------------------------------------------------------------------

/// The output of a `SourceParser::parse` call.
///
/// Contains the validated source text and optional path hint for diagnostics.
#[derive(Debug, Clone)]
pub struct ParsedDocument {
    /// The original source text (UTF-8 string).
    pub source_text: String,
    /// Optional file path for diagnostic messages.
    pub path: Option<String>,
}

// ---------------------------------------------------------------------------
// SourceParser trait
// ---------------------------------------------------------------------------

/// A pluggable source frontend that validates and normalises source input.
///
/// # Object safety
///
/// This trait is object-safe and can be used as `Box<dyn SourceParser>`.
pub trait SourceParser: Send + Sync {
    /// IANA media type this parser handles (e.g. `"text/markdown"`).
    fn media_type(&self) -> &str;

    /// Parse and validate the source text.
    ///
    /// Returns a `ParsedDocument` on success, or a `FrontendError` if the
    /// source is malformed or cannot be processed.
    fn parse(&self, source: &str, path: Option<&str>) -> Result<ParsedDocument, FrontendError>;
}

// ---------------------------------------------------------------------------
// MarkdownFrontend
// ---------------------------------------------------------------------------

/// A `SourceParser` implementation for Markdown source documents.
///
/// Accepts any valid UTF-8 text and wraps it in a `ParsedDocument`. Since
/// Markdown has no hard parsing errors, this frontend only fails if the
/// source is not valid UTF-8 (which cannot happen at the `&str` API level).
pub struct MarkdownFrontend;

impl SourceParser for MarkdownFrontend {
    fn media_type(&self) -> &str {
        "text/markdown"
    }

    fn parse(&self, source: &str, path: Option<&str>) -> Result<ParsedDocument, FrontendError> {
        // Markdown is inherently tolerant — any valid UTF-8 string is accepted.
        // Future versions may add structural validation (e.g. max nesting depth).
        Ok(ParsedDocument {
            source_text: source.to_owned(),
            path: path.map(str::to_owned),
        })
    }
}
