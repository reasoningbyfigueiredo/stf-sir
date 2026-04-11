# STF-SIR v1 Invariants

This document defines the non-negotiable invariants of the STF-SIR v1 reference baseline. An artifact or compiler execution that violates any invariant below is outside the v1 contract.

## 1. Determinism

1. Given identical source bytes and identical effective compiler configuration, STF-SIR v1 MUST produce byte-identical `.zmd` output.
2. Determinism includes ztoken ordering, relation ordering, identifier allocation, normalization, and YAML field ordering.
3. Any intentional change to parser options, emitted node types, relation emission, normalization policy, or serialization policy is a configuration change and MUST be reflected by a new `compiler.config_hash`.

## 2. Valid Spans

1. Every ztoken span MUST satisfy `0 <= start_byte < end_byte <= source.length_bytes`.
2. Every ztoken span MUST satisfy `1 <= start_line <= end_line`.
3. `L.source_text` MUST equal the exact source slice denoted by `L.span` when the original source bytes are available during validation.
4. Line numbers are 1-based.

## 3. Unique Identifiers

1. Every ztoken id MUST be unique within an artifact.
2. Every relation id MUST be unique within an artifact.
3. The reference implementation allocates ztoken ids as `z1`, `z2`, `z3`, ... in preorder traversal order.
4. The reference implementation allocates relation ids as `r1`, `r2`, `r3`, ... in deterministic emission order.

## 4. Count Consistency

1. `document.token_count` MUST equal the serialized length of `ztokens`.
2. `document.relation_count` MUST equal the serialized length of `relations`.
3. `document.root_token_ids` MUST contain exactly the ztoken ids whose `S.parent_id` is `null`, in serialized token order.

## 5. Reference Integrity

1. Every non-null `S.parent_id` MUST reference an existing ztoken id.
2. Every relation `source` MUST reference an existing ztoken id.
3. Every relation `target` MUST reference an existing ztoken id unless the relation category explicitly permits external targets.
4. Every `Φ.relation_ids` entry MUST reference an existing relation id.

## 6. Semantic Fallback

1. The STF-SIR v1 semantic fallback is mandatory.
2. For every ztoken with non-empty `L.plain_text`, `Σ.gloss` MUST equal `L.normalized_text`.
3. For every ztoken with empty `L.plain_text`, `Σ.gloss` MUST be the empty string.
4. `L.normalized_text` is defined by Unicode NFKC normalization, followed by trimming and collapsing consecutive whitespace to a single ASCII space.

## 7. Relation Provenance

1. `relation.category` classifies the relation as `structural`, `logical`, or `semantic-link`.
2. `relation.stage` records the pipeline stage that emitted the relation; it does not encode the semantic class of the relation.
3. It is therefore valid for a relation to have `category: structural` and `stage: logical` when the logical stage emits a structural ordering or containment relation.

## 8. Validation Surface

1. The JSON Schema defines structural, required-field, and enum constraints.
2. The semantic validator defines cross-reference and consistency rules that are not expressible in JSON Schema alone.
3. A v1 artifact is conformant only when it satisfies both validation layers.
