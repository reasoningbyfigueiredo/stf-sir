# Semantic Token Space (STS): A Multidimensional Formalization for STF-SIR

## 1. Introduction

Traditional tokenization schemes are effective as sequence encodings, but they are not adequate as formal representations of meaning-bearing structure. Statistical subword methods preserve lexical decomposability and support efficient model training, yet they leave syntax, semantics, logical dependency, context, and pragmatic force largely implicit. As a result, downstream systems must repeatedly reconstruct higher-order structure from flattened streams.

STF-SIR addresses this limitation by treating document processing as compilation into a Semantic Intermediate Representation (SIR). In the current operational baseline, this representation is realized through ztokens, relations, deterministic validation, graph projection, and retention scoring. The present document extends that foundation by defining the **Semantic Token Space (STS)**: a formal multidimensional space in which semantic tokens are modeled not merely as tuples of lexical and structural data, but as points in a coupled system of lexical, syntactic, semantic, logical, contextual, pragmatic, temporal, and coherence dimensions.

STF-SIR is therefore interpreted at three layers:

1. **STF** is the framework: the methodological and compiler-oriented architecture.
2. **SIR** is the intermediate representation: the compiled artifact layer.
3. **STS** is the mathematical space: the abstract structure in which semantic tokens are defined, constrained, transformed, and analyzed.

Under this interpretation, STF-SIR may be viewed as a compilation framework over semantic artifacts, STS as the corresponding semantic space, and retention as the metric family used to evaluate preservation across transformations.

The purpose of STS is not to invalidate the existing v1 system. Rather, it generalizes the existing model

\[
z = \langle L, S, \Sigma, \Phi \rangle
\]

into a richer formal space while preserving compatibility with the current implementation.

Throughout this document, \(\Phi\) is treated as the legacy v1 symbol for the logical dimension, while \(\Gamma\) is the preferred STS notation. Accordingly, the alignment

\[
\Phi_{\mathrm{v1}} \equiv \Gamma_{\mathrm{STS}}
\]

is assumed whenever the v1 and STS formalisms are compared.

## 2. Semantic Token Space Definition

Let the semantic token space be the Cartesian product

\[
\mathcal{T} = \mathcal{L} \times \mathcal{S} \times \mathcal{Sem} \times \mathcal{G} \times \mathcal{C} \times \mathcal{P} \times \mathcal{D} \times \mathcal{O}.
\]

For typographical clarity:

- \(\mathcal{L}\) denotes the lexical space,
- \(\mathcal{S}\) denotes the syntactic space,
- \(\mathcal{Sem}\) denotes the semantic space,
- \(\mathcal{G}\) denotes the logical-relational space,
- \(\mathcal{C}\) denotes the contextual space,
- \(\mathcal{P}\) denotes the pragmatic space,
- \(\mathcal{D}\) denotes the temporal or evolutionary space,
- \(\mathcal{O}\) denotes the coherence or validation space.

A semantic token is an element

\[
t \in \mathcal{T}
\]

with coordinate form

\[
t = (L, S, \Sigma, \Gamma, C, P, \Delta, \Omega).
\]

To support information-preserving reasoning, assume that semantic meaning is ordered by a partial order

\[
(\mathcal{Sem}, \preceq_{\mathcal{Sem}})
\]

where \(\Sigma_1 \preceq_{\mathcal{Sem}} \Sigma_2\) means that \(\Sigma_2\) is at least as informative as \(\Sigma_1\). When no ambiguity arises, the shorter notations

\[
\Sigma_1 \preceq \Sigma_2
\qquad\text{and}\qquad
\Sigma_2 \succeq \Sigma_1
\]

will be used. Analogous implementation-dependent information preorders may be assumed on \(\mathcal{G}\) and \(\mathcal{O}\) when enrichment or comparison requires them.

More explicitly, when needed one may write

\[
(\mathcal{G}, \preceq_{\mathcal{G}})
\qquad\text{and}\qquad
(\mathcal{O}, \preceq_{\mathcal{O}})
\]

for implementation-dependent information preorders on logical structure and coherence structure, respectively.

The coherence coordinate is modeled as an artifact-relative validation tuple

\[
\Omega = (\omega_{\mathrm{schema}}, \omega_{\mathrm{inv}}, \omega_{\rho}) \in \mathcal{O},
\]

with

\[
\mathcal{O} = \{0,1\} \times \{0,1\} \times [0,1].
\]

Its components are interpreted as follows:

- \(\omega_{\mathrm{schema}} \in \{0,1\}\): schema validity indicator,
- \(\omega_{\mathrm{inv}} \in \{0,1\}\): invariants validity indicator,
- \(\omega_{\rho} \in [0,1]\): retention-derived coherence score.

Because coherence is global rather than purely local, \(\Omega\) is most naturally evaluated relative to an artifact or compilation context. When needed, this dependence may be written explicitly as \(\Omega_{\mathcal{A}}(t)\) for token \(t\) in artifact \(\mathcal{A}\). For scalar comparisons, assume a monotone scalarization

\[
s_\Omega : \mathcal{O} \to [0,1].
\]

The intended meaning of each dimension is:

| Dimension | Interpretation |
| --- | --- |
| \(L\) | Lexical evidence: source span, surface text, normalized form |
| \(S\) | Syntactic structure: tree position, node type, composition metadata |
| \(\Sigma\) | Semantic meaning: gloss, concept assignment, proposition content |
| \(\Gamma\) | Logical relations: typed dependencies, containment, ordering, support, reference; legacy v1 alias: \(\Phi\) |
| \(C\) | Contextual dependency: local and global conditions needed to interpret the token |
| \(P\) | Pragmatic intent: communicative role, discourse act, speaker or author intention |
| \(\Delta\) | Temporal or evolutionary state: versioning, provenance, revision, temporal scope |
| \(\Omega\) | Global coherence and validation state, operationalized as \((\omega_{\mathrm{schema}}, \omega_{\mathrm{inv}}, \omega_{\rho})\) |

The v1 ztoken model is recovered as a low-dimensional operational slice of STS by collapsing \(\Gamma\) into the current logical relation layer and treating \(C\), \(P\), and \(\Delta\) as implicit or unmaterialized.

## 3. Valid Token Manifold

The Cartesian product \(\mathcal{T}\) contains many combinations that are mathematically admissible as tuples but semantically invalid as STF-SIR tokens. Accordingly, STF-SIR does not treat all of \(\mathcal{T}\) as a realizable token population. Instead, it defines a constrained manifold

\[
\mathcal{M}_{\mathrm{STF}} \subset \mathcal{T}
\]

called the **valid token manifold**.

Formally, let

\[
\mathcal{K} = \{\kappa_1, \kappa_2, \dots, \kappa_m\}
\]

be a family of well-formedness predicates

\[
\kappa_j : \mathcal{T} \to \{0,1\}.
\]

Then the valid manifold is

\[
\mathcal{M}_{\mathrm{STF}} = \{ t \in \mathcal{T} \mid \forall \kappa_j \in \mathcal{K},\ \kappa_j(t)=1 \}.
\]

This definition captures the fact that semantic tokens occupy a constrained region of the full space. The constraints arise from coupling between dimensions. Examples include:

- a lexical span in \(L\) must be compatible with syntactic placement in \(S\),
- semantic content in \(\Sigma\) must be licensed by lexical and structural evidence,
- logical relations in \(\Gamma\) must reference semantically coherent endpoints,
- contextual state \(C\) must not contradict pragmatic intent \(P\),
- coherence \(\Omega\) must certify the compatibility of all other coordinates.

Thus STS is not merely a product space; it is a product space together with a nontrivial admissibility geometry.

## 4. Coupling Functions

The dimensions of STS are not independent. Their dependencies are formalized by coupling functions.

### 4.1 Semantic Coupling

\[
\Sigma = f_\Sigma(L, S, C)
\]

with

\[
f_\Sigma : \mathcal{L} \times \mathcal{S} \times \mathcal{C} \to \mathcal{Sem}.
\]

Conceptually, semantic interpretation is not determined by surface form alone. It depends on lexical evidence, syntactic role, and contextual conditioning. In the current STF-SIR v1 implementation, \(f_\Sigma\) is approximated by a deterministic fallback:

\[
\widehat{f}_\Sigma(L, S, C) = \operatorname{normalize}(L.\mathrm{plain\_text}),
\]

with \(C\) treated implicitly and \(S\) used primarily to delimit compilation units rather than to derive deeper semantic composition.

### 4.2 Logical Coupling

\[
\Gamma = f_\Gamma(\Sigma, C, P)
\]

with

\[
f_\Gamma : \mathcal{Sem} \times \mathcal{C} \times \mathcal{P} \to \mathcal{G}.
\]

Logical relations are induced by meaning, contextual dependency, and pragmatic framing. A statement of support, contradiction, definition, containment, or reference is not purely lexical; it arises from interpreted semantics situated in context and discourse purpose. In v1, \(f_\Gamma\) is approximated by deterministic structural emission, most notably:

- `contains`,
- `precedes`.

These are structurally classified relations, emitted in the logical pipeline stage.

### 4.3 Pragmatic Coupling

\[
P = f_P(C, \Sigma, \Gamma)
\]

with

\[
f_P : \mathcal{C} \times \mathcal{Sem} \times \mathcal{G} \to \mathcal{P}.
\]

Pragmatic intent is modeled as a function of context, semantic content, and the logical organization of the utterance. This reflects the fact that pragmatic force is rarely inferable from lexical material alone. The same proposition may function as assertion, warning, question, hypothesis, or instruction depending on context and relation structure.

In the current implementation, \(P\) is largely unmaterialized. Its inclusion in STS serves to formalize an extension axis already required by realistic language use.

### 4.4 Coherence Coupling

\[
\Omega = f_\Omega(L, S, \Sigma, \Gamma, C, P, \Delta)
\]

with

\[
f_\Omega : \mathcal{L} \times \mathcal{S} \times \mathcal{Sem} \times \mathcal{G} \times \mathcal{C} \times \mathcal{P} \times \mathcal{D} \to \mathcal{O}.
\]

The coherence dimension aggregates global admissibility. It evaluates whether the token remains well-formed and semantically coherent when all dimensions are considered together. In the current system, \(f_\Omega\) is approximated through:

- schema validation,
- semantic validation rules,
- invariants documentation,
- retention baseline scoring.

In operational terms, the current implementation is well approximated by

\[
\Omega = (\omega_{\mathrm{schema}}, \omega_{\mathrm{inv}}, \omega_{\rho}),
\]

where \(\omega_{\mathrm{schema}}\) records schema conformance, \(\omega_{\mathrm{inv}}\) records invariants satisfaction, and \(\omega_{\rho}\) is a monotone summary induced by retention measurements. Thus \(\Omega\) is not identified with a raw scalar; it is a structured coherence object with an associated scalarization \(s_\Omega\) when ordered comparison is required.

## 5. Projection Operators

For each token \(t = (L, S, \Sigma, \Gamma, C, P, \Delta, \Omega)\), define the canonical projections:

\[
\pi_L(t) = L,\quad
\pi_S(t) = S,\quad
\pi_\Sigma(t) = \Sigma,\quad
\pi_\Gamma(t) = \Gamma,
\]

\[
\pi_C(t) = C,\quad
\pi_P(t) = P,\quad
\pi_\Delta(t) = \Delta,\quad
\pi_\Omega(t) = \Omega.
\]

These projections are theoretical primitives. Their current implementation status is as follows.

| Projection | STS meaning | Current implementation mapping |
| --- | --- | --- |
| \(\pi_L\) | lexical coordinate | `ZToken.lexical`, `ZToken::pi_l()` |
| \(\pi_S\) | syntactic coordinate | `ZToken.syntactic`, `ZToken::pi_s()` |
| \(\pi_\Sigma\) | semantic coordinate | `ZToken.semantic`, `ZToken::pi_sigma()` |
| \(\pi_\Gamma\) | logical relation coordinate | local: `ZToken.logical` via `ZToken::pi_phi()`; global: `Artifact.relations`; graph realization: `SirGraph.edges`; v1 alias: \(\Phi\) |
| \(\pi_C\) | contextual coordinate | implicit only; partially approximated by source ordering, containment, and artifact-local adjacency |
| \(\pi_P\) | pragmatic coordinate | not explicitly implemented |
| \(\pi_\Delta\) | temporal or evolution coordinate | partially approximated by `compiler.version`, `compiler.config_hash`, tags, changelog, and provenance metadata |
| \(\pi_\Omega\) | coherence / validation coordinate | validator, invariants, conformance suite, retention baseline |

The current implementation therefore materializes:

- \(L\), \(S\), and \(\Sigma\) directly,
- \(\Gamma\) both locally and globally,
- \(\Omega\) operationally,
- \(C\), \(P\), and \(\Delta\) only partially or implicitly.

## 6. Operations on Tokens

STS supports a family of operations over tokens. These operations are defined at the level of the valid token manifold \(\mathcal{M}_{\mathrm{STF}}\).

### 6.1 Composition

Composition is a partial binary operation

\[
\oplus : \mathcal{M}_{\mathrm{STF}} \times \mathcal{M}_{\mathrm{STF}} \rightharpoonup \mathcal{M}_{\mathrm{STF}}.
\]

For compatible tokens \(t_1\) and \(t_2\),

\[
t_3 = t_1 \oplus t_2
\]

is defined when a compatibility predicate

\[
\kappa_{\oplus}(t_1, t_2)=1
\]

holds. Compatibility may require, for example, consistent source ordering, non-contradictory context, mergeable syntax, or coherence-preserving logical linkage.

At the dimensional level, composition is interpreted schematically as

\[
L_3 = L_1 \boxplus_L L_2,\qquad
S_3 = S_1 \boxplus_S S_2,\qquad
C_3 = C_1 \sqcup C_2,\qquad
\Delta_3 = \Delta_1 \sqcup \Delta_2,
\]

\[
\Sigma_3 = f_\Sigma(L_3, S_3, C_3),
\qquad
\Gamma_3 = \Gamma_1 \cup \Gamma_2 \cup \Gamma_{12},
\qquad
P_3 = f_P(C_3, \Sigma_3, \Gamma_3),
\qquad
\Omega_3 = f_\Omega(L_3, S_3, \Sigma_3, \Gamma_3, C_3, P_3, \Delta_3).
\]

Here \(\Gamma_{12}\) denotes new inter-token relations induced by composition.

Intuitively, composition forms a larger semantic unit from compatible smaller ones.

### 6.2 Equivalence

Semantic equivalence is defined by

\[
t_1 \equiv_\Sigma t_2 \iff \pi_\Sigma(t_1) =_{\mathcal{Sem}} \pi_\Sigma(t_2),
\]

where \(=_{\mathcal{Sem}}\) is a chosen semantic equivalence relation on \(\mathcal{Sem}\). Depending on the implementation level, this may mean identical gloss, identical concept assignment, or a stronger proposition-level equivalence.

Logical equivalence is defined by

\[
t_1 \equiv_\Gamma t_2 \iff \pi_\Gamma(t_1) \cong \pi_\Gamma(t_2),
\]

where \(\cong\) denotes typed-relation equivalence up to irrelevant identifier renaming. In the current graph-oriented realization, this may be approximated by equality of typed incoming and outgoing relation multisets.

Intuitively:

- \(t_1 \equiv_\Sigma t_2\) means the two tokens mean the same thing,
- \(t_1 \equiv_\Gamma t_2\) means the two tokens play the same logical role.

### 6.3 Reduction

Let

\[
r \subseteq \{L,S,\Sigma,\Gamma,C,P,\Delta,\Omega\}
\]

be a retained dimension set. Define the reduction operator

\[
\pi_r : \mathcal{M}_{\mathrm{STF}} \to \prod_{X \in r} \mathcal{X}
\]

by

\[
\pi_r(t) = (\pi_X(t))_{X \in r}.
\]

When a reduced token is re-embedded into STS, omitted dimensions are treated as abstract bottoms or placeholders \(\bot_X\). Thus reduction is a controlled loss of dimensional information.

Intuitively, reduction produces a lower-dimensional view of a token suitable for indexing, transport, or specialized computation.

### 6.4 Enrichment

Define enrichment as a monotone endomorphism

\[
\eta : \mathcal{M}_{\mathrm{STF}} \to \mathcal{M}_{\mathrm{STF}}
\]

such that, for all \(t \in \mathcal{M}_{\mathrm{STF}}\),

\[
\pi_L(\eta(t)) = \pi_L(t),\qquad
\pi_S(\eta(t)) = \pi_S(t),
\]

and, in the information order,

\[
\pi_\Sigma(t) \preceq_{\mathcal{Sem}} \pi_\Sigma(\eta(t)),\qquad
\pi_\Gamma(t) \preceq_{\mathcal{G}} \pi_\Gamma(\eta(t)),\qquad
\pi_\Omega(t) \preceq_{\mathcal{O}} \pi_\Omega(\eta(t)).
\]

Enrichment may also refine \(C\), \(P\), or \(\Delta\), provided it does not invalidate already established lexical or structural evidence.

Intuitively, enrichment makes a token more informative while preserving its identity and evidence base.

## 7. Semantic Conservation Principles

Let \(\succeq_{\mathcal{Sem}}\) denote the inverse order induced by \((\mathcal{Sem}, \preceq_{\mathcal{Sem}})\), so that \(\Sigma_1 \succeq_{\mathcal{Sem}} \Sigma_2\) means that \(\Sigma_1\) is at least as semantically informative as \(\Sigma_2\). Then STS adopts the following conservation principle for composition:

\[
\Sigma(t_1 \oplus t_2) \succeq_{\mathcal{Sem}} \Sigma(t_1) \cup \Sigma(t_2).
\]

This states that semantic composition should be monotone with respect to coverage: a composed token must not erase the meaning content of its constituents unless an explicitly lossy reduction has been applied.

The principle does not require simple set union in every implementation. Rather, it requires that the composed semantic interpretation dominate the constituent interpretations in an information order. This allows normalization, abstraction, and re-expression while disallowing untracked semantic loss.

The corresponding non-loss requirement is therefore:

\[
t_3 = t_1 \oplus t_2 \implies \Sigma(t_3) \not\prec_{\mathcal{Sem}} \Sigma(t_1) \cup \Sigma(t_2)
\]

unless composition is followed by an explicitly declared reduction \(\pi_r\).

### 7.1 Constraints on Conservation

Semantic conservation is constrained by at least three considerations:

1. **Contextual compatibility.** If \(C_1\) and \(C_2\) are incompatible, composition may be undefined.
2. **Pragmatic conflict.** If \(P_1\) and \(P_2\) impose incompatible discourse roles, semantic merger may degrade coherence.
3. **Temporal divergence.** If \(\Delta_1\) and \(\Delta_2\) refer to incompatible revisions or temporal scopes, composition may require explicit version mediation.

### 7.2 Coherence Degradation Bound

Given a monotone scalarization \(s_\Omega : \mathcal{O} \to [0,1]\), STF-SIR adopts the conservative coherence bound

\[
s_\Omega(\Omega(t_1 \oplus t_2)) \le \min(s_\Omega(\Omega(t_1)), s_\Omega(\Omega(t_2))).
\]

This inequality states that composition should not be assumed to improve coherence automatically. Combining units can preserve or degrade global coherence, but should not exceed the weaker constituent without additional evidence.

Because \(\Omega\) is a structured validation object rather than a raw scalar, the inequality is interpreted through the chosen order-preserving scalarization.

## 8. Relationship with STF-SIR v1

The current STF-SIR implementation realizes STS only partially. The correspondence is:

| STS dimension | Current v1 realization | Status |
| --- | --- | --- |
| \(L\) | lexical dimension in `ZToken` | implemented |
| \(S\) | syntactic dimension in `ZToken`, AST-derived structure | implemented |
| \(\Sigma\) | semantic dimension in `ZToken`, currently gloss-centered | implemented, minimal |
| \(\Gamma\) | `Artifact.relations`, `ZToken.logical`, `SirGraph.edges` | implemented |
| \(C\) | implicit context through source ordering, containment, and local adjacency | partially implemented |
| \(P\) | no explicit pragmatic field | future work |
| \(\Delta\) | version, config hash, provenance metadata, release tags | partially implemented |
| \(\Omega\) | validator, invariants, conformance suite, retention baseline | partially implemented, operational |

The current v1 system is therefore best understood as an implemented slice

\[
\mathcal{M}_{\mathrm{v1}} \subset \mathcal{L} \times \mathcal{S} \times \mathcal{Sem} \times \mathcal{G}
\]

with partial operational hooks into \(\mathcal{D}\) and \(\mathcal{O}\), and largely implicit or absent realizations of \(\mathcal{C}\) and \(\mathcal{P}\).

This is compatible with STS because STS is designed as a strict generalization, not a replacement.

## 9. Relationship with SirGraph

SirGraph is the graph realization of the relation-bearing portion of STS. More precisely, it is a projection of tokens into a graph determined primarily by \((\Sigma,\Gamma)\), while preserving token identity from the compiled unit.

Let \(\mathcal{A}\) be an STF-SIR artifact with token set \(T\) and relation set \(R\). Define a graph projection

\[
\Pi_{\mathrm{graph}} : \mathcal{M}_{\mathrm{STF}}^n \to \mathbf{Graph}
\]

such that

\[
\Pi_{\mathrm{graph}}(t_1,\dots,t_n) = (V,E),
\]

where:

- each \(v_i \in V\) corresponds to token \(t_i\),
- each \(e \in E\) corresponds to a relation in \(\Gamma\),
- node identity is inherited from the compiled token id,
- edge identity is inherited from the compiled relation id.

Operationally:

- SirGraph nodes represent compiled tokens,
- SirGraph edges represent realized logical relations,
- SirGraph is a projection of STS, not the entirety of STS.

In particular, SirGraph does not yet realize the full contextual, pragmatic, temporal, or coherence structure of STS. It is therefore accurate to say:

\[
\mathrm{SirGraph} \approx \Pi_{\Sigma,\Gamma}(\mathcal{M}_{\mathrm{STF}})
\]

where the approximation symbol reflects that node identity still comes from the compiled token as a whole, not from a pure semantic-logic quotient.

## 10. Relationship with Retention (\(\rho\))

At the token level, define the retention vector

\[
\rho(t) = (\rho_L(t), \rho_S(t), \rho_\Sigma(t), \rho_\Gamma(t))
\]

with each component in \([0,1]\).

Conceptually:

- \(\rho_L(t)\) measures how faithfully lexical evidence is preserved,
- \(\rho_S(t)\) measures how faithfully syntactic structure is preserved,
- \(\rho_\Sigma(t)\) measures how faithfully semantic content is preserved,
- \(\rho_\Gamma(t)\) measures how faithfully logical relations are preserved.

At the artifact level, the current implementation computes a deterministic aggregate baseline

\[
\overline{\rho}(\mathcal{A}) = \langle \overline{\rho}_L,\overline{\rho}_S,\overline{\rho}_\Sigma,\overline{\rho}_\Phi \rangle
\]

through `Artifact::retention_baseline()`. Here the implementation name `rho_phi` corresponds to the logical dimension \(\Gamma\) in the present formalization.

The important distinctions are:

1. the current implementation is a **baseline**, not the full STS retention theory;
2. it measures internal completeness and consistency, not benchmark-grounded semantic equivalence;
3. future retention should expand from \((L,S,\Sigma,\Gamma)\) to the full STS tuple \((L,S,\Sigma,\Gamma,C,P,\Delta,\Omega)\).

The coherence dimension \(\Omega\) is not measured directly by the current vector, but it is indirectly approximated through validator success, invariants satisfaction, and retention completeness.

## 11. Future Extensions

The remaining STS dimensions are intended to evolve as follows.

### 11.1 Context (\(C\))

The contextual dimension should become explicit rather than implicit. Likely future realizations include:

- cross-token dependency windows,
- section and discourse scope,
- reference frames and antecedent links,
- external context handles for retrieval or memory.

### 11.2 Pragmatics (\(P\))

The pragmatic dimension should model communicative role and discourse force. Possible future realizations include:

- assertion, question, command, hypothesis, warning,
- role-sensitive intent metadata,
- discourse-act classification linked to logical structure.

### 11.3 Temporal or Evolutionary State (\(\Delta\))

The temporal dimension should move beyond version metadata toward token-level or artifact-level evolution:

- semantic diff lineage,
- revision ancestry,
- timestamped validity intervals,
- branch-sensitive compilation provenance.

These extensions should remain monotone with respect to v1 artifacts. That is, they should enrich the STS realization without invalidating existing tokens, relations, or graphs.

## 12. Naming and Terminology

The STF-SIR terminology is fixed as follows.

| Term | Meaning |
| --- | --- |
| **STF** | Semantic Tokenization Framework: the architectural and methodological framework |
| **SIR** | Semantic Intermediate Representation: the compiled representation layer |
| **STS** | Semantic Token Space: the mathematical multidimensional space of semantic tokens |
| **ZToken** | The compiled semantic unit used by the implementation |
| **SirGraph** | The graph projection of compiled tokens and relations |

The relationship between these terms is hierarchical:

- STF provides the framework,
- SIR provides the representation layer,
- STS provides the mathematical foundation,
- ZToken provides the concrete compiled unit,
- SirGraph provides a graph realization of part of the representation.

Accordingly, STS should be understood as the formal v2 theoretical foundation of STF-SIR, while remaining backward-compatible with the operational v1 baseline.
