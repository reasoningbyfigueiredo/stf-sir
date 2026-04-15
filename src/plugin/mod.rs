//! Plugin system for STF-SIR extensibility (EPIC-208).
//!
//! Provides the core `Plugin` trait, `NamespaceRegistry`, and `ExternalEnricher`
//! for third-party enrichers written in any language.

pub mod external;
pub mod namespace;

pub use external::{
    EnricherRequest, EnricherResponse, EnricherToken, ExternalEnricher, TokenEnrichment,
};
pub use namespace::{NamespaceRegistry, RegisteredPlugin};

use crate::model::Artifact;

/// Core plugin trait — object-safe, Send + Sync.
///
/// Plugins enrich artifacts by adding data to `extensions[namespace]` fields.
/// A plugin MUST NOT modify any field outside its declared namespace.
pub trait Plugin: Send + Sync {
    /// Human-readable plugin name.
    fn name(&self) -> &str;

    /// Unique namespace claimed by this plugin (e.g. `"acme.concept-extractor"`).
    /// Must not be a reserved namespace (`stf-sir`, `stf`, `sir`).
    fn namespace(&self) -> &str;

    /// Plugin version string (semver recommended).
    fn version(&self) -> &str;

    /// Apply enrichment to `artifact`. MUST only write to `extensions[self.namespace()]`.
    fn enrich(&self, artifact: &mut Artifact) -> Result<(), PluginError>;
}

/// Errors from the plugin system.
#[derive(Debug, thiserror::Error)]
pub enum PluginError {
    #[error("Plugin '{plugin}' failed: {message}")]
    EnrichmentFailed { plugin: String, message: String },

    #[error("Namespace collision: '{namespace}' is reserved")]
    NamespaceCollision { namespace: String },

    #[error("External protocol error: {0}")]
    ProtocolError(String),
}
