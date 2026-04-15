# ADR-SEM-001: Posicionamento do STF-SIR como Infraestrutura Auditável de Compilação Semântica

**Status:** Aceito  
**Data:** 2026-04-14  
**Autor:** Rogerio Figueiredo  
**Escopo:** Todos os artefatos de implementação, especificação e documentação no repositório STF-SIR

---

## Contexto

O projeto STF-SIR acumulou um núcleo técnico funcional ao lado de documentação e material teórico cujo escopo vai além do que a implementação efetivamente aplica. Essa divergência não é um defeito de ambição, mas um risco estrutural: se a documentação faz afirmações que o motor não consegue verificar, o sistema perde sua propriedade central — a auditabilidade.

### O que a implementação atualmente oferece

Os seguintes componentes estão implementados, testados e considerados estáveis:

| Componente | Localização | Status |
|---|---|---|
| Pipeline de compilação determinística | `src/compiler/` | Estável |
| Formato de artefato `.zmd` | `spec/stf-sir-spec-v1.md`, `src/model/artifact.rs` | Estável (v1 congelado) |
| Modelo de quatro dimensões do ZToken (L, S, Σ, Φ) | `src/model/ztoken.rs` | Estável |
| Representação em grafo SIR | `src/sir/graph.rs` | Estável |
| Função ponte artefato→teoria (`β`) | `src/model/bridge.rs` | Estável |
| AST de fórmulas (subconjunto proposicional) | `src/model/formula.rs` | Estável |
| Vetor de coerência `(C_l, C_c, C_o)` | `src/model/coherence.rs` | Estável |
| Verificador de coerência lógica — backend texto | `src/compiler/coherence.rs` | Estável (MVP) |
| Verificador de coerência lógica — backend AST de fórmulas | `src/compiler/coherence.rs` | Estável |
| Motor de inferência — backend texto | `src/compiler/inference.rs` | Estável (MVP) |
| Motor de inferência — backend AST de fórmulas (modus ponens) | `src/compiler/inference.rs` | Estável |
| Verificador de fundamentação — baseado em proveniência | `src/compiler/grounding.rs` | Estável |
| Verificador de fundamentação — baseado em grafo SIR | `src/compiler/grounding.rs` | Estável |
| Avaliação de coerência `StfEngine<L, I, G>` | `src/compiler/engine.rs` | Estável |
| Métrica de retenção `ρ` (vetor por etapa do pipeline) | `src/retention/mod.rs` | Estável |
| Vetor de retenção unificado | `src/retention/mod.rs` | Estável |
| Taxonomia de erros (Contradiction, Hallucination, NonExecutable) | `src/error.rs` | Estável |
| Suíte de testes (unitários, conformidade, golden, metamórficos, propriedade) | `tests/` | Extensa |

### Onde a superestrutura diverge

1. **A coerência computacional (`C_c`) é uma aproximação por orçamento de passos.** O artigo de coerência (§2 / Teorema A2) associa `C_c = 1` à pertença à classe de complexidade **P**. O motor (`src/compiler/engine.rs`) implementa isso como um orçamento de passos configurável: se o número de comparações ≤ `step_budget`, então `C_c = Satisfied`. Trata-se de uma aproximação pragmática e honesta. Ela não constitui uma caracterização de classe de complexidade, e a documentação NÃO DEVE apresentá-la como tal.

2. **`useful_information` não exige fundamentação.** A implementação atual de `evaluate_statement` (`src/compiler/engine.rs:164`) define `useful_information = logical_ok && operational_ok`. O status de fundamentação é reportado separadamente em `EvaluationResult.grounded` e emite um erro `Hallucination`, mas não bloqueia `useful_information`. A definição teórica (ICE) exige fundamentação como condição necessária, criando uma lacuna entre especificação e aplicação.

3. **O artigo de coerência faz afirmações de escopo cosmológico e cognitivo.** O resumo de `docs/coherence-paper.tex` enuncia uma "hipótese fundacional não-padrão: que a coerência … é a propriedade estrutural primária subjacente à realidade observável" e relaciona o horizonte de inferibilidade à fronteira **P** vs **NP** como uma afirmação epistemológica. Trata-se de enquadramentos filosóficos, não de asserções de engenharia. Eles NÃO DEVEM ser tratados como normativos para decisões de implementação.

4. **O AST de fórmulas cobre apenas um subconjunto proposicional.** `src/model/formula.rs` implementa átomos, negação e implicação. Conjunção, disjunção, quantificadores e operadores modais não estão implementados. Especificações que referenciam um "espaço STS completo" ou universo conjuntístico `U` são aspiracionais.

5. **A função ponte é o ponto de entrada canônico; a CLI é uma interface de usuário.** A semântica do sistema é definida por `artifact_to_theory` em `src/model/bridge.rs`. A CLI (`src/cli.rs`) é uma interface para o pipeline do compilador e o motor. A documentação e os exemplos DEVEM tratar a ponte como central do ponto de vista arquitetural, e não a invocação pela CLI.

---

## Decisão

### Posicionamento primário

O STF-SIR DEVE ser posicionado primariamente como:

> **infraestrutura auditável de compilação semântica e raciocínio ciente de teoria**

Isso significa:

- O STF-SIR é um **sistema de ciência da computação** que compila documentos-fonte em artefatos semânticos estruturados e endereçáveis, e avalia esses artefatos quanto à coerência lógica, computacional e operacional.
- Sua proposta de valor é a **auditabilidade**: cada enunciado em uma teoria compilada carrega proveniência rastreável até o artefato-fonte, via a função ponte β e o grafo SIR.
- Seu critério de correção é **verificável pela implementação**: uma propriedade faz parte do sistema somente se for aplicável em código e falsificável por teste.

### O que o STF-SIR NÃO DEVE ser posicionado como

- Uma teoria da realidade física, da consciência ou da cognição.
- Um sistema que resolve o problema **P** vs **NP** ou faz afirmações sobre classes de complexidade além de sua aproximação por orçamento de passos.
- Uma ontologia formal completa ou framework epistemológico.
- Um sistema cuja correção é definida por interpretação filosófica.

Esses enquadramentos PODEM aparecer em documentos não normativos explicitamente marcados como tal (ver Regra 3.5 abaixo), mas NÃO DEVEM ter qualquer efeito sobre decisões de implementação, contratos de API ou critérios de aceitação de testes.

---

## Regras Normativas

As seguintes regras são vinculantes para todo trabalho futuro de implementação, atualizações de especificação e documentação neste repositório. As palavras-chave DEVE, DEVERIA e PODE seguem a RFC 2119.

### Regra 3.1 — Alinhamento com a Implementação

> **O sistema NÃO DEVE afirmar na documentação propriedades que não sejam aplicáveis em código.**

Uma propriedade é aplicável se:
- é avaliada por um componente do pipeline de compilação ou de coerência, E
- seu resultado é determinístico para uma entrada e configuração fixas, E
- existe (ou pode ser escrito) um teste que falharia se a propriedade fosse violada.

Qualquer propriedade que não satisfaça esses três critérios é aspiracional. Propriedades aspiracionais DEVEM ser rotuladas como tal na documentação.

### Regra 3.2 — Requisito de Fundamentação

> **`useful_information` DEVE exigir coerência lógica (`C_l`), coerência operacional (`C_o`) E fundamentação (`Ground`).**

A implementação atual (`src/compiler/engine.rs`) define:

```
useful_information = logical_ok && operational_ok
```

Isso DEVE ser corrigido para:

```
useful_information = logical_ok && operational_ok && grounded
```

Um enunciado que é logicamente coerente e operacionalmente produtivo, mas não fundamentado, produz `useful_information = false`. O erro `Hallucination` DEVE continuar sendo emitido. Esta correção torna o motor consistente com a definição teórica de Informação Coerentemente Executável (ICE / CEI).

### Regra 3.3 — Honestidade Computacional

> **A coerência computacional (`C_c`) DEVE ser descrita como uma aproximação por orçamento de passos, e não como uma caracterização de classe de complexidade.**

A documentação e os comentários de código DEVEM usar a seguinte linguagem:

- CORRETO: "C_c é `Satisfied` se a verificação de consistência termina dentro do orçamento de passos configurado."
- CORRETO: "C_c é `Violated` se o orçamento de passos é excedido, indicando que a verificação foi intratável para este agente nesta escala."
- CORRETO: "Com `step_budget = usize::MAX`, C_c é `Unknown` (nenhuma afirmação de orçamento é feita)."
- INCORRETO: "C_c = 1 sse a verificação de coerência está em **P**."
- INCORRETO: "O motor determina a pertença a uma classe de complexidade computacional."

O modelo de orçamento de passos é honesto e útil. Ele não precisa ser inflado a uma afirmação de teoria de complexidade para ter valor.

### Regra 3.4 — Separação de Camadas

O sistema é composto por quatro camadas distintas. Essas camadas DEVEM ser mantidas arquiteturalmente separadas em código, documentação e design de API:

| Camada | Definição | Artefato principal |
|---|---|---|
| **Camada de artefato** | O documento `.zmd` compilado: `Artifact`, `ZToken`, `Relation`, `SourceInfo` | `src/model/artifact.rs`, `src/model/ztoken.rs` |
| **Camada SIR** | O grafo de tokens compilados e suas relações estruturais | `src/sir/graph.rs` |
| **Camada de teoria** | O conjunto de objetos `Statement` com proveniência, derivado via a ponte | `src/model/statement.rs`, `src/model/theory.rs` |
| **Camada de coerência** | A avaliação de `(C_l, C_c, C_o)` e o predicado de fundamentação sobre uma teoria | `src/compiler/engine.rs`, `src/model/coherence.rs` |

As dependências entre camadas DEVEM fluir em uma única direção: Coerência → Teoria → SIR → Artefato. Nenhuma camada DEVE importar de uma camada superior.

### Regra 3.5 — Escopo da Filosofia

> **A interpretação filosófica DEVE ser tratada como contexto não normativo. Ela NÃO DEVE definir correção.**

Documentos que enquadram o STF-SIR em termos de epistemologia, ontologia, consciência ou física DEVEM conter um cabeçalho não normativo explícito, como:

```
> **Não normativo.** Este documento fornece contexto filosófico e motivação.
> Ele não define requisitos de implementação nem critérios de aceitação de testes.
```

Isso inclui, mas não se limita a: `docs/coherence-paper.tex`, `docs/sts-paper.tex`, `docs/coherence-foundations.md`.

O enquadramento filosófico é permitido e pode ter valor intelectual genuíno. Ele se torna um risco somente quando é tratado implicitamente como vinculante para a implementação.

---

## Posicionamento Matemático

O STF-SIR define **objetos semânticos operacionais**. Seus componentes formais estão fundamentados na implementação da seguinte forma:

| Objeto matemático | Implementação | Status |
|---|---|---|
| Vetor de coerência `Coh(S) = (C_l, C_c, C_o) ∈ {0,1}³` | `CoherenceVector` em `src/model/coherence.rs` | Implementado |
| Função ponte `β : Artifact → Theory` | `artifact_to_theory` em `src/model/bridge.rs` | Implementado |
| Predicado de fundamentação `Ground(x, W)` | Trait `GroundingChecker` + `ProvenanceGroundingChecker`, `SirGroundingChecker` | Implementado |
| AST de fórmulas (subconjunto proposicional: Atom, Not, Implies) | `Formula` em `src/model/formula.rs` | Implementado |
| Relação de contradição `contradicts(φ, ψ)` | `Formula::contradicts` | Implementado |
| Inferência por modus ponens `MP(Atom(p), Implies(p,q)) → q` | `FormulaInferenceEngine` em `src/compiler/inference.rs` | Implementado |
| Métrica de retenção `ρ = (ρ_l, ρ_s, ρ_σ, ρ_φ)` | `RetentionVector` em `src/retention/mod.rs` | Implementado |
| Vetor de retenção unificado | `UnifiedRetentionVector` em `src/retention/mod.rs` | Implementado |
| Rastreamento delta `Δ` (ancoragem de span na fonte) | `Provenance.anchors`, `LexicalDimension.source_text` | Implementado |
| Taxonomia de erros `Err(m) = (anomalia, alucinação, contradição)` | `ErrorKind` em `src/error.rs` | Implementado |
| Homomorfismo de coerência lexical `ρ_lex` | `CoherenceRetention` em `src/retention/coherence_retention.rs` | Parcialmente implementado |
| Universo de enunciados `U` / espaço STS completo | Não implementado | Aspiracional |
| Conjunção, disjunção, quantificadores na fórmula | Não implementado | Aspiracional |

**Estruturas aspiracionais DEVEM ser rotuladas como tal em toda a documentação.** Elas representam direções futuras pretendidas, não propriedades atuais do sistema.

---

## Posicionamento de Engenharia

O STF-SIR DEVE ser classificado e descrito como os seguintes tipos de sistemas de ciência da computação:

1. **Sistema de compilação semântica.** Transforma documentos-fonte (atualmente `text/markdown`) em artefatos estruturados, versionados e determinísticos (`.zmd`) com quatro dimensões semânticas explícitas por token.

2. **Sistema de representação intermediária (SIR).** O grafo de Representação Semântica Intermediária fornece uma camada estável e endereçável entre a fonte bruta e os componentes de raciocínio a jusante. O teste de pertença ao grafo SIR é a verificação de fundamentação primária para enunciados compilados.

3. **Motor de validação de coerência.** O `StfEngine<L, I, G>` avalia se um enunciado candidato pode ser integrado coerentemente a uma teoria, reportando o triplo de coerência, o status de fundamentação, as consequências derivadas e uma lista de erros tipificados.

4. **Infraestrutura auditável de raciocínio.** Cada enunciado em uma teoria compilada carrega proveniência rastreável (SHA-256 da fonte, âncoras de span em bytes, âncoras de id de ZToken). Essa cadeia de proveniência é o fundamento da garantia de auditabilidade do sistema. Nenhuma etapa de raciocínio é anônima.

---

## Consequências

### Consequências positivas

- Documentação e implementação ficam alinhadas, reduzindo o risco de regressões invisíveis onde os testes passam mas as propriedades declaradas já não valem.
- O modelo de coerência computacional torna-se defensável: uma aproximação por orçamento de passos é honesta e útil; uma afirmação de pertença a classe de complexidade não é.
- A fundamentação torna-se uma porta de entrada de primeira classe para informação útil, fechando a lacuna entre a taxonomia de erros e o motor de coerência.
- Os documentos filosóficos podem ser mantidos e expandidos sem que sejam confundidos com requisitos de implementação.
- Futuros colaboradores têm um critério claro sobre o que constitui uma afirmação válida sobre o sistema.

### Restrições aceitas

- O AST de fórmulas permanecerá proposicional até que uma necessidade concreta de implementação para uma gramática mais rica seja identificada e especificada.
- O espaço STS e o universo conjuntístico estão fora do escopo do trabalho de engenharia de v1 e v2.
- Afirmações sobre a correspondência **P** vs **NP** estão fora do escopo da implementação e NÃO DEVEM aparecer em seções normativas da especificação.

---

## Alternativas Consideradas

### Alternativa A: Manter a documentação atual, aceitar a lacuna

**Rejeitada.** A lacuna entre propriedades afirmadas e propriedades aplicadas é um risco de correção. À medida que o sistema é estendido, lacunas não documentadas tornam-se dependências ocultas. O custo da correção agora é menor do que o custo de manter uma especificação desalinhada.

### Alternativa B: Remover todo enquadramento filosófico e teórico

**Rejeitada.** O enquadramento filosófico (coerência como primitivo, a definição de ICE, a taxonomia de erros) fornece clareza conceitual genuína que orienta decisões de engenharia. Ele deve ser mantido, mas com escopo corretamente definido como não normativo.

### Alternativa C: Implementar caracterização completa de classe de complexidade para `C_c`

**Rejeitada por razões de escopo.** Determinar se um problema de coerência específico está em **P** requer uma redução específica ao problema. O modelo de orçamento de passos é um proxy de engenharia correto, honesto e suficiente para a mesma preocupação prática: esta verificação termina dentro de um custo aceitável para este agente?

### Alternativa D: Expandir o AST de fórmulas para lógica de primeira ordem antes do reposicionamento

**Rejeitada.** A expansão do AST é uma tarefa de engenharia independente. A decisão de posicionamento não está bloqueada pela completude da gramática. O subconjunto proposicional é suficiente para as regras de inferência implementadas (modus ponens) e para a detecção de contradições. A extensão DEVERIA seguir a demanda, não a especulação.

---

## Implicações para a Implementação

As seguintes ações corretivas são OBRIGATÓRIAS para colocar a base de código em conformidade com este ADR:

### I-1: Aplicar fundamentação em `useful_information` (Regra 3.2)

**Arquivo:** `src/compiler/engine.rs`

O método `evaluate_statement` DEVE ser atualizado para que:

```rust
// Antes (atual):
let useful_information = logical_ok && operational_ok;

// Depois (obrigatório):
let useful_information = logical_ok && operational_ok && grounding_result.is_grounded;
```

O método `audit_theory` DEVE ser atualizado de forma consistente:

```rust
// Antes (atual):
useful_information: logical_ok && operational_ok,

// Depois (obrigatório):
useful_information: logical_ok && operational_ok && ungrounded_ids.is_empty(),
```

Essa mudança é retroincompatível para chamadores que inspecionam `useful_information` em enunciados coerentes mas não fundamentados. Todos os testes afetados DEVEM ser atualizados para refletir a semântica corrigida.

### I-2: Adicionar cabeçalho não normativo aos documentos filosóficos (Regra 3.5)

**Arquivos:** `docs/coherence-paper.tex`, `docs/sts-paper.tex`, `docs/coherence-foundations.md`

Cada documento DEVE incluir um aviso não normativo em seu preâmbulo ou resumo. Para documentos LaTeX, um bloco `\begin{quote}...\end{quote}` imediatamente após o resumo é suficiente.

### I-3: Alinhar a seção do modelo computacional em `coherence-paper.tex` (Regra 3.3)

**Arquivo:** `docs/coherence-paper.tex`

A seção que descreve `C_c` e a correspondência com **P** vs **NP** DEVE ser revisada para descrever a aproximação por orçamento de passos como a definição operacional, com o enquadramento de teoria de complexidade explicitamente rebaixado a motivação e contexto. A revisão NÃO DEVE remover a discussão teórica; ela DEVE esclarecer seu caráter não normativo.

### I-4: Adicionar rótulos `[aspiracional]` a estruturas matemáticas não implementadas (Regra 3.1)

**Arquivos:** `docs/sts-formalization.md`, `docs/sir-graph.md`, qualquer documento que referencie o espaço STS completo ou o universo conjuntístico `U`

Qualquer estrutura matemática listada como "Aspiracional" na tabela de Posicionamento Matemático DEVE ser rotulada `[aspiracional]` na documentação onde aparece.

### I-5: Estabelecer a `bridge` como ponto de entrada canônico da API (Regra 3.4)

A superfície de API pública exposta por `src/lib.rs` DEVE documentar `artifact_to_theory` como o ponto de entrada primário para converter artefatos compilados em teorias. O uso da CLI DEVE ser documentado como uma interface voltada ao usuário do pipeline, distinta da API da biblioteca. Nenhum novo exemplo de documentação DEVE usar a invocação da CLI como substituto para demonstrar a API da biblioteca.

### I-6: Priorizar backends cientes de fórmulas na seleção padrão do motor (Regra 3.4)

**Arquivo:** `src/compiler/engine.rs`

O alias de tipo `DefaultEngine` atualmente usa `SimpleLogicChecker` e `RuleBasedInferenceEngine`. Um alias `RecommendedEngine` DEVERIA ser introduzido usando `FormulaCoherenceChecker` e `FormulaInferenceEngine` com um orçamento explícito escolhido pelo chamador. O alias `DefaultEngine` DEVE ser mantido por compatibilidade retroativa, mas DEVERIA ser marcado como `#[deprecated]` em uma versão subsequente.

---

## Direções Futuras

As seguintes direções são consistentes com este ADR e representam um caminho de evolução coerente. Elas não são compromissos.

- **Extensão do AST de fórmulas:** Adicionar `And`, `Or` e eventualmente `ForAll`/`Exists` conforme o motor de inferência os exija. Cada adição DEVE ser motivada por um caso de teste concreto, não por completude teórica.
- **Níveis graduados de fundamentação:** Estender o predicado de fundamentação para retornar uma pontuação graduada em vez de um booleano, permitindo o reporte de fundamentação parcial sem quebrar a porta binária em `useful_information`.
- **Composição de teorias multidocumento:** Definir uma operação de união de teorias com rastreamento de proveniência entre artefatos-fonte.
- **Interface de consulta sobre teorias:** Implementar o motor de consulta semântica (EPIC-203) sobre a teoria derivada pela ponte, e não diretamente sobre ZTokens brutos.
- **Diff semântico:** Implementar diff de teoria (EPIC-204) usando a variação do vetor de coerência como sinal primário.
- **Benchmark de retenção:** Estabelecer valores de referência de `ρ` para classes conhecidas de documentos (EPIC-205).

Essas direções DEVEM permanecer sujeitas à Regra 3.1: nenhuma direção futura passa de aspiracional a implementada enquanto não estiver aplicada em código e coberta por testes.

---

## Referências

- `spec/stf-sir-spec-v1.md` — Especificação operacional v1 (normativa)
- `src/compiler/engine.rs` — Implementação do `StfEngine`
- `src/model/bridge.rs` — Função ponte `artifact_to_theory`
- `src/model/coherence.rs` — `CoherenceVector`, `TruthValue`
- `src/compiler/coherence.rs` — Backends de `LogicalCoherenceChecker`
- `src/compiler/grounding.rs` — Backends de `GroundingChecker`
- `src/compiler/inference.rs` — Backends de `InferenceEngine`
- `src/model/formula.rs` — AST de fórmulas
- `src/retention/mod.rs` — Métrica de retenção
- `docs/coherence-paper.tex` — Artigo de teoria de coerência (não normativo após I-2)
- `docs/v1-invariants.md` — Registro de invariantes v1
- `docs/roadmap/ROADMAP-STF-SIR-V2.md` — Roadmap v2
