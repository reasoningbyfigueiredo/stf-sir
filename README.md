# STF-SIR

![CI](https://github.com/reasoningbyfigueiredo/stf-sir/actions/workflows/ci.yml/badge.svg)

Deterministic semantic compilation system for document-centric AI.

> Status: STF-SIR v1.0.0 is the current stable reference baseline. Future revisions may extend semantics without breaking v1 artifacts.

## What is this?

STF-SIR, short for **Semantic Tokenization Framework - Semantic Intermediate Representation**, compiles source documents into a deterministic semantic artifact instead of treating them only as flat token streams.

The core compiled unit is the **ztoken**:

```text
z = <L, S, Σ, Φ>
```

Where:

- `L` is the lexical dimension
- `S` is the syntactic dimension
- `Σ` is the semantic dimension
- `Φ` is the logical dimension in the v1 artifact model

At the artifact level, STF-SIR produces `.zmd` files containing ztokens, typed relations, validation-ready metadata, and deterministic serialization. In the broader theory, STF-SIR is the compilation framework, SIR is the intermediate representation, SirGraph is its graph projection, and retention `ρ` is its preservation metric.

## Why it matters

Traditional tokenization is efficient for sequence models, but it does not preserve structure and meaning as first-class, addressable units. STF-SIR exists to make semantic structure explicit before retrieval, validation, graph traversal, or agent reasoning begins.

| Need | Conventional token streams | STF-SIR |
| --- | --- | --- |
| Stable unit of meaning | Reconstructed late | Explicit ztoken |
| Document structure | Mostly implicit | Preserved from AST traversal |
| Typed relations | Usually absent | Emitted deterministically |
| Validation | Ad hoc | Schema + invariant checks |
| Reproducibility | Often indirect | Byte-identical `.zmd` under fixed config |

This matters whenever we want a system to reason over documents as structured semantic objects instead of raw text windows.

## Example

The simplest path through STF-SIR is:

```text
.md -> .zmd -> SirGraph
```

### Input

[`examples/sample.md`](examples/sample.md)

```markdown
# AI is transforming software development

Semantic tokenization preserves meaning across structure.
```

### Compile and validate

```bash
cargo run -- compile examples/sample.md -o out.zmd
cargo run -- validate out.zmd
```

Validation should report:

```text
VALID: out.zmd conforms to STF-SIR v1
```

### `.zmd` excerpt

The compiled artifact is a stable YAML document. For the canonical sample, the essential shape is:

```yaml
document:
  token_count: 2
  relation_count: 1

ztokens:
- id: z1
  S:
    node_type: heading
  Σ:
    gloss: AI is transforming software development
- id: z2
  S:
    node_type: paragraph
  Σ:
    gloss: Semantic tokenization preserves meaning across structure.

relations:
- id: r1
  type: precedes
  category: structural
  source: z1
  target: z2
  stage: logical
```

### Graph view

That same artifact projects to a minimal typed graph:

```text
z1 [heading] --precedes/structural--> z2 [paragraph]
```

And can be accessed directly in Rust:

```rust
use std::fs;
use stf_sir::model::Artifact;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let yaml = fs::read_to_string("out.zmd")?;
    let artifact: Artifact = serde_yaml_ng::from_str(&yaml)?;
    let graph = artifact.as_sir_graph();

    assert!(graph.node("z1").is_some());
    assert_eq!(graph.outgoing("z1").len(), 1);
    Ok(())
}
```

## Key concepts

| Concept | Meaning | Current v1 realization |
| --- | --- | --- |
| `ZToken` | Compiled semantic unit | `z = <L, S, Σ, Φ>` with spans, AST position, gloss, and relation references |
| `SIR` | Semantic Intermediate Representation | The `.zmd` artifact containing document metadata, ztokens, and relations |
| `SirGraph` | Typed graph projection of the artifact | One node per ztoken, one edge per relation, deterministic indexes |
| `Retention` | Preservation baseline | `ρ(d) = <ρ_L, ρ_S, ρ_Σ, ρ_Φ>` over artifact completeness and consistency |

A ztoken is not defined as an arbitrary graph node. It is a compiled semantic unit that becomes a graph node in the SirGraph projection.

## Quickstart

### Requirements

- Rust `1.82` or newer

### Build

```bash
cargo build
```

### Hello world

```bash
cargo run -- compile examples/sample.md -o out.zmd
cargo run -- validate out.zmd
```

### Test and release checks

```bash
cargo test --all-features
bash scripts/release-check.sh
```

### Conformance suite

```bash
cargo test --test conformance
```

## Architecture

STF-SIR v1 follows a deterministic four-stage pipeline:

```text
Markdown
  -> lexical
  -> syntactic
  -> semantic
  -> logical
  -> Artifact (.zmd)
  -> SirGraph / retention / validation
```

The stages are intentionally minimal:

- `lexical`: extracts source text, plain text, normalized text, byte spans, and line spans
- `syntactic`: builds a CommonMark-compatible preorder structure over supported block nodes
- `semantic`: applies the MVP fallback `Σ.gloss = normalized_text`
- `logical`: emits deterministic typed relations such as `contains` and `precedes`

The current compiler supports only the v1 MVP node set:

- `heading`
- `paragraph`
- `blockquote`
- `list`
- `list_item`
- `code_block`

Determinism is part of the contract:

- ztoken ids are stable: `z1`, `z2`, ...
- relation ids are stable: `r1`, `r2`, ...
- traversal order is preorder
- YAML serialization is stable
- golden tests enforce byte-for-byte reproducibility

## Why not X?

These systems solve adjacent problems, but not the same one.

| Approach | What it does well | Why STF-SIR still matters |
| --- | --- | --- |
| Embeddings | Similarity search and dense retrieval | They do not expose deterministic, addressable semantic units with explicit structure and typed relations |
| ASTs | Preserve syntax and hierarchy | They do not by themselves define semantic glosses, artifact validation, or a stable semantic serialization contract |
| Knowledge graphs | Represent explicit entities and edges at graph level | They are often inferred or curated; STF-SIR starts earlier as a deterministic compilation of source documents without inference |
| Subword tokenization | Efficient model input encoding | It optimizes sequence handling, not semantic preservation or reproducible semantic IR |

In practice, STF-SIR should be viewed as a semantic compilation system that can complement embeddings, parsers, and graph systems rather than replace them wholesale.

## Paper

The repository includes both operational and formal documents:

- [`docs/stf-sir-article.md`](docs/stf-sir-article.md) for the article-style STF-SIR foundation
- [`docs/sts-formalization.md`](docs/sts-formalization.md) for the multidimensional STS formalization
- [`docs/sts-paper.pdf`](docs/sts-paper.pdf) for the compiled paper
- [`docs/sts-paper.tex`](docs/sts-paper.tex) for the LaTeX source
- [`docs/sir-graph.md`](docs/sir-graph.md) for the SirGraph contract
- [`docs/retention-baseline.md`](docs/retention-baseline.md) for the operational retention model
- [`docs/v1-invariants.md`](docs/v1-invariants.md) for freeze-level invariants
- [`spec/stf-sir-spec-v1.md`](spec/stf-sir-spec-v1.md) for the implementation-facing specification

## Status (v1.0.0)

STF-SIR v1.0.0 guarantees:

- deterministic compilation from Markdown to `.zmd`
- schema validation plus semantic invariant checks
- stable ztoken and relation identity
- a minimal SirGraph projection over the compiled artifact
- a deterministic retention baseline
- CI enforcement through `fmt`, `clippy`, tests, fixtures, and goldens

What v1.0.0 does not attempt:

- inferred semantics beyond the MVP gloss fallback
- entity linking or external knowledge integration
- pragmatics, temporal modeling, or world-state reasoning

The current system prioritizes determinism and structural fidelity over semantic completeness.

## Roadmap

- richer graph query APIs over SirGraph
- expanded retention beyond the baseline completeness model
- formal context, pragmatics, and temporal dimensions from STS
- additional source frontends beyond Markdown
- external implementations validated through the conformance kit

## License

This project is licensed under the Apache License 2.0. See [LICENSE](LICENSE) for details.
