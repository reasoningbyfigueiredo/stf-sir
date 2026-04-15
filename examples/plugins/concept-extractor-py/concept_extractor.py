#!/usr/bin/env python3
"""
STF-SIR External Enricher — Concept Extractor (stf-sir-enricher-v1)

Reads a JSON request from stdin, extracts simple concepts (capitalized words
and technical terms) from each token's gloss, and writes a JSON response to
stdout.

Protocol: stf-sir-enricher-v1
Namespace: example.concept-extractor

Usage:
    echo '{"protocol":"stf-sir-enricher-v1","artifact_id":"test","tokens":[
      {"id":"z1","gloss":"The System design pattern","node_type":"paragraph","extensions":{}}
    ]}' | python3 concept_extractor.py
"""

import json
import re
import sys
from typing import Any

PROTOCOL = "stf-sir-enricher-v1"
MAX_CONCEPTS = 5

# Technical terms that should be recognized even when lowercase
TECHNICAL_TERMS = {
    "api", "url", "uri", "json", "yaml", "html", "css", "sql", "xml",
    "http", "https", "rest", "graphql", "grpc", "tcp", "ip", "dns",
}


def extract_concepts(gloss: str) -> list[str]:
    """
    Extract concepts from a gloss string.

    Rules:
    1. Capitalized words (Title Case) are treated as named concepts.
    2. Well-known technical terms (regardless of case) are included.
    3. Stop words are excluded.
    4. Maximum MAX_CONCEPTS concepts are returned (most salient first).
    5. Duplicates are removed while preserving order.
    """
    stop_words = {
        "The", "A", "An", "In", "On", "At", "To", "For", "Of", "And",
        "Or", "But", "Is", "Are", "Was", "Were", "Be", "Been", "Has",
        "Have", "Had", "Do", "Does", "Did", "Will", "Would", "Can",
        "Could", "Should", "May", "Might", "This", "That", "These",
        "Those", "It", "Its", "With", "From", "By", "As", "If", "So",
    }

    concepts: list[str] = []
    seen: set[str] = set()

    # Find capitalized words
    capitalized = re.findall(r'\b([A-Z][a-zA-Z0-9]+(?:-[A-Z][a-zA-Z0-9]+)*)\b', gloss)
    for word in capitalized:
        if word not in stop_words and word not in seen:
            concepts.append(word)
            seen.add(word)

    # Find technical terms
    words_lower = re.findall(r'\b([a-zA-Z][a-zA-Z0-9_-]+)\b', gloss.lower())
    for word in words_lower:
        if word in TECHNICAL_TERMS and word.upper() not in seen and word not in seen:
            concepts.append(word.upper())
            seen.add(word.upper())

    return concepts[:MAX_CONCEPTS]


def process_request(request: dict[str, Any]) -> dict[str, Any]:
    """Process a single enricher request and return the response."""
    if request.get("protocol") != PROTOCOL:
        return {
            "protocol": PROTOCOL,
            "enrichments": [],
            "error": f"unsupported protocol: {request.get('protocol')!r}",
        }

    enrichments = []
    for token in request.get("tokens", []):
        token_id = token.get("id", "")
        gloss = token.get("gloss", "")
        concepts = extract_concepts(gloss)
        if concepts:
            enrichments.append({
                "token_id": token_id,
                "extensions": {
                    "concepts": concepts,
                    "source": "concept-extractor-v1",
                },
            })

    return {
        "protocol": PROTOCOL,
        "enrichments": enrichments,
    }


def main() -> None:
    """Main loop: read NDJSON from stdin, write NDJSON to stdout."""
    for line in sys.stdin:
        line = line.strip()
        if not line:
            continue

        try:
            request = json.loads(line)
        except json.JSONDecodeError as exc:
            error_response = {
                "protocol": PROTOCOL,
                "enrichments": [],
                "error": f"JSON parse error: {exc}",
            }
            print(json.dumps(error_response), flush=True)
            continue

        response = process_request(request)
        print(json.dumps(response), flush=True)


if __name__ == "__main__":
    main()
