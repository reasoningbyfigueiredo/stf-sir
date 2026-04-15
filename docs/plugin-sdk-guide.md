# Plugin SDK Guide

**Version:** 1.0.0  
**Target audience:** Developers who want to enrich STF-SIR artifacts with custom data.  
**Goal:** Enable a developer to write and test a new enricher in under 30 minutes.

---

## Overview

STF-SIR's plugin system lets you extend compiled ZMD artifacts with custom data — without
modifying or forking the core compiler. There are two ways to write a plugin:

1. **Rust plugin** — implement the `Plugin` trait in Rust, statically linked.
2. **External enricher** — a subprocess in any language (Python, TypeScript, etc.) that
   communicates via the **External Enricher Protocol v1** (JSON-over-stdin/stdout).

Both approaches produce the same result: your data appears in `token.extensions[your-namespace]`
for every enriched token.

---

## Part 1: Writing a Rust Plugin

### Step 1 — Implement the `Plugin` trait

```rust
use stf_sir::plugin::{Plugin, PluginError};
use stf_sir::model::Artifact;

pub struct MyConceptPlugin;

impl Plugin for MyConceptPlugin {
    fn name(&self) -> &str { "my-concept-plugin" }
    fn namespace(&self) -> &str { "acme.concepts" }
    fn version(&self) -> &str { "1.0.0" }

    fn enrich(&self, artifact: &mut Artifact) -> Result<(), PluginError> {
        for token in &mut artifact.ztokens {
            // Extract concepts from the token's gloss
            let concepts: Vec<&str> = token.semantic.gloss
                .split_whitespace()
                .filter(|w| w.chars().next().map_or(false, |c| c.is_uppercase()))
                .collect();

            if !concepts.is_empty() {
                let value = serde_yaml_ng::to_value(&concepts)
                    .map_err(|e| PluginError::EnrichmentFailed {
                        plugin: self.name().to_string(),
                        message: e.to_string(),
                    })?;
                token.extensions.insert(self.namespace().to_string(), value);
            }
        }
        Ok(())
    }
}
```

**Rules:**
- Your `enrich` method MUST only write to `token.extensions[self.namespace()]`.
- Do NOT modify `token.id`, `token.lexical`, `token.syntactic`, `token.semantic`, or `token.logical`.
- Return `Err(PluginError::EnrichmentFailed { ... })` on any error; do not panic.

### Step 2 — Register the plugin

```rust
use stf_sir::plugin::NamespaceRegistry;

let mut registry = NamespaceRegistry::new();
let plugin = MyConceptPlugin;
registry.register(&plugin).expect("namespace must not be reserved or duplicate");
```

### Step 3 — Apply the plugin to an artifact

```rust
use stf_sir::compiler;

let mut artifact = compiler::compile_markdown("# Hello World\n\nParagraph.", None)?;
plugin.enrich(&mut artifact)?;

// Check the result
for token in &artifact.ztokens {
    if let Some(concepts) = token.extensions.get("acme.concepts") {
        println!("{}: {:?}", token.id, concepts);
    }
}
```

### Step 4 — Test your plugin

```rust
#[test]
fn my_plugin_adds_concepts() {
    use stf_sir::compiler;
    let mut artifact = compiler::compile_markdown("# System Design", None).unwrap();
    let plugin = MyConceptPlugin;
    plugin.enrich(&mut artifact).unwrap();

    let heading = artifact.ztokens.iter().find(|t| t.syntactic.node_type == "heading").unwrap();
    assert!(heading.extensions.contains_key("acme.concepts"));
}
```

---

## Part 2: Writing an External Enricher (Python)

### Step 1 — Create `my_enricher.py`

```python
#!/usr/bin/env python3
"""External enricher for STF-SIR (stf-sir-enricher-v1 protocol)."""
import json
import sys

PROTOCOL = "stf-sir-enricher-v1"

def enrich_token(token: dict) -> dict | None:
    """Return enrichment data or None to skip this token."""
    gloss = token.get("gloss", "")
    concepts = [w for w in gloss.split() if w[0].isupper()] if gloss else []
    if not concepts:
        return None
    return {"token_id": token["id"], "extensions": {"concepts": concepts[:5]}}

for line in sys.stdin:
    line = line.strip()
    if not line:
        continue
    request = json.loads(line)
    if request.get("protocol") != PROTOCOL:
        continue
    enrichments = [e for t in request["tokens"] if (e := enrich_token(t)) is not None]
    print(json.dumps({"protocol": PROTOCOL, "enrichments": enrichments}), flush=True)
```

### Step 2 — Test it manually

```bash
echo '{"protocol":"stf-sir-enricher-v1","artifact_id":"test","tokens":[
  {"id":"z1","gloss":"The System is a Platform","node_type":"paragraph","extensions":{}}
]}' | python3 my_enricher.py
```

Expected output:
```json
{"protocol": "stf-sir-enricher-v1", "enrichments": [{"token_id": "z1", "extensions": {"concepts": ["System", "Platform"]}}]}
```

### Step 3 — Wire it into Rust

```rust
use stf_sir::plugin::ExternalEnricher;

let enricher = ExternalEnricher::new(
    "my-enricher",
    "acme.concepts",
    "1.0.0",
    vec!["python3".to_string(), "my_enricher.py".to_string()],
);

// Build the protocol request
let request = ExternalEnricher::build_request(&artifact);
let request_json = serde_json::to_string(&request)?;

// Spawn the enricher, write request, read response (pseudo-code):
// let response_json = run_subprocess(&enricher.command, &request_json)?;
// let response: EnricherResponse = serde_json::from_str(&response_json)?;
// ExternalEnricher::apply_response(&mut artifact, &response, &enricher.namespace);
```

### Step 4 — TypeScript enricher example

```typescript
import * as readline from "readline";

const PROTOCOL = "stf-sir-enricher-v1";
const rl = readline.createInterface({ input: process.stdin });

rl.on("line", (line: string) => {
    const request = JSON.parse(line);
    if (request.protocol !== PROTOCOL) return;

    const enrichments = request.tokens
        .map((token: { id: string; gloss: string }) => {
            const concepts = token.gloss
                .split(/\s+/)
                .filter((w: string) => /^[A-Z]/.test(w));
            return concepts.length
                ? { token_id: token.id, extensions: { concepts } }
                : null;
        })
        .filter(Boolean);

    console.log(JSON.stringify({ protocol: PROTOCOL, enrichments }));
});
```

---

## Namespace Registration

Every plugin (Rust or external) must claim a unique namespace. Namespaces follow
reverse-DNS convention: `<org>.<plugin-name>`.

**Reserved namespaces** (will always reject):
- `stf-sir`
- `stf`
- `sir`

**Valid examples:**
- `acme.concept-extractor`
- `myorg.sentiment`
- `research-lab.named-entity`

The `NamespaceRegistry` enforces this at registration time:

```rust
let mut registry = NamespaceRegistry::new();

// OK
registry.register(&MyPlugin { namespace: "acme.concepts" }).unwrap();

// Err(NamespaceCollision { namespace: "stf-sir" })
registry.register(&MyPlugin { namespace: "stf-sir" }).unwrap_err();

// Err(NamespaceCollision { namespace: "acme.concepts" }) — duplicate
registry.register(&AnotherPlugin { namespace: "acme.concepts" }).unwrap_err();
```

---

## Error Handling

| Situation                          | Correct behavior                                              |
|------------------------------------|---------------------------------------------------------------|
| Enrichment logic fails             | Return `Err(PluginError::EnrichmentFailed { ... })`           |
| Writing outside declared namespace | Architecture violation — tests will catch this                |
| External enricher not found        | Caller handles `std::io::Error` from process spawn            |
| External enricher timeout          | Host returns original tokens unchanged, logs warning          |
| Non-JSON enricher output           | Host returns hard error                                       |

---

## Testing Your Plugin

### Rust plugin

```bash
# Run all plugin system tests
cargo test plugin_system

# Run your own plugin tests
cargo test my_concept_plugin
```

### External enricher

```bash
# Unit test (no STF-SIR required)
echo '{"protocol":"stf-sir-enricher-v1","artifact_id":"x","tokens":[{"id":"z1","gloss":"Hello World","node_type":"paragraph","extensions":{}}]}' \
    | python3 my_enricher.py

# Integration test (requires compiled artifact)
cargo run -- compile examples/sample.md -o /tmp/sample.zmd
# Then use the Rust integration test in tests/plugin_system.rs
```

### Checklist

- [ ] Namespace registered in `NamespaceRegistry` without error
- [ ] Plugin does not modify any field outside `token.extensions[namespace]`
- [ ] `enrich` returns `Ok(())` on valid input
- [ ] `enrich` returns `Err(...)` (not panic) on invalid input
- [ ] External enricher outputs valid JSON for every valid input line
- [ ] External enricher echoes `"protocol": "stf-sir-enricher-v1"` in every response

---

## Reference Files

| File | Description |
|------|-------------|
| `src/plugin/mod.rs` | `Plugin` trait and `PluginError` |
| `src/plugin/namespace.rs` | `NamespaceRegistry` |
| `src/plugin/external.rs` | `ExternalEnricher` host-side adapter |
| `docs/spec/external-enricher-protocol-v1.md` | Full protocol specification |
| `examples/plugins/concept-extractor-py/` | Reference Python enricher |
| `tests/plugin_system.rs` | Integration tests |
