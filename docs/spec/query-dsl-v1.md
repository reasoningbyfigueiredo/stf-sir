# STF-SIR Query DSL v1 — Specification

**Version:** 1.0.0-alpha  
**Status:** Draft  
**EPIC:** EPIC-203 (Semantic Query Engine)  
**Implements:** D-203-1, D-203-5  

---

## 1. Overview

The STF-SIR Query DSL is a typed, composable expression language for querying the semantic graph produced by the STF-SIR compiler. A query operates over a `SirGraph` + `Artifact` pair and returns a `QueryResult` containing sorted, deduplicated node (ZToken) and relation IDs.

### Design goals

- **Determinism:** identical graph + identical query → identical result, every time (INV-203-1)
- **Composability:** all operators combine freely via `And`, `Or`, `Not`
- **Safety:** queries are read-only (INV-203-3); the engine never panics on any input
- **Completeness:** a filter query returns ALL matching nodes, never a subset (INV-203-2)

### Architecture

```
Query AST  →  QueryExecutor  →  QueryResult
               ↓                  ↓
           SirGraph          sorted Vec<token_id>
           Artifact          sorted Vec<relation_id>
```

---

## 2. Query Pattern Catalog

The DSL defines 10 named query patterns, each corresponding to a variant of the `Query` enum in `src/sir/query/ast.rs`.

### 2.1 `Path`

Find a shortest path between two nodes (BFS over outgoing edges).

```
Path { from: "tok-001", to: "tok-005" }
```

Returns: all node IDs on the path from `from` to `to`, or empty if unreachable.

### 2.2 `Ancestors`

Transitive closure over incoming edges from a given node.

```
Ancestors { id: "tok-007" }
```

Returns: all ancestor node IDs (sorted). The start node is not included unless a self-loop exists.

### 2.3 `Descendants`

Transitive closure over outgoing edges from a given node.

```
Descendants { id: "tok-001" }
```

Returns: all descendant node IDs (sorted).

### 2.4 `ByType`

Filter nodes by their `syntactic.node_type` field.

```
ByType { node_type: "heading" }
ByType { node_type: "paragraph" }
ByType { node_type: "code_block" }
```

Returns: all ZToken IDs whose `syntactic.node_type` equals the given string.

### 2.5 `ByCategory`

Filter relations (and their source/target nodes) by relation category.

```
ByCategory { category: "structural" }
ByCategory { category: "logical" }
ByCategory { category: "semantic-link" }
```

Returns: all node IDs that participate in a relation of the given category, plus the matching relation IDs.

### 2.6 `Subgraph`

Extract the subgraph rooted at a node, up to an optional depth limit.

```
Subgraph { root_id: "tok-001", max_depth: Some(2) }
Subgraph { root_id: "tok-001", max_depth: None }
```

Returns: all node IDs reachable from `root_id` within `max_depth` hops (inclusive). The root itself is included.

### 2.7 `DepthRange`

Select nodes by syntactic depth range (inclusive bounds).

```
DepthRange { min: 0, max: 1 }
DepthRange { min: 2, max: 4 }
```

Returns: all ZToken IDs whose `syntactic.depth` falls in `[min, max]`.

### 2.8 `RegexGloss`

Select nodes whose `semantic.gloss` contains the given pattern.

```
RegexGloss { pattern: "introduction" }
RegexGloss { pattern: "§ 5" }
```

Returns: all ZToken IDs whose `semantic.gloss` contains the pattern as a substring. Full regex support is planned for v1.1.

### 2.9 `DimensionFilter`

Select nodes by exact field value in a specific ZToken dimension.

```
DimensionFilter { dimension: Dimension::Syntactic, field: "node_type", value: "heading" }
DimensionFilter { dimension: Dimension::Lexical, field: "plain_text", value: "Introduction" }
DimensionFilter { dimension: Dimension::Semantic, field: "gloss", value: "introductory paragraph" }
DimensionFilter { dimension: Dimension::Logical, field: "relation_ids", value: "rel-001" }
```

Dimensions: `Lexical`, `Syntactic`, `Semantic`, `Logical`.

Supported fields per dimension:

| Dimension  | Fields                                    |
|------------|-------------------------------------------|
| Lexical    | `source_text`, `plain_text`, `normalized_text` |
| Syntactic  | `node_type`, `parent_id`, `depth`, `path` |
| Semantic   | `gloss`, `confidence`, `concepts`         |
| Logical    | `relation_ids`                            |

### 2.10 Combinators: `And`, `Or`, `Not`

Boolean operators for composing any two queries.

```
// Intersection: headings at depth 0
And(ByType { node_type: "heading" }, DepthRange { min: 0, max: 0 })

// Union: headings or paragraphs
Or(ByType { node_type: "heading" }, ByType { node_type: "paragraph" })

// Complement: all nodes that are not headings
Not(ByType { node_type: "heading" })
```

---

## 3. Query API

### Rust API

```rust
use stf_sir::compiler::compile_markdown;
use stf_sir::sir::query::{Query, QueryExecutor};

let artifact = compile_markdown(source, None)?;
let graph = artifact.as_sir_graph();
let executor = QueryExecutor::new(&graph, &artifact);

let result = executor.execute(&Query::ByType {
    node_type: "heading".to_string(),
});

println!("Found {} headings", result.token_count());
for id in &result.token_ids {
    println!("  - {}", id);
}
```

### Composition example

```rust
// Find headings that are also at depth 0 (top-level headings)
let q = Query::and(
    Query::ByType { node_type: "heading".to_string() },
    Query::DepthRange { min: 0, max: 0 },
);
let result = executor.execute(&q);
```

---

## 4. Query Executor

### `QueryExecutor<'a>`

```rust
pub struct QueryExecutor<'a> {
    graph: &'a SirGraph,
    artifact: &'a Artifact,
}

impl<'a> QueryExecutor<'a> {
    pub fn new(graph: &'a SirGraph, artifact: &'a Artifact) -> Self;
    pub fn execute(&self, query: &Query) -> QueryResult;
}
```

The executor is **read-only**: it borrows `SirGraph` and `Artifact` immutably and never modifies them (INV-203-3).

---

## 5. Result Format

### `QueryResult`

```rust
pub struct QueryResult {
    pub query_id: String,
    pub token_ids: Vec<String>,    // sorted, deduplicated ZToken IDs
    pub relation_ids: Vec<String>, // sorted, deduplicated Relation IDs
    pub execution_time_us: u64,    // wall-clock microseconds
}
```

Serializes to JSON:

```json
{
  "query_id": "q-0x7f...",
  "token_ids": ["tok-001", "tok-003", "tok-007"],
  "relation_ids": ["rel-002"],
  "execution_time_us": 42
}
```

### Determinism guarantee

For any query `Q` and artifact `A`:

```
execute(Q, A) == execute(Q, A)  // for any call count, any OS, any thread
```

This is guaranteed by:
1. All internal sets use `BTreeSet` (deterministic iteration order)
2. All depth maps use `BTreeMap`
3. All output vectors are sorted and deduplicated before returning

---

## 6. Worked Examples

### Example 1: All headings

```rust
Query::ByType { node_type: "heading".to_string() }
```

### Example 2: Top-level headings (depth 0)

```rust
Query::and(
    Query::ByType { node_type: "heading".to_string() },
    Query::DepthRange { min: 0, max: 0 },
)
```

### Example 3: All descendants of the first section

```rust
Query::Descendants { id: first_section_id.to_string() }
```

### Example 4: Structural relations

```rust
Query::ByCategory { category: "structural".to_string() }
```

### Example 5: Nodes with gloss containing "introduction"

```rust
Query::RegexGloss { pattern: "introduction".to_string() }
```

### Example 6: Subgraph rooted at node, max 2 hops

```rust
Query::Subgraph {
    root_id: "tok-001".to_string(),
    max_depth: Some(2),
}
```

### Example 7: Filter by syntactic path prefix (DimensionFilter)

```rust
Query::DimensionFilter {
    dimension: Dimension::Syntactic,
    field: "path".to_string(),
    value: "/document/section".to_string(),
}
```

### Example 8: All non-paragraph nodes

```rust
Query::not(Query::ByType { node_type: "paragraph".to_string() })
```

### Example 9: Headings OR code blocks

```rust
Query::or(
    Query::ByType { node_type: "heading".to_string() },
    Query::ByType { node_type: "code_block".to_string() },
)
```

### Example 10: Path between two specific nodes

```rust
Query::Path {
    from: "tok-001".to_string(),
    to: "tok-050".to_string(),
}
```

---

## 7. CLI Usage (planned — FEAT-203-3)

```bash
# Query all headings in a compiled artifact
stf-sir query artifact.zmd --query 'by_type(heading)'

# Query top-level headings (depth 0)
stf-sir query artifact.zmd --query 'and(by_type(heading), depth_range(0, 0))'

# Output as JSON
stf-sir query artifact.zmd --query 'by_type(paragraph)' --format json

# Output as YAML
stf-sir query artifact.zmd --query 'descendants(tok-001)' --format yaml

# Read query from file
stf-sir query artifact.zmd --query-file query.dsl
```

Exit codes:
- `0` — query executed successfully, results printed
- `1` — query parse error
- `2` — no results found
- `3` — invalid artifact file

---

## 8. Invariants

| ID | Invariant |
|----|-----------|
| INV-203-1 | Query determinism: execute(Q, A) == execute(Q, A) always |
| INV-203-2 | Query completeness: no matching node is silently dropped |
| INV-203-3 | Query isolation: queries are read-only |
| INV-203-4 | Result serialization is deterministic |
| INV-203-5 | Engine never panics; all errors are returned as `Err` |

---

## 9. Future work

- Full regex support in `RegexGloss` (FEAT-203-x)
- PEG grammar file for DSL string parsing (FEAT-203-1)
- CLI subcommand (FEAT-203-3)
- `SpanRange` query pattern (byte-range filter)
- Query planner and cost model
- Multi-artifact joins (EPIC-205)
