#!/usr/bin/env bash
# validate-migration-v1-v2.sh
# Validates one or more .zmd JSON files against the ZMD v2 schema.
#
# Usage:
#   ./scripts/validate-migration-v1-v2.sh <file-or-dir> [file-or-dir ...]
#
# Exit codes:
#   0 — all files pass v2 schema validation
#   1 — one or more files fail validation or a required tool is missing
#
# Dependencies:
#   check-jsonschema (pip install check-jsonschema)  — preferred
#   OR ajv-cli (npm install -g ajv-cli)             — fallback
#   OR python3 with jsonschema package               — fallback
#
# The script auto-detects which validator is available and uses the first one found.

set -euo pipefail

SCHEMA_PATH="$(dirname "$0")/../schemas/zmd-v2.schema.json"
PASS=0
FAIL=0
ERRORS=()

# ── detect validator ──────────────────────────────────────────────────────────
detect_validator() {
    if command -v check-jsonschema &>/dev/null; then
        echo "check-jsonschema"
    elif command -v ajv &>/dev/null; then
        echo "ajv"
    elif python3 -c "import jsonschema" &>/dev/null 2>&1; then
        echo "python3"
    else
        echo "none"
    fi
}

validate_file_check_jsonschema() {
    local file="$1"
    check-jsonschema --schemafile "$SCHEMA_PATH" "$file" &>/dev/null
}

validate_file_ajv() {
    local file="$1"
    ajv validate -s "$SCHEMA_PATH" -d "$file" &>/dev/null
}

validate_file_python() {
    local file="$1"
    python3 - <<EOF
import json, sys
try:
    import jsonschema
except ImportError:
    sys.exit(2)
with open("$SCHEMA_PATH") as f:
    schema = json.load(f)
with open("$file") as f:
    instance = json.load(f)
try:
    jsonschema.validate(instance, schema)
except jsonschema.ValidationError as e:
    print(e.message, file=sys.stderr)
    sys.exit(1)
EOF
}

validate_file() {
    local file="$1"
    local validator="$2"
    case "$validator" in
        check-jsonschema) validate_file_check_jsonschema "$file" ;;
        ajv)              validate_file_ajv "$file" ;;
        python3)          validate_file_python "$file" ;;
    esac
}

# ── argument handling ─────────────────────────────────────────────────────────
if [[ $# -eq 0 ]]; then
    echo "Usage: $0 <file-or-dir> [file-or-dir ...]" >&2
    exit 1
fi

if [[ ! -f "$SCHEMA_PATH" ]]; then
    echo "ERROR: Schema not found at $SCHEMA_PATH" >&2
    exit 1
fi

VALIDATOR="$(detect_validator)"
if [[ "$VALIDATOR" == "none" ]]; then
    echo "ERROR: No JSON Schema validator found." >&2
    echo "Install one of: check-jsonschema, ajv-cli, or python3 jsonschema" >&2
    exit 1
fi

echo "Using validator: $VALIDATOR"
echo "Schema: $SCHEMA_PATH"
echo ""

# ── collect files ─────────────────────────────────────────────────────────────
FILES=()
for arg in "$@"; do
    if [[ -d "$arg" ]]; then
        while IFS= read -r -d '' f; do
            FILES+=("$f")
        done < <(find "$arg" -name "*.zmd" -o -name "*.json" | sort | tr '\n' '\0')
    elif [[ -f "$arg" ]]; then
        FILES+=("$arg")
    else
        echo "WARNING: $arg not found, skipping" >&2
    fi
done

if [[ ${#FILES[@]} -eq 0 ]]; then
    echo "No .zmd or .json files found in the given paths." >&2
    exit 1
fi

# ── validate ──────────────────────────────────────────────────────────────────
for file in "${FILES[@]}"; do
    printf "  %-60s " "$file"
    if validate_file "$file" "$VALIDATOR"; then
        echo "PASS"
        ((PASS++))
    else
        echo "FAIL"
        ((FAIL++))
        ERRORS+=("$file")
    fi
done

echo ""
echo "Results: $PASS passed, $FAIL failed"

if [[ $FAIL -gt 0 ]]; then
    echo ""
    echo "Failed files:"
    for f in "${ERRORS[@]}"; do
        echo "  - $f"
    done
    exit 1
fi

exit 0
