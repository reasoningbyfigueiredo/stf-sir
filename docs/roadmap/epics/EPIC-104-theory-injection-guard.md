---
id: EPIC-104
title: Guarda de Injeção na Teoria
version: 1.0.0
status: implementado
sdd: SDD-CORE-SEMANTICS
adr_ref: ADR-SEM-001 Regra 3.4 (separação de camadas)
priority: media
created: 2026-04-14
target: 2026-06-15
depends_on:
  - EPIC-101
blocks:
  - EPIC-201
---

# EPIC-104 — Guarda de Injeção na Teoria

## Descrição

Introduzir validação de proveniência na camada de teoria, de forma que enunciados inseridos
em um `Theory` sem proveniência explícita sejam sinalizados como não confiáveis em vez de
aceitos silenciosamente.

## Motivação

A fundamentação do SIR-graph (`SirGroundingChecker`) verifica se um `Statement.id` existe
no grafo compilado. Porém, um caller pode criar um `Statement` com um id arbitrário e inseri-lo
diretamente em `Theory::insert` sem passar pelo pipeline de compilação, contornando o verificador.

Isso significa que a garantia de auditabilidade — "todo enunciado tem proveniência rastreável"
— não é enforced pela camada de teoria; é apenas assumed. A Regra 3.4 do ADR-SEM-001
exige que as camadas tenham responsabilidades definidas e que a camada de teoria não aceite
silenciosamente enunciados que a camada de coerência classificaria como alucinação.

A solução é additive e retrocompatível: `Theory::insert` continua funcionando, mas um método
`Theory::insert_guarded` e um struct `InsertionOutcome` permitem inspecionar o status de
confiança de cada enunciado inserido.

## Escopo

- **Em escopo:** `src/model/theory.rs` (adição de `TrustLevel`, `InsertionOutcome`,
  `Theory::insert_guarded`), testes em `tests/grounding.rs`.
- **Fora de escopo:** Modificação de `Theory::insert` (retrocompatibilidade), validação
  semântica completa na inserção, alteração da lógica de `StfEngine`.

## Entregáveis

| # | Artefato | Caminho | Formato |
|---|---|---|---|
| D-104-1 | `TrustLevel`, `InsertionOutcome`, `Theory::insert_guarded` | `src/model/theory.rs` | Rust |
| D-104-2 | Reexportações públicas | `src/lib.rs` | Rust |
| D-104-3 | Testes de guarda | `tests/grounding.rs` | Rust |

## Critérios de Sucesso

- [x] `Theory::insert_guarded` aceita enunciado e retorna `InsertionOutcome` com `TrustLevel`
- [x] Enunciado sem `source_ids`, `anchors` e `grounded=false` retorna `TrustLevel::Untrusted`
- [x] Enunciado com qualquer proveniência válida retorna `TrustLevel::Trusted`
- [x] `Theory::insert` continua funcionando sem modificação (retrocompatibilidade)
- [x] `InsertionOutcome` é documentado com exemplos
- [x] Todos os testes existentes passam

## Riscos

| ID | Risco | Mitigação |
|---|---|---|
| R-104-1 | Confundir `TrustLevel` com resultado de `GroundingChecker` — duas fontes de verdade | Documentar claramente: `insert_guarded` usa heurística de proveniência inline, sem instanciar `GroundingChecker`; para verificação completa, usar `StfEngine` |
| R-104-2 | Callers que iterem sobre `Theory` e inspecionem trust level não existem ainda | API é additive; nenhum breaking change |

---

## CONTRATO DO EPIC

```yaml
contract:
  id: CONTRACT-EPIC-104
  version: 1.0.0

  inputs:
    - id: I-104-1
      description: Theory::insert atual (src/model/theory.rs)
      required: true
    - id: I-104-2
      description: Definição de Provenance (src/model/statement.rs)
      required: true
    - id: I-104-3
      description: EPIC-101 fechado (semântica de fundamentação estável)
      required: false

  outputs:
    - id: O-104-1
      artifact: src/model/theory.rs
      constraint: |
        Contém TrustLevel enum, InsertionOutcome struct,
        Theory::insert_guarded(stmt: Statement) -> InsertionOutcome
    - id: O-104-2
      artifact: tests/grounding.rs
      constraint: Contém INV-104-1 e INV-104-2

  invariants:
    - INV-104-1: |
        Theory::insert CONTINUA FUNCIONANDO sem mudança de comportamento.
        Para todo enunciado S: theory.insert(S) comporta-se identicamente ao código anterior.
    - INV-104-2: |
        insert_guarded(S).trust_level == TrustLevel::Untrusted
        ⟺ S.provenance.source_ids.is_empty()
             ∧ S.provenance.anchors.is_empty()
             ∧ !S.provenance.grounded
    - INV-104-3: |
        insert_guarded sempre insere o enunciado (mesmo que Untrusted).
        O enunciado fica presente em Theory após insert_guarded independentemente do trust_level.

  preconditions:
    - PRE-104-1: Todos os testes existentes passam em main
    - PRE-104-2: Provenance é uma struct com source_ids, anchors, grounded

  postconditions:
    - POST-104-1: cargo test retorna 0
    - POST-104-2: Theory::insert_guarded é pub e documentado
    - POST-104-3: TrustLevel e InsertionOutcome são pub e reexportados de src/lib.rs

  validation:
    automated:
      - script: cargo test --test grounding theory_injection
        description: Testes INV-104-1, INV-104-2, INV-104-3
        asserts: [POST-104-1]
      - script: cargo doc --no-deps
        description: Documentação compila
        asserts: [POST-104-2, POST-104-3]

  metrics:
    - metric: retrocompatibilidade
      formula: testes_existentes_falhando_apos_epic
      target: 0
      measurement: cargo test (suíte completa)
    - metric: cobertura_de_invariantes
      formula: invariantes_com_teste / invariantes_total
      target: 1.0
      measurement: INV-104-1, INV-104-2, INV-104-3 — 3/3

  failure_modes:
    - FAIL-104-1:
        condition: INV-104-1 violado (Theory::insert muda de comportamento)
        action: Reverter qualquer modificação a Theory::insert; apenas adicionar insert_guarded
        severity: critical
    - FAIL-104-2:
        condition: INV-104-2 violado (trust_level incorreto)
        action: Corrigir lógica de classify_provenance; bloquear PR
        severity: error
    - FAIL-104-3:
        condition: INV-104-3 violado (insert_guarded não insere quando Untrusted)
        action: insert_guarded DEVE sempre inserir; trust_level é apenas informativo
        severity: critical
```

---

## Features

### FEAT-104-1: `TrustLevel` e `InsertionOutcome`

**Descrição:** Definir os tipos de dados que representam o resultado da inserção com guarda.

**Comportamento:**
```rust
/// Nível de confiança de um enunciado inserido em Theory.
///
/// Determina se o enunciado tem proveniência verificável.
/// Este é um nível de confiança estático (heurística de proveniência inline),
/// não uma avaliação completa de fundamentação pelo GroundingChecker.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TrustLevel {
    /// O enunciado tem pelo menos uma fonte, âncora ou flag grounded=true.
    Trusted,
    /// O enunciado não tem nenhuma proveniência verificável.
    /// Pode ser uma alucinação; requer verificação via StfEngine.
    Untrusted,
}

/// Resultado de uma inserção com guarda em Theory.
#[derive(Debug, Clone)]
pub struct InsertionOutcome {
    /// O enunciado foi inserido (sempre true — a inserção nunca é rejeitada).
    pub inserted: bool,
    /// Nível de confiança determinado pela proveniência do enunciado.
    pub trust_level: TrustLevel,
    /// Id do enunciado inserido.
    pub statement_id: StatementId,
    /// Mensagem de diagnóstico quando trust_level == Untrusted.
    pub diagnostic: Option<String>,
}
```

**Restrições:**
- `InsertionOutcome.inserted` é sempre `true` (a guarda nunca rejeita)
- `TrustLevel` é determinado apenas por `Provenance`; não usa `GroundingChecker`
- Tipos são `pub` e documentados

**Contrato da Feature:**
```yaml
contract:
  id: CONTRACT-FEAT-104-1
  inputs: [Provenance struct de src/model/statement.rs]
  outputs: [TrustLevel enum, InsertionOutcome struct em src/model/theory.rs]
  invariants:
    - TrustLevel tem exatamente dois variants: Trusted, Untrusted
    - InsertionOutcome.inserted é sempre true
  postconditions:
    - Tipos compilam e documentam sem erro
  failure_modes:
    - InsertionOutcome.inserted = false → corrigir; a guarda nunca rejeita
```

#### Tarefas

**TASK-104-1-1: Definir `TrustLevel` em `src/model/theory.rs`**

- **Feature vinculada:** FEAT-104-1
- **Escopo de arquivo:** `src/model/theory.rs`
- **Descrição:** Adicionar enum conforme especificado
- **Critério de aceitação:** Compila; documentação presente
- **Requisitos de teste:** Implícito via TASK-104-2-1

**TASK-104-1-2: Definir `InsertionOutcome` em `src/model/theory.rs`**

- **Feature vinculada:** FEAT-104-1
- **Escopo de arquivo:** `src/model/theory.rs`
- **Descrição:** Adicionar struct conforme especificado
- **Critério de aceitação:** Compila; `diagnostic` é `Option<String>`
- **Requisitos de teste:** Implícito via TASK-104-2-1

---

### FEAT-104-2: `Theory::insert_guarded`

**Descrição:** Adicionar o método `insert_guarded` à struct `Theory`.

**Comportamento:**
```rust
impl Theory {
    /// Insere um enunciado e retorna seu nível de confiança baseado na proveniência.
    ///
    /// A inserção SEMPRE ocorre, independente do nível de confiança.
    /// Use o `InsertionOutcome` para inspecionar a proveniência e tomar
    /// decisões downstream (ex: marcar enunciados não confiáveis para
    /// verificação pelo StfEngine).
    ///
    /// # Classificação de confiança
    ///
    /// Um enunciado é `Trusted` se tiver pelo menos um de:
    /// - `provenance.source_ids` não vazio
    /// - `provenance.anchors` não vazio
    /// - `provenance.grounded == true`
    ///
    /// Caso contrário é `Untrusted` (candidato a alucinação).
    pub fn insert_guarded(&mut self, stmt: Statement) -> InsertionOutcome {
        let trust_level = classify_provenance(&stmt.provenance);
        let id = stmt.id.clone();
        let diagnostic = if trust_level == TrustLevel::Untrusted {
            Some(format!(
                "enunciado '{}' inserido sem proveniência verificável (candidato a alucinação)",
                id
            ))
        } else {
            None
        };
        self.insert(stmt);   // delega para insert existente — sem duplicação
        InsertionOutcome {
            inserted: true,
            trust_level,
            statement_id: id,
            diagnostic,
        }
    }
}

/// Classificação de proveniência inline (sem instanciar GroundingChecker).
fn classify_provenance(p: &Provenance) -> TrustLevel {
    if !p.source_ids.is_empty() || !p.anchors.is_empty() || p.grounded {
        TrustLevel::Trusted
    } else {
        TrustLevel::Untrusted
    }
}
```

**Restrições:**
- `insert_guarded` delega para `self.insert(stmt)` internamente — sem lógica de inserção duplicada
- `classify_provenance` é uma função privada no mesmo módulo
- O critério de classificação espelha `ProvenanceGroundingChecker::check_grounding` para consistência

**Contrato da Feature:**
```yaml
contract:
  id: CONTRACT-FEAT-104-2
  inputs:
    - Theory::insert existente
    - Provenance struct
  outputs:
    - Theory::insert_guarded(stmt: Statement) -> InsertionOutcome
    - classify_provenance (privada)
  invariants:
    - INV-104-1: Theory::insert inalterado
    - INV-104-2: trust_level correto por INV-104-2 do EPIC
    - INV-104-3: inserted = true sempre
  postconditions:
    - Após insert_guarded, theory.contains(&outcome.statement_id) == true
    - insert_guarded compila e documenta
  failure_modes:
    - FAIL-104-3: inserted=false → corrigir imediatamente
    - FAIL-104-2: trust_level errado → corrigir classify_provenance
```

#### Tarefas

**TASK-104-2-1: Implementar `classify_provenance` e `Theory::insert_guarded`**

- **Feature vinculada:** FEAT-104-2
- **Escopo de arquivo:** `src/model/theory.rs`
- **Descrição:** Adicionar a função privada e o método público conforme especificado
- **Critério de aceitação:** `cargo test` passa; nenhum teste existente falha
- **Requisitos de teste:** UT-104-1 a UT-104-4

**TASK-104-2-2: Reexportar `TrustLevel` e `InsertionOutcome` em `src/model/mod.rs` e `src/lib.rs`**

- **Feature vinculada:** FEAT-104-2
- **Escopo de arquivo:** `src/model/mod.rs`, `src/lib.rs`
- **Descrição:** Adicionar reexportações públicas para os dois novos tipos
- **Critério de aceitação:** `use stf_sir::{TrustLevel, InsertionOutcome}` compila
- **Requisitos de teste:** Compilação

**TASK-104-2-3: Adicionar testes unitários de `insert_guarded`**

- **Feature vinculada:** FEAT-104-2
- **Escopo de arquivo:** `tests/grounding.rs`
- **Descrição:**
  - Caso 1: enunciado com `source_ids` → `Trusted`
  - Caso 2: enunciado com `anchors` → `Trusted`
  - Caso 3: enunciado com `grounded=true` → `Trusted`
  - Caso 4: enunciado sem nada → `Untrusted` + diagnostic presente
  - Caso 5: após `insert_guarded`, `theory.contains(id)` é `true` em todos os casos
- **Critério de aceitação:** 5 casos cobertos; falham ao reverter `classify_provenance`

---

## Plano de Testes

### Testes Unitários

| ID | Arquivo | Função | Caso | Resultado esperado |
|---|---|---|---|---|
| UT-104-1 | `tests/grounding.rs` | `insert_guarded_with_source_ids_is_trusted` | `source_ids` não vazio | `TrustLevel::Trusted` |
| UT-104-2 | `tests/grounding.rs` | `insert_guarded_with_anchors_is_trusted` | `anchors` não vazio | `TrustLevel::Trusted` |
| UT-104-3 | `tests/grounding.rs` | `insert_guarded_with_grounded_flag_is_trusted` | `grounded=true` | `TrustLevel::Trusted` |
| UT-104-4 | `tests/grounding.rs` | `insert_guarded_without_provenance_is_untrusted` | Sem nenhuma proveniência | `TrustLevel::Untrusted`, `diagnostic.is_some()` |
| UT-104-5 | `tests/grounding.rs` | `insert_guarded_always_inserts` | Qualquer enunciado | `inserted=true`, `theory.contains(id)=true` |
| UT-104-6 | `tests/grounding.rs` | `theory_insert_unchanged_after_epic` | `Theory::insert` com enunciado sem proveniência | Inserção bem-sucedida, sem pânico |

### Testes Adversariais

| ID | Arquivo | Função | Caso adversarial |
|---|---|---|---|
| ADV-104-1 | `tests/grounding.rs` | `insert_guarded_conflicting_ids` | Dois `insert_guarded` com mesmo id, diferentes proveniências: o segundo deve sobrescrever (comportamento de `BTreeMap::insert`) |
| ADV-104-2 | `tests/grounding.rs` | `insert_guarded_empty_source_id_string` | `source_ids` contém string vazia: `TrustLevel::Trusted` (conjunto não vazio) |

### Testes de Regressão

| ID | Arquivo | Função | Proteção |
|---|---|---|---|
| REG-104-1 | `tests/grounding.rs` | (existentes) | `Theory::insert` sem mudança de comportamento |
| REG-104-2 | `tests/bridge_semantics.rs` | (existentes) | Bridge + Theory funcionam sem regressão |
