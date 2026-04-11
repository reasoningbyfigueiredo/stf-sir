//! JSON Schema validation layer for `.zmd` v1 artifacts.
//!
//! The canonical schema is embedded from `schemas/zmd-v1.schema.json` so the
//! binary is self-contained. YAML is first parsed into a generic
//! `serde_json::Value` (since every valid `.zmd` is a JSON-compatible tree)
//! and then validated against the compiled schema. All structural and
//! field-level constraints live in the JSON Schema; cross-reference rules
//! are enforced in `validator.rs`.

use std::sync::OnceLock;

use jsonschema::{Draft, JSONSchema};
use thiserror::Error;

use crate::compiler::validator::ValidationError;

/// The embedded schema as raw JSON text.
pub const SCHEMA_JSON: &str = include_str!("../../schemas/zmd-v1.schema.json");

/// Lazily compiled schema shared across calls.
fn schema() -> &'static JSONSchema {
    static SCHEMA: OnceLock<JSONSchema> = OnceLock::new();
    SCHEMA.get_or_init(|| {
        let value: serde_json::Value = serde_json::from_str(SCHEMA_JSON)
            .expect("embedded STF-SIR schema is invalid JSON — build-time bug");
        JSONSchema::options()
            .with_draft(Draft::Draft202012)
            .compile(&value)
            .expect("embedded STF-SIR schema failed to compile — build-time bug")
    })
}

#[derive(Debug, Error)]
pub enum SchemaLoadError {
    #[error("failed to parse artifact as YAML: {0}")]
    Yaml(#[from] serde_yaml_ng::Error),
}

/// Parse a YAML `.zmd` document into a JSON value suitable for schema
/// validation. Returns an error only if the YAML itself is malformed.
pub fn parse_yaml_as_json(yaml: &str) -> Result<serde_json::Value, SchemaLoadError> {
    // `serde_yaml_ng` can deserialize directly into any type implementing
    // `Deserialize`, including `serde_json::Value`, as long as the YAML is a
    // JSON-compatible tree (string keys, scalar leaves). Our `.zmd` format
    // guarantees this.
    let value: serde_json::Value = serde_yaml_ng::from_str(yaml)?;
    Ok(value)
}

/// Validate a parsed artifact value against the compiled STF-SIR v1 schema.
///
/// Returns an empty vector on success. Each failure is rendered as a
/// `ValidationError` with rule code `SCHEMA_VIOLATION` and a message that
/// carries the instance path for precise diagnosis.
pub fn validate_value(instance: &serde_json::Value) -> Vec<ValidationError> {
    match schema().validate(instance) {
        Ok(()) => Vec::new(),
        Err(errors) => errors
            .map(|err| ValidationError {
                rule: "SCHEMA_VIOLATION",
                message: format!("{} at {}", err, err.instance_path),
            })
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_schema_compiles() {
        // Accessing the OnceLock forces compilation; a build-time bug would
        // panic here rather than deeper in the CLI.
        let _ = schema();
    }

    #[test]
    fn schema_rejects_empty_object() {
        let value = serde_json::json!({});
        let errors = validate_value(&value);
        assert!(!errors.is_empty(), "empty object must fail schema");
    }
}
