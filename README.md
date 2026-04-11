# STF-SIR - Semantic Tokenization Framework

From tokens to meaning: a new foundation for AI context representation.

## Overview

STF-SIR (Semantic Tokenization Framework - Semantic Intermediate Representation) is a novel approach to tokenization that transforms text and structured data into **ztokens**: semantic units that preserve:

- Lexical information
- Syntactic structure
- Semantic meaning
- Logical relationships

Unlike traditional tokenization (BPE, WordPiece), STF-SIR introduces a Semantic Intermediate Representation (SIR) designed for:

- Advanced AI reasoning
- Next-generation RAG systems
- Efficient context compression
- Semantic query execution

## The Problem

Current tokenization methods:

- Lose structural and semantic information
- Fragment meaning into statistical chunks
- Increase token cost for LLMs
- Limit reasoning capabilities

## The STF Approach

STF-SIR defines a compilation process:

```text
D -> z
```

Where:

- `D` = original document
- `z` = semantic representation (ztokens)

Each ztoken is defined as:

```text
z = <L, S, Σ, Φ>
```

| Component | Description        |
| --- | --- |
| `L` | Lexical layer      |
| `S` | Syntax (AST)       |
| `Σ` | Semantic meaning   |
| `Φ` | Logical relations  |

## Architecture

```text
Input (.md / .json)
        ↓
Lexical Analysis
        ↓
Syntax Tree (AST)
        ↓
Semantic Extraction
        ↓
Logical Modeling
        ↓
STF Compiler
        ↓
Output (.zmd / ztokens)
```

## File Format: `.zmd`

```yaml
header:
  version: STF-SIR/1.0
  encoding: semantic-binary

body:
  ztokens:
    - id: z1
      type: paragraph
      semantic_hash: 0xA91F...
      structure_ref: ast_12
      logic_ref: graph_3
```

## Example

### Input

```markdown
# AI is transforming software development
```

### Output (conceptual)

```yaml
ztoken:
  type: statement
  semantic: "AI impacts software engineering"
  logic: cause-effect
```

## Key Innovations

- Semantic Tokenization (ztokens)
- Lossless Information Retention
- Direct execution on compressed representation
- Query without decompression
- Semantic + Logical IR

## Comparison

| Feature | Traditional Tokens | STF-SIR |
| --- | --- | --- |
| Unit | Subword | Semantic unit |
| Context | Local | Global |
| Compression | Statistical | Semantic |
| Reasoning | Limited | Native |
| Reversibility | Partial | High |

## Use Cases

- Semantic RAG (no chunking)
- AI agents operating on meaning
- Knowledge graphs
- Semantic versioning and diff
- Natural language compilation

## Roadmap

- STF Formal Specification
- `.zmd` format definition
- CLI Compiler (`stf compile`)
- Semantic Query Engine
- Agent integration

## License

Licensed under the Apache License 2.0.

## Author

Rogério Figueiredo  
DevSecOps | Systems Architecture | AI Systems

