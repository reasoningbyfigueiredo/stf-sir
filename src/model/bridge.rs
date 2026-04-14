//! Official bridge: Artifact → Theory.
//!
//! This module closes the architectural gap identified in the peer review:
//! the conversion between the "artefact layer" (Artifact, ZToken) and the
//! "theory layer" (Theory, Statement) was only implicit in the CLI.
//!
//! ## Provenance mapping (ZToken → Statement)
//!
//! | ZToken field              | Statement / Provenance field |
//! |---------------------------|------------------------------|
//! | `token.id`                | `Statement.id`               |
//! | `token.lexical.normalized_text` | `Statement.text`       |
//! | `token.syntactic.node_type` | `Statement.domain`         |
//! | `artifact.source.sha256`  | `Provenance.source_ids`      |
//! | `token.id` (ztoken anchor)| `Provenance.anchors`         |
//! | `token.lexical.span` → `"<start_byte>:<end_byte>"` | `Provenance.anchors` |
//! | `token.logical.relation_ids` | `Statement.metadata["relation_ids"]` |
//! | `token.syntactic.path`    | `Statement.metadata["path"]`  |
//!
//! A ZToken is considered grounded (`Provenance.grounded = true`) if its
//! `lexical.source_text` is non-empty, establishing that the token has a
//! verifiable span in the source artefact — the Δ-tracking predicate.

use crate::model::artifact::Artifact;
use crate::model::formula::Formula;
use crate::model::statement::{Provenance, Statement, StatementKind};
use crate::model::theory::Theory;

/// Convert an `Artifact` into a `Theory` with rich provenance.
///
/// This is the canonical bridge between the artefact layer and the theory
/// layer.  It should be preferred over constructing a `Theory` manually from
/// raw token data.
pub fn artifact_to_theory(artifact: &Artifact) -> Theory {
    let mut theory = Theory::new();

    for token in &artifact.ztokens {
        let text = token.lexical.normalized_text.clone();

        // -- Provenance --
        let mut provenance = Provenance::default();

        // Source anchor: sha256 of the source file (Δ-tracking).
        provenance.source_ids.insert(artifact.source.sha256.clone());

        // Span anchor: "<start_byte>:<end_byte>" for exact byte-level grounding.
        let span = &token.lexical.span;
        if span.start_byte < span.end_byte {
            provenance
                .anchors
                .insert(format!("{}:{}", span.start_byte, span.end_byte));
        }

        // ZToken id anchor: the compiled id is itself a stable pointer into the artefact.
        provenance.anchors.insert(token.id.clone());

        // A token is grounded if it has a non-empty source_text (the exact
        // bytes from the source file were verified during compilation).
        provenance.grounded = !token.lexical.source_text.is_empty();

        // -- Metadata --
        let mut metadata = std::collections::BTreeMap::new();
        metadata.insert("path".to_string(), token.syntactic.path.clone());
        if !token.logical.relation_ids.is_empty() {
            metadata.insert(
                "relation_ids".to_string(),
                token.logical.relation_ids.join(","),
            );
        }
        if let Some(parent_id) = &token.syntactic.parent_id {
            metadata.insert("parent_id".to_string(), parent_id.clone());
        }
        metadata.insert("depth".to_string(), token.syntactic.depth.to_string());

        // -- Kind --
        let kind = StatementKind::Atomic;

        theory.insert(Statement {
            id: token.id.clone(),
            text,
            kind,
            domain: token.syntactic.node_type.clone(),
            provenance,
            metadata,
        });
    }

    theory
}

/// Convert an `Artifact` to a `Theory` and also parse `Formula`s for each statement.
///
/// This is the richer variant: each `Statement` gets a `formula` derived by
/// parsing its normalized text.  Used by the `FormulaCoherenceChecker` and
/// the `FormulaInferenceEngine`.
pub fn artifact_to_theory_with_formulas(artifact: &Artifact) -> Vec<(Statement, Option<Formula>)> {
    artifact_to_theory(artifact)
        .statements
        .into_values()
        .map(|stmt| {
            let formula = Formula::parse(&stmt.text);
            (stmt, formula)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn minimal_artifact() -> Artifact {
        use crate::model::{
            artifact::{Artifact, CompilerInfo, DocumentInfo, SourceInfo},
            ztoken::{
                LexicalDimension, LogicalDimension, SemanticDimension, SourceSpan,
                SyntacticDimension, ZToken,
            },
        };
        use std::collections::BTreeMap;

        let token = ZToken {
            id: "z1".into(),
            lexical: LexicalDimension {
                source_text: "Hello".into(),
                plain_text: "Hello".into(),
                normalized_text: "Hello".into(),
                span: SourceSpan {
                    start_byte: 0,
                    end_byte: 5,
                    start_line: 1,
                    end_line: 1,
                },
            },
            syntactic: SyntacticDimension {
                node_type: "paragraph".into(),
                parent_id: None,
                depth: 0,
                sibling_index: 0,
                path: "0".into(),
            },
            semantic: SemanticDimension {
                gloss: "Hello".into(),
                concepts: vec![],
                confidence: None,
            },
            logical: LogicalDimension {
                relation_ids: vec![],
            },
            extensions: BTreeMap::new(),
        };

        Artifact {
            format: "stf-sir.zmd".into(),
            version: 1,
            source: SourceInfo {
                path: None,
                media_type: "text/markdown".into(),
                encoding: "utf-8".into(),
                length_bytes: 5,
                sha256: "sha256:abc123".into(),
            },
            compiler: CompilerInfo {
                name: "test".into(),
                version: "0.0.0".into(),
                config_hash: "sha256:cfg".into(),
                profile: None,
            },
            document: DocumentInfo {
                language: "und".into(),
                token_count: 1,
                relation_count: 0,
                root_token_ids: vec!["z1".into()],
            },
            ztokens: vec![token],
            relations: vec![],
            diagnostics: vec![],
            extensions: BTreeMap::new(),
        }
    }

    #[test]
    fn converts_ztoken_to_grounded_statement() {
        let artifact = minimal_artifact();
        let theory = artifact_to_theory(&artifact);

        let stmt = theory.statements.get("z1").expect("z1 must be present");
        assert!(stmt.provenance.grounded, "non-empty source_text must imply grounded");
        assert!(
            stmt.provenance.source_ids.contains("sha256:abc123"),
            "source sha256 must be in source_ids"
        );
        assert!(
            stmt.provenance.anchors.contains("z1"),
            "ztoken id must be in anchors"
        );
        assert!(
            stmt.provenance.anchors.contains("0:5"),
            "span must be in anchors"
        );
    }

    #[test]
    fn metadata_carries_path_and_depth() {
        let artifact = minimal_artifact();
        let theory = artifact_to_theory(&artifact);
        let stmt = theory.statements.get("z1").unwrap();
        assert_eq!(stmt.metadata.get("path").map(String::as_str), Some("0"));
        assert_eq!(stmt.metadata.get("depth").map(String::as_str), Some("0"));
    }
}
