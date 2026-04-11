# STF-SIR: A Semantic Intermediate Representation for Meaning-Preserving Tokenization

## 1. Abstract

This paper introduces the Semantic Tokenization Framework and Semantic Intermediate Representation (STF-SIR), a compiler-oriented model for transforming source documents into semantically structured tokens suitable for retrieval, reasoning, and agent execution. Contemporary tokenization schemes such as byte-pair encoding and WordPiece optimize sequence efficiency and vocabulary coverage, but they do not treat syntax, meaning, or logical dependency as first-class objects. As a consequence, downstream AI systems repeatedly reconstruct structure and semantics from flattened token streams.

STF-SIR addresses this limitation by defining a semantic token, or ztoken, as a four-dimensional object

\[
z = \langle L, S, \Sigma, \Phi \rangle
\]

where \(L\) denotes lexical information, \(S\) syntactic structure, \(\Sigma\) semantic interpretation, and \(\Phi\) logical relations. A compiler \(C_\theta : \mathcal{L} \to \mathcal{Z}\), parameterized by configuration \(\theta\), maps admissible source documents in language space \(\mathcal{L}\) to structured artifacts in representation space \(\mathcal{Z}\). The resulting artifact is interpreted as a Semantic Intermediate Representation (SIR), which can be modeled as a typed attributed graph over ztokens and their relations.

The paper contributes three results. First, it formalizes semantic tokenization as a representation problem rather than a compression-only problem. Second, it introduces an information-retention framework for reasoning about lexical, syntactic, semantic, and logical fidelity. Third, it characterizes core properties of STF-SIR, including traceability, determinism under fixed compilation parameters, monotone enrichment, and bounded reversibility. The objective is not to replace all statistical tokenization, but to provide a rigorous intermediate layer between raw content and downstream intelligent computation.

## 2. Introduction

Most AI systems consume text through token streams that are efficient for neural training and inference but weak as explicit knowledge-bearing structures. This choice has practical consequences. Retrieval systems partition documents into coarse chunks because surface tokens alone provide limited semantic locality. Reasoning systems recover syntax, entity structure, and logical dependency at runtime because those properties are not preserved directly by the tokenization layer. Agent systems repeatedly re-interpret the same context because no stable semantic intermediate form is shared across components.

Compiler theory offers a useful contrast. In programming language design, intermediate representations separate source syntax from downstream optimization and execution. They make structure explicit, enable reproducible transformations, and support analysis without repeatedly parsing the raw source. STF-SIR applies this principle to natural language and structured documents. Its premise is that language processing for AI systems should expose a representation layer where lexical realization, structural role, semantic meaning, and logical relation are jointly represented rather than reconstructed on demand.

The central unit of STF-SIR is the ztoken. A ztoken is not a subword fragment. It is a semantic carrier associated with a source span, a structural position, a semantic interpretation, and a set of typed relations. A ztoken is also not definitionally identical to a graph node. Rather, it is the primary compiled unit and a potential node anchor within the SIR graph. The central artifact of STF-SIR is the Semantic Intermediate Representation (SIR), a document-level structure composed of ztokens and their relations. STF-SIR therefore reframes tokenization as compilation from surface documents into an analyzable semantic representation.

## 3. Problem Statement

Let \(\mathcal{L}\) denote the set of admissible source documents, such as Markdown, JSON, or other structured textual inputs. Given a document \(d \in \mathcal{L}\), the objective is to construct a representation \(C_\theta(d) \in \mathcal{Z}\) that satisfies the following requirements:

| Requirement | Description |
| --- | --- |
| Lexical traceability | The representation must remain anchored to the source document. |
| Structural explicitness | Document organization and syntactic role must be directly represented. |
| Semantic availability | Meaning-bearing information must be encoded without requiring full raw-text reconstruction. |
| Logical connectivity | Relations such as dependency, reference, support, contradiction, or containment must be representable as typed objects. |
| Deterministic compilation | Repeated compilation under fixed parameters should produce identical artifacts. |
| Extensible refinement | Richer semantic or logical analyses should refine the representation without invalidating previously emitted structure. |

Traditional token streams satisfy lexical coverage but do not satisfy the full set of requirements above. In particular, the representation consumed by downstream systems is often insufficient for compositional querying over meaning. The practical problem is therefore to design a representation that preserves more of the informational structure already present in the source document while remaining compilable and operationally useful.

## 4. Limitations of Traditional Tokenization

Subword tokenization methods are successful at vocabulary compression, coverage over unseen words, and computational compatibility with transformer architectures. Their limitations emerge when they are asked to serve as semantic representations rather than merely sequence encodings.

| Limitation | Consequence |
| --- | --- |
| Surface segmentation is optimized statistically | Semantic units are split across multiple low-level tokens. |
| Syntax is implicit in sequence position | Tree structure must be reconstructed after tokenization. |
| Logical relations are not first-class | Dependency, entailment, reference, and support remain latent. |
| Token identity is context-light | Higher-order meaning is distributed over long spans and model state. |
| Reconstruction is surface-oriented | Preserving byte recoverability does not imply preserving semantic fidelity. |

These limitations are not failures of subword tokenization with respect to its original design goals. Rather, they indicate a mismatch between sequence encoding objectives and meaning-preserving representation objectives. STF-SIR addresses the latter.

## 5. STF-SIR Model

STF-SIR defines a semantic token as

\[
z_i = \langle L_i, S_i, \Sigma_i, \Phi_i \rangle
\]

with the following interpretation:

| Component | Role |
| --- | --- |
| \(L_i\) | Lexical realization: source span, surface form, and normalized lexical content |
| \(S_i\) | Syntactic role: structural type, tree position, and local composition metadata |
| \(\Sigma_i\) | Semantic interpretation: meaning-bearing abstraction, concept assignments, or proposition-level gloss |
| \(\Phi_i\) | Typed relations partitioned into structural, logical, and semantic-link categories |

At the artifact level, STF-SIR compiles a document into a finite collection of ztokens together with their structural and logical relations. Informally,

\[
C_\theta(d) = \{z_1, z_2, \ldots, z_n\} \;\cup\; R_d
\]

where \(R_d\) is a relation set induced by syntax, semantics, and explicit logic. STF-SIR is therefore local and global at once: each ztoken is a local carrier of four dimensions, while the full artifact is a global structure over those carriers.

This distinction matters. The ztoken is the unit of addressing, storage, and transport. The SIR artifact is the unit of document-level interpretation, analysis, and execution. Granularity is orthogonal to this distinction: the same formalism may emit block-level, sentence-level, or entity-level ztokens under different compilation profiles.

## 6. Mathematical Formalization

### 6.1 Notation

To avoid overloading symbols, this paper uses:

| Symbol | Meaning |
| --- | --- |
| \(\mathcal{L}\) | Input language space |
| \(\mathcal{Z}\) | SIR artifact space |
| \(d\) | Source document |
| \(C_\theta\) | STF compiler under fixed configuration \(\theta\) |
| \(z_i\) | Individual ztoken |
| \(\pi_L, \pi_S, \pi_\Sigma, \pi_\Phi\) | Projection operators for the four dimensions |

Calligraphic symbols denote spaces of objects. In particular, \(\mathcal{L}\) and \(\mathcal{Z}\) denote input and artifact spaces, whereas plain \(L\) is reserved for the lexical component of a ztoken.

### 6.2 Compiler Mapping

The STF compiler is a total or partial mapping

\[
C_\theta : \mathcal{L} \to \mathcal{Z}
\]

where \(\theta\) contains all compilation parameters required for deterministic interpretation: parser version, normalization rules, ontology resources, relation extractors, and serialization policy.

For a compiled document \(d\), define

\[
C_\theta(d) = \mathcal{A}_d = (Z_d, G_d, M_d)
\]

where:

- \(Z_d = (z_1, \dots, z_n)\) is an ordered sequence of ztokens.
- \(G_d = (V_d, E_d)\) is a relation graph.
- \(M_d\) is metadata sufficient for provenance and reproducibility.

### 6.3 ZToken Components

Each ztoken \(z_i\) is a quadruple

\[
z_i = \langle L_i, S_i, \Sigma_i, \Phi_i \rangle.
\]

One useful formalization is:

\[
L_i = (\sigma_i, \iota_i, \nu_i), \quad
S_i = (\tau_i, p_i, a_i), \quad
\Sigma_i = (\omega_i, \kappa_i), \quad
\Phi_i = \{r_{i1}, \dots, r_{im}\}
\]

with each relation represented as

\[
r_{ij} = (\chi_{ij}, \lambda_{ij}, t_{ij}, \mu_{ij})
\]

where:

- \(\sigma_i\) is a surface form or source slice,
- \(\iota_i\) is a source interval or offset set,
- \(\nu_i\) is a deterministic lexical normalization,
- \(\tau_i\) is a syntactic type,
- \(p_i\) is a structural position, for example a tree path,
- \(a_i\) is additional syntactic metadata,
- \(\omega_i\) is a semantic object, gloss, or proposition,
- \(\kappa_i\) is optional semantic context such as concept assignments,
- \(\chi_{ij} \in \{\texttt{structural}, \texttt{logical}, \texttt{semantic-link}\}\) is the relation category,
- \(\lambda_{ij}\) is the relation label,
- \(t_{ij}\) is the relation target,
- \(\mu_{ij}\) is optional relation metadata.

The projection operators are straightforward:

\[
\pi_L(z_i) = L_i,\quad
\pi_S(z_i) = S_i,\quad
\pi_\Sigma(z_i) = \Sigma_i,\quad
\pi_\Phi(z_i) = \Phi_i.
\]

### 6.4 Well-Formedness

An artifact \(\mathcal{A}_d\) is well-formed if the following conditions hold:

1. Every ztoken has a unique identifier in \(Z_d\).
2. Every lexical interval in \(L_i\) refers to a valid source region in \(d\).
3. The structure induced by \(S_i\) is acyclic and rooted or forest-like.
4. Every relation in \(\Phi_i\) refers to a valid endpoint in \(Z_d\) or another declared semantic object.
5. Metadata \(M_d\) is sufficient to reproduce compilation under the same \(\theta\).

These conditions define a correctness envelope for both theoretical analysis and operational validation.

## 7. Information Retention Theory

### 7.1 Retention as a Multi-Dimensional Objective

STF-SIR is motivated by the observation that information preservation is not one-dimensional. A representation may preserve bytes while losing syntax, or preserve syntax while omitting semantic dependency. Let

\[
\mathbb{D} = \{L, S, \Sigma, \Phi\}
\]

be the set of representational dimensions. For each \(X \in \mathbb{D}\), define:

- an extraction function \(E_X(d)\) capturing the dimension-specific information available in the source document,
- a reconstruction function \(R_X(C_\theta(d))\) recovering that dimension from the compiled artifact,
- a similarity or equivalence function \(\operatorname{sim}_X\).

Then the dimension-specific retention score is

\[
\rho_X(d) = \operatorname{sim}_X\big(E_X(d), R_X(C_\theta(d))\big), \qquad 0 \le \rho_X(d) \le 1.
\]

This yields a retention vector

\[
\rho(d) = \langle \rho_L(d), \rho_S(d), \rho_\Sigma(d), \rho_\Phi(d) \rangle.
\]

The goal of STF-SIR is not necessarily to maximize a single scalar objective, but to improve the lower-bound fidelity across all four dimensions relative to flattening representations.

### 7.2 Information-Theoretic Interpretation

If documents are treated as random variables, retention can also be expressed using normalized mutual information:

\[
\rho_X = \frac{I(E_X(D); \pi_X(C_\theta(D)))}{H(E_X(D))}
\]

when \(H(E_X(D)) > 0\). This formulation emphasizes that a representation can only preserve what is encoded and recoverable. It also clarifies a limitation: semantic normalization may deliberately collapse distinctions that matter lexically but not conceptually. Hence perfect semantic retention does not imply perfect lexical retention, and vice versa.

### 7.3 Retention Classes

For practical use, the following classes are helpful:

| Class | Condition |
| --- | --- |
| Exact retention | \(\rho_X(d) = 1\) under strict equivalence |
| Faithful retention | \(\rho_X(d)\) exceeds a configured threshold under task-relevant equivalence |
| Lossy retention | \(\rho_X(d)\) is below threshold but still informative |
| Null retention | The dimension is not represented in a recoverable form |

STF-SIR is intended to make null retention for \(S\), \(\Sigma\), and \(\Phi\) unnecessary in the common case.

## 8. Semantic Intermediate Representation (SIR)

The document-level artifact induced by STF-SIR is the Semantic Intermediate Representation. A useful abstraction is a typed attributed multigraph

\[
\operatorname{SIR}(d) = (V, E, \tau, \alpha)
\]

where:

- \(V\) is a finite set of vertices,
- \(E \subseteq V \times \mathcal{R} \times V\) is a typed edge set over relation alphabet \(\mathcal{R}\),
- \(\tau : V \to \mathcal{T}\) assigns node types,
- \(\alpha\) assigns attributes to nodes and edges.

At minimum, \(V\) contains one vertex for each ztoken-backed unit. More expressive variants may also include entity nodes, proposition nodes, section nodes, or external knowledge references. In this view, each ztoken is a boundary object between local annotation and global graph structure:

- \(L\) binds the token to source evidence.
- \(S\) places it in the document tree.
- \(\Sigma\) assigns meaning-bearing content.
- \(\Phi\) links it into a relational graph.

A ztoken is therefore a compiled unit and a potential graph node anchor, but the two notions are not identical. The graph may contain auxiliary nodes that are not themselves standalone ztokens, and a ztoken may participate in the graph through one or more node realizations depending on the compilation profile.

The SIR abstraction is therefore richer than an annotated token sequence and lighter than a full theorem-proving environment. It is intended as an intermediate form, not as a final knowledge ontology.

## 9. Properties and Guarantees

Theoretical usefulness requires explicit statements about what STF-SIR can and cannot guarantee.

### 9.1 Traceability

**Proposition 1.** If every emitted ztoken retains a valid source interval \(\iota_i\), then every ztoken is traceable to documentary evidence in the source document.

**Consequence.** Downstream systems can inspect, quote, or verify the source basis of any semantic object without re-parsing the full document.

### 9.2 Determinism Under Fixed Configuration

**Proposition 2.** Let \(\theta\) fix parser behavior, normalization rules, semantic extraction procedures, logical relation rules, and serialization policy. If each stage is deterministic, then \(C_\theta\) is deterministic:

\[
\forall d \in \mathcal{L}, \quad C_\theta(d) = C_\theta'(d) \text{ for all repeated executions with } \theta' = \theta.
\]

**Consequence.** Artifacts can support golden tests, reproducible indexing, and stable semantic diffing.

### 9.3 Monotone Enrichment

**Proposition 3.** Suppose a refinement operator \(U\) only appends or enriches attributes in \(\Sigma\) and \(\Phi\) without mutating existing identifiers, spans, or structural coordinates. Then the projection to the previous artifact schema is preserved.

**Consequence.** STF-SIR supports incremental semantic improvement without invalidating prior references.

### 9.4 Bounded Reversibility

**Proposition 4.** Exact lexical reconstruction is possible when \(L\) preserves sufficient source slices or references. Exact semantic reconstruction of the author's intent is, in general, not guaranteed because multiple surface forms may normalize to the same semantic object and some pragmatic information may remain implicit.

**Consequence.** STF-SIR should be evaluated using explicit equivalence criteria rather than assuming a single notion of reversibility.

### 9.5 Compositional Assembly

If a document is partitioned into subdocuments \(d_1, \dots, d_k\), each compiled separately and later merged with boundary metadata, then the assembled artifact preserves local ztoken identity and can recover global ordering relations. This property is essential for scalable indexing and distributed compilation.

## 10. Example (Formal)

Consider the source document

```markdown
# Model assumptions

A compiler preserves meaning when it preserves structure and relations.
```

Let \(d \in \mathcal{L}\) denote this document. A simple STF-SIR compilation may emit two principal ztokens:

| ztoken | Formal characterization |
| --- | --- |
| \(z_1\) | \(\langle L_1, S_1, \Sigma_1, \Phi_1 \rangle\), where \(L_1\) is the heading span, \(S_1\) is a level-1 heading node, \(\Sigma_1\) denotes the concept of model assumptions, and \(\Phi_1\) includes a structural ordering relation to \(z_2\) |
| \(z_2\) | \(\langle L_2, S_2, \Sigma_2, \Phi_2 \rangle\), where \(L_2\) is the paragraph span, \(S_2\) is a paragraph node, \(\Sigma_2\) is the proposition \(preserve(compiler, meaning) \Leftarrow preserve(compiler, structure) \land preserve(compiler, relations)\), and \(\Phi_2\) may include logical dependency or semantic-link relations |

A minimal ztoken-backed graph view of the same artifact may be summarized as

\[
V = \{z_1, z_2\}, \qquad
E = \{(z_1, \texttt{precedes}, z_2), (z_1, \texttt{scopes}, z_2)\}.
\]

The formal value of the example is not that one unique semantic encoding must be chosen, but that the representation explicitly separates surface evidence, structural role, proposition content, and logical linkage.

## 11. Implications for AI Systems

STF-SIR has several implications for the design of AI pipelines.

| System concern | Implication of STF-SIR |
| --- | --- |
| Retrieval | Indexing can target semantically meaningful units rather than arbitrary chunks. |
| Reasoning | Logical edges reduce the need to reconstruct relations from flat context windows. |
| Memory | Agents can store and update ztokens or subgraphs instead of re-storing raw passages. |
| Attribution | Lexical traceability enables source-grounded explanations and audits. |
| Evaluation | Fidelity can be measured dimension by dimension rather than only by text reconstruction quality. |

The framework is especially relevant where context cost is high and repeated interpretation is expensive. By externalizing semantics into an intermediate representation, STF-SIR may reduce redundant computation across indexing, retrieval, planning, and execution layers.

## 12. Future Work

Several research directions remain open.

| Area | Open question |
| --- | --- |
| Canonical semantics | How should semantic objects in \(\Sigma\) be normalized across domains and languages? |
| Logical expressivity | Which fragment of logic is sufficient for practical inference without overburdening compilation? |
| Evaluation | What benchmark suite best measures \(\rho_L, \rho_S, \rho_\Sigma,\) and \(\rho_\Phi\)? |
| Learning integration | How should neural models consume SIR artifacts directly rather than reconstructed text? |
| Compression | What are the storage and latency trade-offs between raw text, embeddings, and SIR artifacts? |
| Versioning | How should semantic diffs and artifact compatibility be defined over evolving schemas? |

Future work should distinguish clearly between what is normative for interoperability and what is exploratory for research.

## 13. Conclusion

STF-SIR formalizes semantic tokenization as a compiler problem over meaning-bearing structure. Its core claim is that a useful token for AI systems should preserve more than lexical segmentation: it should preserve lexical evidence, syntactic role, semantic interpretation, and logical relation in a unified object. The ztoken

\[
z = \langle L, S, \Sigma, \Phi \rangle
\]

provides that object, while the Semantic Intermediate Representation provides the document-level structure in which those objects interact.

The framework does not eliminate the utility of statistical tokenization for model training or low-level inference. Instead, it introduces a formally motivated intermediate layer between source content and intelligent computation. This separation enables a more rigorous treatment of fidelity, reproducibility, and downstream semantics, and it establishes a foundation for future research on representation-aware AI systems.
