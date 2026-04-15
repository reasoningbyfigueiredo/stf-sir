---
id: EPIC-103
title: Segurança na Retenção
version: 1.0.0
status: implementado
sdd: SDD-CORE-SEMANTICS
adr_ref: ADR-SEM-001 Regra 3.1
priority: media
created: 2026-04-14
target: 2026-06-01
depends_on: []
blocks:
  - EPIC-205
---

# EPIC-103 — Segurança na Retenção

## Descrição

Introduzir uma métrica de alerta `rho_alert = min(ρ_L, ρ_S, ρ_Σ, ρ_Φ)` no
`RetentionVector` para expor colapso dimensional que a média geométrica mascara,
e uma flag `unsafe` no `RetentionScore` quando qualquer dimensão cai abaixo
de um limiar configurável.

## Motivação

A média geométrica `(ρ_L · ρ_S · ρ_Σ · ρ_Φ)^¼` pode reportar um score composto alto
mesmo quando uma dimensão está em colapso total. Por exemplo:

```
ρ_L=1.0, ρ_S=1.0, ρ_Σ=1.0, ρ_Φ=0.0 → média geométrica = 0.0  ← correto
ρ_L=1.0, ρ_S=1.0, ρ_Σ=0.5, ρ_Φ=0.5 → média geométrica = 0.707  ← mascarado
```

O segundo caso reporta 70,7% de retenção enquanto duas dimensões perdem metade do conteúdo.
O colapso de `ρ_Φ` (dimensão lógica) é especialmente crítico porque eliminam-se relações
que fundamentam a coerência do artefato.

A regra ADR-SEM-001 3.1 exige que propriedades declaradas sejam verificáveis: uma score
de retenção "seguro" deve significar que nenhuma dimensão colapsou.

## Escopo

- **Em escopo:** `src/retention/mod.rs` (adição de `rho_alert`, `RetentionAlert`,
  `RetentionScore`), `src/lib.rs` (reexportação), testes em `tests/retention.rs`.
- **Fora de escopo:** Alteração dos algoritmos de cálculo existentes de `ρ_L`, `ρ_S`,
  `ρ_Σ`, `ρ_Φ`, mudança em `UnifiedRetentionVector`, implementação de benchmarks (EPIC-205).

## Entregáveis

| # | Artefato | Caminho | Formato |
|---|---|---|---|
| D-103-1 | `RetentionAlert` e campo `rho_alert` | `src/retention/mod.rs` | Rust |
| D-103-2 | `RetentionScore` com flag `unsafe_flag` | `src/retention/mod.rs` | Rust |
| D-103-3 | Testes de alerta | `tests/retention.rs` | Rust |

## Critérios de Sucesso

- [x] `RetentionVector.rho_alert()` retorna `min(ρ_L, ρ_S, ρ_Σ, ρ_Φ)`
- [x] `RetentionScore::from_vector(v, threshold)` sinaliza `unsafe_flag = true` quando `rho_alert < threshold`
- [x] A média geométrica alta NÃO impede o alerta quando uma dimensão está abaixo do limiar
- [x] Todos os cálculos existentes de `RetentionVector` permanecem inalterados
- [x] Todos os testes existentes passam

## Riscos

| ID | Risco | Mitigação |
|---|---|---|
| R-103-1 | Limiar padrão muito alto — alerta dispara em artefatos legítimos | Calibrar limiar contra corpus golden; usar 0.5 como padrão conservador |
| R-103-2 | `RetentionScore` quebra compatibilidade se adicionado a structs existentes | Adicionar como struct nova; não modificar `RetentionVector` nem `UnifiedRetentionVector` |

---

## CONTRATO DO EPIC

```yaml
contract:
  id: CONTRACT-EPIC-103
  version: 1.0.0

  inputs:
    - id: I-103-1
      description: RetentionVector atual (src/retention/mod.rs)
      required: true
    - id: I-103-2
      description: RetentionScore atual (src/retention/mod.rs)
      required: true

  outputs:
    - id: O-103-1
      artifact: src/retention/mod.rs
      constraint: |
        RetentionVector tem método rho_alert() → f64
        RetentionScore tem campo unsafe_flag: bool e threshold: f64
    - id: O-103-2
      artifact: tests/retention.rs
      constraint: Contém INV-103-1 e INV-103-2

  invariants:
    - INV-103-1: |
        rho_alert(v) = min(v.rho_l, v.rho_s, v.rho_sigma, v.rho_phi)
        Para qualquer RetentionVector v.
    - INV-103-2: |
        SE rho_alert(v) < threshold
        ENTÃO RetentionScore::from_vector(v, threshold).unsafe_flag == true
        (independentemente da média geométrica)
    - INV-103-3: |
        A média geométrica de RetentionVector NÃO É ALTERADA por este EPIC.
        score_existente(v) == score_existente_antes_do_epic(v) para todo v.

  preconditions:
    - PRE-103-1: Todos os testes de retenção existentes passam em main
    - PRE-103-2: RetentionVector e RetentionScore são structs públicas

  postconditions:
    - POST-103-1: cargo test --test retention retorna 0
    - POST-103-2: rho_alert() é um método público documentado em RetentionVector
    - POST-103-3: RetentionScore::from_vector existe e está documentado

  validation:
    automated:
      - script: cargo test --test retention rho_alert
        description: Testes INV-103-1 e INV-103-2
        asserts: [POST-103-1]
      - script: cargo doc --no-deps
        description: Documentação compila
        asserts: [POST-103-2, POST-103-3]

  metrics:
    - metric: false_negative_rate
      formula: |
        (casos_com_dimensao_colapsada_e_unsafe_false) / (total_casos_colapsados)
      target: 0.0
      measurement: UT-103-2 (caso adversarial)
    - metric: retrocompatibilidade
      formula: testes_de_retencao_existentes_falhando
      target: 0
      measurement: cargo test retention

  failure_modes:
    - FAIL-103-1:
        condition: INV-103-2 violado (rho_alert < threshold mas unsafe_flag = false)
        action: Corrigir lógica de from_vector; bloquear merge
        severity: critical
    - FAIL-103-2:
        condition: INV-103-3 violado (média geométrica alterada)
        action: Reverter mudanças em RetentionVector; preservar cálculo existente
        severity: error
    - FAIL-103-3:
        condition: rho_alert() retorna valor fora de [0.0, 1.0]
        action: Investigar valores de entrada; adicionar assert_debug
        severity: error
```

---

## Features

### FEAT-103-1: Método `rho_alert` em `RetentionVector`

**Descrição:** Adicionar o método `rho_alert()` a `RetentionVector` que retorna o mínimo
das quatro dimensões.

**Comportamento:**
```rust
impl RetentionVector {
    /// Retorna a dimensão de menor retenção (rho_alert).
    ///
    /// Um valor abaixo do limiar de segurança indica colapso dimensional —
    /// uma degradação que a média geométrica pode mascarar.
    ///
    /// # Invariante
    /// rho_alert() == min(rho_l, rho_s, rho_sigma, rho_phi)
    pub fn rho_alert(&self) -> f64 {
        self.rho_l
            .min(self.rho_s)
            .min(self.rho_sigma)
            .min(self.rho_phi)
    }

    /// Retorna true se rho_alert() < threshold.
    pub fn is_unsafe(&self, threshold: f64) -> bool {
        self.rho_alert() < threshold
    }
}
```

**Restrições:**
- Nenhum campo existente de `RetentionVector` é modificado
- O método é `pub` e documentado
- `rho_alert()` é O(1) e sem alocação

**Contrato da Feature:**
```yaml
contract:
  id: CONTRACT-FEAT-103-1
  inputs: [RetentionVector (rho_l, rho_s, rho_sigma, rho_phi)]
  outputs: [RetentionVector.rho_alert() → f64, RetentionVector.is_unsafe(f64) → bool]
  invariants:
    - INV-103-1: rho_alert() == min das quatro dimensões
    - rho_alert() ∈ [0.0, 1.0] quando todas as dimensões ∈ [0.0, 1.0]
  postconditions:
    - Método compila e documenta sem erro
  failure_modes:
    - rho_alert() fora de [0.0, 1.0] → assert em debug mode
```

#### Tarefas

**TASK-103-1-1: Implementar `rho_alert()` e `is_unsafe()` em `RetentionVector`**

- **Feature vinculada:** FEAT-103-1
- **Escopo de arquivo:** `src/retention/mod.rs`, impl block de `RetentionVector`
- **Descrição:** Adicionar os dois métodos conforme especificado
- **Critério de aceitação:** `cargo test` passa; `rho_alert()` acessível externamente
- **Requisitos de teste:** UT-103-1

**TASK-103-1-2: Adicionar testes unitários de `rho_alert`**

- **Feature vinculada:** FEAT-103-1
- **Escopo de arquivo:** `tests/retention.rs`
- **Descrição:** Casos: todas iguais, uma menor, todas iguais mas pequenas
- **Critério de aceitação:** Testes passam e falham ao reverter implementação

---

### FEAT-103-2: `RetentionScore` com flag de alerta

**Descrição:** Adicionar `RetentionScore` como struct que encapsula o `RetentionVector`
com alerta de segurança, limiar e flag booleana.

**Comportamento:**
```rust
/// Score de retenção com alerta de segurança dimensional.
///
/// Diferente do score composto (média geométrica), `RetentionScore` expõe
/// explicitamente se alguma dimensão colapsou abaixo de um limiar de segurança.
#[derive(Debug, Clone, PartialEq)]
pub struct RetentionScore {
    /// Vetor de retenção original com as quatro dimensões.
    pub vector: RetentionVector,
    /// Score composto: média geométrica das quatro dimensões.
    pub composite: f64,
    /// Dimensão de menor retenção.
    pub rho_alert: f64,
    /// Limiar de segurança configurado.
    pub threshold: f64,
    /// True se rho_alert < threshold (colapso dimensional detectado).
    pub unsafe_flag: bool,
}

impl RetentionScore {
    pub fn from_vector(vector: RetentionVector, threshold: f64) -> Self {
        let rho_alert = vector.rho_alert();
        let composite = (vector.rho_l * vector.rho_s * vector.rho_sigma * vector.rho_phi)
            .powf(0.25);
        Self {
            vector,
            composite,
            rho_alert,
            threshold,
            unsafe_flag: rho_alert < threshold,
        }
    }

    /// Limiar padrão de segurança (conservador).
    pub const DEFAULT_THRESHOLD: f64 = 0.5;
}
```

**Restrições:**
- `RetentionScore` é additive — não substitui `RetentionVector`
- O campo `composite` reproduz exatamente o cálculo existente (sem divergência)
- Nenhum código existente é modificado para usar `RetentionScore`

**Contrato da Feature:**
```yaml
contract:
  id: CONTRACT-FEAT-103-2
  inputs: [RetentionVector, threshold: f64]
  outputs: [RetentionScore com composite, rho_alert, unsafe_flag]
  invariants:
    - INV-103-2: unsafe_flag = (rho_alert < threshold)
    - composite reproduz média geométrica existente exatamente
  postconditions:
    - RetentionScore::from_vector compila e documenta
    - RetentionScore é pub em src/lib.rs
  failure_modes:
    - FAIL-103-1: unsafe_flag=false com rho_alert < threshold → corrigir from_vector
```

#### Tarefas

**TASK-103-2-1: Implementar `RetentionScore`**

- **Feature vinculada:** FEAT-103-2
- **Escopo de arquivo:** `src/retention/mod.rs`
- **Descrição:** Adicionar struct e implementação conforme especificado
- **Critério de aceitação:** Compila; `DEFAULT_THRESHOLD` acessível
- **Requisitos de teste:** UT-103-2, UT-103-3

**TASK-103-2-2: Reexportar `RetentionScore` em `src/lib.rs`**

- **Feature vinculada:** FEAT-103-2
- **Escopo de arquivo:** `src/lib.rs`
- **Descrição:** Adicionar `RetentionScore` à linha `pub use retention::...`
- **Critério de aceitação:** `use stf_sir::RetentionScore` compila em crate externa
- **Requisitos de teste:** Compilação

**TASK-103-2-3: Adicionar testes de `RetentionScore`**

- **Feature vinculada:** FEAT-103-2
- **Escopo de arquivo:** `tests/retention.rs`
- **Descrição:**
  - Caso normal: todas as dimensões acima do limiar → `unsafe_flag = false`
  - Caso de colapso: uma dimensão em 0.0, demais em 1.0 → `unsafe_flag = true`
  - Caso adversarial: média geométrica alta mas uma dimensão abaixo do limiar →
    `unsafe_flag = true` mesmo que `composite > threshold`
- **Critério de aceitação:** Três casos cobertos; falham ao reverter `from_vector`

---

## Plano de Testes

### Testes Unitários

| ID | Arquivo | Função | Caso | Resultado esperado |
|---|---|---|---|---|
| UT-103-1 | `tests/retention.rs` | `rho_alert_is_minimum` | `(1.0, 0.3, 1.0, 1.0)` | `rho_alert() == 0.3` |
| UT-103-2 | `tests/retention.rs` | `rho_alert_triggers_on_collapsed_dimension` | `(1.0, 1.0, 1.0, 0.0)`, limiar=0.5 | `unsafe_flag = true` |
| UT-103-3 | `tests/retention.rs` | `geometric_mean_does_not_mask_alert` | `(1.0, 0.8, 0.8, 0.3)`, limiar=0.5 | `unsafe_flag = true`, `composite > 0.5` |
| UT-103-4 | `tests/retention.rs` | `retention_score_safe_when_all_above_threshold` | `(0.9, 0.9, 0.8, 0.7)`, limiar=0.5 | `unsafe_flag = false` |
| UT-103-5 | `tests/retention.rs` | `rho_alert_in_unit_interval` | Qualquer `RetentionVector` válido | `rho_alert() ∈ [0.0, 1.0]` |

### Testes Adversariais

| ID | Arquivo | Função | Caso adversarial |
|---|---|---|---|
| ADV-103-1 | `tests/retention.rs` | `high_composite_with_collapsed_phi` | `ρ_Φ = 0.01`, demais = 1.0: composite = ~0.316, mas alerta deve disparar |
| ADV-103-2 | `tests/retention.rs` | `all_dimensions_at_threshold_boundary` | `(0.5, 0.5, 0.5, 0.5)`, limiar=0.5: `unsafe_flag = false` (boundary não é unsafe) |

### Testes de Regressão

| ID | Arquivo | Função | Proteção |
|---|---|---|---|
| REG-103-1 | `tests/retention.rs` | (existentes) | Cálculos existentes de `RetentionVector` e `RetentionBaseline` não são alterados |
