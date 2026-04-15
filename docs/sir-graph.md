# SirGraph

SirGraph is the first graph-oriented projection of STF-SIR. It defines a deterministic, typed graph view over the existing in-memory `Artifact` model without changing compilation, validation, or `.zmd` serialization.

## 1. Scope

SirGraph v0/v1.1-minimum is intentionally narrow:

- every ztoken is materialized as a graph node,
- every relation is materialized as a graph edge,
- node and edge ids are preserved exactly,
- deterministic indexes for outgoing and incoming adjacency are built,
- no inference, auxiliary nodes, or alternative graph serialization are introduced.

The graph is therefore a computed view, not a second persisted representation.

## 2. Layering

The relationship between STF-SIR layers is:

1. `.zmd` is the canonical serialized artifact.
2. `Artifact` is the canonical typed Rust model.
3. `SirGraph` is a deterministic projection computed from `Artifact`.

The module boundary is deliberate. v1 does not yet define a graph-specific interchange format, so the graph must remain derivable from the artifact rather than competing with it.

## 3. Data Model

The current SirGraph module defines:

| Type | Purpose |
| --- | --- |
| `SirNodeKind` | Closed node-kind enum for the current graph layer |
| `SirNode` | Addressable node record with `id` and `kind` |
| `SirEdge` | Addressable edge record with `id`, `edge_type`, `category`, `source`, `target`, and `stage` |
| `SirGraph` | Deterministic graph container with nodes, edges, and indexes |

### 3.1 Node Semantics

The only supported node kind at this stage is the ztoken-backed node:

- `SirNodeKind::ZToken { node_type }`

This means SirGraph is currently a graph view over compiled units, not over a richer ontology of entities, propositions, or inferred concepts.

### 3.2 Edge Semantics

Each `SirEdge` is copied directly from an artifact relation:

- `id` preserves the relation id,
- `edge_type` preserves `relation.type`,
- `category` preserves relation classification,
- `source` and `target` preserve relation endpoints,
- `stage` preserves relation provenance.

SirGraph does not reinterpret relation meaning. If the artifact says a relation is `category: structural` and `stage: logical`, the graph preserves exactly that distinction.

## 4. Construction Rules

The conversion `Artifact::as_sir_graph()` is normative for the current graph layer and applies the following rules:

1. One node is emitted for each ztoken in artifact order.
2. One edge is emitted for each relation in artifact order.
3. `node_by_id` is built as a deterministic lookup from node id to node index.
4. `outgoing` is built as a deterministic lookup from node id to edge indexes where the node is the source.
5. `incoming` is built as a deterministic lookup from node id to edge indexes where the node is the target.

No additional nodes or edges are synthesized during construction.

## 5. Query Surface

The initial query APIs are intentionally small:

| Method | Behavior |
| --- | --- |
| `node(id)` | Returns a node by id if present |
| `outgoing(id)` | Returns outgoing edges in deterministic edge order |
| `incoming(id)` | Returns incoming edges in deterministic edge order |
| `neighbors(id)` | Returns adjacent nodes reachable through outgoing or incoming edges, without duplicates |
| `edges_by_category(category)` | Returns all edges of the given relation category |

These methods are query conveniences over the artifact-derived graph. They do not add reasoning or transitive closure.

## 6. ZToken and Graph Nodes

A ztoken is a compiled unit. In SirGraph v1, each ztoken is materialized as a node because the graph must remain directly traversable over the stable artifact.

That does not make a ztoken identical to an arbitrary graph node in the general STF-SIR theory. Future graph layers may introduce auxiliary nodes, derived nodes, or domain-specific overlays while keeping ztokens as the stable compiled substrate.

In the current scope:

- ztoken -> required graph node
- relation -> required graph edge
- no auxiliary nodes
- no inferred edges

## 7. Determinism

SirGraph inherits determinism from the underlying artifact:

- node order follows artifact ztoken order,
- edge order follows artifact relation order,
- indexes use deterministic maps,
- repeated projection of the same artifact yields an identical graph.

This makes SirGraph suitable as the first stable graph layer over STF-SIR v1.

## 8. Current Limitations

SirGraph does not yet provide:

- **[aspiracional]** sentence-level or entity-level materialization,
- **[aspiracional]** inference or semantic closure,
- **[aspiracional]** external knowledge graph integration,
- **[aspiracional]** graph serialization beyond `.zmd`,
- **[aspiracional]** speculative node or edge kinds not already present in the artifact.

Those are intentionally deferred so the first graph layer can remain minimal, deterministic, and stable.
