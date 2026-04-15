---
id: PROFILES-V2
version: 2.0.0-alpha
status: draft
created: 2026-04-14
updated: 2026-04-14
normative: true
tags:
  - profiles
  - v2
  - ztoken
---

# STF-SIR Profile Registry v2

This document is the normative registry of ZToken profiles for STF-SIR v2.
Profiles define which fields are mandatory (MUST), recommended (SHOULD), or
optional (MAY) for a given compilation granularity.

---

## 1. Overview

A **profile** is a named configuration that constrains:
1. Which `node_type` values are valid for this granularity.
2. Which ZToken dimensions and fields are required.
3. Which validation rules (VAL_XX) apply.

The profile identifier MUST be declared in `compiler.profile` in the artifact.
A v2 reader MUST reject an artifact whose `node_type` values do not match the
declared profile.

---

## 2. Profile: `stf-sir-spec-v1-mvp` (preserved from v1)

**Granularity:** Block-level (paragraphs, headings, lists, etc.)
**Stability:** Stable (frozen at v1)

### 2.1 Valid `node_type` values

`heading`, `paragraph`, `blockquote`, `list`, `list_item`, `code_block`,
`table`, `footnote_definition`

### 2.2 Field optionality

| Field | Required |
|---|---|
| `id` | MUST |
| `lexical.source_text` | MUST |
| `lexical.span` | MUST |
| `syntactic.node_type` | MUST |
| `syntactic.depth` | MUST |
| `syntactic.sibling_index` | MUST |
| `syntactic.parent_id` | MAY (absent for root tokens) |
| `semantic.gloss` | MUST |
| `logical.formula` | MAY |
| `logical.relation_ids` | MAY |
| `context.*` | MUST NOT (not part of v1 profile) |
| `pragmatic.*` | MUST NOT |
| `temporal.*` | MUST NOT |
| `coherence_eval.*` | MUST NOT |

### 2.3 Applicable validation rules

VAL_01 through VAL_18 (all v1 rules).

---

## 3. Profile: `stf-sir-spec-v2-block`

**Granularity:** Block-level (same `node_type` set as v1-mvp)
**Stability:** Draft

This profile is identical to `stf-sir-spec-v1-mvp` in `node_type` set and
L/S/Σ/Φ requirements, but adds support for the Ω dimension.

### 3.1 Valid `node_type` values

Same as `stf-sir-spec-v1-mvp`.

### 3.2 Field optionality

| Field | Required |
|---|---|
| `id` | MUST |
| `lexical.*` | MUST (same as v1) |
| `syntactic.*` | MUST (same as v1) |
| `semantic.*` | MUST (same as v1) |
| `logical.*` | MAY (same as v1) |
| `context.*` | MAY |
| `pragmatic.*` | MAY |
| `temporal.*` | MAY |
| `coherence_eval.coherence_score` | SHOULD |
| `coherence_eval.validation_flags` | MAY |
| `coherence_eval.useful_information` | SHOULD |

### 3.3 Applicable validation rules

VAL_01 through VAL_30.

---

## 4. Profile: `stf-sir-spec-v2-sentence`

**Granularity:** Sentence-level (fine-grained linguistic units)
**Stability:** Draft

### 4.1 Valid `node_type` values

`sentence`, `clause`, `phrase`

### 4.2 Field optionality

| Field | Required |
|---|---|
| `id` | MUST |
| `lexical.source_text` | MUST |
| `lexical.span` | MUST |
| `syntactic.node_type` | MUST (from sentence set) |
| `syntactic.depth` | MUST |
| `syntactic.sibling_index` | MUST |
| `syntactic.parent_id` | MAY |
| `semantic.gloss` | MUST |
| `logical.formula` | MAY |
| `context.context_id` | SHOULD |
| `context.scope` | MAY |
| `pragmatic.intent` | SHOULD |
| `pragmatic.speech_act` | MAY |
| `temporal.*` | MAY |
| `coherence_eval.*` | MAY |

### 4.3 Applicable validation rules

VAL_01–VAL_18, VAL_19, VAL_21, VAL_22, VAL_23, VAL_24, VAL_25, VAL_28.

### 4.4 Notes

- A sentence-profile artifact SHOULD contain at least one `sentence` token.
- Parent tokens at sentence level are typically `paragraph` (block-level) tokens.
  Cross-profile parent references are permitted.

---

## 5. Profile: `stf-sir-spec-v2-entity`

**Granularity:** Entity/span-level (named entities, mentions, coreferences)
**Stability:** Draft

### 5.1 Valid `node_type` values

`entity`, `span`, `mention`

### 5.2 Field optionality

| Field | Required |
|---|---|
| `id` | MUST |
| `lexical.source_text` | MUST |
| `lexical.span` | MUST |
| `syntactic.node_type` | MUST (from entity set) |
| `syntactic.depth` | MUST |
| `syntactic.sibling_index` | MAY |
| `syntactic.parent_id` | SHOULD (points to containing sentence or block) |
| `semantic.gloss` | MUST |
| `semantic.tags` | SHOULD (entity type labels, e.g., `["PERSON", "LOCATION"]`) |
| `logical.formula` | MAY |
| `context.context_id` | SHOULD |
| `context.reference_frame` | MAY |
| `pragmatic.*` | MAY |
| `temporal.*` | MAY |
| `coherence_eval.*` | MAY |

### 5.3 Applicable validation rules

VAL_01–VAL_18, VAL_19, VAL_21, VAL_24, VAL_25, VAL_29.

### 5.4 Notes

- Entity tokens SHOULD be connected to their containing block or sentence
  via a `refers_to` relation.
- Coreference chains are represented as `refers_to` relations between
  `mention` tokens sharing the same `context.context_id`.

---

## 6. Profile Registration

New profiles MUST be registered in this document before use. A profile
registration MUST include:

1. Profile identifier (string, unique, no spaces)
2. Granularity description
3. Valid `node_type` values
4. Field optionality table
5. Applicable validation rules
6. At least 2 valid and 2 invalid conformance fixtures

Experimental profiles MAY use the prefix `exp:` (e.g., `exp:my-profile`).
They are not subject to the conformance kit requirement until stabilized.
