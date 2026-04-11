# STF-SIR

From tokens to meaning: a foundation for semantic AI context representation.

> Status: STF-SIR is currently documented as an architectural framework and evolving specification for semantic tokenization and intermediate representation.

## Overview

STF-SIR, short for **Semantic Tokenization Framework - Semantic Intermediate Representation**, proposes a different abstraction for how language and structured knowledge are prepared for AI systems.

Instead of reducing input to statistical subword fragments alone, STF-SIR compiles text and structured data into **ztokens**: semantic units that preserve multiple layers of information in a single representation:

- lexical form
- syntactic structure
- semantic meaning
- logical relationships

The central idea is to treat language processing more like compilation. In the same way that LLVM IR or WebAssembly provide a structured layer between source code and execution, STF-SIR introduces a **Semantic Intermediate Representation (SIR)** between source content and downstream AI tasks such as reasoning, retrieval, and agent execution.

This repository currently defines the core model, formal notation, architecture direction, and proposed artifact format for that representation.

## The Problem

Most modern language systems still rely on tokenization schemes such as BPE and WordPiece. Those methods are effective for model training and text compression, but they are not designed to preserve meaning as a first-class structure.

| Limitation in traditional tokenization | Why it matters |
| --- | --- |
| Text is split into statistical fragments | Meaning is distributed across many low-level tokens |
| Syntax and document structure are flattened | Structural cues become harder to recover reliably |
| Semantic and logical relations are implicit | Reasoning systems must reconstruct them at runtime |
| Context is optimized for sequence consumption | Retrieval and execution remain tightly coupled to raw text |
| Reversibility is only partial at the semantic level | Compactness does not imply faithful preservation of intent |

In practice, this means current tokenization often increases context cost while forcing downstream systems to reconstruct the very structure they need in order to reason well.

## The STF-SIR Approach

STF-SIR introduces **semantic tokenization** as an alternative representation strategy.

Rather than treating a document as a flat sequence of subword units, STF-SIR models it as a compiled semantic structure composed of ztokens. Each ztoken carries not only text-level information, but also the syntactic, semantic, and logical context required to interpret that unit in relation to the whole.

This approach is designed to support:

- higher-fidelity context representation
- semantic compression with structural awareness
- direct querying over meaning-bearing units
- reasoning workflows that do not depend entirely on raw-text reconstruction
- a shared intermediate layer for AI pipelines, retrieval systems, and agents

STF-SIR is therefore best understood not as a tokenizer replacement in isolation, but as a proposed **representation layer** for AI-native systems.

## Mathematical Definition

At the document level, STF-SIR defines a compilation from source content `D` into semantic representation `z`:

```text
D -> z
```

Each semantic token, or ztoken, is defined as:

```text
z = <L, S, Σ, Φ>
```

| Component | Meaning |
| --- | --- |
| `L` | Lexical layer: original lexical form, symbols, and surface text |
| `S` | Syntactic layer: structural role, parse relationships, or AST-aligned context |
| `Σ` | Semantic layer: normalized meaning or conceptual interpretation |
| `Φ` | Logical layer: relations, constraints, dependencies, or inference links |

Together, these layers aim to preserve not just what was written, but how it is structured, what it means, and how it relates to other units in the document.

## Architecture

STF-SIR follows a compiler-style pipeline from source input to semantic representation:

```text
Input (.md | .json | structured data)
        ↓
Lexical Analysis
        ↓
Syntactic Representation
        ↓
Semantic Extraction
        ↓
Logical Modeling
        ↓
STF Compiler
        ↓
SIR Artifacts (.zmd / ztokens / graph references)
```

Conceptually, each stage refines the input into a representation that is progressively less dependent on raw token order and progressively more aligned with semantic execution. The output is intended to support downstream operations such as semantic lookup, structured retrieval, and reasoning over relationships rather than over text fragments alone.

## File Format: `.zmd`

STF-SIR proposes `.zmd` as a container format for serialized semantic representation artifacts. The format below is conceptual and illustrates the shape of the representation rather than a finalized specification.

```yaml
header:
  version: STF-SIR/1.0
  encoding: semantic-binary

body:
  ztokens:
    - id: z1
      type: paragraph
      lexical_ref: src_4
      structure_ref: ast_12
      semantic_hash: 0xA91F...
      logic_ref: graph_3
```

The intent of `.zmd` is to make semantic artifacts portable, inspectable, and usable as a stable exchange layer between compilers, retrieval systems, and reasoning engines.

## Example

### Input

```markdown
# AI is transforming software development
```

### Conceptual ztoken output

```yaml
ztoken:
  id: z1
  type: statement
  L: "AI is transforming software development"
  S:
    role: heading
    structure: proposition
  Σ:
    meaning: "Artificial intelligence is changing software engineering practice"
  Φ:
    relation: transformation
    target: software_development
```

This example is illustrative. The purpose is not to prescribe a final serialization, but to show how STF-SIR treats a unit of text as a structured semantic object rather than a sequence of subword fragments.

## Key Innovations

- A semantic token primitive, `ztoken`, defined as a multi-layer representation rather than a statistical fragment
- A Semantic Intermediate Representation for language and structured data
- Compiler-style processing for AI context preparation
- A path toward querying and reasoning over compressed semantic units
- A representation model that keeps lexical, structural, semantic, and logical information in the same abstraction

## Comparison with Traditional Tokenization

| Feature | Traditional tokenization | STF-SIR |
| --- | --- | --- |
| Primary unit | Subword or statistical token | Semantic token (`ztoken`) |
| Structural awareness | Limited | Explicit |
| Semantic preservation | Indirect | Central design goal |
| Logical relations | Usually implicit | Representable in `Φ` |
| Query model | Text-first | Meaning-first |
| Reversibility | Surface-form oriented | High-fidelity semantic reconstruction is a design objective |
| Role in the stack | Input encoding | Intermediate representation layer |

## Use Cases

- Semantic retrieval systems that operate on meaning-bearing units instead of raw chunks
- AI agents that need structured context for planning, execution, or memory
- Knowledge systems that bridge documents, graphs, and symbolic relations
- Semantic diffing and versioning of meaning, not only text
- Research on compiler-inspired NLP and representation learning

## Roadmap

- Formal STF-SIR specification
- Canonical `.zmd` schema
- Reference compiler for source-to-SIR transformation
- Semantic query and execution model
- Integration patterns for RAG systems and AI agents

## License

This project is licensed under the Apache License 2.0. See [LICENSE](LICENSE) for details.

## Author

**Rogério Figueiredo**  
DevSecOps | Systems Architecture | AI Systems
