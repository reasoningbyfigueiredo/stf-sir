---
id: EPIC-102
title: Alias RecommendedEngine
version: 1.0.0
status: implementado
sdd: SDD-CORE-SEMANTICS
adr_ref: ADR-SEM-001 Implicação I-6
priority: alta
created: 2026-04-14
target: 2026-05-15
depends_on: []
blocks: []
---

# EPIC-102 — Alias RecommendedEngine

## Descrição

Introduzir um alias de tipo `RecommendedEngine` que canoniza a seleção de backends para o
`StfEngine`, e expô-lo como ponto de entrada primário na API pública e na CLI.

## Motivação

Consumidores da API atualmente escolhem backends manualmente ao construir `StfEngine<L, I, G>`.
Isso produz:

1. **Inconsistência semântica:** código diferente pode usar `SimpleLogicChecker` (texto) ou
   `FormulaCoherenceChecker` (AST), produzindo resultados diferentes para a mesma entrada.
2. **Superfície de API frágil:** a escolha de backend não é guiada; não há "caminho feliz" documentado.
3. **Divergência CLI/biblioteca:** a CLI pode usar um backend diferente do que o código de
   biblioteca padrão prescreve.

O alias `RecommendedEngine` resolve esses três problemas com uma mudança additive.

## Nota de Implementação: Restrição de Lifetime em `SirGroundingChecker`

O `SirGroundingChecker<'a>` requer uma referência de vida `'a` a um `SirGraph`, o que impede
seu uso em um alias de tipo estático simples. Por isso:

- O alias estático `RecommendedEngine` usa `ProvenanceGroundingChecker` (sem lifetime)
- Uma função construtora `recommended_engine_with_sir(graph: &SirGraph)` é provida separadamente
  para uso quando um `SirGraph` está disponível (caso de uso mais forte, conforme ADR-SEM-001 I-5)

O tipo `IdentityDomainMapper` referenciado na especificação original é aspiracional (v2).
Esta feature não o implementa.

## Escopo

- **Em escopo:** `src/compiler/engine.rs` (alias `RecommendedEngine`, construtor
  `recommended_engine`, `recommended_engine_with_sir`), `src/lib.rs` (reexportação),
  `src/cli.rs` (migração para `recommended_engine`), deprecação de `default_engine`.
- **Fora de escopo:** Remoção de `DefaultEngine` ou `default_engine`, implementação de
  `IdentityDomainMapper`, mudança na lógica de avaliação.

## Entregáveis

| # | Artefato | Caminho | Formato |
|---|---|---|---|
| D-102-1 | Alias e construtores | `src/compiler/engine.rs` | Rust |
| D-102-2 | Reexportação pública | `src/lib.rs` | Rust |
| D-102-3 | Migração da CLI | `src/cli.rs` | Rust |
| D-102-4 | Testes de consistência | `tests/coherence.rs` | Rust |

## Critérios de Sucesso

- [x] `RecommendedEngine` é acessível como `stf_sir::RecommendedEngine`
- [x] `recommended_engine()` constrói um engine com orçamento padrão explícito (não `usize::MAX`)
- [x] A CLI usa `recommended_engine()` por padrão
- [x] `DefaultEngine` e `default_engine()` são marcados `#[deprecated]`
- [x] Todos os caminhos de coerência produzem resultados idênticos via API e CLI para a mesma entrada
- [x] Todos os testes existentes passam

## Riscos

| ID | Risco | Mitigação |
|---|---|---|
| R-102-1 | `DefaultEngine` é usado em testes existentes por nome | Manter alias; adicionar `#[allow(deprecated)]` onde necessário nos testes |
| R-102-2 | Orçamento padrão de passos escolhido muito baixo — `C_c = Violated` em artefatos legítimos | Calibrar orçamento contra corpus golden; usar `n²` para n=1000 como baseline |
| R-102-3 | CLI muda silenciosamente de comportamento para artefatos grandes | Adicionar flag `--engine default` para forçar comportamento antigo durante transição |

---

## CONTRATO DO EPIC

```yaml
contract:
  id: CONTRACT-EPIC-102
  version: 1.0.0

  inputs:
    - id: I-102-1
      description: StfEngine genérico atual (src/compiler/engine.rs)
      required: true
    - id: I-102-2
      description: FormulaCoherenceChecker, FormulaInferenceEngine, ProvenanceGroundingChecker
      required: true
    - id: I-102-3
      description: CLI atual (src/cli.rs)
      required: true

  outputs:
    - id: O-102-1
      artifact: src/compiler/engine.rs
      constraint: |
        Contém RecommendedEngine, recommended_engine(), recommended_engine_with_sir(),
        e DefaultEngine marcado como deprecated
    - id: O-102-2
      artifact: src/lib.rs
      constraint: RecommendedEngine reexportado como pub
    - id: O-102-3
      artifact: src/cli.rs
      constraint: Usa recommended_engine() em vez de default_engine()

  invariants:
    - INV-102-1: |
        Para qualquer entrada E e configuração C:
        recommended_engine().evaluate_statement(T, S)
          == formula_engine_with_budget(ORÇAMENTO_PADRÃO).evaluate_statement(T, S)
        (RecommendedEngine é determinístico e consistente com FormulaEngine no mesmo orçamento)
    - INV-102-2: |
        RecommendedEngine É ACESSÍVEL como stf_sir::RecommendedEngine em código externo
        (verificado por teste de compilação de crate)
    - INV-102-3: |
        DefaultEngine CONTINUA COMPILANDO após a mudança
        (compatibilidade retroativa — sem remoção)

  preconditions:
    - PRE-102-1: FormulaCoherenceChecker e FormulaInferenceEngine são estáveis
    - PRE-102-2: Todos os testes passam em main antes da mudança

  postconditions:
    - POST-102-1: cargo test retorna 0
    - POST-102-2: cargo doc compila sem erros
    - POST-102-3: grep -r "default_engine" src/cli.rs retorna 0

  validation:
    automated:
      - script: cargo test --test coherence recommended_engine
        description: Testes de consistência do RecommendedEngine
        asserts: [POST-102-1]
      - script: cargo doc --no-deps
        description: Documentação compila
        asserts: [POST-102-2]
      - script: "! grep -q 'default_engine' src/cli.rs"
        description: CLI migrada para recommended_engine
        asserts: [POST-102-3]

  metrics:
    - metric: api_surface_stability
      formula: (tipos_publicos_removidos)
      target: 0
      measurement: cargo semver-checks
    - metric: cli_engine_consistency
      formula: |
        diferenca_entre_saida_api_e_cli_para_mesma_entrada
      target: 0
      measurement: teste de integração IT-102-1

  failure_modes:
    - FAIL-102-1:
        condition: INV-102-1 violado (resultados divergentes entre recommended_engine e formula_engine no mesmo orçamento)
        action: Investigar parâmetros do construtor; bloquear merge
        severity: critical
    - FAIL-102-2:
        condition: RecommendedEngine não acessível externamente (INV-102-2 violado)
        action: Verificar reexportação em src/lib.rs; bloquear PR
        severity: error
    - FAIL-102-3:
        condition: CLI ainda usa default_engine após migração
        action: Corrigir import em src/cli.rs
        severity: error
```

---

## Features

### FEAT-102-1: Definição de `RecommendedEngine` e construtores

**Descrição:** Adicionar o alias de tipo `RecommendedEngine` e as funções construtoras
em `src/compiler/engine.rs`.

**Comportamento:**
```rust
/// Motor canônico recomendado para uso em produção.
///
/// Usa backends baseados em AST de fórmulas para coerência lógica e inferência,
/// com verificação de fundamentação por proveniência e orçamento de passos explícito.
///
/// Para uso com SirGraph, use `recommended_engine_with_sir`.
pub type RecommendedEngine = StfEngine<
    FormulaCoherenceChecker,
    FormulaInferenceEngine,
    ProvenanceGroundingChecker,
>;

/// Orçamento de passos padrão para o RecommendedEngine.
/// Calibrado para artefatos com até ~1000 tokens (n² pares = 1_000_000 passos).
pub const RECOMMENDED_STEP_BUDGET: usize = 1_000_000;

/// Constrói o RecommendedEngine com orçamento de passos padrão.
pub fn recommended_engine() -> RecommendedEngine {
    StfEngine {
        logic: FormulaCoherenceChecker,
        inference: FormulaInferenceEngine,
        grounding: ProvenanceGroundingChecker,
        step_budget: RECOMMENDED_STEP_BUDGET,
    }
}

/// Constrói o RecommendedEngine com orçamento de passos explícito.
pub fn recommended_engine_with_budget(budget: usize) -> RecommendedEngine {
    StfEngine {
        logic: FormulaCoherenceChecker,
        inference: FormulaInferenceEngine,
        grounding: ProvenanceGroundingChecker,
        step_budget: budget,
    }
}
```

**Construtores para SirGraph (lifetime-bound, sem alias estático):**
```rust
/// Constrói um engine com fundamentação baseada em grafo SIR.
/// Retorna um StfEngine concreto (não um alias) por limitação de lifetime.
pub fn recommended_engine_with_sir(
    graph: &SirGraph,
) -> StfEngine<FormulaCoherenceChecker, FormulaInferenceEngine, SirGroundingChecker<'_>> {
    StfEngine {
        logic: FormulaCoherenceChecker,
        inference: FormulaInferenceEngine,
        grounding: SirGroundingChecker { graph },
        step_budget: RECOMMENDED_STEP_BUDGET,
    }
}
```

**Deprecação de `DefaultEngine` e `default_engine`:**
```rust
/// Motor padrão — DEPRECIADO. Use `RecommendedEngine` e `recommended_engine()`.
///
/// Este alias usa backends baseados em texto (SimpleLogicChecker, RuleBasedInferenceEngine)
/// que são menos precisos do que os backends de AST de fórmulas.
/// Mantido para compatibilidade retroativa; será removido em v2.0.0.
#[deprecated(since = "1.1.0", note = "Use RecommendedEngine e recommended_engine()")]
pub type DefaultEngine = StfEngine<
    SimpleLogicChecker,
    RuleBasedInferenceEngine,
    ProvenanceGroundingChecker,
>;

#[deprecated(since = "1.1.0", note = "Use recommended_engine()")]
pub fn default_engine() -> DefaultEngine { ... }
```

**Restrições:**
- `DefaultEngine` e `default_engine` NÃO devem ser removidos
- O orçamento `RECOMMENDED_STEP_BUDGET` deve ser documentado com a justificativa de calibração
- `recommended_engine_with_sir` deve estar no mesmo módulo mas NÃO no alias de tipo

**Contrato da Feature:**
```yaml
contract:
  id: CONTRACT-FEAT-102-1
  inputs:
    - src/compiler/engine.rs (StfEngine, FormulaCoherenceChecker, FormulaInferenceEngine,
      ProvenanceGroundingChecker, SirGroundingChecker)
  outputs:
    - src/compiler/engine.rs (RecommendedEngine, RECOMMENDED_STEP_BUDGET,
      recommended_engine, recommended_engine_with_budget, recommended_engine_with_sir)
  invariants:
    - INV-102-1: recommended_engine() == formula_engine_with_budget(RECOMMENDED_STEP_BUDGET)
    - INV-102-3: DefaultEngine ainda compila
  postconditions:
    - Todos os símbolos compilam sem erros
    - Documentação doc-comment presente em todos os itens públicos
  failure_modes:
    - FAIL-102-1: Divergência de resultado → verificar parâmetros do construtor
```

#### Tarefas

**TASK-102-1-1: Adicionar `RECOMMENDED_STEP_BUDGET` e `RecommendedEngine`**

- **Feature vinculada:** FEAT-102-1
- **Escopo de arquivo:** `src/compiler/engine.rs`, após linha de `FormulaEngine`
- **Descrição:** Adicionar constante e alias de tipo conforme especificado
- **Critério de aceitação:** `cargo build` passa; `cargo doc` renderiza corretamente
- **Requisitos de teste:** Compilação

**TASK-102-1-2: Adicionar `recommended_engine`, `recommended_engine_with_budget`, `recommended_engine_with_sir`**

- **Feature vinculada:** FEAT-102-1
- **Escopo de arquivo:** `src/compiler/engine.rs`
- **Descrição:** Adicionar as três funções construtoras com doc-comments
- **Critério de aceitação:** Funções são chamáveis e produzem engines funcionais
- **Requisitos de teste:** UT-102-1 (TASK-102-1-4)

**TASK-102-1-3: Marcar `DefaultEngine` e `default_engine` como `#[deprecated]`**

- **Feature vinculada:** FEAT-102-1
- **Escopo de arquivo:** `src/compiler/engine.rs`
- **Descrição:** Adicionar atributo `#[deprecated]` com mensagem de migração
- **Critério de aceitação:** `cargo clippy` emite aviso de depreciação; nenhum erro
- **Requisitos de teste:** Verificação de clippy em CI

**TASK-102-1-4: Adicionar testes unitários de construção**

- **Feature vinculada:** FEAT-102-1
- **Escopo de arquivo:** `tests/coherence.rs`
- **Descrição:** Testes que constroem `RecommendedEngine`, `recommended_engine()`,
  `recommended_engine_with_budget(n)`, `recommended_engine_with_sir(&graph)`
  e executam `evaluate_statement` com uma teoria minimal
- **Critério de aceitação:** Todos os construtores compilam e executam sem pânico

---

### FEAT-102-2: Reexportação pública e migração da CLI

**Descrição:** Expor `RecommendedEngine` e `recommended_engine` no crate público e migrar
a CLI para usar o novo construtor.

**Comportamento em `src/lib.rs`:**
```rust
pub use compiler::{
    RecommendedEngine, recommended_engine, recommended_engine_with_budget,
    recommended_engine_with_sir, RECOMMENDED_STEP_BUDGET,
    // manter por compatibilidade:
    DefaultEngine, EvaluationResult, FormulaEngine,
};
```

**Comportamento em `src/cli.rs`:**
A CLI deve substituir qualquer chamada a `default_engine()` por `recommended_engine()`.

**Restrições:**
- O comportamento da CLI muda: C_c pode agora ser `Satisfied` ou `Violated` em vez de
  sempre `Unknown`. Documentar no CHANGELOG.

**Contrato da Feature:**
```yaml
contract:
  id: CONTRACT-FEAT-102-2
  inputs:
    - src/lib.rs (reexportações atuais)
    - src/cli.rs (uso de default_engine)
  outputs:
    - src/lib.rs (RecommendedEngine e construtores exportados)
    - src/cli.rs (usa recommended_engine)
  invariants:
    - INV-102-2: RecommendedEngine acessível externamente
  postconditions:
    - grep 'pub use compiler::.*RecommendedEngine' src/lib.rs retorna match
    - grep 'recommended_engine()' src/cli.rs retorna match
  failure_modes:
    - FAIL-102-2: RecommendedEngine não exportado → verificar src/lib.rs
    - FAIL-102-3: CLI ainda usa default_engine → verificar src/cli.rs
```

#### Tarefas

**TASK-102-2-1: Atualizar `src/lib.rs` com reexportações**

- **Feature vinculada:** FEAT-102-2
- **Escopo de arquivo:** `src/lib.rs`
- **Descrição:** Adicionar `RecommendedEngine`, `recommended_engine`,
  `recommended_engine_with_budget`, `recommended_engine_with_sir`, `RECOMMENDED_STEP_BUDGET`
  à linha `pub use compiler::...`
- **Critério de aceitação:** `use stf_sir::RecommendedEngine` compila em crate externa (teste de compilação)
- **Requisitos de teste:** UT-102-2

**TASK-102-2-2: Migrar `src/cli.rs` para `recommended_engine()`**

- **Feature vinculada:** FEAT-102-2
- **Escopo de arquivo:** `src/cli.rs`
- **Descrição:** Localizar todas as chamadas a `default_engine()` e substituir por `recommended_engine()`
- **Critério de aceitação:** `! grep -q 'default_engine' src/cli.rs` (salvo comentários)
- **Requisitos de teste:** IT-102-1

---

## Plano de Testes

### Testes Unitários

| ID | Arquivo | Função | Caso | Resultado esperado |
|---|---|---|---|---|
| UT-102-1 | `tests/coherence.rs` | `recommended_engine_builds_and_evaluates` | Construir e avaliar enunciado minimal | Nenhum pânico; resultado válido |
| UT-102-2 | `tests/coherence.rs` | `recommended_engine_is_public` | `use stf_sir::RecommendedEngine` | Compila sem erro |
| UT-102-3 | `tests/coherence.rs` | `recommended_engine_budget_is_explicit` | `recommended_engine().step_budget != usize::MAX` | True |
| UT-102-4 | `tests/coherence.rs` | `recommended_engine_consistent_with_formula_engine` | Mesma entrada para `recommended_engine` e `formula_engine_with_budget(RECOMMENDED_STEP_BUDGET)` | Resultados idênticos |
| UT-102-5 | `tests/coherence.rs` | `deprecated_default_engine_still_compiles` | `#[allow(deprecated)] let _ = default_engine()` | Compila com aviso |

### Testes de Integração

| ID | Arquivo | Função | Descrição |
|---|---|---|---|
| IT-102-1 | `tests/coherence.rs` | `cli_and_api_produce_same_coherence_result` | Compilar artefato golden, avaliar via API (`recommended_engine`) e simular avaliação CLI; verificar que `coherence.label()` é idêntico |

### Testes Adversariais

| ID | Arquivo | Função | Caso adversarial |
|---|---|---|---|
| ADV-102-1 | `tests/coherence.rs` | `recommended_engine_large_theory_budget_respected` | Teoria com 1500 enunciados: `C_c` deve ser `Violated` (excede `RECOMMENDED_STEP_BUDGET`) |
| ADV-102-2 | `tests/coherence.rs` | `recommended_engine_with_sir_uses_graph_grounding` | Enunciado cujo id existe no SirGraph mas sem proveniência: deve ser grounded via `SirGroundingChecker` |

### Testes de Regressão

| ID | Arquivo | Função | Proteção |
|---|---|---|---|
| REG-102-1 | `tests/golden.rs` | (existente) | Golden gate: saída do compilador não muda |
| REG-102-2 | `tests/coherence.rs` | (existente) | Todos os testes de coerência existentes passam mesmo com `DefaultEngine` deprecado |
