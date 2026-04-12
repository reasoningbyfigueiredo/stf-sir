//! STF-SIR v1 artifact validator.
//!
//! Validation runs in two passes:
//!
//! 1. **Schema pass** — the YAML document is parsed as a generic
//!    `serde_json::Value` and validated against the embedded JSON Schema in
//!    [`crate::compiler::schema`]. This enforces all structural and
//!    field-level rules: required fields, constant values (`format ==
//!    "stf-sir.zmd"`, `version == 1`), enum categories, and scalar types.
//! 2. **Semantic pass** — if and only if the schema pass succeeds, the
//!    artifact is deserialized into the typed [`Artifact`] model and the
//!    numbered rules from spec §9 are checked. These are cross-reference
//!    invariants that cannot be expressed in JSON Schema alone: unique ids,
//!    count consistency, reference resolution, exact source-slice equality.
//!
//! Each failure is reported as a [`ValidationError`] with a stable rule code,
//! enabling structured CI gates and external tooling.

use std::collections::HashSet;

use crate::compiler::schema;
use crate::compiler::semantic::normalize_text;
use crate::model::{Artifact, RelationCategory};

/// A single validation failure, carrying a short machine code and a
/// human-readable message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationError {
    pub rule: &'static str,
    pub message: String,
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.rule, self.message)
    }
}

/// Validate a raw YAML `.zmd` document in two passes (schema then semantic).
///
/// `source_bytes` is the original Markdown source required by rule 16
/// (`L.source_text` must equal the exact source slice). Pass `None` if the
/// source is unavailable — rule 16 is then skipped.
///
/// Returns the concatenated error list (schema errors first, then semantic).
/// The semantic pass is only run when the schema pass succeeds, because
/// deserializing a structurally broken artifact into the typed model would
/// either fail or produce misleading downstream errors.
pub fn validate_yaml_str(yaml: &str, source_bytes: Option<&[u8]>) -> Vec<ValidationError> {
    // Pass 1: parse as JSON-compatible Value and schema-validate.
    let instance = match schema::parse_yaml_as_json(yaml) {
        Ok(value) => value,
        Err(err) => {
            return vec![ValidationError {
                rule: "YAML_PARSE",
                message: format!("failed to parse artifact as YAML: {err}"),
            }]
        }
    };

    let schema_errors = schema::validate_value(&instance);
    if !schema_errors.is_empty() {
        return schema_errors;
    }

    // Pass 2: deserialize into typed model and run §9 semantic rules.
    let artifact: Artifact = match serde_json::from_value(instance) {
        Ok(artifact) => artifact,
        Err(err) => {
            return vec![ValidationError {
                rule: "MODEL_DESERIALIZE",
                message: format!(
                "artifact passed schema but could not be deserialized into the typed model: {err}"
            ),
            }]
        }
    };

    validate(&artifact, source_bytes)
}

/// Validate an in-memory artifact. `source_bytes` is the original source
/// buffer required by rule 16 (`L.source_text` must equal the exact slice).
/// Pass `None` if the source is unavailable — in which case rule 16 becomes
/// a length check only.
pub fn validate(artifact: &Artifact, source_bytes: Option<&[u8]>) -> Vec<ValidationError> {
    let mut errors = Vec::new();

    // Rule 1
    if artifact.format != "stf-sir.zmd" {
        errors.push(err(
            "VAL_01_FORMAT",
            format!(
                "format must be \"stf-sir.zmd\", got \"{}\"",
                artifact.format
            ),
        ));
    }

    // Rule 2
    if artifact.version != 1 {
        errors.push(err(
            "VAL_02_VERSION",
            format!("version must be 1, got {}", artifact.version),
        ));
    }

    // Rule 3
    if artifact.source.media_type != "text/markdown" {
        errors.push(err(
            "VAL_03_MEDIA_TYPE",
            format!(
                "source.media_type must be \"text/markdown\", got \"{}\"",
                artifact.source.media_type
            ),
        ));
    }

    // Rule 4
    if artifact.source.encoding != "utf-8" {
        errors.push(err(
            "VAL_04_ENCODING",
            format!(
                "source.encoding must be \"utf-8\", got \"{}\"",
                artifact.source.encoding
            ),
        ));
    }

    // Rule 5: unique ztoken ids
    {
        let mut seen = HashSet::new();
        for token in &artifact.ztokens {
            if !seen.insert(token.id.as_str()) {
                errors.push(err(
                    "VAL_05_TOKEN_ID_UNIQUE",
                    format!("duplicate ztoken id \"{}\"", token.id),
                ));
            }
        }
    }

    // Rule 6: unique relation ids
    {
        let mut seen = HashSet::new();
        for relation in &artifact.relations {
            if !seen.insert(relation.id.as_str()) {
                errors.push(err(
                    "VAL_06_RELATION_ID_UNIQUE",
                    format!("duplicate relation id \"{}\"", relation.id),
                ));
            }
        }
    }

    // Rule 7
    if artifact.document.token_count != artifact.ztokens.len() {
        errors.push(err(
            "VAL_07_TOKEN_COUNT",
            format!(
                "document.token_count ({}) does not match serialized ztokens ({})",
                artifact.document.token_count,
                artifact.ztokens.len()
            ),
        ));
    }

    // Rule 8
    if artifact.document.relation_count != artifact.relations.len() {
        errors.push(err(
            "VAL_08_RELATION_COUNT",
            format!(
                "document.relation_count ({}) does not match serialized relations ({})",
                artifact.document.relation_count,
                artifact.relations.len()
            ),
        ));
    }

    let token_ids: HashSet<&str> = artifact.ztokens.iter().map(|t| t.id.as_str()).collect();
    let relation_ids: HashSet<&str> = artifact.relations.iter().map(|r| r.id.as_str()).collect();

    // Rule 9: every S.parent_id is null or references an existing ztoken
    for token in &artifact.ztokens {
        if let Some(parent) = &token.syntactic.parent_id {
            if !token_ids.contains(parent.as_str()) {
                errors.push(err(
                    "VAL_09_PARENT_REF",
                    format!(
                        "ztoken {} references unknown parent_id \"{}\"",
                        token.id, parent
                    ),
                ));
            }
        }
    }

    // Rules 10, 11, 12, 18
    const RELATION_STAGES: &[&str] = &["lexical", "syntactic", "semantic", "logical"];
    for relation in &artifact.relations {
        if !token_ids.contains(relation.source.as_str()) {
            errors.push(err(
                "VAL_10_RELATION_SOURCE",
                format!(
                    "relation {} source \"{}\" does not reference an existing ztoken",
                    relation.id, relation.source
                ),
            ));
        }
        // Rule 11: external targets are permitted only for semantic-link in v1
        let external_allowed = matches!(relation.category, RelationCategory::SemanticLink);
        if !token_ids.contains(relation.target.as_str()) && !external_allowed {
            errors.push(err(
                "VAL_11_RELATION_TARGET",
                format!(
                    "relation {} target \"{}\" does not reference an existing ztoken",
                    relation.id, relation.target
                ),
            ));
        }
        // Rule 12 is guaranteed statically by the enum, but we leave a hook in
        // case a deserialized artifact ever extends the type downstream.
        let _ = relation.category;
        // Rule 18: stage is a provenance coordinate and must be drawn from a
        // closed set of pipeline stages. See spec §3.10 normative note.
        if !RELATION_STAGES.contains(&relation.stage.as_str()) {
            errors.push(err(
                "VAL_18_RELATION_STAGE",
                format!(
                    "relation {} stage \"{}\" is not one of {:?}",
                    relation.id, relation.stage, RELATION_STAGES
                ),
            ));
        }
    }

    // Rule 13: every Φ.relation_ids entry references an existing relation
    for token in &artifact.ztokens {
        for rid in &token.logical.relation_ids {
            if !relation_ids.contains(rid.as_str()) {
                errors.push(err(
                    "VAL_13_PHI_RELATION_REF",
                    format!(
                        "ztoken {} references unknown relation_id \"{}\"",
                        token.id, rid
                    ),
                ));
            }
        }
    }

    // Rule 14: byte span bounds
    for token in &artifact.ztokens {
        let span = &token.lexical.span;
        if !(span.start_byte < span.end_byte && span.end_byte <= artifact.source.length_bytes) {
            errors.push(err(
                "VAL_14_BYTE_SPAN",
                format!(
                    "ztoken {} has invalid byte span [{}..{}] (length_bytes={})",
                    token.id, span.start_byte, span.end_byte, artifact.source.length_bytes
                ),
            ));
        }
    }

    // Rule 15: line span bounds
    for token in &artifact.ztokens {
        let span = &token.lexical.span;
        if !(1 <= span.start_line && span.start_line <= span.end_line) {
            errors.push(err(
                "VAL_15_LINE_SPAN",
                format!(
                    "ztoken {} has invalid line span [{}..{}]",
                    token.id, span.start_line, span.end_line
                ),
            ));
        }
    }

    // Rule 16: L.source_text equals the exact source slice.
    if let Some(bytes) = source_bytes {
        for token in &artifact.ztokens {
            let span = &token.lexical.span;
            let start = span.start_byte;
            let end = span.end_byte;
            if end > bytes.len() {
                // Already reported under rule 14; skip here to avoid cascade.
                continue;
            }
            let slice = &bytes[start..end];
            if slice != token.lexical.source_text.as_bytes() {
                errors.push(err(
                    "VAL_16_SOURCE_TEXT_SLICE",
                    format!(
                        "ztoken {} L.source_text does not match source[{}..{}]",
                        token.id, start, end
                    ),
                ));
            }
        }
    }

    // Rule 17: STF-SIR v1 semantic fallback.
    for token in &artifact.ztokens {
        let expected_gloss = if token.lexical.plain_text.is_empty() {
            String::new()
        } else {
            normalize_text(&token.lexical.plain_text)
        };

        if token.semantic.gloss != expected_gloss {
            errors.push(err(
                "VAL_17_SEMANTIC_FALLBACK",
                format!(
                    "ztoken {} has Σ.gloss {:?}, expected {:?} under the v1 semantic fallback",
                    token.id, token.semantic.gloss, expected_gloss
                ),
            ));
        }
    }

    errors
}

fn err(rule: &'static str, message: String) -> ValidationError {
    ValidationError { rule, message }
}
