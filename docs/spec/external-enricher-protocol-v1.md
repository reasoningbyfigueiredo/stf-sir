# External Enricher Protocol v1

**Version:** 1.0.0  
**Status:** Draft  
**Author:** STF-SIR Project  
**Date:** 2026-04-14

---

## Overview

The **External Enricher Protocol v1** defines how external processes written in any programming
language (Python, TypeScript, Julia, etc.) communicate with the STF-SIR compiler to enrich
ZToken artifacts. Communication is **JSON-over-stdin/stdout** (one JSON object per line, NDJSON).

No Rust FFI or shared libraries are required. The host (STF-SIR) and enricher exchange
newline-delimited JSON messages. This keeps the protocol language-agnostic and simple to
implement.

---

## Protocol Version Field

Every message MUST include `"protocol": "stf-sir-enricher-v1"`. The host rejects responses
that carry a different protocol string.

---

## Request Format

The host writes a single JSON line to the enricher's **stdin**:

```json
{
  "protocol": "stf-sir-enricher-v1",
  "artifact_id": "<artifact-sha256>",
  "tokens": [
    {
      "id": "z1",
      "gloss": "Introduction to semantic compilation",
      "node_type": "heading",
      "extensions": {}
    },
    {
      "id": "z2",
      "gloss": "The system processes Markdown documents",
      "node_type": "paragraph",
      "extensions": {}
    }
  ]
}
```

| Field         | Type            | Description                                             |
|---------------|-----------------|---------------------------------------------------------|
| `protocol`    | `string`        | Must be `"stf-sir-enricher-v1"`                         |
| `artifact_id` | `string`        | SHA-256 of the source document                          |
| `tokens`      | `array<object>` | ZTokens to be enriched                                  |
| `token.id`    | `string`        | Unique ZToken ID (e.g. `"z1"`)                          |
| `token.gloss` | `string`        | Semantic gloss (`Σ.gloss`) — the primary text for NLP   |
| `token.node_type` | `string`   | Syntactic node type (`"heading"`, `"paragraph"`, etc.)  |
| `token.extensions` | `object`  | Current extension data for the token (read-only)        |

---

## Response Format

The enricher writes a single JSON line to its **stdout**:

```json
{
  "protocol": "stf-sir-enricher-v1",
  "enrichments": [
    {
      "token_id": "z1",
      "extensions": {
        "concepts": ["semantic compilation", "introduction"]
      }
    },
    {
      "token_id": "z2",
      "extensions": {
        "concepts": ["system", "Markdown", "document processing"]
      }
    }
  ]
}
```

| Field                       | Type            | Description                                        |
|-----------------------------|-----------------|----------------------------------------------------|
| `protocol`                  | `string`        | Must echo `"stf-sir-enricher-v1"`                  |
| `enrichments`               | `array<object>` | One entry per enriched token (may omit tokens)     |
| `enrichment.token_id`       | `string`        | Must match a `token.id` from the request           |
| `enrichment.extensions`     | `object`        | Data merged into `token.extensions[namespace]`     |

**Important:** Enrichments may be a strict subset of the requested tokens. Tokens absent from
the response are left unchanged.

---

## Example: Python Enricher

Below is a minimal Python enricher that extracts concepts (capitalized words) from each token's
gloss and returns them in the `concepts` field.

```python
#!/usr/bin/env python3
"""Minimal STF-SIR external enricher (stf-sir-enricher-v1)."""
import json
import re
import sys

PROTOCOL = "stf-sir-enricher-v1"

def extract_concepts(gloss: str) -> list[str]:
    """Return capitalized words and technical terms from gloss."""
    words = re.findall(r'\b[A-Z][a-zA-Z0-9]+\b', gloss)
    return list(dict.fromkeys(words))[:5]  # deduplicate, max 5

def main() -> None:
    for line in sys.stdin:
        line = line.strip()
        if not line:
            continue
        try:
            request = json.loads(line)
        except json.JSONDecodeError as e:
            print(json.dumps({"error": f"JSON parse error: {e}"}), flush=True)
            continue

        if request.get("protocol") != PROTOCOL:
            print(json.dumps({"error": "unknown protocol"}), flush=True)
            continue

        enrichments = []
        for token in request.get("tokens", []):
            concepts = extract_concepts(token.get("gloss", ""))
            if concepts:
                enrichments.append({
                    "token_id": token["id"],
                    "extensions": {"concepts": concepts}
                })

        response = {"protocol": PROTOCOL, "enrichments": enrichments}
        print(json.dumps(response), flush=True)

if __name__ == "__main__":
    main()
```

### Testing with echo

```bash
echo '{"protocol":"stf-sir-enricher-v1","artifact_id":"test","tokens":[{"id":"z1","gloss":"The System design pattern","node_type":"paragraph","extensions":{}}]}' \
  | python3 concept_extractor.py
```

Expected output:
```json
{"protocol": "stf-sir-enricher-v1", "enrichments": [{"token_id": "z1", "extensions": {"concepts": ["System"]}}]}
```

---

## Error Handling Rules

1. **Invalid JSON input** — the enricher SHOULD write a JSON error object to stdout and
   continue processing subsequent lines. It MUST NOT crash.

2. **Wrong protocol version** — if `protocol` is not `"stf-sir-enricher-v1"`, the enricher
   MUST reject the request with an error response.

3. **Unknown token IDs** — if `enrichment.token_id` does not match any token in the request,
   the host silently ignores the enrichment entry.

4. **Process timeout** — the host enforces a per-request timeout (default: 5 seconds). If the
   enricher does not respond within the timeout, the host returns the original tokens unmodified
   and logs a warning. The enricher process is terminated.

5. **Non-JSON output** — if the enricher writes a non-JSON line to stdout, the host treats it
   as a hard error and rejects the entire response.

---

## Namespace Isolation Guarantee

Every enrichment result is stored at `token.extensions[namespace]`, where `namespace` is the
namespace registered in the `NamespaceRegistry` by the host-side `ExternalEnricher` descriptor.

The host performs this merge — the enricher never writes directly to the token struct. This
means:

- Enrichers cannot overwrite each other's data (namespaces are isolated).
- Enrichers cannot modify core ZToken fields (`id`, `gloss`, `node_type`, spans).
- The `stf-sir`, `stf`, and `sir` namespaces are permanently reserved and cannot be used.

An enricher that returns data for a namespace other than its declared namespace will have that
data silently dropped by the host.

---

## Protocol Version History

| Version | Date       | Changes              |
|---------|------------|----------------------|
| v1      | 2026-04-14 | Initial release      |
