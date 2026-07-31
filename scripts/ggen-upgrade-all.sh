#!/usr/bin/env bash
set -euo pipefail

# Review and upgrade the complete tracked code graph through one bounded ggen
# projection. This script deliberately renders only code-modernization-pack;
# repository-global and sibling packs are outside this increment's write set.

ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
cd "$ROOT"

OUT=${GGEN_MODERNIZATION_OUT:-target/ggen-code-modernization}
mkdir -p "$OUT/empty-project-templates"

for command in git python3 ggen rustc b3sum; do
    command -v "$command" >/dev/null 2>&1 || {
        echo "ggen-upgrade-all: REFUSED: required command not found: $command" >&2
        exit 1
    }
done

TMP=$(mktemp -d)
cp ggen.toml "$TMP/ggen.toml"
if [[ -f ggen.lock ]]; then
    cp ggen.lock "$TMP/ggen.lock"
fi

restore_repository_config() {
    cp "$TMP/ggen.toml" ggen.toml
    if [[ -f "$TMP/ggen.lock" ]]; then
        cp "$TMP/ggen.lock" ggen.lock
    else
        rm -f ggen.lock
    fi
}

cleanup() {
    restore_repository_config
    rm -rf "$TMP"
}
trap cleanup EXIT INT TERM

write_isolated_config() {
    cat > ggen.toml <<TOML
[project]
name = "praxis-code-modernization"

[ontology]
source = "packs/code-modernization-pack/ontology.ttl"

[ontology.prefixes]
gmod = "https://praxis.chatman.io/ontology/ggen-modernization#"
rdf = "http://www.w3.org/1999/02/22-rdf-syntax-ns#"
rdfs = "http://www.w3.org/2000/01/rdf-schema#"
xsd = "http://www.w3.org/2001/XMLSchema#"

[packs]
code-modernization-pack = { path = "packs/code-modernization-pack", extra_ontologies = ["packs/code-modernization-pack/instances.ttl"] }

[templates]
dir = "$OUT/empty-project-templates"
TOML
    rm -f ggen.lock
}

render_once() {
    write_isolated_config
    ggen sync run
    restore_repository_config
}

python3 scripts/ggen-code-inventory.py build
render_once

# Rebuild after first render. The generated report, generated Rust verifier,
# instances graph, receipt, and ggen.lock are excluded from the admitted input
# fold, so this must converge rather than chase its own projections.
python3 scripts/ggen-code-inventory.py build
python3 scripts/ggen-code-inventory.py verify

first_test=$(b3sum crates/ggen/tests/ggen_code_inventory.rs | cut -d' ' -f1)
first_report=$(b3sum docs/standing/GGEN_CODE_MODERNIZATION.md | cut -d' ' -f1)
first_instances=$(b3sum packs/code-modernization-pack/instances.ttl | cut -d' ' -f1)

render_once
python3 scripts/ggen-code-inventory.py verify

second_test=$(b3sum crates/ggen/tests/ggen_code_inventory.rs | cut -d' ' -f1)
second_report=$(b3sum docs/standing/GGEN_CODE_MODERNIZATION.md | cut -d' ' -f1)
second_instances=$(b3sum packs/code-modernization-pack/instances.ttl | cut -d' ' -f1)

test "$first_test" = "$second_test" || {
    echo "ggen-upgrade-all: BUILD_BROKEN: Rust verifier is not idempotent" >&2
    exit 1
}
test "$first_report" = "$second_report" || {
    echo "ggen-upgrade-all: BUILD_BROKEN: modernization report is not idempotent" >&2
    exit 1
}
test "$first_instances" = "$second_instances" || {
    echo "ggen-upgrade-all: BUILD_BROKEN: admitted instances graph is not stable" >&2
    exit 1
}

CARGO_MANIFEST_DIR="$ROOT/crates/ggen" \
    rustc --edition=2021 --test \
        crates/ggen/tests/ggen_code_inventory.rs \
        -o "$OUT/ggen-code-inventory-test"
"$OUT/ggen-code-inventory-test" --nocapture

cp packs/code-modernization-pack/instances.ttl "$TMP/instances.ttl"
printf '\n# TAMPER\n' >> packs/code-modernization-pack/instances.ttl
if python3 scripts/ggen-code-inventory.py verify; then
    echo "ggen-upgrade-all: BUILD_BROKEN: tampered inventory was accepted" >&2
    exit 1
fi
mv "$TMP/instances.ttl" packs/code-modernization-pack/instances.ttl
python3 scripts/ggen-code-inventory.py verify

restore_repository_config
mkdir -p "$OUT"
git diff --stat | tee "$OUT/diff-stat.txt"
git diff --binary > "$OUT/generated.patch"
cp packs/code-modernization-pack/instances.ttl "$OUT/"
cp packs/code-modernization-pack/instances.receipt.json "$OUT/"
cp docs/standing/GGEN_CODE_MODERNIZATION.md "$OUT/"
cp crates/ggen/tests/ggen_code_inventory.rs "$OUT/"

cat > "$OUT/execution.receipt.json" <<JSON
{
  "schema": "praxis-ggen-upgrade-execution.v1",
  "head_sha": "$(git rev-parse HEAD)",
  "generated_test_blake3": "$second_test",
  "generated_report_blake3": "$second_report",
  "instances_blake3": "$second_instances",
  "tamper_refused": true,
  "second_render_identical": true,
  "generated_verifier_executed": true
}
JSON

echo "ggen-upgrade-all: PARTIAL_ALIVE"
echo "generated_test_blake3=$second_test"
echo "generated_report_blake3=$second_report"
echo "instances_blake3=$second_instances"
