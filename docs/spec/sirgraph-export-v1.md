---
id: SIRGRAPH-EXPORT-V1
version: 1.0.0-alpha
status: draft
created: 2026-04-14
updated: 2026-04-14
normative: true
tags:
  - sirgraph
  - export
  - graph
  - v2
---

# SirGraph Export Sub-Format v1

This document specifies the optional `sirgraph` section of the ZMD v2 artifact format.
The export section serializes the SirGraph in a compact adjacency-list JSON encoding,
enabling external graph tools to consume STF-SIR artifacts without a custom reader.

---

## 1. Overview

The `sirgraph` section is an **optional** top-level property of a ZMD v2 artifact.
Its absence does not affect artifact validity. When present, it MUST conform to this spec.

**Format identifier:** `stf-sir-sirgraph-v1`

**Design goals:**
1. Minimal — encode only the structural graph; full ZToken data lives in `ztokens`.
2. Deterministic — same SirGraph always produces byte-for-byte identical export.
3. Interoperable — importable by standard graph tools (NetworkX, Gephi, Cytoscape) with
   a thin adapter.
4. Round-trip safe — `ztokens` + `sirgraph` together fully reconstruct the artifact.

---

## 2. Format Structure

```json
{
  "sirgraph": {
    "format": "stf-sir-sirgraph-v1",
    "nodes": [
      { "id": "z1", "node_type": "paragraph" },
      { "id": "z2", "node_type": "heading" }
    ],
    "edges": [
      {
        "id": "r1",
        "type": "contains",
        "source": "z1",
        "target": "z2",
        "category": "structural"
      }
    ]
  }
}
```

### 2.1 Top-level fields

| Field    | Type     | Required | Description |
|----------|----------|----------|-------------|
| `format` | `string` | MUST     | Always `"stf-sir-sirgraph-v1"` |
| `nodes`  | `array`  | MUST     | Array of node descriptors |
| `edges`  | `array`  | MUST     | Array of edge descriptors |

### 2.2 Node descriptor

| Field       | Type     | Required | Description |
|-------------|----------|----------|-------------|
| `id`        | `string` | MUST     | ZToken ID (matches `ztokens[].id`) |
| `node_type` | `string` | MUST     | Syntactic node type (matches `ztokens[].S.node_type`) |

### 2.3 Edge descriptor

| Field      | Type     | Required | Description |
|------------|----------|----------|-------------|
| `id`       | `string` | MUST     | Relation ID (matches `relations[].id`) |
| `type`     | `string` | MUST     | Relation type (matches `relations[].type`) |
| `source`   | `string` | MUST     | Source ZToken ID |
| `target`   | `string` | MUST     | Target ZToken ID |
| `category` | `string` | MUST     | Relation category (`structural`, `logical`, `semantic-link`) |

---

## 3. Ordering and Determinism

To guarantee byte-for-byte identical serialization:

1. **`nodes`** are listed in the same order as `ztokens` (insertion order).
2. **`edges`** are listed in the same order as `relations` (insertion order).
3. Both arrays MUST NOT be reordered during serialization.
4. Object fields within each node and edge MUST follow the field order specified in §2.2 and §2.3.

---

## 4. Semantics

### 4.1 Node identity

Every node in `sirgraph.nodes` MUST have a corresponding ZToken in `ztokens` with the same `id`.
A node MAY NOT appear in `sirgraph.nodes` without a corresponding ZToken.

### 4.2 Edge identity

Every edge in `sirgraph.edges` MUST have a corresponding relation in `relations` with the same `id`.
The `source`, `target`, and `category` fields in the edge MUST match the values in the corresponding
relation.

### 4.3 Completeness

A valid `sirgraph` section MUST contain:
- All ZTokens that participate in at least one relation (as source or target).
- All relations.
- ZTokens not participating in any relation MAY be omitted from `nodes`; their absence does not
  affect validity.

### 4.4 Round-trip property

Given `ztokens` and `sirgraph`, a consumer can reconstruct the full SirGraph:
- Nodes: merge `sirgraph.nodes` with `ztokens` on `id`.
- Edges: merge `sirgraph.edges` with `relations` on `id`.

This round-trip MUST be lossless for graph topology and relation metadata.

---

## 5. Interoperability

### 5.1 NetworkX (Python)

```python
import json
import networkx as nx

with open("artifact.zmd") as f:
    artifact = json.load(f)

sg = artifact["sirgraph"]
G = nx.DiGraph()
G.add_nodes_from((n["id"], {"node_type": n["node_type"]}) for n in sg["nodes"])
G.add_edges_from(
    (e["source"], e["target"], {"id": e["id"], "type": e["type"], "category": e["category"]})
    for e in sg["edges"]
)
```

### 5.2 CSV export (for Gephi / Cytoscape)

```python
import csv

with open("nodes.csv", "w") as f:
    w = csv.DictWriter(f, fieldnames=["id", "node_type"])
    w.writeheader()
    w.writerows(sg["nodes"])

with open("edges.csv", "w") as f:
    w = csv.DictWriter(f, fieldnames=["id", "type", "source", "target", "category"])
    w.writeheader()
    w.writerows(sg["edges"])
```

---

## 6. Constraints

| ID         | Level   | Statement |
|------------|---------|-----------|
| SGE-01     | MUST    | `format` MUST be `"stf-sir-sirgraph-v1"` |
| SGE-02     | MUST    | Every node `id` MUST match a ZToken `id` in the same artifact |
| SGE-03     | MUST    | Every edge `id` MUST match a relation `id` in the same artifact |
| SGE-04     | MUST    | Edge `source` and `target` MUST each reference a node in `sirgraph.nodes` |
| SGE-05     | MUST    | Node and edge arrays MUST be in the deterministic order specified in §3 |
| SGE-06     | SHOULD  | All ZTokens SHOULD appear in `nodes` when `sirgraph` is present |
| SGE-07     | MAY     | Producers MAY add a `metadata` object to `sirgraph` for extension use |

---

## 7. Example

```json
{
  "format": "stf-sir.zmd",
  "version": 2,
  "sirgraph": {
    "format": "stf-sir-sirgraph-v1",
    "nodes": [
      { "id": "z1", "node_type": "heading" },
      { "id": "z2", "node_type": "paragraph" },
      { "id": "z3", "node_type": "paragraph" }
    ],
    "edges": [
      { "id": "r1", "type": "contains",  "source": "z1", "target": "z2", "category": "structural" },
      { "id": "r2", "type": "precedes",  "source": "z2", "target": "z3", "category": "structural" },
      { "id": "r3", "type": "supports",  "source": "z2", "target": "z3", "category": "logical" }
    ]
  }
}
```
