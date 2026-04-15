---
id: SDD-CORE-SEMANTICS
title: Pacote SDD — Semântica Central STF-SIR
version: 1.0.0
status: accepted
adr_ref: ADR-SEM-001
created: 2026-04-14
author: Rogerio Figueiredo
---

# SDD-CORE-SEMANTICS — Pacote de Desenvolvimento Orientado por Especificação

> Alinhado com: **ADR-SEM-001** (posicionamento do STF-SIR como infraestrutura auditável de compilação semântica)

---

## 1. Objetivo

Este pacote traduz as cinco recomendações arquiteturais priorizadas do ADR-SEM-001 em
artefatos de engenharia auditáveis: EPICs, Features, Tarefas, Contratos e Planos de Teste.

Todos os artefatos são rastreáveis até o código-fonte e verificáveis pela suíte de testes existente.

---

## 2. Índice de EPICs

| ID | Título | Prioridade | Status | Arquivo |
|---|---|---|---|---|
| [EPIC-101](epics/EPIC-101-grounded-useful-information.md) | Informação Útil com Fundamentação | CRÍTICA | planejado | `epics/EPIC-101-grounded-useful-information.md` |
| [EPIC-102](epics/EPIC-102-recommended-engine.md) | Alias RecommendedEngine | ALTA | planejado | `epics/EPIC-102-recommended-engine.md` |
| [EPIC-103](epics/EPIC-103-retention-safety.md) | Segurança na Retenção | MÉDIA | planejado | `epics/EPIC-103-retention-safety.md` |
| [EPIC-104](epics/EPIC-104-theory-injection-guard.md) | Guarda de Injeção na Teoria | MÉDIA | planejado | `epics/EPIC-104-theory-injection-guard.md` |
| [EPIC-201](epics/EPIC-201-spec-v2.md) + [FEAT-201-5](epics/EPIC-201-spec-v2.md#feat-201-5) | Dimensões Semânticas v2 (materialização) | BAIXA | planejado | `epics/EPIC-201-spec-v2.md` |

---

## 3. Princípio de Design

> **O sistema DEVE aplicar em código o que afirma em teoria.**

---

## 4. Restrições Globais

| Restrição | Alcance |
|---|---|
| NÃO quebrar testes existentes | Todos os EPICs |
| NÃO remover comportamento de fallback ainda | EPIC-102 |
| DEVE preservar compatibilidade retroativa | EPIC-101, EPIC-104 |
| DEVE adicionar diagnósticos para novas regras | EPIC-101, EPIC-103, EPIC-104 |
| DEVE documentar todos os invariantes | Todos os EPICs |

---

## 5. Caminho Crítico

```
EPIC-101 ──▶ EPIC-104   (guarda de injeção depende de semântica de fundamentação estável)
EPIC-102                 (independente; pode ser executado em paralelo)
EPIC-103                 (independente; pode ser executado em paralelo)
EPIC-101 ──▶ EPIC-201   (SemanticDimensions incorpora CoherenceVector estabilizado)
```

---

## 6. Mapeamento de Invariantes

| Invariante | EPIC | Classe | Teste |
|---|---|---|---|
| INV-101-1 | EPIC-101 | Segurança | `tests/grounding.rs::useful_information_requires_grounding` |
| INV-101-2 | EPIC-101 | Segurança | `tests/grounding.rs::audit_theory_useful_requires_all_grounded` |
| INV-102-1 | EPIC-102 | Determinismo | `tests/coherence.rs::recommended_engine_consistent_with_formula_engine` |
| INV-102-2 | EPIC-102 | Segurança | `tests/coherence.rs::recommended_engine_is_public` |
| INV-103-1 | EPIC-103 | Segurança | `tests/retention.rs::rho_alert_triggers_on_collapsed_dimension` |
| INV-103-2 | EPIC-103 | Segurança | `tests/retention.rs::geometric_mean_does_not_mask_alert` |
| INV-104-1 | EPIC-104 | Segurança | `tests/grounding.rs::theory_insert_flags_ungrounded` |
| INV-104-2 | EPIC-104 | Segurança | `tests/grounding.rs::guarded_theory_tracks_trust_level` |
| INV-201-5 | EPIC-201 | Segurança | `tests/coherence.rs::semantic_dimensions_attach_to_statement` |

---

## 7. Referências

- [ADR-SEM-001](../adr/ADR-SEM-001-positioning.md) — Decisão de posicionamento
- [CONTRACT-MODEL](contracts/CONTRACT-MODEL.md) — Esquema canônico de contratos
- [AUDIT-MODEL](audit/AUDIT-MODEL.md) — Pipeline de auditoria contínua
- `src/compiler/engine.rs` — Motor de coerência (alvo de EPIC-101, EPIC-102)
- `src/retention/mod.rs` — Métricas de retenção (alvo de EPIC-103)
- `src/model/theory.rs` — Camada de teoria (alvo de EPIC-104)
- `src/model/statement.rs` — Statement (alvo de EPIC-201)
