//! Integration tests for the plugin system (EPIC-208).
//!
//! Requires: `pub mod plugin;` in `src/lib.rs` (added as part of EPIC-208 wiring).

use stf_sir::plugin::{ExternalEnricher, NamespaceRegistry, Plugin, PluginError};
use stf_sir::model::Artifact;

// ── Minimal test fixtures ────────────────────────────────────────────────────

fn empty_artifact() -> Artifact {
    stf_sir::compiler::compile_markdown("# Test\n\nParagraph.", None)
        .expect("compile_markdown should not fail on valid input")
}

// ── Stub plugin ──────────────────────────────────────────────────────────────

struct StubPlugin {
    name: &'static str,
    namespace: &'static str,
}

impl Plugin for StubPlugin {
    fn name(&self) -> &str {
        self.name
    }
    fn namespace(&self) -> &str {
        self.namespace
    }
    fn version(&self) -> &str {
        "0.1.0"
    }
    fn enrich(&self, _artifact: &mut Artifact) -> Result<(), PluginError> {
        Ok(())
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

/// Reserved namespaces must be rejected at registration time.
#[test]
fn namespace_registry_rejects_reserved_namespace() {
    let mut registry = NamespaceRegistry::new();

    for reserved in &["stf-sir", "stf", "sir"] {
        let plugin = StubPlugin {
            name: "test-plugin",
            namespace: reserved,
        };
        let result = registry.register(&plugin);
        assert!(
            result.is_err(),
            "expected error for reserved namespace '{reserved}'"
        );
        match result.unwrap_err() {
            PluginError::NamespaceCollision { namespace } => {
                assert_eq!(&namespace, reserved);
            }
            other => panic!("expected NamespaceCollision, got {other:?}"),
        }
    }
}

/// Custom (non-reserved) namespaces should be registered successfully.
#[test]
fn namespace_registry_allows_custom_namespace() {
    let mut registry = NamespaceRegistry::new();

    let plugin = StubPlugin {
        name: "concept-extractor",
        namespace: "acme.concept-extractor",
    };
    let result = registry.register(&plugin);
    assert!(result.is_ok(), "expected Ok, got {result:?}");

    let registered = registry.get("acme.concept-extractor");
    assert!(registered.is_some(), "plugin should appear in registry after registration");
    let reg = registered.unwrap();
    assert_eq!(reg.name, "concept-extractor");
    assert_eq!(reg.namespace, "acme.concept-extractor");
}

/// Duplicate namespace registrations must be rejected.
#[test]
fn namespace_registry_rejects_duplicate_namespace() {
    let mut registry = NamespaceRegistry::new();
    let p1 = StubPlugin { name: "first", namespace: "org.extractor" };
    let p2 = StubPlugin { name: "second", namespace: "org.extractor" };

    assert!(registry.register(&p1).is_ok());
    let result = registry.register(&p2);
    assert!(result.is_err(), "expected error on duplicate namespace");
}

/// `build_request` must include every ZToken in the artifact.
#[test]
fn external_enricher_build_request_includes_all_tokens() {
    let artifact = empty_artifact();
    let request = ExternalEnricher::build_request(&artifact);

    assert_eq!(request.protocol, "stf-sir-enricher-v1");
    assert_eq!(request.artifact_id, artifact.source.sha256);
    assert_eq!(
        request.tokens.len(),
        artifact.ztokens.len(),
        "request must include all {} ztokens",
        artifact.ztokens.len()
    );

    // Token IDs must match
    for (req_token, ztoken) in request.tokens.iter().zip(artifact.ztokens.iter()) {
        assert_eq!(req_token.id, ztoken.id);
        assert_eq!(req_token.gloss, ztoken.semantic.gloss);
        assert_eq!(req_token.node_type, ztoken.syntactic.node_type);
    }
}

/// `apply_response` must merge enrichment data into `token.extensions[namespace]`.
#[test]
fn external_enricher_apply_response_merges_extensions() {
    use stf_sir::plugin::external::{EnricherResponse, TokenEnrichment};

    let mut artifact = empty_artifact();
    assert!(!artifact.ztokens.is_empty(), "artifact must have at least one token");

    let first_token_id = artifact.ztokens[0].id.clone();

    let response = EnricherResponse {
        protocol: "stf-sir-enricher-v1".to_string(),
        enrichments: vec![TokenEnrichment {
            token_id: first_token_id.clone(),
            extensions: serde_json::json!({
                "concepts": ["System", "Design"]
            }),
        }],
    };

    ExternalEnricher::apply_response(&mut artifact, &response, "acme.concept-extractor");

    let token = artifact
        .ztokens
        .iter()
        .find(|t| t.id == first_token_id)
        .expect("token must exist");

    assert!(
        token.extensions.contains_key("acme.concept-extractor"),
        "extension namespace key must be set after apply_response"
    );
}
