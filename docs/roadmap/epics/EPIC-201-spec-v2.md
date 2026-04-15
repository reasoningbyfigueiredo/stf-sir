---
id: EPIC-201
title: Spec v2
version: 2.0.0-alpha
status: implementado
roadmap: ROADMAP-STF-SIR-V2
priority: critical
created: 2026-04-12
target: 2026-06-01
depends_on: []
blocks:
  - EPIC-202
  - EPIC-207
---

# EPIC-201 — Spec v2

## Description

Produce the normative STF-SIR v2 specification document that extends v1 with:

1. The four deferred STS dimensions (C — contextual, P — pragmatic, Δ — temporal, Ω — coherence/validation)
2. A v2 ZToken structure compatible with the formal STS multidimensional space
3. Enricher trait specification (monotone semantic enrichment contract)
4. Extended relation taxonomy (adds `supports`, `refers_to`, `contradicts`, `elaborates`, `cites`)
5. Sentence-level and entity-level profiles (alongside the existing block-level profile)
6. Language detection requirement (no more unconditional `"und"`)
7. Backward-compatibility guarantees: v1 `.zmd` must validate against a v2 reader

## Scope

- **In scope:** Normative spec text, formal grammar for ZToken v2, updated validation rules (VAL_01–VAL_30), conformance kit v2 specification, profile taxonomy
- **Out of scope:** Implementation in Rust (that is EPIC-207), embedding or vector fields (those are EPIC-206)

## Deliverables

| # | Artifact | Path | Format |
|---|---|---|---|
| D-201-1 | STF-SIR Spec v2 document | `spec/stf-sir-spec-v2.md` | Markdown (normative) |
| D-201-2 | ZToken v2 formal grammar | `spec/ztoken-v2-grammar.ebnf` | EBNF |
| D-201-3 | Validation rules v2 table | (embedded in D-201-1, §9) | Markdown table |
| D-201-4 | Relation taxonomy v2 table | (embedded in D-201-1, §5) | Markdown table |
| D-201-5 | Profile registry v2 | `spec/profiles-v2.md` | Markdown |
| D-201-6 | Conformance kit v2 spec | (embedded in D-201-1, §10) | Markdown |
| D-201-7 | Backward-compat migration guide | `spec/migration-v1-to-v2.md` | Markdown |

## Success Criteria

- [x] All v1 validation rules (VAL_01–VAL_18) preserved verbatim or superseded by a strict superset
- [x] New dimensions C, P, Δ, Ω are formally defined with typed fields and invariants
- [x] Enricher trait is formally specified with monotonicity proof sketch
- [x] Language detection is a MUST requirement (not a SHOULD)
- [x] Five new relation types are defined with their categories and stages
- [x] Sentence-level profile is defined with its own node_type set
- [ ] Two independent reviewers sign off spec document (git-signed commits)

## Risks

| ID | Risk | Mitigation |
|---|---|---|
| R-201-1 | STS C/P/Δ/Ω semantics are underspecified in the formalization paper | Schedule spec review with paper author before D-201-1 draft |
| R-201-2 | New relation types conflict with existing `structural`/`logical` categories | Define third category `semantic-link` emitters before taxonomy freeze |
| R-201-3 | Backward-compat rule is too strict, blocking valid v2 extensions | Allow additive-only changes under `version: 2` gate; document exceptions |

---

## EPIC CONTRACT

```yaml
contract:
  id: CONTRACT-EPIC-201
  version: 1.0.0

  inputs:
    - id: I-201-1
      description: STF-SIR v1 spec (spec/stf-sir-spec-v1.md)
      required: true
    - id: I-201-2
      description: STS formalization paper (docs/sts-formalization.md)
      required: true
    - id: I-201-3
      description: STS paper (docs/sts-paper.pdf)
      required: true
    - id: I-201-4
      description: v1 validation rules VAL_01–VAL_18
      required: true

  outputs:
    - id: O-201-1
      artifact: spec/stf-sir-spec-v2.md
      schema: normative-markdown-spec
    - id: O-201-2
      artifact: spec/ztoken-v2-grammar.ebnf
    - id: O-201-3
      artifact: spec/profiles-v2.md
    - id: O-201-4
      artifact: spec/migration-v1-to-v2.md

  invariants:
    - INV-201-1: |
        Every v1 validation rule ID (VAL_01–VAL_18) MUST appear verbatim or as a
        superseding rule in spec v2. No rule may be silently dropped.
    - INV-201-2: |
        The v2 ZToken structure MUST be a strict superset of v1: all v1 fields are
        present with identical names and types; new fields are optional or versioned.
    - INV-201-3: |
        The Enricher trait contract MUST satisfy monotonicity: applying an enricher
        to an artifact A yields A' such that A ⊑ A' (no existing field value is removed
        or overwritten with a weaker value).
    - INV-201-4: |
        All five new relation types MUST be assigned to exactly one category and one
        valid stage from the extended taxonomy.

  preconditions:
    - PRE-201-1: v1 spec is published at spec/stf-sir-spec-v1.md
    - PRE-201-2: STS formalization paper is present and reviewed
    - PRE-201-3: All v1 golden tests pass (CI green on main)

  postconditions:
    - POST-201-1: spec/stf-sir-spec-v2.md exists, passes markdown lint, word count ≥ 5000
    - POST-201-2: spec/ztoken-v2-grammar.ebnf is parseable by standard EBNF tooling
    - POST-201-3: migration guide covers all breaking changes with examples
    - POST-201-4: CI lint step validates spec structure (section numbering, rule IDs)

  validation:
    automated:
      - script: scripts/lint-spec.sh spec/stf-sir-spec-v2.md
        description: Checks section numbering, rule ID format, mandatory sections
      - script: scripts/check-ebnf.sh spec/ztoken-v2-grammar.ebnf
        description: Parses EBNF and validates non-terminal coverage
    manual:
      - review: Two independent reviewers must approve spec PR
      - review: Author cross-references STS paper §3 for each new dimension

  metrics:
    - metric: spec_completeness
      formula: (rules_defined / rules_required) * 100
      target: 100%
    - metric: backward_compat_coverage
      formula: (v1_rules_preserved / 18) * 100
      target: 100%

  failure_modes:
    - FAIL-201-1:
        condition: INV-201-1 violated (dropped v1 rule)
        action: Block EPIC-202 gating; reopen EPIC-201
    - FAIL-201-2:
        condition: EBNF grammar is ambiguous
        action: Flag as spec defect; resolve before EPIC-207 begins
```

---

## Features

### FEAT-201-1: ZToken v2 Formal Structure

**Description:** Define the extended ZToken structure `z = <L, S, Σ, Φ, C, P, Δ, Ω>` with
formal field types, constraints, and optionality rules for each new dimension.

**Inputs:**
- STS formalization (all 8 dimensions)
- v1 ZToken structure (4 dimensions, all MUST fields)

**Outputs:**
- ZToken v2 type table (embedded in spec §3)
- EBNF grammar for ZToken v2

**Acceptance Criteria:**
- [ ] C (contextual) dimension defined: `context_id`, `scope`, `reference_frame` (all optional for block profile)
- [ ] P (pragmatic) dimension defined: `intent`, `speech_act`, `register` (all optional for block profile)
- [ ] Δ (temporal) dimension defined: `created_at`, `modified_at`, `valid_from`, `valid_to` (all optional)
- [ ] Ω (coherence) dimension defined: `coherence_score` f64 [0,1], `validation_flags` Vec<String>
- [ ] All new fields have `minimum`, `maximum`, `pattern` constraints in JSON Schema
- [ ] EBNF grammar covers all 8 dimensions unambiguously

**Metrics:**
- Grammar ambiguity: 0 ambiguous productions (verified by parser)
- Field coverage: 100% of STS paper dimensions represented

**Feature Contract:**
```yaml
contract:
  id: CONTRACT-FEAT-201-1
  inputs: [STS paper §2–§4, v1 ZToken fields]
  outputs: [ZToken v2 type table, EBNF grammar]
  invariants:
    - All v1 field names unchanged
    - New dimensions are optional at block profile
  postconditions:
    - EBNF grammar parsed successfully by external tool
    - All 8 dimensions have at least one defined field
  failure_modes:
    - ambiguous grammar → block EPIC-202 schema work
```

#### Tasks

**TASK-201-1-1: Draft C, P, Δ, Ω dimension field tables**
- Description: Write field-by-field tables for all four new dimensions, drawing directly from STS formalization §3–§6
- Definition of done: Table renders in markdown, each field has name/type/required/description/example
- Testability: CI markdown lint passes; manual review confirms STS paper traceability
- Artifacts: Spec section §3.5–§3.8 draft
- Contract: Input = STS paper, Output = 4 field tables, Invariant = no v1 field conflicts

**TASK-201-1-2: Write EBNF grammar for ZToken v2**
- Description: Extend the block-level grammar to include all 8 dimensions with proper optionality
- Definition of done: Grammar file at `spec/ztoken-v2-grammar.ebnf` passes `scripts/check-ebnf.sh`
- Testability: Script parses file and reports 0 errors; all non-terminals reachable
- Artifacts: `spec/ztoken-v2-grammar.ebnf`
- Contract: Input = field tables from TASK-201-1-1, Output = valid EBNF, Invariant = v1 productions preserved

**TASK-201-1-3: Define profile-specific field optionality rules**
- Description: Document which fields are MUST/SHOULD/MAY for block, sentence, and entity profiles
- Definition of done: Three-column optionality table (block/sentence/entity) for every ZToken field
- Testability: Table coverage = 100% of defined fields; manual review
- Artifacts: Spec §4 (profiles) optionality annex

---

### FEAT-201-2: Validation Rules v2 (VAL_01–VAL_30)

**Description:** Extend the v1 validation rule set from 18 to ~30 rules, adding rules for new
dimensions, new relation types, and profile-specific constraints.

**Inputs:**
- v1 rules VAL_01–VAL_18
- New dimension fields from FEAT-201-1
- New relation types from FEAT-201-3

**Outputs:**
- Complete validation rules table in spec §9
- Machine-readable rules index (YAML)

**Acceptance Criteria:**
- [ ] All 18 v1 rules preserved with same codes
- [ ] New rules VAL_19–VAL_30 defined with codes, descriptions, error categories
- [ ] Each new dimension has at least one validation rule
- [ ] Rules are uniquely coded, no gaps, no duplicates
- [ ] Machine-readable index at `spec/validation-rules-v2.yaml`

**Feature Contract:**
```yaml
contract:
  id: CONTRACT-FEAT-201-2
  inputs: [v1 rules, new dimensions, new relation types]
  outputs: [complete rules table, spec/validation-rules-v2.yaml]
  invariants:
    - VAL_01–VAL_18 codes unchanged
    - No two rules share a code
  postconditions:
    - YAML file parses without error
    - CI lint script validates rule ID sequence
  failure_modes:
    - duplicate rule code → spec defect, block EPIC-207
```

#### Tasks

**TASK-201-2-1: Audit v1 rules for completeness gaps**
- Description: Cross-reference all 18 v1 rules against the v2 field set; identify uncovered fields
- Definition of done: Gap analysis document listing each field without a rule
- Testability: Manual + automated count of fields vs rule references
- Artifacts: Gap analysis in spec §9 annex

**TASK-201-2-2: Write VAL_19–VAL_30 rules**
- Description: Cover C/P/Δ/Ω dimension invariants, language detection requirement, new relation validation
- Definition of done: Each rule has: code, severity, description, example-valid, example-invalid
- Testability: examples compile/fail as expected using EPIC-207 implementation
- Artifacts: spec §9 extension table; `spec/validation-rules-v2.yaml`

**TASK-201-2-3: Write machine-readable rules index**
- Description: Export all 30 rules as structured YAML for consumption by audit pipeline
- Definition of done: `spec/validation-rules-v2.yaml` passes `yamllint`, all required fields present
- Testability: `scripts/validate-rules-index.sh` exits 0
- Artifacts: `spec/validation-rules-v2.yaml`

---

### FEAT-201-3: Extended Relation Taxonomy

**Description:** Define five new relation types (`supports`, `refers_to`, `contradicts`,
`elaborates`, `cites`) with categories, stages, directionality rules, and validation constraints.

**Inputs:**
- v1 relation taxonomy (contains, precedes)
- STS paper §5 (logical relations)
- SemanticLink category (defined in v1 schema but never emitted)

**Outputs:**
- Relation taxonomy table (spec §5.4)
- Updated relation JSON Schema fragment

**Acceptance Criteria:**
- [ ] Each new type assigned to exactly one category and valid stage set
- [ ] Directionality (source/target role) documented for each type
- [ ] `semantic-link` category emitters defined for `refers_to`, `cites`
- [ ] `logical` category emitters defined for `supports`, `contradicts`, `elaborates`
- [ ] JSON Schema updated to include new type enums

**Feature Contract:**
```yaml
contract:
  id: CONTRACT-FEAT-201-3
  inputs: [v1 taxonomy, STS §5, SemanticLink category]
  outputs: [taxonomy table, schema fragment]
  invariants:
    - v1 relation types (contains, precedes) unchanged
    - Every type has exactly one category
  postconditions:
    - Schema fragment passes JSON Schema meta-validation
  failure_modes:
    - type assigned to multiple categories → spec defect
```

#### Tasks

**TASK-201-3-1: Define new relation type specifications**
- Description: Write per-type specification (name, category, stage, source-role, target-role, description, example)
- Definition of done: 5 type specs written, peer-reviewed
- Artifacts: spec §5.4 table

**TASK-201-3-2: Write JSON Schema fragment for new types**
- Description: Extend the `type` enum in `zmd-v2.schema.json` and add per-type constraints
- Definition of done: Schema fragment is valid JSON Schema Draft 2020-12
- Testability: `scripts/validate-schema-fragment.sh` exits 0
- Artifacts: Schema fragment in `schemas/relations-v2-fragment.json`

---

### FEAT-201-4: Conformance Kit v2 Specification

**Description:** Specify the v2 conformance kit: structure, fixture categories, required fixture
counts, and pass/fail criteria for independent implementations.

**Inputs:**
- v1 conformance kit (10 valid + 10 invalid fixtures)
- v2 spec from FEAT-201-1/2/3

**Outputs:**
- Conformance kit v2 specification (spec §10)
- Fixture category registry

**Acceptance Criteria:**
- [ ] Spec defines minimum fixture counts (≥ 20 valid, ≥ 20 invalid)
- [ ] New fixture categories for: new dimensions, new relation types, sentence profile, entity profile, language detection
- [ ] Pass/fail criteria machine-checkable (exit codes, output diff)
- [ ] External impl guide included

**Feature Contract:**
```yaml
contract:
  id: CONTRACT-FEAT-201-4
  inputs: [v1 conformance kit structure, v2 spec]
  outputs: [conformance kit v2 spec, fixture registry]
  invariants:
    - All v1 fixture categories preserved
  postconditions:
    - Spec §10 present, parseable, reviewed
  failure_modes:
    - Undefined fixture category → external impls non-comparable
```

#### Tasks

**TASK-201-4-1: Define fixture category taxonomy**
- Description: List all fixture categories for v2, with min-count and description
- Artifacts: Fixture registry table in spec §10.1

**TASK-201-4-2: Write external implementation guide**
- Description: How a third-party impl registers and runs the conformance kit
- Artifacts: spec §10.4 external impl guide; `spec/migration-v1-to-v2.md`

---

### FEAT-201-5: Materialização de `SemanticDimensions` em Rust {#feat-201-5}

> **Referência:** SDD-CORE-SEMANTICS (prioridade BAIXA) — ADR-SEM-001 Posicionamento Matemático (estruturas aspiracionais)

**Descrição:** Definir e implementar a struct `SemanticDimensions` em `src/model/` que
materializa as quatro dimensões semânticas C/P/Δ/Ω como objeto Rust de primeira classe,
pronta para ser anexada a `Statement` e propagada pelo pipeline.

**Comportamento:**

```rust
/// Dimensões semânticas de segunda ordem para um Statement ou ZToken.
///
/// Materializa os quatro eixos C/P/Δ/Ω do espaço STS:
///
/// - `coherence` (C): vetor de coerência (C_l, C_c, C_o) — implementado
/// - `provenance_score` (P): score escalar de qualidade da proveniência — implementado
/// - `transformation_delta` (Δ): distância semântica da fonte original — parcialmente implementado
/// - `execution_score` (Ω): score de executabilidade operacional — parcialmente implementado
///
/// Esta struct é ASPIRACIONAL no sentido de que `transformation_delta` e
/// `execution_score` são aproximações; consulte os comentários dos campos.
#[derive(Debug, Clone, PartialEq)]
pub struct SemanticDimensions {
    /// C — coerência: o vetor (C_l, C_c, C_o) avaliado pelo StfEngine.
    /// Implementado: src/model/coherence.rs, src/compiler/engine.rs
    pub coherence: CoherenceVector,

    /// P — proveniência: score escalar ∈ [0.0, 1.0] derivado de Provenance.
    ///
    /// Cálculo: 1.0 se grounded=true ∧ source_ids não vazio ∧ anchors não vazio;
    ///          0.5 se apenas grounded=true ou apenas source_ids não vazio;
    ///          0.0 se nenhuma proveniência.
    /// [parcialmente implementado — heurística simples]
    pub provenance_score: f32,

    /// Δ — transformação: distância semântica normalizada da fonte.
    ///
    /// Em v1: sempre 0.0 (sem transformação — compilação direta da fonte).
    /// Em v2: derivado de SemanticDiff (EPIC-204) quando disponível.
    /// [aspiracional para v2]
    pub transformation_delta: f32,

    /// Ω — execução: score de executabilidade operacional ∈ [0.0, 1.0].
    ///
    /// Cálculo: 1.0 se C_o = Satisfied; 0.0 se C_o = Violated; 0.5 se Unknown.
    /// [parcialmente implementado — baseado em C_o]
    pub execution_score: f32,
}

impl SemanticDimensions {
    /// Constrói SemanticDimensions a partir de um EvaluationResult.
    pub fn from_evaluation(result: &EvaluationResult) -> Self {
        let execution_score = match result.coherence.operational {
            TruthValue::Satisfied => 1.0,
            TruthValue::Violated  => 0.0,
            TruthValue::Unknown   => 0.5,
        };
        Self {
            coherence: result.coherence.clone(),
            provenance_score: if result.grounded { 1.0 } else { 0.0 },
            transformation_delta: 0.0,  // v1: sem transformação
            execution_score,
        }
    }

    /// Retorna true se todas as dimensões implementadas indicam estado saudável.
    pub fn is_healthy(&self) -> bool {
        self.coherence.is_full()
            && self.provenance_score >= 0.5
            && self.execution_score >= 0.5
    }
}
```

**Propagação pelo pipeline:**

A struct é ADDITIVE — não substitui campos existentes de `Statement`:

```rust
// Em src/model/statement.rs — campo opcional, sem breaking change:
pub struct Statement {
    // ... campos existentes ...
    /// Dimensões semânticas de segunda ordem (calculadas pelo StfEngine).
    /// None se o Statement não foi avaliado pelo engine.
    pub semantic_dimensions: Option<SemanticDimensions>,
}
```

**Restrições:**
- `transformation_delta` DEVE ser `0.0` em v1 (sem lógica de diff disponível)
- `SemanticDimensions` é OPCIONAL em `Statement` — nenhum código existente é quebrado
- A struct DEVE ser documentada com o status de implementação de cada campo
- O campo `[aspiracional]` em `transformation_delta` deve ser legível em `cargo doc`

**Saída esperada:**
- `SemanticDimensions` acessível como `stf_sir::SemanticDimensions`
- `SemanticDimensions::from_evaluation` integra com `StfEngine` existente
- `Statement.semantic_dimensions` é `Option<SemanticDimensions>`

**Contrato da Feature:**
```yaml
contract:
  id: CONTRACT-FEAT-201-5
  version: 1.0.0
  inputs:
    - src/model/coherence.rs (CoherenceVector, TruthValue)
    - src/compiler/engine.rs (EvaluationResult)
    - src/model/statement.rs (Statement)
  outputs:
    - src/model/semantic_dimensions.rs (SemanticDimensions struct)
    - src/model/statement.rs (campo semantic_dimensions: Option<SemanticDimensions>)
    - src/model/mod.rs (reexportação)
    - src/lib.rs (reexportação pública)
  invariants:
    - INV-201-5: |
        SemanticDimensions::from_evaluation(r).coherence == r.coherence
        para todo EvaluationResult r.
    - INV-201-6: |
        Statement.semantic_dimensions = None NÃO impede inserção em Theory.
        (campo opcional não é breaking)
    - INV-201-7: |
        transformation_delta == 0.0 em todas as construções via from_evaluation em v1.
  preconditions:
    - PRE-201-5: EvaluationResult é estável (EPIC-101 fechado ou independente)
    - PRE-201-6: Todos os testes existentes passam em main
  postconditions:
    - POST-201-5: cargo test retorna 0
    - POST-201-6: SemanticDimensions acessível como stf_sir::SemanticDimensions
    - POST-201-7: Statement compila com campo semantic_dimensions = None por padrão
  validation:
    automated:
      - script: cargo test --test coherence semantic_dimensions
        description: INV-201-5, INV-201-6, INV-201-7
        asserts: [POST-201-5]
      - script: cargo doc --no-deps
        description: Documentação com status aspiracional visível
        asserts: [POST-201-6]
  metrics:
    - metric: retrocompatibilidade
      formula: testes_existentes_falhando
      target: 0
      measurement: cargo test (suíte completa)
  failure_modes:
    - FAIL-201-5:
        condition: INV-201-5 violado (coherence não copiado corretamente)
        action: Verificar from_evaluation; bloquear merge
        severity: critical
    - FAIL-201-6:
        condition: Statement não compila após adição do campo
        action: Verificar Default impl; o campo deve ser Option com default None
        severity: error
```

#### Tarefas

**TASK-201-5-1: Criar `src/model/semantic_dimensions.rs`**
- **Descrição:** Implementar `SemanticDimensions` e `SemanticDimensions::from_evaluation`
  conforme especificado. Incluir doc-comments com status de implementação por campo.
- **Critério de aceitação:** Arquivo compila; `cargo doc` renderiza sem warning
- **Requisitos de teste:** UT-201-5-1

**TASK-201-5-2: Adicionar campo `semantic_dimensions` a `Statement`**
- **Descrição:** Adicionar `pub semantic_dimensions: Option<SemanticDimensions>` a `Statement`.
  Atualizar `Statement::atomic`, `Statement::grounded`, `Statement::with_formula` para incluir
  `semantic_dimensions: None` nos construtores (sem breaking change).
- **Critério de aceitação:** Todos os construtores existentes compilam; `Default` funciona
- **Requisitos de teste:** UT-201-5-2

**TASK-201-5-3: Reexportar em `src/model/mod.rs` e `src/lib.rs`**
- **Descrição:** Adicionar `pub mod semantic_dimensions` e reexportação `pub use semantic_dimensions::SemanticDimensions`
- **Critério de aceitação:** `use stf_sir::SemanticDimensions` compila em crate externa
- **Requisitos de teste:** Compilação

**TASK-201-5-4: Adicionar testes de `SemanticDimensions`**
- **Escopo de arquivo:** `tests/coherence.rs`
- **Descrição:**
  - UT-201-5-1: `from_evaluation` preserva `CoherenceVector`
  - UT-201-5-2: `transformation_delta == 0.0` em v1
  - UT-201-5-3: `Statement` com `semantic_dimensions = None` insere em `Theory` sem erro
  - UT-201-5-4: `is_healthy()` retorna false quando `C_l = Violated`
- **Critério de aceitação:** 4 casos cobertos; falham ao reverter a struct
