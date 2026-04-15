---
id: coherence-foundations
version: 2.0.0-draft
status: draft
created: 2026-04-14
updated: 2026-04-14
authors:
  - Rogerio Figueiredo
  - AI Architect Auditor (claude-sonnet-4-6)
license: Apache-2.0
---

# Fundamentos Computacionais da Coerencia

> **Versao:** 2.0.0-draft | **Status:** rascunho formal-computacional | **Data:** 2026-04-14

> **NAO NORMATIVO.**
> Este documento apresenta contexto filosofico, hipoteses fundacionais e motivacao
> teorica para o sistema STF-SIR. Ele **nao** define requisitos de implementacao,
> contratos de API nem criterios de aceitacao de testes. O posicionamento normativo
> do sistema esta estabelecido em `docs/adr/ADR-SEM-001-positioning.md`.

---

## Resumo

Este documento apresenta uma formulacao computacional da coerencia para sistemas
semanticos orientados a artefatos, teoria e auditoria. Em vez de tratar
coerencia como propriedade informal recuperada apos a interpretacao, o projeto
STF-SIR a trata como um invariante verificavel sobre representacoes semanticas estruturadas.
Definimos tres camadas de coerencia:

- **coerencia logica**, associada a ausencia de contradicao interna;
- **coerencia computacional**, associada a verificabilidade tratavel sob um
  orcamento de recursos;
- **coerencia operacional**, associada a capacidade de produzir consequencias
  nao-triviais e executaveis.

Com base nessas camadas, definimos quando informacao e admissivel, util e
auditavel em um sistema dirigido por teoria. Tambem formalizamos grounding como
uma relacao verificavel entre representacoes derivadas e artefatos-fonte ou
estruturas SIR, introduzimos uma ponte explicita entre semantica de artefato e
raciocinio em nivel de teoria, e distinguimos tres classes centrais de erro:
contradicao, anomalia e alucinacao.

O objetivo deste documento e posicionar STF-SIR como infraestrutura de sistemas
semanticos, IR e metodos formais, e nao como teoria baseada em fisica,
cosmologia ou hipoteses metafisicas.

---

## 1. Modelo de Sistema e Escopo

STF-SIR e tratado como um projeto de ciencia da computacao focado em:

- compilacao semantica deterministica;
- validacao de coerencia;
- grounding e proveniencia verificavel;
- retencao como metrica operacional de preservacao;
- informacao executavel em camadas de teoria.

Neste contexto:

- coerencia e tratada como um invariante verificavel sobre representacoes semanticas estruturadas;
- informacao so e util quando preserva coerencia e produz consequencias
  executaveis;
- grounding e uma relacao verificavel com artefatos-fonte ou estruturas SIR;
- contradicao, anomalia e alucinacao sao classes distintas de erro;
- `Artifact`, `Statement`, `Theory`, `Formula` e `SIR` sao as abstracoes
  internas centrais.

Este documento e normativo no nivel do modelo formal. Ele nao depende de
hipoteses fisicas para justificar o sistema, e separa deliberadamente:

- **formalismo de pesquisa**;
- **arquitetura de implementacao**;
- **metodos de auditoria e validacao**.

---

## 2. Modelo Formal do Sistema

### 2.1 Artifact

Um `Artifact` e a unidade compilada e serializavel do sistema. Ele preserva:

- informacoes de fonte;
- metadados de compilacao;
- colecao ordenada de `ztokens`;
- relacoes tipadas;
- diagnosticos e extensoes.

O artefato e o objeto canonical de serializacao, validacao e reproducibilidade.

### 2.2 SIR

`SIR` e a representacao intermediaria semantica do documento compilado. Na
implementacao de referencia, o artefato `.zmd` e a realizacao persistente
principal dessa representacao, enquanto `SirGraph` e sua projecao orientada a
grafo.

### 2.3 Statement

Um `Statement` e a unidade teorica de raciocinio. Ele representa uma proposicao
ou afirmacao auditavel, acompanhada de:

- texto normalizado;
- dominio interpretativo;
- proveniencia;
- metadados estruturais;
- formula logica opcional.

### 2.4 Theory

Uma `Theory` e um conjunto estruturado de `Statement`s sobre o qual:

- coerencia pode ser verificada;
- inferencia pode ser executada;
- grounding pode ser auditado;
- erros formais podem ser classificados.

### 2.5 Formula

Uma `Formula` e a representacao logica minima usada por STF-SIR para:

- detectar contradicoes estruturais;
- aplicar regras inferenciais;
- evitar dependencia excessiva de comparacao textual superficial.

No baseline atual, a AST suporta ao menos:

- atomos;
- negacao;
- implicacao.

---

## 3. Coerencia como Invariante Estrutural

Definimos a coerencia de uma teoria \(T\) como um triplo:

\[
\mathrm{Coh}(T) = (C_l,\; C_c,\; C_o).
\]

### 3.1 Coerencia Logica

\[
C_l(T) = 1 \iff T \not\models \bot.
\]

`C_l` indica ausencia de contradicao interna. Uma teoria e logicamente coerente
quando nenhum subconjunto relevante de suas proposicoes gera inconsistencia.

### 3.2 Coerencia Computacional

\[
C_c(T, B) = 1 \iff \exists\, V_B \text{ tal que } V_B(T) \text{ verifica coerencia em } \leq B \text{ passos}.
\]

`C_c` modela a verificabilidade sob restricao de recursos. O parametro `B`
representa um orcamento de passos, tempo ou outro recurso computacional.

No formalismo geral, `C_c` esta associado a verificacao tratavel. Na
implementacao, ele pode ser aproximado por contagem de passos ou orcamentos
operacionais explicitos.

### 3.3 Coerencia Operacional

\[
C_o(T) = 1 \iff \exists\, I \text{ tal que } I(T) \neq \varnothing.
\]

`C_o` mede a capacidade de produzir consequencias nao-triviais por meio de um
motor de inferencia `I`. Uma teoria pode ser consistente e verificavel, mas
operacionalmente esteril se nao gera novas consequencias.

### 3.4 Classificacao

| \(C_l\) | \(C_c\) | \(C_o\) | Classificacao |
|:---:|:---:|:---:|---|
| 0 | - | - | Contraditoria |
| 1 | 0 | - | Intratavel |
| 1 | 1 | 0 | Esteril |
| 1 | 1 | 1 | Plenamente coerente |

Essa classificacao trata coerencia como propriedade auditavel e mensuravel do
sistema, nao como qualidade retorica.

---

## 4. Informacao Admissivel, Util e Auditavel

### 4.1 Informacao Admissivel

Uma mensagem \(m\) e admissivel em relacao a uma teoria \(T\) e um contexto de
grounding \(A\) quando:

\[
\mathrm{Adm}(m, T, A) = 1 \iff C_l(T \cup \{m\}) = 1 \wedge \mathrm{Ground}(m, A) = 1.
\]

Admissibilidade exige integracao sem contradicao e ancoragem verificavel.

### 4.2 Informacao Executavel

Uma mensagem \(m\) e executavel quando a teoria estendida produz consequencias
nao-triviais:

\[
\mathrm{Exec}(m, T) = 1 \iff C_o(T \cup \{m\}) = 1.
\]

### 4.3 Informacao Util

Para STF-SIR, informacao util deve satisfazer simultaneamente:

\[
\mathrm{Useful}(m, T, A) = 1 \iff \mathrm{Adm}(m, T, A) = 1 \wedge \mathrm{Exec}(m, T) = 1.
\]

Essa definicao incorpora a intuicao central do projeto: informacao so e
operacionalmente valiosa quando e coerente, grounded e executavel.

**Teorema (Caracterizacao da Informacao Util).**
Uma mensagem e util se, e somente se, for admissivel e executavel.

\[
\mathrm{Useful}(m, T, A) = 1
\iff
C_l(T \cup \{m\}) = 1
\;\wedge\;
C_o(T \cup \{m\}) = 1
\;\wedge\;
\mathrm{Ground}(m, A) = 1.
\]

**Prova.**
Segue diretamente das definicoes de admissibilidade e executabilidade:

\[
\mathrm{Adm}(m, T, A) = 1 \iff C_l(T \cup \{m\}) = 1 \wedge \mathrm{Ground}(m, A) = 1
\]

e

\[
\mathrm{Exec}(m, T) = 1 \iff C_o(T \cup \{m\}) = 1.
\]

Substituindo ambas na definicao de `Useful`, obtemos o resultado.

### 4.4 Informacao Auditavel

Uma mensagem e auditavel quando, alem de admissivel, existe trilha de
proveniencia suficiente para reconstruir:

- sua origem artefatual;
- seus anchors estruturais;
- suas dependencias inferenciais;
- os invariantes usados para aceita-la.

---

## 5. Grounding

Definimos grounding como uma relacao verificavel entre uma representacao derivada
e um objeto-fonte do sistema:

\[
\mathrm{Ground}(s, A) = 1
\]

quando existe evidencia verificavel de que o `Statement` \(s\) e ancorado em:

- um artefato-fonte;
- um `ztoken` ou anchor SIR;
- spans ou coordenadas de origem;
- proveniencia declarada e validavel.

Em STF-SIR, grounding nao e intuicao epistemologica, mas uma propriedade
operacional verificavel por:

- `source_ids`;
- `anchors`;
- spans do artefato;
- ids estaveis de `ztokens`;
- estruturas SIR derivadas.

Grounding e central para separar coerencia interna de correspondencia verificavel.

---

## 6. Ponte entre Semantica de Artefato e Raciocinio em Nivel de Teoria

Uma das contribuicoes estruturais do projeto e a ponte:

\[
\beta : \mathrm{Artifact} \to \mathrm{Theory}.
\]

Essa ponte permite converter informacao compilada em unidades explicitamente
raciocinaveis.

### 6.1 Mapeamento Basico

Para cada `ztoken` \(z\) em um `Artifact`, a ponte produz um `Statement`
correspondente que preserva:

- identidade estavel;
- texto normalizado;
- dominio estrutural;
- anchors e spans;
- ids de relacao;
- formula logica pre-processada, quando disponivel.

Para a acao da ponte em nivel de token, ainda denotada por \(\beta\), exigimos
os seguintes invariantes:

\[
\forall z_1 \neq z_2,\; \beta(z_1) \neq \beta(z_2)
\]

e

\[
\mathrm{Ground}(\beta(z), A) = 1 \iff \mathrm{Ground}(z, A) = 1.
\]

O primeiro expressa preservacao de identidade; o segundo expressa preservacao de
grounding entre a camada de artefato e a camada de teoria.

### 6.2 Consequencia Arquitetural

A ponte elimina uma fragilidade comum em sistemas semanticos: a separacao rigida
entre o objeto compilado e o objeto raciocinavel. Em STF-SIR:

- o artefato continua sendo o objeto canonical de serializacao e validacao;
- a teoria torna-se o objeto canonical de auditoria e inferencia;
- a ponte garante correspondencia rastreavel entre os dois niveis.

---

## 7. Taxonomia de Erros

### 7.1 Contradicao

\[
\mathrm{Contradiction}(T) \iff T \models \bot.
\]

Contradicao e erro de coerencia logica. Ela e detectavel por verificacao formal
interna e independe de referencia externa.

### 7.2 Alucinacao

\[
\mathrm{Hallucination}(s, A) \iff C_l(\{s\}) = 1 \wedge \mathrm{Ground}(s, A) = 0.
\]

Alucinacao e um item localmente coerente, mas sem ancoragem verificavel.
Por isso, ela nao se confunde com contradicao. O sistema pode ser internamente
fluente e ainda assim falhar em grounding.

Uma formulacao mais forte, orientada pela ponte artefato-teoria, e:

\[
\mathrm{Hallucination}(s, A)
\iff
C_l(\{s\}) = 1
\;\wedge\;
\neg \exists a \in A : \beta(a) = s.
\]

Essa formulacao torna explicito que a alucinacao e a falha de encontrar um
predecessor verificavel na camada de artefato ou em estruturas SIR admissiveis.

### 7.3 Anomalia

\[
\mathrm{Anomaly}(x, \mathcal{D}) \iff P_{\mathcal{D}}(x) < \epsilon.
\]

Anomalia e desvio estatistico ou distribucional em relacao a um dominio
\(\mathcal{D}\). Ela nao implica necessariamente contradicao nem falta de
grounding.

### 7.4 Distincao Estrutural

| Classe | Coerencia logica | Grounding | Natureza |
|---|---|---|---|
| Contradicao | violada | irrelevante | inconsistencia formal |
| Alucinacao | satisfeita | ausente | desconexao de fonte |
| Anomalia | pode estar satisfeita | pode estar presente | desvio distribucional |

Essa separacao impede que o sistema trate todos os erros como simples "falhas de
verdade". Cada classe exige mecanismo proprio de deteccao.

---

## 8. Coerencia Lexical, Retencao e Preservacao

### 8.1 Retencao como Proxy de Preservacao de Coerencia

Quando STF-SIR transforma ou mapeia representacoes, a preservacao estrutural
pode ser avaliada por retencao:

\[
\rho(d) = \langle \rho_L,\; \rho_S,\; \rho_\Sigma,\; \rho_\Phi \rangle.
\]

No baseline atual, `rho` mede completude e consistencia operacional das
dimensoes do artefato. Em niveis mais avancados, a mesma ideia pode ser estendida
para:

- coerencia lexical entre dominios;
- preservacao de grounding;
- preservacao de formulas;
- preservacao de relacoes SIR.

Retencao, portanto, nao substitui coerencia, mas fornece superficie mensuravel
para auditoria de preservacao semantica.

Como principio operacional, adotamos a seguinte implicacao:

\[
\rho(d) \geq \theta \Rightarrow \mathrm{Coh}(d) \text{ preservada}
\]

para um limiar \(\theta\) definido pelo perfil, corpus ou contrato de validacao.
Essa regra nao afirma que `rho` e identica a coerencia, mas que, no pipeline
auditavel do sistema, ela funciona como proxy mensuravel de preservacao de
coerencia.

### 8.2 Interpretacao Computacional

\[
C_c(T, B) \approx \text{verificacao em } \mathbf{P}
\]

Coerencia computacional aproxima a nocao de verificacao tratavel e estabelece
uma correspondencia com classes classicas de complexidade. Em particular:

- `C_c` conecta verificacao de coerencia a limites operacionais de custo;
- a distincao entre verificacao e construcao acompanha a distincao classica
  entre testemunho verificavel e busca inferencial;
- o uso de orcamentos de passos, tempo ou recursos fornece uma realizacao
  operacional dessa camada no sistema.

Assim, `C_c` liga diretamente o modelo de coerencia de STF-SIR a temas como:

- `P` vs `NP`;
- verificacao vs construcao;
- decidibilidade operacional sob orcamento.

---

## 9. Realizacao do Sistema

No ecossistema STF-SIR, estes fundamentos sao instanciados como:

- **compilacao semantica**: documentos sao compilados em `Artifact`s
  deterministas;
- **representacao intermediaria**: `SIR` e a camada estruturada de semantica
  compilada;
- **ponte artefato-teoria**: `Artifact` e convertido em `Theory` por uma
  funcao canonica;
- **coerencia logica**: `Formula` e usada para detectar contradicoes;
- **coerencia operacional**: motores de inferencia avaliam se a teoria produz
  consequencias nao-triviais;
- **grounding**: `Statement`s preservam `source_ids`, anchors e spans;
- **retencao**: `rho` fornece baselines de preservacao e consistencia;
- **auditoria**: erros e invariantes sao verificaveis no pipeline.

Essa instanciacao posiciona o projeto como infraestrutura de raciocinio e IR
semantico auditavel.

---

## 10. Limites Normativos

Este documento estabelece tres limites importantes:

1. Ele nao usa fisica, cosmologia ou hipoteses metafisicas como fundamento do
   sistema.
2. Ele separa formalismo de pesquisa de detalhes contingentes de implementacao.
3. Ele trata coerencia como objeto computacional e arquitetural, nao como
   especulacao filosofica normativa.

Consequentemente:

- referencias filosoficas podem inspirar discussao conceitual;
- o desenho normativo do sistema deve ser sustentado por logica, semantica,
  validacao e arquitetura;
- alegacoes de corretude devem ser formuladas em termos de invariantes,
  proveniencia, grounding, retencao e auditabilidade.

---

## Referencias Conceituais

- Tarski, A. (1936). *Der Wahrheitsbegriff in den formalisierten Sprachen.*
- Bar-Hillel, Y. & Carnap, R. (1953). *Semantic Information.*
- Hoare, C. A. R. (1969). *An Axiomatic Basis for Computer Programming.*
- Cousot, P. & Cousot, R. (1977). *Abstract Interpretation: A Unified Lattice Model for Static Analysis of Programs by Construction or Approximation of Fixpoints.*
- Ji, Z. et al. (2023). *Survey of Hallucination in Natural Language Generation.* ACM Computing Surveys.
- Figueiredo, R. (2026). *Semantic Token Space: A Multidimensional Formalization.* [docs/sts-formalization.md](sts-formalization.md)
- Figueiredo, R. (2026). [docs/spec/SPEC-STF-CORE-SEMANTICS.md](spec/SPEC-STF-CORE-SEMANTICS.md)
