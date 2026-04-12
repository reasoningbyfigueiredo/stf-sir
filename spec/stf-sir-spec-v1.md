# STF-SIR Operational Specification v1

## 1. Overview

This document defines the minimal operational contract for STF-SIR v1. It specifies how an implementation compiles a source document into a `.zmd` artifact containing ztokens and typed relations. The scope of v1 is intentionally narrow:

- it MUST be implementable with standard parsing and serialization tooling,
- it MUST preserve the four STF-SIR dimensions in explicit fields,
- it MUST be deterministic under a fixed compiler configuration,
- it SHOULD allow richer semantic and logical analyses to be added later without breaking existing artifacts.

The keywords `MUST`, `SHOULD`, `MAY`, and `MUST NOT` are normative.

## 2. Core Concepts

### 2.1 ZToken Structure

A ztoken is the smallest addressable compiled unit in STF-SIR. Every ztoken MUST have a stable identifier and MUST expose the four STF-SIR dimensions:

\[
z = \langle L, S, \Sigma, \Phi \rangle
\]

The v1 ztoken data model is:

| Field | Required | Description |
| --- | --- | --- |
| `id` | Yes | Artifact-local identifier, for example `z1`, `z2`, `z3` |
| `L` | Yes | Lexical dimension |
| `S` | Yes | Syntactic dimension |
| `Σ` | Yes | Semantic dimension |
| `Φ` | Yes | Logical dimension |
| `extensions` | No | Namespaced implementation-specific data |

A ztoken is the primary compiled unit, not a synonym for an arbitrary graph node. In STF-SIR, a ztoken MAY be materialized as an internal node anchor in the relation graph, while future profiles MAY also introduce auxiliary non-ztoken nodes. In v1, the only required internal identifiers are ztoken ids.

### 2.2 Dimensions

| Dimension | Required fields | Purpose |
| --- | --- | --- |
| `L` | `source_text`, `plain_text`, `normalized_text`, `span` | Preserve lexical evidence and source anchoring |
| `S` | `node_type`, `parent_id`, `depth`, `sibling_index`, `path` | Preserve syntactic placement in the document structure |
| `Σ` | `gloss` | Preserve a minimum semantic interpretation |
| `Φ` | `relation_ids` | Preserve references to categorized relations involving the ztoken |

### 2.3 Source Document Scope

STF-SIR v1 defines normative behavior for `text/markdown` inputs. Other media types MAY be supported, but they are out of scope for conformance unless separately specified.

### 2.4 Granularity

Granularity is the semantic size of the unit emitted as a ztoken. STF-SIR is designed to support at least the following granularities:

- `block`
- `sentence`
- `entity`

STF-SIR v1 conformance requires `block` granularity only. Sentence-level and entity-level emission MAY be introduced by future profiles or extensions without changing the core ztoken model.

## 3. Data Structures

### 3.1 Top-Level Artifact

A `.zmd` artifact MUST be a single document with the following top-level fields in this order:

| Field | Required | Description |
| --- | --- | --- |
| `format` | Yes | Constant string `stf-sir.zmd` |
| `version` | Yes | Constant integer `1` |
| `source` | Yes | Information about the compiled source document |
| `compiler` | Yes | Information required for reproducibility |
| `document` | Yes | Aggregate document metadata |
| `ztokens` | Yes | Ordered list of ztokens |
| `relations` | Yes | Ordered list of logical relations |
| `diagnostics` | Yes | Ordered list of warnings and errors emitted during compilation |
| `extensions` | No | Namespaced artifact-level extensions |

### 3.2 `source` Object

| Field | Required | Type | Description |
| --- | --- | --- | --- |
| `path` | No | string | Source path as supplied to the compiler |
| `media_type` | Yes | string | For v1 conformance, `text/markdown` |
| `encoding` | Yes | string | MUST be `utf-8` |
| `length_bytes` | Yes | integer | Total length of the original source in bytes |
| `sha256` | Yes | string | SHA-256 of the original source bytes |

### 3.3 `compiler` Object

| Field | Required | Type | Description |
| --- | --- | --- | --- |
| `name` | Yes | string | Compiler implementation name |
| `version` | Yes | string | Compiler implementation version |
| `config_hash` | Yes | string | Hash of the effective compiler configuration |
| `profile` | No | string | Optional build or ruleset profile name |

### 3.4 `document` Object

| Field | Required | Type | Description |
| --- | --- | --- | --- |
| `language` | Yes | string | BCP-47 tag or `und` if unknown |
| `token_count` | Yes | integer | Number of emitted ztokens |
| `relation_count` | Yes | integer | Number of emitted relations |
| `root_token_ids` | Yes | array of string | Token ids with `S.parent_id = null` |

### 3.5 ZToken Object

| Field | Required | Type | Description |
| --- | --- | --- | --- |
| `id` | Yes | string | Unique artifact-local token id |
| `L` | Yes | object | Lexical dimension |
| `S` | Yes | object | Syntactic dimension |
| `Σ` | Yes | object | Semantic dimension |
| `Φ` | Yes | object | Logical dimension |
| `extensions` | No | object | Namespaced token-level extensions |

### 3.6 `L` Object

| Field | Required | Type | Description |
| --- | --- | --- | --- |
| `source_text` | Yes | string | Exact source slice corresponding to the token span |
| `plain_text` | Yes | string | Parser-extracted textual content without source markup wrappers |
| `normalized_text` | Yes | string | Deterministic normalization of `plain_text` |
| `span` | Yes | object | Source coordinates |

The `span` object MUST contain:

| Field | Required | Type | Description |
| --- | --- | --- | --- |
| `start_byte` | Yes | integer | Inclusive byte offset in the source |
| `end_byte` | Yes | integer | Exclusive byte offset in the source |
| `start_line` | Yes | integer | 1-based line number |
| `end_line` | Yes | integer | 1-based line number |

### 3.7 `S` Object

| Field | Required | Type | Description |
| --- | --- | --- | --- |
| `node_type` | Yes | string | Normalized Markdown AST node type |
| `parent_id` | Yes | string or null | Parent ztoken id, or `null` for root tokens |
| `depth` | Yes | integer | Tree depth where root tokens have depth `0` |
| `sibling_index` | Yes | integer | Zero-based position among siblings |
| `path` | Yes | string | Stable tree path such as `0`, `1/0`, `1/0/2` |

### 3.8 `Σ` Object

| Field | Required | Type | Description |
| --- | --- | --- | --- |
| `gloss` | Yes | string | Minimum semantic interpretation of the token |
| `concepts` | No | array | Optional normalized concept assignments |
| `confidence` | No | number | Optional score in `[0,1]` when a semantic model emits one |

### 3.9 `Φ` Object

| Field | Required | Type | Description |
| --- | --- | --- | --- |
| `relation_ids` | Yes | array of string | All relation ids incident to the token |

The `Φ` object is the ztoken-local view of the global relation set. Every referenced relation MUST declare a `category` from the closed v1 set `{structural, logical, semantic-link}`.

### 3.10 Relation Object

| Field | Required | Type | Description |
| --- | --- | --- | --- |
| `id` | Yes | string | Unique artifact-local relation id |
| `type` | Yes | string | Relation type |
| `category` | Yes | string | One of `structural`, `logical`, `semantic-link` |
| `source` | Yes | string | Source ztoken id |
| `target` | Yes | string | Target ztoken id or external reference |
| `stage` | Yes | string | Pipeline stage that emitted the relation. One of `lexical`, `syntactic`, `semantic`, `logical` |
| `attributes` | No | object | Additional deterministic relation metadata |
| `extensions` | No | object | Namespaced relation-level extensions |

**Normative note on `stage` vs `category`.** These two fields are orthogonal.
`category` is a *classification* of the relation (`structural`, `logical`,
`semantic-link`). `stage` is a *provenance* coordinate — it records which
pipeline stage emitted the relation and MUST be one of the four closed
values above. It is therefore consistent and expected for a relation to
have `category: structural` and `stage: logical`, for example when the
logical stage derives a structural ordering relation such as `precedes`
from the syntactic tree. Future enrichers MAY emit relations with
`category: structural` and `stage: semantic` when reclassifying existing
structure; this is not a conformance violation.

A corresponding enum is enforced by the reference validator
(`VAL_18_RELATION_STAGE`) and by the JSON Schema.

### 3.11 Diagnostic Object

| Field | Required | Type | Description |
| --- | --- | --- | --- |
| `code` | Yes | string | Stable diagnostic code |
| `severity` | Yes | string | One of `info`, `warning`, `error` |
| `message` | Yes | string | Human-readable diagnostic text |
| `token_id` | No | string | Related token id if applicable |
| `stage` | Yes | string | Emitting stage |

## 4. `.zmd` File Format (v1)

The `.zmd` file format v1 is a UTF-8 YAML 1.2 document with the following constraints:

| Rule | Requirement |
| --- | --- |
| Encoding | MUST be UTF-8 |
| Root value | MUST be a mapping object |
| Line endings | SHOULD be LF |
| Indentation | SHOULD use two spaces |
| Duplicate keys | MUST NOT occur |
| Unknown fields | MAY occur only inside `extensions` |

The canonical field order for top-level objects, ztokens, and relations MUST follow the order defined in this specification. Producers SHOULD preserve this order to simplify diffing and golden tests.

## 5. Compiler Pipeline

The STF-SIR compiler pipeline has four required stages. An implementation MAY use additional internal stages, but conformance is defined against the externally visible behavior below.

### 5.1 Lexical Stage

The lexical stage MUST:

1. Read the source as UTF-8 bytes.
2. Compute `source.sha256` from the original bytes.
3. Build a line index for byte-to-line conversion.
4. Extract exact source spans for emitted Markdown AST nodes.

The lexical stage MUST populate `L.source_text` and `L.span` for every ztoken.

### 5.2 Syntactic Stage

The syntactic stage MUST:

1. Parse the source using a deterministic CommonMark-compatible parser.
2. Traverse the Markdown AST in preorder.
3. Emit ztokens for block-level nodes at minimum.
4. Assign `S.node_type`, `S.parent_id`, `S.depth`, `S.sibling_index`, and `S.path`.

For v1, block granularity is the only required emission profile.

For v1, the minimum conforming node types are:

- `heading`
- `paragraph`
- `list`
- `list_item`
- `blockquote`
- `code_block`
- `table` when supported by the parser

If a parser does not support a node type listed above, the compiler MUST emit a diagnostic and MAY fall back to a parent block token that preserves the source span.

### 5.3 Semantic Stage

The semantic stage MUST emit `Σ.gloss` for every ztoken.

The minimum conforming semantic behavior is:

| Input condition | Required `Σ.gloss` behavior |
| --- | --- |
| `plain_text` is non-empty | `gloss` MUST equal `normalized_text` |
| `plain_text` is empty | `gloss` MUST be the empty string |

This rule makes the semantic stage implementable without an ontology or machine-learned model. Richer semantic enrichment MAY add `concepts` or `confidence`, but it MUST NOT remove or rewrite existing mandatory fields in a way that breaks determinism.

### 5.4 Logical Stage

The logical stage MUST emit deterministic typed relations.

Every relation emitted in v1 MUST declare one of the following categories:

| Category | Meaning |
| --- | --- |
| `structural` | Relations induced directly by document structure or ordering |
| `logical` | Relations expressing dependency, support, contradiction, or other inference-oriented linkage |
| `semantic-link` | Relations connecting a ztoken to semantically associated units such as entities, concepts, or external knowledge references |

The minimum conforming relation set is:

| Relation type | Category | Required behavior |
| --- | --- | --- |
| `contains` | `structural` | MUST be emitted from a parent token to each child token |
| `precedes` | `structural` | MUST be emitted between consecutive sibling tokens |

The logical stage MAY emit additional relation types such as `supports` under `logical` or `refers_to` under `semantic-link`, provided they are deterministic and documented by the compiler implementation.

## 6. Minimal Viable ZToken (MVP Definition)

A compiler conforms to STF-SIR v1 if it can emit the following minimum ztoken shape for every block-level Markdown node:

```yaml
id: z1
L:
  source_text: "# Example"
  plain_text: "Example"
  normalized_text: "Example"
  span:
    start_byte: 0
    end_byte: 9
    start_line: 1
    end_line: 1
S:
  node_type: heading
  parent_id: null
  depth: 0
  sibling_index: 0
  path: "0"
Σ:
  gloss: "Example"
Φ:
  relation_ids: ["r1"]
```

The MVP intentionally avoids mandatory ontology bindings, embeddings, or probabilistic semantic graphs. Those may be layered on later as extensions.

## 7. Serialization Rules

Serialization MUST follow these rules:

| Rule | Requirement |
| --- | --- |
| Token order | `ztokens` MUST be serialized in preorder traversal order |
| Relation order | `relations` MUST be serialized in creation order, with `contains` before `precedes` for the same source token |
| Identifier format | Token ids SHOULD be `z1`, `z2`, `z3`, and relation ids SHOULD be `r1`, `r2`, `r3` |
| String normalization | `normalized_text` MUST be produced by Unicode NFKC followed by whitespace trimming and collapsing consecutive whitespace to a single ASCII space |
| Null handling | `null` MAY be used only where explicitly allowed |
| Empty collections | Empty arrays MUST be serialized as `[]`; empty mappings SHOULD be serialized as `{}` |

Writers SHOULD avoid volatile fields such as timestamps in canonical artifacts. If a producer records runtime metadata, it SHOULD do so outside the canonical artifact or under a namespaced extension that is excluded from reproducibility checks.

## 8. Example (`.md` -> `.zmd`)

### 8.1 Input

```markdown
# AI is transforming software development

Semantic tokenization preserves meaning across structure.
```

### 8.2 Output

```yaml
format: stf-sir.zmd
version: 1
source:
  path: examples/sample.md
  media_type: text/markdown
  encoding: utf-8
  length_bytes: 101
  sha256: "sha256:REPLACE_WITH_SOURCE_HASH"
compiler:
  name: stf-sir-ref
  version: 1.0.0
  config_hash: "sha256:REPLACE_WITH_CONFIG_HASH"
document:
  language: en
  token_count: 2
  relation_count: 1
  root_token_ids: [z1, z2]
ztokens:
  - id: z1
    L:
      source_text: "# AI is transforming software development"
      plain_text: "AI is transforming software development"
      normalized_text: "AI is transforming software development"
      span:
        start_byte: 0
        end_byte: 41
        start_line: 1
        end_line: 1
    S:
      node_type: heading
      parent_id: null
      depth: 0
      sibling_index: 0
      path: "0"
    Σ:
      gloss: "AI is transforming software development"
    Φ:
      relation_ids: [r1]
  - id: z2
    L:
      source_text: "Semantic tokenization preserves meaning across structure."
      plain_text: "Semantic tokenization preserves meaning across structure."
      normalized_text: "Semantic tokenization preserves meaning across structure."
      span:
        start_byte: 43
        end_byte: 100
        start_line: 3
        end_line: 3
    S:
      node_type: paragraph
      parent_id: null
      depth: 0
      sibling_index: 1
      path: "1"
    Σ:
      gloss: "Semantic tokenization preserves meaning across structure."
    Φ:
      relation_ids: [r1]
relations:
  - id: r1
    type: precedes
    category: structural
    source: z1
    target: z2
    stage: logical
diagnostics: []
extensions: {}
```

This example demonstrates the minimum viable behavior. A richer compiler MAY also emit section-scoping relations, concept assignments, or semantic links, but those are not required for v1 conformance.

## 9. Validation Rules

A `.zmd` artifact is valid if and only if all of the following hold:

1. `format` is `stf-sir.zmd`.
2. `version` is `1`.
3. `source.media_type` is `text/markdown`.
4. `source.encoding` is `utf-8`.
5. All ztoken ids are unique.
6. All relation ids are unique.
7. `document.token_count` equals the number of serialized ztokens.
8. `document.relation_count` equals the number of serialized relations.
9. Every `S.parent_id` is either `null` or references an existing ztoken.
10. Every relation `source` references an existing ztoken.
11. Every relation `target` references an existing ztoken unless the relation type explicitly permits external targets.
12. Every relation `category` is one of `structural`, `logical`, or `semantic-link`.
13. Every `Φ.relation_ids` entry references an existing relation.
14. Every byte span satisfies `0 <= start_byte < end_byte <= source.length_bytes`.
15. Every line span satisfies `1 <= start_line <= end_line`.
16. `L.source_text` MUST equal the exact source slice denoted by `L.span`.
17. `Σ.gloss` MUST be present even if empty.

Validators SHOULD reject artifacts that contain required-field omissions, duplicate identifiers, invalid spans, or undeclared top-level fields.

## 10. Error Handling

Compilers MUST emit diagnostics deterministically. The following baseline behavior is required:

| Condition | Required behavior |
| --- | --- |
| Invalid UTF-8 source | Compilation MUST fail and emit an `error` diagnostic |
| Parser failure | Compilation MUST fail and emit an `error` diagnostic |
| Unsupported Markdown construct | Compilation SHOULD continue when a parent span can still be emitted; a `warning` diagnostic MUST be emitted |
| Semantic enrichment unavailable | Compilation MUST continue using MVP semantic fallback; an `info` or `warning` diagnostic MAY be emitted |
| Relation generation failure for optional relation types | Compilation MAY continue if required `contains` and `precedes` relations are still emitted |
| Validation failure after serialization | Artifact MUST be rejected as non-conforming |

Recommended diagnostic codes:

- `SRC_UTF8_INVALID`
- `SYN_PARSE_FAILED`
- `SYN_NODE_UNSUPPORTED`
- `SEM_FALLBACK_APPLIED`
- `LOG_RELATION_SKIPPED`
- `VAL_SCHEMA_FAILED`

## 11. Determinism and Reproducibility

An STF-SIR v1 compiler is deterministic if repeated compilation of the same source bytes with the same effective configuration yields byte-identical `.zmd` output.

To satisfy this requirement, implementations MUST:

1. Use a fixed parser and parser configuration.
2. Use deterministic traversal order.
3. Use deterministic identifier assignment.
4. Use deterministic text normalization.
5. Use stable field ordering during serialization.
6. Exclude runtime-volatile metadata from canonical artifacts.
7. Record the effective configuration in `compiler.config_hash`.

If an implementation adds model-based semantic enrichment, it MUST either disable nondeterministic decoding or mark the artifact as non-canonical and outside strict reproducibility guarantees.

## 12. Extensibility Model

STF-SIR v1 is intentionally minimal. Extensions MUST follow these rules:

| Rule | Requirement |
| --- | --- |
| Namespacing | Extension keys MUST be namespaced, for example `org.example.sentiment` |
| Placement | Extensions MAY appear only under `extensions` fields defined by this spec |
| Compatibility | Consumers MUST ignore unknown extensions without failing validation |
| Non-destructive evolution | Extensions MUST NOT redefine the meaning of required core fields |
| Promotion path | Widely useful extensions SHOULD be candidates for future core-spec adoption |

Recommended extension targets include:

- ontology bindings,
- embeddings or vector references,
- confidence calibration metadata,
- external knowledge graph identifiers,
- domain-specific logical relation types,
- future sentence-level or entity-level granularity profiles.

## 13. Suggested Folder Structure

The following repository structure is recommended for the next engineering phase:

```text
/docs
  stf-sir-article.md
/spec
  stf-sir-spec-v1.md
/schemas
  zmd-v1.schema.json
/examples
  sample.md
  sample.zmd
/src
  /stf_sir
    /compiler
      lexical.py
      syntactic.py
      semantic.py
      logical.py
      serializer.py
    /model
      ztoken.py
      relation.py
      artifact.py
    /cli
      main.py
/tests
  /fixtures
  /golden
  /unit
  /integration
```

The names above are illustrative. Equivalent organization is acceptable if the separation between model, compiler stages, schema, examples, and tests is preserved.

## 14. Next Implementation Steps

The recommended sequence for implementation is:

1. Define a machine-readable schema for `.zmd` v1 and add a validator.
2. Build a deterministic Markdown parser adapter that extracts byte spans and line spans.
3. Implement preorder ztoken emission for block-level nodes.
4. Implement lexical normalization and the MVP semantic fallback for `Σ.gloss`.
5. Implement required logical relations: `contains` and `precedes`.
6. Add golden tests that compare expected `.zmd` output for representative Markdown fixtures.
7. Add a command-line interface such as `stf-sir compile input.md -o output.zmd`.
8. Add optional semantic enrichment only after the deterministic MVP pipeline is stable.

This sequence keeps the project aligned with the STF-SIR design principles: formal correctness first, minimal viable execution, and extensibility by design.
