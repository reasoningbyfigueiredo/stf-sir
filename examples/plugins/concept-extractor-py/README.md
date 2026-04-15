# Concept Extractor — STF-SIR Reference External Enricher

A reference implementation of the **STF-SIR External Enricher Protocol v1** written in Python.

It extracts simple "concepts" from each token's semantic gloss: capitalized words (Title Case)
and well-known technical terms (API, JSON, etc.).

## Requirements

- Python 3.9 or later
- No third-party packages required (stdlib only)

## Quick Start

```bash
echo '{"protocol":"stf-sir-enricher-v1","artifact_id":"test","tokens":[
  {"id":"z1","gloss":"The System design pattern","node_type":"paragraph","extensions":{}}
]}' | python3 concept_extractor.py
```

Expected output:
```json
{"protocol": "stf-sir-enricher-v1", "enrichments": [{"token_id": "z1", "extensions": {"concepts": ["System"], "source": "concept-extractor-v1"}}]}
```

## Testing

Run the self-contained test suite:

```bash
python3 -m doctest concept_extractor.py -v
```

Or run a multi-token roundtrip:

```bash
echo '{"protocol":"stf-sir-enricher-v1","artifact_id":"abc123","tokens":[
  {"id":"z1","gloss":"Introduction to the API design","node_type":"heading","extensions":{}},
  {"id":"z2","gloss":"The System uses REST and JSON over HTTP","node_type":"paragraph","extensions":{}},
  {"id":"z3","gloss":"a simple lowercase gloss","node_type":"paragraph","extensions":{}}
]}' | python3 concept_extractor.py
```

Expected: tokens `z1` and `z2` are enriched; `z3` has no concepts and is omitted from
the response.

## Protocol

See `docs/spec/external-enricher-protocol-v1.md` for the full protocol specification.

## Namespace

This enricher uses the namespace `example.concept-extractor`. In production, replace
`example` with your organization identifier (e.g. `acme.concept-extractor`).
