//! Namespace registry — prevents reserved namespaces from being claimed by external plugins.

use std::collections::{BTreeMap, BTreeSet};

use super::{Plugin, PluginError};

/// Metadata about a registered plugin.
#[derive(Debug, Clone)]
pub struct RegisteredPlugin {
    pub name: String,
    pub namespace: String,
    pub version: String,
}

/// Registry that tracks claimed namespaces and enforces reservation rules.
///
/// Reserved namespaces (`stf-sir`, `stf`, `sir`) can never be claimed by an
/// external plugin. Attempting to register a plugin whose `namespace()` matches
/// a reserved name returns `PluginError::NamespaceCollision`.
pub struct NamespaceRegistry {
    reserved: BTreeSet<String>,
    registered: BTreeMap<String, RegisteredPlugin>,
}

impl NamespaceRegistry {
    /// Create a registry pre-populated with the STF-SIR reserved namespaces.
    pub fn new() -> Self {
        let mut reserved = BTreeSet::new();
        for ns in &["stf-sir", "stf", "sir"] {
            reserved.insert(ns.to_string());
        }
        Self {
            reserved,
            registered: BTreeMap::new(),
        }
    }

    /// Register a plugin, claiming its namespace.
    ///
    /// Returns `Err(NamespaceCollision)` if the namespace is reserved **or**
    /// already claimed by another plugin.
    pub fn register(&mut self, plugin: &dyn Plugin) -> Result<(), PluginError> {
        let ns = plugin.namespace().to_string();
        if self.reserved.contains(&ns) {
            return Err(PluginError::NamespaceCollision { namespace: ns });
        }
        if self.registered.contains_key(&ns) {
            return Err(PluginError::NamespaceCollision { namespace: ns });
        }
        self.registered.insert(
            ns.clone(),
            RegisteredPlugin {
                name: plugin.name().to_string(),
                namespace: ns,
                version: plugin.version().to_string(),
            },
        );
        Ok(())
    }

    /// Return `true` if `namespace` is in the reserved set.
    pub fn is_reserved(&self, namespace: &str) -> bool {
        self.reserved.contains(namespace)
    }

    /// Look up a registered plugin by namespace.
    pub fn get(&self, namespace: &str) -> Option<&RegisteredPlugin> {
        self.registered.get(namespace)
    }

    /// Return all registered plugins in deterministic (BTreeMap) order.
    pub fn all(&self) -> Vec<&RegisteredPlugin> {
        self.registered.values().collect()
    }
}

impl Default for NamespaceRegistry {
    fn default() -> Self {
        Self::new()
    }
}
