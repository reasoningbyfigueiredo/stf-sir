---
id: EPIC-101
title: Informação Útil com Fundamentação
version: 1.0.0
status: implementado
sdd: SDD-CORE-SEMANTICS
adr_ref: ADR-SEM-001 Regra 3.2
priority: critica
created: 2026-04-14
target: 2026-05-01
depends_on: []
blocks:
  - EPIC-104
  - EPIC-201
---

# EPIC-101 — Informação Útil com Fundamentação

## Descrição

Corrigir a implementação de `useful_information` em `StfEngine` para exigir fundamentação
(`Ground`) como condição necessária, alinhando o motor com a definição teórica de
Informação Coerentemente Executável (ICE / CEI).

## Motivação

A implementação atual em `src/compiler/engine.rs` define:

```rust
let useful_information = logical_ok && operational_ok;
```

A definição teórica (ICE) exige que informação útil satisfaça:

```
ICE(m, A) = C_l(m) ∧ C_o(m) ∧ Ground(m, W)
```

O estado atual cria uma classe silenciosa de enunciados: coerentes, operacionalmente
produtivos, mas não fundamentados — classificados como `useful_information = true` enquanto
emitem simultaneamente um erro `Hallucination`. Essa contradição entre a flag e o erro
inviabiliza o uso da flag como critério de qualidade.

## Escopo

- **Em escopo:** `src/compiler/engine.rs` (métodos `evaluate_statement` e `audit_theory`),
  atualização de testes afetados, adição do invariante à suíte de testes de propriedade.
- **Fora de escopo:** Alteração da semântica de `CoherenceVector`, remoção do erro
  `Hallucination`, modificação na lógica de fundamentação em si.

## Entregáveis

| # | Artefato | Caminho | Formato |
|---|---|---|---|
| D-101-1 | Motor corrigido | `src/compiler/engine.rs` | Rust |
| D-101-2 | Testes unitários atualizados | `tests/grounding.rs` | Rust |
| D-101-3 | Testes de propriedade atualizados | `tests/proptest_invariants.rs` | Rust |
| D-101-4 | Teste de regressão | `tests/regression_semantic_bridge.rs` | Rust |

## Critérios de Sucesso

- [x] `useful_information = false` para enunciados coerentes mas não fundamentados
- [x] `useful_information = false` para enunciados fundamentados mas não executáveis
- [x] `useful_information = true` somente quando `C_l ∧ C_o ∧ Ground` são todos satisfeitos
- [x] Todos os testes existentes passam (retrocompatibilidade)
- [x] O invariante INV-101-1 é verificado por teste de propriedade com ≥ 256 casos

## Riscos

| ID | Risco | Mitigação |
|---|---|---|
| R-101-1 | Testes existentes assumem `useful_information = true` para enunciados não fundamentados | Auditar todos os usos de `useful_information` antes da mudança; atualizar fixtures |
| R-101-2 | A correção expõe regressões semânticas latentes no corpus golden | Executar o corpus golden antes e depois; abrir issues para cada regressão encontrada |
| R-101-3 | Callers externos da API inspecionam `useful_information` com semântica antiga | Documentar a mudança como breaking em CHANGELOG; bump de versão patch |

---

## CONTRATO DO EPIC

```yaml
contract:
  id: CONTRACT-EPIC-101
  version: 1.0.0

  inputs:
    - id: I-101-1
      description: Implementação atual de StfEngine (src/compiler/engine.rs)
      required: true
    - id: I-101-2
      description: Definição teórica de ICE (docs/coherence-paper.tex §2)
      required: true
    - id: I-101-3
      description: Suíte de testes existente (tests/)
      required: true

  outputs:
    - id: O-101-1
      artifact: src/compiler/engine.rs
      constraint: useful_information = logical_ok && operational_ok && grounded
    - id: O-101-2
      artifact: tests/grounding.rs
      constraint: Contém testes INV-101-1 e INV-101-2
    - id: O-101-3
      artifact: tests/proptest_invariants.rs
      constraint: Propriedade INV-101-1 coberta com ≥ 256 casos

  invariants:
    - INV-101-1: |
        SE useful_information == true
        ENTÃO grounded == true
        (válido para todos os EvaluationResult produzidos por evaluate_statement e audit_theory)
    - INV-101-2: |
        SE grounded == false
        ENTÃO useful_information == false
        (contrapositiva de INV-101-1; ambas devem ser verificadas independentemente)

  preconditions:
    - PRE-101-1: Todos os testes existentes passam em main antes de qualquer mudança
    - PRE-101-2: ADR-SEM-001 Regra 3.2 está aceita

  postconditions:
    - POST-101-1: cargo test (suíte completa) retorna 0
    - POST-101-2: O teste INV-101-1 em proptest_invariants passa com 256 casos
    - POST-101-3: Nenhum teste de regressão golden falha

  validation:
    automated:
      - script: cargo test --test grounding useful_information
        description: Verifica os três casos de useful_information
        asserts: [POST-101-1]
      - script: PROPTEST_CASES=256 cargo test proptest_invariants useful_information_grounding_invariant
        description: Verificação de propriedade para INV-101-1
        asserts: [POST-101-2]
      - script: cargo test golden
        description: Golden gate — nenhuma regressão de representação
        asserts: [POST-101-3]

  metrics:
    - metric: useful_information_false_positive_rate
      formula: (casos_nao_fundamentados_com_useful_true) / (total_casos_nao_fundamentados)
      target: 0.0
      measurement: Suíte de teste proptest com 256 casos
    - metric: regressao_golden
      formula: testes_golden_falhando
      target: 0
      measurement: cargo test golden

  failure_modes:
    - FAIL-101-1:
        condition: INV-101-1 violado (useful_information=true com grounded=false)
        action: Bloquear merge; reabrir EPIC-101
        severity: critical
    - FAIL-101-2:
        condition: Regressão no corpus golden após a correção
        action: Identificar fixture afetada; atualizar baseline ou corrigir engine
        severity: error
    - FAIL-101-3:
        condition: Testes de propriedade falham em < 256 casos
        action: Investigar gerador de casos; aumentar cobertura
        severity: error
```

---

## Features

### FEAT-101-1: Correção de `evaluate_statement`

**Descrição:** Modificar o método `evaluate_statement` em `StfEngine` para incluir
fundamentação no cálculo de `useful_information`.

**Comportamento atual:**
```rust
// src/compiler/engine.rs:164
let useful_information = logical_ok && operational_ok;
```

**Comportamento requerido:**
```rust
let useful_information = logical_ok && operational_ok && grounding_result.is_grounded;
```

**Restrições:**
- O erro `Hallucination` DEVE continuar sendo emitido quando `grounded = false`
- A ordem de avaliação não muda: `C_l` → `C_c` → `Ground` → `C_o`
- `grounding_result` já está disponível no escopo antes da linha afetada

**Saída esperada:** `EvaluationResult.useful_information` respeita INV-101-1 em todos os casos.

**Contrato da Feature:**
```yaml
contract:
  id: CONTRACT-FEAT-101-1
  inputs:
    - src/compiler/engine.rs (método evaluate_statement, linha ~134)
  outputs:
    - src/compiler/engine.rs (linha ~164 corrigida)
  invariants:
    - INV-101-1: useful_information → grounded
  preconditions:
    - grounding_result disponível no escopo antes da linha de useful_information
  postconditions:
    - useful_information = logical_ok && operational_ok && grounding_result.is_grounded
  failure_modes:
    - FAIL-101-1: useful_information=true com grounded=false → reverter commit
```

#### Tarefas

**TASK-101-1-1: Alterar linha de `useful_information` em `evaluate_statement`**

- **Feature vinculada:** FEAT-101-1
- **Escopo de arquivo:** `src/compiler/engine.rs`, linha ~164
- **Descrição da implementação:**
  Substituir:
  ```rust
  let useful_information = logical_ok && operational_ok;
  ```
  Por:
  ```rust
  let useful_information = logical_ok && operational_ok && grounding_result.is_grounded;
  ```
- **Critério de aceitação:** `cargo test` passa; INV-101-1 verificado manualmente
- **Requisitos de teste:** Nenhum teste novo nesta tarefa; os testes de TASK-101-1-3 cobrem

---

### FEAT-101-2: Correção de `audit_theory`

**Descrição:** Modificar o método `audit_theory` em `StfEngine` para que `useful_information`
exija que nenhum enunciado na teoria esteja sem fundamentação.

**Comportamento atual:**
```rust
useful_information: logical_ok && operational_ok,
```

**Comportamento requerido:**
```rust
useful_information: logical_ok && operational_ok && ungrounded_ids.is_empty(),
```

**Restrições:**
- `ungrounded_ids` já é computado antes da construção do `EvaluationResult`
- A lista de erros `Hallucination` por `id` DEVE continuar sendo emitida

**Contrato da Feature:**
```yaml
contract:
  id: CONTRACT-FEAT-101-2
  inputs:
    - src/compiler/engine.rs (método audit_theory, linha ~284)
  outputs:
    - src/compiler/engine.rs (campo useful_information na struct literal corrigido)
  invariants:
    - INV-101-2: !grounded → !useful_information (para audit_theory)
  postconditions:
    - useful_information = logical_ok && operational_ok && ungrounded_ids.is_empty()
  failure_modes:
    - FAIL-101-1: useful_information=true com ungrounded_ids não vazio → reverter
```

#### Tarefas

**TASK-101-2-1: Alterar campo `useful_information` em `audit_theory`**

- **Feature vinculada:** FEAT-101-2
- **Escopo de arquivo:** `src/compiler/engine.rs`, linha ~300
- **Descrição da implementação:**
  Substituir:
  ```rust
  useful_information: logical_ok && operational_ok,
  ```
  Por:
  ```rust
  useful_information: logical_ok && operational_ok && ungrounded_ids.is_empty(),
  ```
- **Critério de aceitação:** `cargo test --test coherence` passa sem regressão
- **Requisitos de teste:** Coberto por TASK-101-2-2

**TASK-101-2-2: Adicionar teste de integração para `audit_theory` com teoria mista**

- **Feature vinculada:** FEAT-101-2
- **Escopo de arquivo:** `tests/grounding.rs`
- **Descrição da implementação:**
  Adicionar teste `audit_theory_with_mixed_grounding` que:
  1. Cria uma teoria com dois enunciados: um fundamentado, um não fundamentado
  2. Executa `engine.audit_theory(&theory)`
  3. Asserta `result.useful_information == false`
  4. Asserta `result.grounded == false`
  5. Asserta que `result.errors` contém pelo menos um `ErrorKind::Hallucination`
- **Critério de aceitação:** Teste passa e falha conforme esperado ao reverter TASK-101-2-1
- **Requisitos de teste:** Unitário (sem dependência de artefato externo)

---

### FEAT-101-3: Invariante de propriedade para `useful_information`

**Descrição:** Adicionar um teste de propriedade (proptest) que verifique INV-101-1 para
entradas arbitrárias geradas.

**Comportamento:** Para qualquer `EvaluationResult` gerado pelo motor:
```
result.useful_information == true → result.grounded == true
```

**Contrato da Feature:**
```yaml
contract:
  id: CONTRACT-FEAT-101-3
  inputs:
    - Gerador de Statement arbitrário (proptest)
    - StfEngine com FormulaCoherenceChecker, FormulaInferenceEngine, ProvenanceGroundingChecker
  outputs:
    - tests/proptest_invariants.rs (função de propriedade adicionada)
  invariants:
    - INV-101-1 verificado via proptest com ≥ 256 casos
  postconditions:
    - PROPTEST_CASES=256 cargo test proptest_invariants useful_information_grounding_invariant retorna 0
  metrics:
    - metric: proptest_cases
      target: ≥ 256
  failure_modes:
    - FAIL-101-3: Contração de caso em < 10 tentativas → investigar gerador
```

#### Tarefas

**TASK-101-3-1: Implementar gerador de `Statement` arbitrário para proptest**

- **Feature vinculada:** FEAT-101-3
- **Escopo de arquivo:** `tests/proptest_invariants.rs`
- **Descrição da implementação:**
  Usando `proptest::arbitrary` ou estratégias manuais, gerar `Statement` com:
  - `provenance.grounded` em `{true, false}` de forma aleatória
  - `text` variando entre enunciados simples e com conectivos lógicos
  - `id` único por caso
- **Critério de aceitação:** Gerador produz casos válidos sem pânico; distribuição ~50/50 de `grounded`
- **Requisitos de teste:** Verificado implicitamente pela propriedade em TASK-101-3-2

**TASK-101-3-2: Implementar propriedade `useful_information_grounding_invariant`**

- **Feature vinculada:** FEAT-101-3
- **Escopo de arquivo:** `tests/proptest_invariants.rs`
- **Descrição da implementação:**
  ```rust
  proptest! {
      #[test]
      fn useful_information_grounding_invariant(stmt in arb_statement()) {
          let theory = Theory::new();
          let engine = formula_engine_with_budget(1024);
          let result = engine.evaluate_statement(&theory, &stmt);
          if result.useful_information {
              prop_assert!(result.grounded,
                  "useful_information=true mas grounded=false para stmt id={}", stmt.id);
          }
      }
  }
  ```
- **Critério de aceitação:** Passa com `PROPTEST_CASES=256`; falha ao reverter TASK-101-1-1
- **Requisitos de teste:** Teste de propriedade; depende de TASK-101-3-1

---

## Plano de Testes

### Testes Unitários

| ID | Arquivo | Função | Caso | Resultado esperado |
|---|---|---|---|---|
| UT-101-1 | `tests/grounding.rs` | `coherent_ungrounded_not_useful` | `C_l=true, C_o=true, Ground=false` | `useful_information=false` |
| UT-101-2 | `tests/grounding.rs` | `grounded_non_executable_not_useful` | `C_l=true, C_o=false, Ground=true` | `useful_information=false` |
| UT-101-3 | `tests/grounding.rs` | `fully_grounded_coherent_executable_is_useful` | `C_l=true, C_o=true, Ground=true` | `useful_information=true` |
| UT-101-4 | `tests/grounding.rs` | `contradictory_not_useful` | `C_l=false` | `useful_information=false` |
| UT-101-5 | `tests/grounding.rs` | `audit_theory_with_mixed_grounding` | Teoria mista | `useful_information=false` |

### Testes de Integração

| ID | Arquivo | Função | Descrição |
|---|---|---|---|
| IT-101-1 | `tests/grounding.rs` | `bridge_derived_theory_useful_information` | Compilar artefato real → bridge → evaluate; verificar que todos os enunciados com `source_text` não vazio são grounded e podem produzir `useful_information=true` |

### Testes Adversariais

| ID | Arquivo | Função | Caso adversarial |
|---|---|---|---|
| ADV-101-1 | `tests/hallucination_boundaries.rs` | `hallucination_with_modus_ponens` | Enunciado não fundamentado que dispara modus ponens: deve ser `useful_information=false` mesmo produzindo derivação |
| ADV-101-2 | `tests/hallucination_boundaries.rs` | `fabricated_grounded_flag` | `Provenance.grounded=true` sem `source_ids` nem `anchors`: deve passar no verificador de proveniência mas falhar no verificador SIR |

### Testes de Regressão

| ID | Arquivo | Função | Proteção |
|---|---|---|---|
| REG-101-1 | `tests/golden.rs` | (existente) | Golden gate: nenhuma saída do compilador muda |
| REG-101-2 | `tests/coherence.rs` | (existente) | Todos os testes de coerência existentes passam |
| REG-101-3 | `tests/proptest_invariants.rs` | `useful_information_grounding_invariant` | INV-101-1 permanece válido após qualquer mudança futura no engine |
